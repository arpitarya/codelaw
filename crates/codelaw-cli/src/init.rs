//! `codelaw init` (SR-RULE-PACKS decision 6, SR-CONFIG decision 8,
//! SR-CONSUMER-LAYOUT decision 8): detect the languages, install their rule
//! packs with the package manager the repo already uses, and write the
//! config, the rules folder and the `.gitignore` line; with `--agents`, the
//! `[agents] targets` and the agent files (SR-AGENT-SURFACES decision 2).

use crate::fail;
use clap::Args;
use codelaw_core::Language;
use codelaw_rules::FIRST_PARTY_PACKS;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, ExitCode};

#[derive(Args)]
pub struct InitArgs {
    /// Print the install commands instead of running them.
    #[arg(long)]
    no_install: bool,
    /// Install every pack from one registry: `npm` or `pypi`.
    #[arg(long)]
    registry: Option<String>,
    /// Agent files to write: claude, cowork, copilot, codex, kiro (SR-AGENT-SURFACES).
    #[arg(long, value_delimiter = ',')]
    agents: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Registry {
    Npm,
    Pypi,
}

/// The pack each language needs (decision 6).
fn pack_for(language: Language) -> &'static str {
    match language {
        Language::Javascript | Language::Jsx | Language::Typescript | Language::Tsx => "js",
        Language::Html => "html",
        Language::Css => "css",
        Language::Python => "python",
        Language::Json | Language::Yaml | Language::Toml | Language::Markdown => "data",
    }
}

fn detect_languages(root: &Path) -> BTreeSet<Language> {
    let mut out = BTreeSet::new();
    let walk = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_global(false)
        .require_git(false)
        .filter_entry(|e| !codelaw_config::DEFAULT_EXCLUDES.iter().any(|x| e.file_name() == *x))
        .build();
    for e in walk.flatten() {
        if e.file_type().is_some_and(|t| t.is_file())
            && let Some(l) = Language::from_path(e.path())
        {
            out.insert(l);
        }
    }
    out
}

fn depends_on(package_json: &str, name: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(package_json) else { return false };
    ["dependencies", "devDependencies", "peerDependencies"].iter().any(|k| v.get(k).and_then(|d| d.get(name)).is_some())
}

/// Each pack's `recommended` presets for what the repo holds.
fn presets(packs: &BTreeSet<&str>, languages: &BTreeSet<Language>, package_json: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    for p in packs {
        match *p {
            "js" => {
                out.push("codelaw:js/recommended".into());
                if languages.iter().any(|l| l.is_typescript()) {
                    out.push("codelaw:ts/recommended".into());
                }
                if package_json.is_some_and(|j| depends_on(j, "react")) {
                    out.push("codelaw:react/recommended".into());
                }
                if package_json.is_some_and(|j| depends_on(j, "@stencil/core")) {
                    out.push("codelaw:stencil/recommended".into());
                }
            }
            "html" => out.push("codelaw:html/recommended".into()),
            "css" => out.push("codelaw:css/recommended".into()),
            "python" => out.push("codelaw:py/recommended".into()),
            "data" => {
                out.push("codelaw:data/recommended".into());
                if languages.contains(&Language::Markdown) {
                    out.push("codelaw:md/recommended".into());
                }
            }
            _ => {}
        }
    }
    out
}

/// The install command for one registry, with the repo's own manager.
fn install_command(root: &Path, registry: Registry, packages: &[&str]) -> Vec<String> {
    let has = |f: &str| root.join(f).exists();
    let mut cmd: Vec<String> = match registry {
        Registry::Npm if has("pnpm-lock.yaml") => vec!["pnpm".into(), "add".into(), "-D".into()],
        Registry::Npm if has("yarn.lock") => vec!["yarn".into(), "add".into(), "-D".into()],
        Registry::Npm if has("bun.lockb") || has("bun.lock") => vec!["bun".into(), "add".into(), "-d".into()],
        Registry::Npm => vec!["npm".into(), "install".into(), "-D".into()],
        Registry::Pypi if has("uv.lock") => vec!["uv".into(), "add".into(), "--dev".into()],
        Registry::Pypi if has("poetry.lock") => vec!["poetry".into(), "add".into(), "--group".into(), "dev".into()],
        Registry::Pypi => vec!["pip".into(), "install".into()],
    };
    cmd.extend(packages.iter().map(|p| p.to_string()));
    cmd
}

pub fn init(args: InitArgs) -> ExitCode {
    let targets = match codelaw_agents::parse_targets(&args.agents) {
        Ok(t) => t,
        Err(e) => return fail(format!("--agents: {e}")),
    };
    let forced = match args.registry.as_deref() {
        None => None,
        Some("npm") => Some(Registry::Npm),
        Some("pypi") => Some(Registry::Pypi),
        Some(other) => return fail(format!("--registry is `npm` or `pypi`, not `{other}`")),
    };
    let root = match std::env::current_dir().and_then(std::fs::canonicalize) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    let mut warnings = Vec::new();
    if let Ok(Some(existing)) = codelaw_config::load_file(&root, &mut warnings) {
        return fail(format!("{} already configures codelaw here", codelaw_core::normalize_path(&existing.path)));
    }

    let languages = detect_languages(&root);
    let packs: BTreeSet<&str> = languages.iter().map(|l| pack_for(*l)).collect();
    let package_json = std::fs::read_to_string(root.join("package.json")).ok();
    let has_npm = package_json.is_some();
    let has_pypi = root.join("pyproject.toml").is_file();
    let extends = presets(&packs, &languages, package_json.as_deref());

    // Which registry each pack comes from: `--registry`, else the one the
    // repo uses; a repo with both takes the Python pack from PyPI and the
    // rest from npm.
    let registry_for = |pack: &str| -> Registry {
        if let Some(r) = forced {
            return r;
        }
        match (has_npm, has_pypi) {
            (true, true) => {
                if pack == "python" { Registry::Pypi } else { Registry::Npm }
            }
            (false, true) => Registry::Pypi,
            (true, false) => Registry::Npm,
            (false, false) => {
                if languages.iter().all(|l| *l == Language::Python || pack_for(*l) == "data") && languages.contains(&Language::Python) {
                    Registry::Pypi
                } else {
                    Registry::Npm
                }
            }
        }
    };
    let mut by_registry: std::collections::BTreeMap<Registry, Vec<&str>> = Default::default();
    for p in &packs {
        let (_, npm, pypi, _) = FIRST_PARTY_PACKS.iter().find(|x| x.0 == *p).expect("a first-party pack");
        let r = registry_for(p);
        by_registry.entry(r).or_default().push(if r == Registry::Npm { npm } else { pypi });
    }

    // codelaw.toml: `extends` and nothing else (SR-CONFIG decision 8).
    let mut config = String::from("# codelaw configuration — see `codelaw config --print` and SR-CONFIG.\n");
    let quoted: Vec<String> = extends.iter().map(|e| format!("\"{e}\"")).collect();
    config.push_str(&format!("extends = [{}]\n", quoted.join(", ")));
    if !targets.is_empty() {
        let names: Vec<String> = targets.iter().map(|t| format!("\"{}\"", t.name())).collect();
        config.push_str(&format!("\n[agents]\ntargets = [{}]\n", names.join(", ")));
    }
    let rules_dir = root.join(".codelaw").join("rules");
    let written = std::fs::write(root.join("codelaw.toml"), &config)
        .and_then(|_| std::fs::create_dir_all(&rules_dir))
        .and_then(|_| {
            std::fs::write(
                rules_dir.join("README.md"),
                "# Project rules\n\nOne Markdown file per rule, at `<namespace>/<name>.md`. Start one with\n`codelaw new-rule <namespace>/<name> --lang <languages>`, then run `codelaw test`.\n",
            )
        })
        .and_then(|_| {
            let gi = root.join(".gitignore");
            let current = std::fs::read_to_string(&gi).unwrap_or_default();
            if current.lines().any(|l| l.trim() == ".codelaw/cache/" || l.trim() == ".codelaw/cache") {
                return Ok(());
            }
            let sep = if current.is_empty() || current.ends_with('\n') { "" } else { "\n" };
            std::fs::write(&gi, format!("{current}{sep}.codelaw/cache/\n"))
        });
    if let Err(e) = written {
        return fail(e);
    }
    let langs: Vec<&str> = languages.iter().map(|l| l.name()).collect();
    println!("languages: {}", if langs.is_empty() { "none found".into() } else { langs.join(", ") });
    println!("wrote codelaw.toml, .codelaw/rules/README.md and the .gitignore line");

    let commands: Vec<Vec<String>> = by_registry.iter().map(|(r, pkgs)| install_command(&root, *r, pkgs)).collect();
    if args.no_install && !commands.is_empty() {
        println!("install the rule packs with:");
        for c in &commands {
            println!("  {}", c.join(" "));
        }
    } else {
        for c in &commands {
            println!("running: {}", c.join(" "));
            match Command::new(&c[0]).args(&c[1..]).current_dir(&root).status() {
                Ok(s) if s.success() => {}
                Ok(s) => return fail(format!("`{}` exited with {s}; run it by hand once it can succeed", c.join(" "))),
                Err(e) => return fail(format!("cannot run `{}`: {e}; run it by hand", c.join(" "))),
            }
        }
    }
    // After the packs, so the per-language instructions list their rules.
    if !targets.is_empty() {
        match codelaw_agents::write_for_init(&root, &targets, None) {
            Ok((written, notes)) => {
                println!("wrote {} agent files for {}", written.len(), targets.iter().map(|t| t.name()).collect::<Vec<_>>().join(", "));
                for n in notes {
                    println!("\n{n}");
                }
            }
            Err(e) => return fail(e),
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_follow_the_repo() {
        let langs: BTreeSet<Language> = [Language::Tsx, Language::Markdown, Language::Python].into();
        let packs: BTreeSet<&str> = langs.iter().map(|l| pack_for(*l)).collect();
        let pj = r#"{"dependencies":{"react":"^19"}}"#;
        assert_eq!(
            presets(&packs, &langs, Some(pj)),
            ["codelaw:data/recommended", "codelaw:md/recommended", "codelaw:js/recommended", "codelaw:ts/recommended", "codelaw:react/recommended", "codelaw:py/recommended"]
        );
    }
}
