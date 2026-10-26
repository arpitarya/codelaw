//! `codelaw sync-agents [--check]` (SR-AGENT-SURFACES decisions 1–3): every
//! agent file of the decision-2 matrix, generated from the skills codelaw
//! ships and the project's rules. A file codelaw owns is written whole; a
//! file the team shares is touched only inside its marker block
//! (`CLAUDE.md`, `AGENTS.md`) or at its `codelaw` key (`.mcp.json`,
//! `.vscode/mcp.json`, `.kiro/settings/mcp.json`, the hook entry of
//! `.claude/settings.json`). `--check` writes nothing and fails on drift.
//!
//! User-level config (Codex) is printed, never written: codelaw does not
//! touch the home directory (decision 2).
//!
//! Every text comes from the installed skills package (W-77,
//! [`crate::package`]); with none at this binary's version, both commands
//! refuse with exit 2 and the install line.

use crate::package::{self, AgentFiles};
use crate::skills::{rule_author, skills, with_generated_note};
use crate::{AgentsConfig, Target, launcher};
use clap::Args;
use codelaw_core::template::json_escape;
use codelaw_core::Language;
use codelaw_engine::{EngineOptions, Project};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

#[derive(Args, Clone, Debug, Default)]
pub struct SyncArgs {
    /// Write nothing; exit 1 when a generated file differs from what the
    /// rules and skills would produce.
    #[arg(long)]
    pub check: bool,
}

pub const BEGIN: &str = "<!-- codelaw:begin -->";
pub const END: &str = "<!-- codelaw:end -->";

/// The generated note of a shared file's block and of the steering files
/// (templates/agents/note-block.md, SR-LAW-10).
fn note(files: &AgentFiles) -> Result<&str, String> {
    Ok(files.text("note-block.md")?.trim_end())
}

/// Render a template that holds JSON into a value.
fn json_template(files: &AgentFiles, path: &str, slots: &[(&str, &str)]) -> Result<Value, String> {
    let text = files.render(path, slots)?;
    serde_json::from_str(&text).map_err(|e| format!("{}/{path}: not valid JSON once rendered ({e})", files.root.display()))
}

/// A list of words as a JSON array, for an `args` slot.
fn json_array(words: &[String]) -> String {
    serde_json::to_string(words).expect("strings serialise")
}

/// One planned file: its path from the root and its full new text, or
/// `None` for a stale generated file to remove.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub path: String,
    pub content: Option<String>,
}

/// What a sync produces: the files, and the snippets to print.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub files: Vec<Planned>,
    pub notes: Vec<String>,
}

/// One rule as the path-scoped surfaces list it.
#[derive(Clone, Debug)]
pub struct RuleLine {
    pub id: String,
    pub title: String,
    pub fix: &'static str,
    pub languages: Vec<Language>,
}

/// What the generated files are made from besides the skills.
#[derive(Clone, Debug, Default)]
pub struct Sources {
    /// `extends` of the root config.
    pub presets: Vec<String>,
    /// The enabled rules; `None` when the rules could not be loaded (an
    /// `init --no-install` before the packs are installed).
    pub rules: Option<Vec<RuleLine>>,
}

impl Sources {
    pub fn from_project(project: &Project) -> Sources {
        let presets = project.config.files.first().map(|f| f.raw.extends.clone()).unwrap_or_default();
        let rules = project
            .list_rules()
            .into_iter()
            .zip(project.compiled.iter())
            .filter(|(row, _)| row.severity != "off" && !row.preview)
            .map(|(row, c)| RuleLine { id: row.id, title: row.title, fix: row.fix.name(), languages: c.loaded.rule.languages.clone() })
            .collect();
        Sources { presets, rules: Some(rules) }
    }
}

/// The extensions `Language::from_path` maps to each language, as a glob.
fn glob(language: Language) -> &'static str {
    match language {
        Language::Javascript => "**/*.{js,mjs,cjs}",
        Language::Jsx => "**/*.jsx",
        Language::Typescript => "**/*.{ts,mts,cts}",
        Language::Tsx => "**/*.tsx",
        Language::Python => "**/*.{py,pyi}",
        Language::Json => "**/*.{json,jsonc}",
        Language::Yaml => "**/*.{yml,yaml}",
        Language::Toml => "**/*.toml",
        Language::Markdown => "**/*.{md,mdx}",
        Language::Css => "**/*.css",
        Language::Scss => "**/*.scss",
        Language::Html => "**/*.{html,htm}",
    }
}

/// The always-on text (decision 3: under 40 lines; what codelaw is, ask
/// before you write, check what you changed, the presets, the skills).
pub fn always_on(files: &AgentFiles, sources: &Sources, cmd: &str) -> Result<String, String> {
    let presets = if sources.presets.is_empty() {
        "none (local rules only)".to_owned()
    } else {
        sources.presets.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")
    };
    files.render("always-on.md", &[("cmd", cmd), ("presets", &presets)])
}

/// The path-scoped text for one language (decision 3: one line per rule).
fn path_scoped(files: &AgentFiles, language: Language, rules: &[&RuleLine], cmd: &str) -> Result<String, String> {
    let lines = rules
        .iter()
        .map(|r| files.render("path-scoped-rule.md", &[("id", r.id.as_str()), ("title", r.title.as_str()), ("fix", r.fix)]))
        .collect::<Result<String, String>>()?;
    files.render("path-scoped.md", &[("language", language.display()), ("cmd", cmd), ("rules", &lines)])
}

/// Replace the marker block of a shared file, or append one.
pub fn with_block(files: &AgentFiles, existing: Option<&str>, body: &str) -> Result<String, String> {
    let block = files.render("block.md", &[("note", note(files)?), ("body", body)])?;
    let Some(text) = existing.filter(|t| !t.is_empty()) else { return Ok(block) };
    if let (Some(b), Some(e)) = (text.find(BEGIN), text.find(END))
        && b < e
    {
        let after = &text[e + END.len()..];
        let after = after.strip_prefix('\n').unwrap_or(after);
        return Ok(format!("{}{block}{after}", &text[..b]));
    }
    let sep = if text.ends_with("\n\n") {
        ""
    } else if text.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    Ok(format!("{text}{sep}{block}"))
}

fn pretty(v: &Value) -> String {
    format!("{}\n", serde_json::to_string_pretty(v).expect("json"))
}

/// Set `key.codelaw` of a shared JSON file, keeping the rest.
fn with_json_key(existing: Option<&str>, path: &str, key: &str, value: Value) -> Result<String, String> {
    let mut v = parse_json(existing, path)?;
    let obj = v.as_object_mut().ok_or_else(|| format!("{path}: not a JSON object; codelaw cannot add its entry"))?;
    let section = obj.entry(key).or_insert_with(|| json!({}));
    let section = section.as_object_mut().ok_or_else(|| format!("{path}: `{key}` is not an object"))?;
    section.insert("codelaw".into(), value);
    Ok(pretty(&v))
}

fn parse_json(existing: Option<&str>, path: &str) -> Result<Value, String> {
    match existing.filter(|t| !t.trim().is_empty()) {
        None => Ok(json!({})),
        Some(t) => serde_json::from_str(t).map_err(|e| format!("{path}: not valid JSON ({e}); fix it or remove it, then re-run `codelaw sync-agents`")),
    }
}

/// Whether a Claude Code hook entry is codelaw's: a command that runs
/// `codelaw … hook`.
fn is_codelaw_hook(entry: &Value) -> bool {
    entry["hooks"].as_array().is_some_and(|hs| {
        hs.iter().any(|h| h["command"].as_str().is_some_and(|c| c.contains("codelaw") && c.split_whitespace().any(|w| w == "hook")))
    })
}

/// Claude Code's `PostToolUse` group (Claude Code and the Cowork plugin).
fn claude_hook_group(files: &AgentFiles, cmd: &str) -> Result<Value, String> {
    json_template(files, "claude/hook-group.json", &[("cmd", &json_escape(cmd))])
}

/// `.claude/settings.json` with codelaw's `PostToolUse` entry replaced.
fn with_claude_hook(files: &AgentFiles, existing: Option<&str>, path: &str, cmd: &str) -> Result<String, String> {
    let mut v = parse_json(existing, path)?;
    let obj = v.as_object_mut().ok_or_else(|| format!("{path}: not a JSON object"))?;
    let hooks = obj.entry("hooks").or_insert_with(|| json!({}));
    let hooks = hooks.as_object_mut().ok_or_else(|| format!("{path}: `hooks` is not an object"))?;
    let post = hooks.entry("PostToolUse").or_insert_with(|| json!([]));
    let post = post.as_array_mut().ok_or_else(|| format!("{path}: `hooks.PostToolUse` is not a list"))?;
    post.retain(|e| !is_codelaw_hook(e));
    post.push(claude_hook_group(files, cmd)?);
    Ok(pretty(&v))
}

/// The skills folder of each target that has one in the repo.
fn skills_dir(target: Target) -> &'static str {
    match target {
        Target::Claude => ".claude/skills",
        Target::Cowork => ".claude/plugins/codelaw/skills",
        Target::Copilot => ".github/skills",
        Target::Codex => ".agents/skills",
        Target::Kiro => ".kiro/skills",
    }
}

/// The Cowork plugin's folder: the matrix bundles Cowork's skills, MCP and
/// hook in a plugin and names no path; it lives under `.claude/`, which
/// SR-CONSUMER-LAYOUT's tree lists as generated.
pub const COWORK_PLUGIN: &str = ".claude/plugins/codelaw";

/// Plan every file for `cfg.targets` from the skills package's `agents`.
pub fn plan(root: &Path, cfg: &AgentsConfig, sources: &Sources, agents: &AgentFiles) -> Result<Plan, String> {
    let catalogue = skills(agents)?;
    let words = launcher(root, cfg.command.as_deref());
    let cmd = words.join(" ");
    let mcp_command = words[0].clone();
    let mut mcp_args: Vec<String> = words[1..].to_vec();
    mcp_args.push("mcp".into());
    let read = |p: &str| std::fs::read_to_string(root.join(p)).ok();
    let mut files: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut notes = Vec::new();
    let body = always_on(agents, sources, &cmd)?;
    let note = note(agents)?;
    let targets = &cfg.targets;
    let has = |t: Target| targets.contains(&t);

    // Skills: the catalogue in every target's folder; a `codelaw-*` skill
    // the catalogue no longer has is removed.
    for t in targets {
        let dir = skills_dir(*t);
        for s in &catalogue {
            files.insert(format!("{dir}/{}/SKILL.md", s.name), Some(with_generated_note(agents, s.text)?));
        }
        for e in std::fs::read_dir(root.join(dir)).into_iter().flatten().flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("codelaw-") && !catalogue.iter().any(|s| s.name == name) && e.path().join("SKILL.md").is_file() {
                files.insert(format!("{dir}/{name}/SKILL.md"), None);
            }
        }
    }

    // Path-scoped surfaces (Copilot, Kiro): one file per language that has
    // rules; stale ones are removed.
    let scoped: Vec<(&str, &str, &str)> = [(Target::Copilot, ".github/instructions", ".instructions.md"), (Target::Kiro, ".kiro/steering", ".md")]
        .into_iter()
        .filter(|(t, _, _)| has(*t))
        .map(|(t, d, ext)| (t.name(), d, ext))
        .collect();
    match &sources.rules {
        Some(rules) => {
            for (target, dir, ext) in &scoped {
                let mut wanted = Vec::new();
                for l in Language::ALL {
                    let these: Vec<&RuleLine> = rules.iter().filter(|r| r.languages.contains(&l)).collect();
                    if these.is_empty() {
                        continue;
                    }
                    let path = format!("{dir}/codelaw-{}{ext}", l.name());
                    let tpl = if *target == "copilot" { "copilot/instructions.md" } else { "kiro/steering-language.md" };
                    let text = agents.render(tpl, &[("glob", glob(l)), ("note", note), ("body", &path_scoped(agents, l, &these, &cmd)?)])?;
                    files.insert(path.clone(), Some(text));
                    wanted.push(path);
                }
                for e in std::fs::read_dir(root.join(dir)).into_iter().flatten().flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let path = format!("{dir}/{name}");
                    let is_lang = name.strip_prefix("codelaw-").and_then(|r| r.strip_suffix(ext)).is_some_and(|l| Language::from_name(l).is_some());
                    if is_lang && !wanted.contains(&path) {
                        files.insert(path, None);
                    }
                }
            }
        }
        None if !scoped.is_empty() => notes.push(format!(
            "the rules did not load, so the per-language instructions were not written; install the rule packs, then run `{cmd} sync-agents`"
        )),
        None => {}
    }

    // Always-on instructions.
    if has(Target::Claude) {
        files.insert("CLAUDE.md".into(), Some(with_block(agents, read("CLAUDE.md").as_deref(), &body)?));
    }
    if has(Target::Copilot) || has(Target::Codex) {
        files.insert("AGENTS.md".into(), Some(with_block(agents, read("AGENTS.md").as_deref(), &body)?));
    }
    if has(Target::Kiro) {
        files.insert(".kiro/steering/codelaw.md".into(), Some(agents.render("kiro/steering.md", &[("note", note), ("body", &body)])?));
    }

    // Custom agent.
    if has(Target::Claude) {
        files.insert(".claude/agents/codelaw-rule-author.md".into(), Some(with_generated_note(agents, rule_author(agents)?)?));
    }
    if has(Target::Copilot) {
        files.insert(".github/agents/codelaw-rule-author.agent.md".into(), Some(with_generated_note(agents, rule_author(agents)?)?));
    }

    // Hooks and MCP registration.
    let args_json = json_array(&mcp_args);
    let command_json = json_escape(&mcp_command);
    let stdio = json_template(agents, "mcp-server.json", &[("command", &command_json), ("args", &args_json)])?;
    if has(Target::Claude) {
        files.insert(".claude/settings.json".into(), Some(with_claude_hook(agents, read(".claude/settings.json").as_deref(), ".claude/settings.json", &cmd)?));
        files.insert(".mcp.json".into(), Some(with_json_key(read(".mcp.json").as_deref(), ".mcp.json", "mcpServers", stdio.clone())?));
    }
    if has(Target::Cowork) {
        let hooks = json_template(agents, "cowork/hooks.json", &[("group", &claude_hook_group(agents, &cmd)?.to_string())])?;
        let mcp = json_template(agents, "cowork/mcp.json", &[("server", &stdio.to_string())])?;
        let manifest = json_template(agents, "cowork/plugin.json", &[("version", &json_escape(env!("CARGO_PKG_VERSION")))])?;
        files.insert(format!("{COWORK_PLUGIN}/.claude-plugin/plugin.json"), Some(pretty(&manifest)));
        files.insert(format!("{COWORK_PLUGIN}/hooks/hooks.json"), Some(pretty(&hooks)));
        files.insert(format!("{COWORK_PLUGIN}/.mcp.json"), Some(pretty(&mcp)));
    }
    if has(Target::Copilot) {
        let hook = json_template(agents, "copilot/hook.json", &[("cmd", &json_escape(&cmd))])?;
        files.insert(".github/hooks/codelaw.json".into(), Some(pretty(&hook)));
        let server = json_template(agents, "copilot/mcp-server.json", &[("command", &command_json), ("args", &args_json)])?;
        files.insert(".vscode/mcp.json".into(), Some(with_json_key(read(".vscode/mcp.json").as_deref(), ".vscode/mcp.json", "servers", server)?));
    }
    if has(Target::Kiro) {
        let patterns: Vec<&str> = Language::ALL.iter().flat_map(|l| ext_alternatives(*l)).collect();
        let matcher = patterns.join("|");
        let hook = json_template(agents, "kiro/hook.json", &[("extensions", &json_escape(&matcher)), ("cmd", &json_escape(&cmd))])?;
        files.insert(".kiro/hooks/codelaw.json".into(), Some(pretty(&hook)));
        files.insert(".kiro/settings/mcp.json".into(), Some(with_json_key(read(".kiro/settings/mcp.json").as_deref(), ".kiro/settings/mcp.json", "mcpServers", stdio.clone())?));
    }
    if has(Target::Codex) {
        let args: Vec<String> = mcp_args.iter().map(|a| format!("\"{a}\"")).collect();
        let snippet = agents.render("codex/config.toml", &[("command", &mcp_command), ("args", &args.join(", ")), ("cmd", &cmd)])?;
        notes.push(format!("Codex reads its MCP servers and hooks from your user config, which codelaw never edits; add this to ~/.codex/config.toml:\n\n{snippet}"));
    }

    Ok(Plan { files: files.into_iter().map(|(path, content)| Planned { path, content }).collect(), notes })
}

/// The extensions of a language, for Kiro's file-save regex.
fn ext_alternatives(language: Language) -> Vec<&'static str> {
    let g = glob(language).trim_start_matches("**/*.");
    g.trim_start_matches('{').trim_end_matches('}').split(',').collect()
}

/// The files whose disk text differs from the plan.
pub fn drift(root: &Path, plan: &Plan) -> Vec<String> {
    plan.files
        .iter()
        .filter(|f| {
            let current = std::fs::read_to_string(root.join(&f.path)).ok();
            match &f.content {
                Some(c) => current.as_deref() != Some(c.as_str()),
                None => current.is_some(),
            }
        })
        .map(|f| f.path.clone())
        .collect()
}

/// Write the plan; returns the paths changed (written or removed).
pub fn apply(root: &Path, plan: &Plan) -> Result<Vec<String>, String> {
    let changed = drift(root, plan);
    for f in plan.files.iter().filter(|f| changed.contains(&f.path)) {
        let abs = root.join(&f.path);
        match &f.content {
            Some(c) => {
                if let Some(dir) = abs.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", f.path))?;
                }
                std::fs::write(&abs, c).map_err(|e| format!("{}: {e}", f.path))?;
            }
            None => {
                std::fs::remove_file(&abs).map_err(|e| format!("{}: {e}", f.path))?;
                if let Some(dir) = abs.parent() {
                    let _ = std::fs::remove_dir(dir); // only when empty
                }
            }
        }
    }
    Ok(changed)
}

/// `init --agents`: write the agent files for `targets` in `root`, from the
/// project's rules when they load. Returns the paths written and the notes.
pub fn write_for_init(root: &Path, targets: &[Target], command: Option<&str>) -> Result<(Vec<String>, Vec<String>), String> {
    let cfg = AgentsConfig { targets: targets.to_vec(), command: command.map(str::to_owned) };
    let agents = package::locate(root)?;
    let sources = match Project::load(root, EngineOptions::from_env()) {
        Ok(p) => Sources::from_project(&p),
        Err(_) => {
            let mut warnings = Vec::new();
            let presets = codelaw_config::load_file(root, &mut warnings).ok().flatten().map(|f| f.raw.extends).unwrap_or_default();
            Sources { presets, rules: None }
        }
    };
    let plan = plan(root, &cfg, &sources, &agents)?;
    Ok((apply(root, &plan)?, plan.notes))
}

/// `codelaw sync-agents [--check]`: `0` in sync (or written), `1` drift
/// under `--check`, `2` a config, rule or file error.
pub fn sync_agents(args: SyncArgs) -> ExitCode {
    let fail = |e: String| {
        eprintln!("error: {e}");
        ExitCode::from(2)
    };
    let config = match codelaw_config::find_config(Path::new(".")) {
        Ok(c) => c,
        Err(e) => return fail(e.to_string()),
    };
    let cfg = match AgentsConfig::from_config(&config) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    if cfg.targets.is_empty() {
        // L5 decision 4: no agents, no agent files — and no rules to load.
        println!("no `[agents] targets` in the config: no agent files to write");
        return ExitCode::SUCCESS;
    }
    let project = match Project::from_config(config, EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e.to_string()),
    };
    let root = project.root().to_path_buf();
    let agents = match package::locate(&root) {
        Ok(a) => a,
        Err(e) => return fail(e),
    };
    let plan = match plan(&root, &cfg, &Sources::from_project(&project), &agents) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    if args.check {
        let d = drift(&root, &plan);
        if d.is_empty() {
            println!("codelaw sync-agents: {} agent files up to date", plan.files.iter().filter(|f| f.content.is_some()).count());
            return ExitCode::SUCCESS;
        }
        for p in &d {
            println!("out of date: {p}");
        }
        println!("{} agent files differ from the rules and skills; run `codelaw sync-agents` and commit the result", d.len());
        return ExitCode::from(1);
    }
    match apply(&root, &plan) {
        Ok(changed) => {
            for p in &changed {
                let removed = plan.files.iter().any(|f| &f.path == p && f.content.is_none());
                println!("{} {p}", if removed { "removed" } else { "wrote" });
            }
            if changed.is_empty() {
                println!("codelaw sync-agents: every agent file is up to date");
            }
            for n in &plan.notes {
                println!("\n{n}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_block_leaves_the_team_text_alone() {
        let a = crate::skills::source();
        let with_block = |e: Option<&str>, b: &str| with_block(&a, e, b).unwrap();
        let first = with_block(Some("# Team notes\n\nOur text."), "body\n");
        assert_eq!(first, format!("# Team notes\n\nOur text.\n\n{BEGIN}\n{}\n\nbody\n{END}\n", note(&a).unwrap()));
        let edited = first.replace("Our text.", "Our text, edited.") + "\nMore team text.\n";
        let second = with_block(Some(&edited), "new body\n");
        assert!(second.starts_with("# Team notes\n\nOur text, edited.\n\n"));
        assert!(second.ends_with(&format!("new body\n{END}\n\nMore team text.\n")), "{second}");
        assert!(!second.contains("\nbody\n"));
        assert_eq!(with_block(Some(&second), "new body\n"), second, "idempotent");
    }

    #[test]
    fn always_on_stays_under_forty_lines() {
        let s = Sources { presets: vec!["codelaw:js/recommended".into()], rules: None };
        let text = always_on(&crate::skills::source(), &s, "npx --no-install codelaw").unwrap();
        assert!(text.lines().count() < 40 - 3, "{}", text.lines().count());
        assert!(text.contains("`codelaw:js/recommended`") && text.contains("`npx --no-install codelaw check <files>`"));
    }

    #[test]
    fn a_shared_json_keeps_its_other_entries() {
        let mine = r#"{"hooks":{"PostToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"./lint.sh"}]},{"matcher":"Edit","hooks":[{"type":"command","command":"codelaw hook"}]}]},"model":"x"}"#;
        let a = crate::skills::source();
        let out = with_claude_hook(&a, Some(mine), "s", "uv run codelaw").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        let post = v["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(post.len(), 2);
        assert_eq!(post[0]["hooks"][0]["command"], "./lint.sh");
        assert_eq!(post[1]["hooks"][0]["command"], "uv run codelaw hook");
        assert_eq!(v["model"], "x");
        assert_eq!(with_claude_hook(&a, Some(&out), "s", "uv run codelaw").unwrap(), out);
        let mcp = with_json_key(Some(r#"{"mcpServers":{"other":{"command":"o"}}}"#), "m", "mcpServers", json!({"command": "codelaw", "args": ["mcp"]})).unwrap();
        let v: Value = serde_json::from_str(&mcp).unwrap();
        assert_eq!(v["mcpServers"]["other"]["command"], "o");
        assert_eq!(v["mcpServers"]["codelaw"]["args"][0], "mcp");
        assert!(with_json_key(Some("{ not json"), "m", "mcpServers", json!({})).unwrap_err().contains("not valid JSON"));
    }

    #[test]
    fn kiro_matches_every_l3_extension() {
        let all: Vec<&str> = Language::ALL.iter().flat_map(|l| ext_alternatives(*l)).collect();
        for ext in ["js", "mjs", "ts", "tsx", "py", "json", "yaml", "yml", "toml", "md", "css", "html"] {
            assert!(all.contains(&ext), "{ext}");
        }
    }
}
