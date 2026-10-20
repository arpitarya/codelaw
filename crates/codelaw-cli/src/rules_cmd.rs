//! `codelaw rules`, `rules lint`, `rules eject`, `explain`, `new-rule`, and
//! what `codelaw test` runs (SR-RULE-AUTHORING decisions 1 and 3,
//! SR-RULE-FORMAT decisions 7 and 8, SR-RULE-PACKS decision 10).

use crate::{fail, load};
use clap::{Args, Subcommand};
use codelaw_cache::Key;
use codelaw_config::Config;
use codelaw_core::{FixClass, Language};
use codelaw_report::Format;
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
    /// text or json.
    #[arg(long, default_value = "text")]
    format: String,
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

pub fn rules(args: RulesArgs) -> ExitCode {
    match args.action {
        Some(RulesAction::Lint { paths }) => return lint(paths),
        Some(RulesAction::Eject { id }) => return eject(&id),
        None => {}
    }
    let json = match crate::format(&args.format) {
        Ok(Format::Json) => true,
        Ok(_) => false,
        Err(e) => return fail(e),
    };
    let mut project = match load(&args.for_path.iter().cloned().collect::<Vec<_>>(), EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    if let Some(id) = args.why {
        return match project.rule_why(&id, args.for_path.as_deref()) {
            Ok(w) if json => print_json(&codelaw_report::why_json(&w)),
            Ok(w) => {
                print!("{}", codelaw_report::why_text(&w));
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        };
    }
    let rows = match &args.for_path {
        Some(p) => match project.rules_for(p) {
            Ok(r) => r,
            Err(e) => return fail(e),
        },
        None => project.list_rules(),
    };
    if json {
        return print_json(&codelaw_report::rules_json(&rows, args.for_path.as_deref().map(codelaw_core::normalize_path).as_deref()));
    }
    print!("{}", codelaw_report::rules_text(&rows));
    ExitCode::SUCCESS
}

fn print_json(v: &serde_json::Value) -> ExitCode {
    println!("{}", serde_json::to_string_pretty(v).expect("json"));
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

pub fn explain(id: &str, format: &str) -> ExitCode {
    let json = match crate::format(format) {
        Ok(Format::Json) => true,
        Ok(_) => false,
        Err(e) => return fail(e),
    };
    if let Some(e) = codelaw_rules::engine_rule(id) {
        if json {
            return print_json(&codelaw_report::explain_engine_json(e));
        }
        print!("{}", codelaw_report::explain_engine_text(e));
        return ExitCode::SUCCESS;
    }
    let project = match load(&[], EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    match project.find_rule(id) {
        Ok(r) if json => print_json(&codelaw_report::explain_json(r)),
        Ok(r) => {
            print!("{}", codelaw_report::explain_text(r));
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
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
