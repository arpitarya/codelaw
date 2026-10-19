//! `codelaw rules`, `rules lint`, `rules eject`, `explain`, `new-rule`, and
//! what `codelaw test` runs (SR-RULE-AUTHORING decisions 1 and 3,
//! SR-RULE-FORMAT decisions 7 and 8, SR-RULE-PACKS decision 10).

use crate::{fail, load};
use clap::{Args, Subcommand};
use codelaw_cache::Key;
use codelaw_config::Config;
use codelaw_core::{FixClass, Language};
use codelaw_engine::{EngineOptions, Project};
use codelaw_rules::{LoadError, LoadedRule, RuleFile, RuleSet, RuleSource, load_pack, load_rules_dir, parse_cases, parse_rule};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

#[derive(Args)]
pub struct RulesArgs {
    #[command(subcommand)]
    action: Option<RulesAction>,
    /// Only the rules that apply to this file.
    #[arg(long = "for")]
    for_path: Option<PathBuf>,
    /// Why a rule is active, and from which source.
    #[arg(long)]
    why: Option<String>,
}

#[derive(Subcommand)]
enum RulesAction {
    /// Check rule files against the template (SR-RULE-FORMAT decision 7).
    Lint { paths: Vec<PathBuf> },
    /// Copy a pack rule into `.codelaw/rules/` to customise it.
    Eject { id: String },
}

#[derive(Args)]
pub struct NewRuleArgs {
    /// `<namespace>/<name>`.
    id: String,
    /// The languages it applies to (comma-separated).
    #[arg(long, value_delimiter = ',', required = true)]
    lang: Vec<String>,
    /// `file` or `project`.
    #[arg(long, default_value = "file")]
    scope: String,
    /// `auto`, `suggest` or `manual`.
    #[arg(long, default_value = "auto")]
    fix: String,
}

fn local(rule: RuleFile) -> LoadedRule {
    LoadedRule {
        rule: Arc::new(rule),
        source: RuleSource::Local,
        default: None,
        enabled_by: None,
        shadows: Vec::new(),
        data: Default::default(),
        preview_blocked: false,
        external_allowed: true,
    }
}

fn errors_text(errors: &[LoadError]) -> String {
    errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
}

/// Rules from a pack folder, a rules folder or one rule file.
fn rules_at(path: &Path) -> Result<Vec<LoadedRule>, String> {
    if path.join("codelaw-pack.toml").is_file() {
        let pack = load_pack(path).map_err(|e| errors_text(&e))?;
        let source = RuleSource::Pack {
            name: pack.manifest.name.clone(),
            version: pack.manifest.version.clone(),
            first_party: pack.first_party,
            dir: pack.dir.clone(),
        };
        return Ok(pack
            .rules
            .iter()
            .map(|r| LoadedRule { source: source.clone(), data: pack.data.clone(), ..local(r.clone()) })
            .collect());
    }
    if path.is_dir() {
        return Ok(load_rules_dir(path).map_err(|e| errors_text(&e))?.into_iter().map(local).collect());
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let ns = path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or_default();
    let expected = format!("{ns}/{name}");
    let mut rule = parse_rule(path, &text, Some(&expected)).map_err(|e| errors_text(&e))?;
    let cases = path.with_file_name(format!("{name}.cases.md"));
    if let Ok(c) = std::fs::read_to_string(&cases) {
        rule.cases = parse_cases(&cases, &c).map_err(|e| errors_text(&e))?;
    }
    Ok(vec![local(rule)])
}

/// Compile rules as the engine would; any error is returned.
fn compile_check(rules: &[LoadedRule]) -> Result<(), String> {
    let mut rs: Vec<LoadedRule> = rules.to_vec();
    rs.sort_by(|a, b| a.rule.id.cmp(&b.rule.id));
    let set = RuleSet { rules: rs, packs: Vec::new(), warnings: Vec::new(), fingerprint: Key([0; 32]) };
    Project::from_parts(Config::defaults(PathBuf::from(".")), set, EngineOptions { threads: Some(1), no_cache: true, ..Default::default() })
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The rules `codelaw test [target]` runs.
pub fn rules_for_test(target: Option<&str>) -> Result<Vec<LoadedRule>, String> {
    match target {
        Some(t) if Path::new(t).exists() => {
            let rules = rules_at(Path::new(t))?;
            compile_check(&rules)?;
            Ok(rules)
        }
        Some(id) => {
            let project = load(&[], EngineOptions::from_env()).map_err(|e| e.to_string())?;
            project.rules.get(id).cloned().map(|r| vec![r]).ok_or_else(|| {
                let near = codelaw_config::did_you_mean(id, project.rules.rules.iter().map(|r| r.rule.id.as_str()))
                    .map(|n| format!("; did you mean `{n}`?"))
                    .unwrap_or_default();
                format!("no rule `{id}` in this project{near}")
            })
        }
        None => Ok(load(&[], EngineOptions::from_env()).map_err(|e| e.to_string())?.rules.rules),
    }
}

fn severity_of(cfg: &Option<codelaw_config::RuleConfig>) -> String {
    cfg.as_ref().and_then(|c| c.severity).map(|s| s.name().to_owned()).unwrap_or_else(|| "off".into())
}

pub fn rules(args: RulesArgs) -> ExitCode {
    match args.action {
        Some(RulesAction::Lint { paths }) => return lint(paths),
        Some(RulesAction::Eject { id }) => return eject(&id),
        None => {}
    }
    let project = match load(&args.for_path.iter().cloned().collect::<Vec<_>>(), EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let file = args.for_path.as_ref().map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()));
    let rel = file.as_ref().map(|f| codelaw_core::normalize_path(f.strip_prefix(project.root()).unwrap_or(f)));
    let language = file.as_ref().and_then(|f| Language::from_path(f));
    if args.for_path.is_some() && language.is_none() {
        return fail("--for needs a file codelaw checks (a known extension)");
    }
    let layers = file.as_ref().map(|f| project.config.layers_for(f));

    if let Some(id) = args.why {
        let Some(r) = project.rules.get(&id) else {
            return fail(format!("no rule `{id}` in this project"));
        };
        println!("{id}  {}", r.rule.title);
        println!("  source:     {} ({})", r.source.label(), codelaw_core::normalize_path(&r.rule.path));
        if !r.shadows.is_empty() {
            println!("  shadows:    {}", r.shadows.join(", "));
        }
        println!("  enabled by: {}", r.enabled_by.as_deref().unwrap_or("nothing (not in any extended preset)"));
        println!("  default:    {}", severity_of(&r.default));
        if r.preview_blocked {
            println!("  preview:    yes — runs only with `preview = true`");
        }
        if let (Some(layers), Some(rel), Some(lang)) = (&layers, &rel, language) {
            let compiled = project.compiled.iter().find(|c| c.id() == id).expect("compiled");
            for l in layers.iter().filter(|l| l.rules.contains_key(&id)) {
                println!("  set by:     {} = {}", l.source, l.rules[&id].severity().unwrap_or("(options)"));
            }
            let effective = project.rules.effective(r, layers);
            let applies = compiled.applies_to_language(lang) && compiled.applies_to_path(rel);
            println!("  for {rel}:  {}", if applies { severity_of(&effective) } else { "does not apply (language or `files`)".into() });
        }
        return ExitCode::SUCCESS;
    }

    for c in &project.compiled {
        let r = &c.loaded;
        let sev = match (&layers, &rel, language) {
            (Some(layers), Some(rel), Some(lang)) => {
                if !c.applies_to_language(lang) || !c.applies_to_path(rel) {
                    continue;
                }
                severity_of(&project.rules.effective(r, layers))
            }
            _ => severity_of(&r.default),
        };
        if args.for_path.is_some() && sev == "off" {
            continue;
        }
        let preview = if r.preview_blocked { "  (preview)" } else { "" };
        println!("{:<40} {:<6} {:<8} {}{preview}", r.rule.id, sev, r.rule.fix.name(), r.source.label());
    }
    ExitCode::SUCCESS
}

fn lint(paths: Vec<PathBuf>) -> ExitCode {
    // The config only: loading the rule set would refuse the very rules
    // lint is asked about.
    let paths = if paths.is_empty() {
        match codelaw_config::find_config(Path::new(".")) {
            Ok(c) => vec![c.rules_dir()],
            Err(e) => return fail(e),
        }
    } else {
        paths
    };
    let mut problems = 0;
    let mut checked = 0;
    for p in &paths {
        match rules_at(p).and_then(|rules| {
            checked += rules.len();
            compile_check(&rules)
        }) {
            Ok(()) => {}
            Err(e) => {
                problems += e.lines().count();
                println!("{e}");
            }
        }
    }
    println!("{checked} rule files · {problems} problems");
    if problems == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

fn eject(id: &str) -> ExitCode {
    let project = match load(&[], EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let Some(r) = project.rules.get(id) else {
        return fail(format!("no rule `{id}` in this project"));
    };
    if r.source == RuleSource::Local {
        return fail(format!("`{id}` is already a local rule ({})", codelaw_core::normalize_path(&r.rule.path)));
    }
    let (ns, name) = id.split_once('/').expect("ids are namespace/name");
    let dest = project.config.rules_dir().join(ns).join(format!("{name}.md"));
    if dest.exists() {
        return fail(format!("{} exists", codelaw_core::normalize_path(&dest)));
    }
    let copy = |from: &Path, to: &Path| -> std::io::Result<()> {
        std::fs::create_dir_all(to.parent().expect("a folder"))?;
        std::fs::copy(from, to).map(|_| ())
    };
    if let Err(e) = copy(&r.rule.path, &dest) {
        return fail(e);
    }
    let cases = r.rule.path.with_file_name(format!("{name}.cases.md"));
    if cases.is_file() {
        let _ = copy(&cases, &dest.with_file_name(format!("{name}.cases.md")));
    }
    println!("ejected {id} from {} to {}; the local copy now shadows it", r.source.label(), codelaw_core::normalize_path(&dest));
    ExitCode::SUCCESS
}

pub fn explain(id: &str) -> ExitCode {
    if let Some(e) = codelaw_rules::engine_rule(id) {
        println!("{}  {}\n\n  severity: {}  ·  fix: {}  ·  source: engine", e.id, e.title, e.severity, e.fix);
        for (i, s) in e.steps.iter().enumerate() {
            println!("  {}. {s}", i + 1);
        }
        return ExitCode::SUCCESS;
    }
    let project = match load(&[], EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let Some(r) = project.rules.get(id) else {
        let near = codelaw_config::did_you_mean(id, project.rules.rules.iter().map(|r| r.rule.id.as_str()))
            .map(|n| format!("; did you mean `{n}`?"))
            .unwrap_or_default();
        return fail(format!("unknown rule `{id}`{near}"));
    };
    let rule = &r.rule;
    let langs: Vec<&str> = rule.languages.iter().map(|l| l.name()).collect();
    println!("{}  {}\n", rule.id, rule.title);
    println!("{}\n", rule.summary);
    println!("Why: {}\n", rule.why);
    println!("  languages: {}  ·  severity: {}  ·  fix: {}  ·  source: {}", langs.join(", "), severity_of(&r.default), rule.fix, r.source.label());
    if let Some(reason) = &rule.fix_reason {
        println!("  not automatic because: {reason}");
    }
    if !rule.ported_from.is_empty() {
        println!("  ported from: {}", rule.ported_from.join(", "));
    }
    if rule.fix == FixClass::Manual || !rule.steps.is_empty() {
        println!("\nHow to fix:");
        for (i, s) in rule.steps.iter().enumerate() {
            println!("  {}. {s}", i + 1);
        }
    }
    println!("\n{}", codelaw_core::normalize_path(&rule.path));
    ExitCode::SUCCESS
}

/// The rule template with every required field filled
/// (SR-RULE-FORMAT decision 8).
pub fn template(id: &str, languages: &[Language], scope: &str, fix: FixClass) -> String {
    let names: Vec<&str> = languages.iter().map(|l| l.name()).collect();
    let mut s = format!("---\nid: {id}\ntitle: TODO one line on what the rule requires\nlanguages: [{}]\n", names.join(", "));
    if scope == "project" {
        s.push_str("scope: project\n");
    }
    s.push_str(&format!("fix: {fix}\n"));
    if fix != FixClass::Auto {
        s.push_str("fix_reason: \"TODO: one line on why the fix can't be automatic\"\n");
    }
    s.push_str(&format!("stability: preview\nsince: {}\n---\n\n", codelaw_core::VERSION));
    s.push_str("TODO: one paragraph on what the rule requires.\n\n**Why:** TODO: the reason, in one or two sentences.\n\n```codelaw\n");
    if scope == "project" {
        s.push_str("graph:\n  query: TODO\n");
    } else {
        s.push_str("match:\n  regex: \"^TODO$\"\n");
    }
    s.push_str("message: \"TODO: what is wrong\"\n");
    if fix != FixClass::Manual {
        s.push_str("fix: \"TODO\"\n");
    }
    s.push_str("```\n\n");
    if fix == FixClass::Manual {
        s.push_str("## How to fix\n\n1. TODO: the first step.\n\n");
    }
    s.push_str("## Examples\n");
    for l in languages {
        s.push_str(&format!("\n### {}\n\n#### Don't\n\n```{ext}\nTODO\n```\n\n#### Do\n\n```{ext}\n```\n", l.display(), ext = l.extension()));
    }
    s
}

pub fn new_rule(args: NewRuleArgs) -> ExitCode {
    let Some((ns, name)) = args.id.split_once('/').filter(|(a, b)| !a.is_empty() && !b.is_empty() && !b.contains('/')) else {
        return fail("a rule id is `<namespace>/<name>`");
    };
    let mut languages = Vec::new();
    for l in &args.lang {
        match Language::from_name(l) {
            Some(x) => languages.push(x),
            None => return fail(format!("unknown language `{l}`")),
        }
    }
    let Some(fix) = FixClass::parse(&args.fix) else {
        return fail("--fix is `auto`, `suggest` or `manual`");
    };
    if args.scope != "file" && args.scope != "project" {
        return fail("--scope is `file` or `project`");
    }
    let project = match load(&[], EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let dest = project.config.rules_dir().join(ns).join(format!("{name}.md"));
    if dest.exists() {
        return fail(format!("{} exists", codelaw_core::normalize_path(&dest)));
    }
    let text = template(&args.id, &languages, &args.scope, fix);
    if let Err(e) = std::fs::create_dir_all(dest.parent().expect("a folder")).and_then(|_| std::fs::write(&dest, text)) {
        return fail(e);
    }
    println!("wrote {}; next: write a failing Don't, then `codelaw tree` and `codelaw test {}`", codelaw_core::normalize_path(&dest), args.id);
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_scaffold_passes_rules_lint() {
        for fix in [FixClass::Auto, FixClass::Suggest, FixClass::Manual] {
            let text = template("acme/new", &[Language::Typescript, Language::Python, Language::Css, Language::Toml], "file", fix);
            let rule = parse_rule(Path::new("acme/new.md"), &text, Some("acme/new")).unwrap_or_else(|e| panic!("{fix}: {}", errors_text(&e)));
            compile_check(&[local(rule)]).unwrap_or_else(|e| panic!("{fix}: {e}"));
        }
    }
}
