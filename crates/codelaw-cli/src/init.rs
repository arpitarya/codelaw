//! `codelaw init` (SR-RULE-PACKS decision 6, SR-CONFIG decision 8,
//! SR-CONSUMER-LAYOUT decision 8): detect the languages, install their rule
//! packs with the package manager the repo already uses, and write the
//! config, the rules folder and the `.gitignore` line; with `--agents`, the
//! `[agents] targets` and the agent files (SR-AGENT-SURFACES decision 2).
//! The spell pack too, with its words file filled by `spell learn`, unless
//! `--no-spell` (SR-SPELL decision 1, W-112).

use crate::fail;
use clap::Args;
use codelaw_core::{Language, template};
use codelaw_rules::first_party;
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
    /// Install git hooks: `pre-commit` (blocks) and `post-commit` (reminds, never blocks); both when none is named. In a repo codelaw already configures, it installs only the hooks (SR-CLI-MCP decision 7).
    #[arg(long, value_delimiter = ',', num_args = 0.., value_name = "HOOKS")]
    git_hooks: Option<Vec<String>>,
    /// Write a CI workflow. `github` alone: the check workflow, which runs codelaw's GitHub Action at this version (SR-CLI-MCP decision 12a). `bitbucket`: `bitbucket-pipelines.yml`, a step that checks and sends the findings to Code Insights, SARIF a switch (decision 12b). With `--autofix` (`github` or `gitlab`): the scheduled auto-fix workflow (decision 12). In a repo codelaw already configures, it writes only the workflow.
    #[arg(long, value_name = "PROVIDER")]
    ci: Option<String>,
    /// How often the CI workflow runs the safe fixes, the formatter and `baseline prune`, and keeps one PR open: `daily` or `weekly`.
    #[arg(long, value_name = "WHEN", requires = "ci")]
    autofix: Option<String>,
    /// With `--autofix`: also apply `suggest` fixes, on a second branch and PR.
    #[arg(long = "unsafe", requires = "autofix")]
    unsafe_fixes: bool,
    /// Leave the spell pack out: no `codelaw:spell/recommended`, no `.codelaw/spell/words.txt`, no `spell learn` (SR-SPELL decision 1).
    #[arg(long)]
    no_spell: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Registry {
    Npm,
    Pypi,
}

/// The packs a repo needs (decision 6): one per language, `ts` for
/// TypeScript, and `react`, `stencil`, `tanstack-query` or `redux-toolkit`
/// when `package.json` depends on the library.
fn packs_for(languages: &BTreeSet<Language>, package_json: Option<&str>) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    for l in languages {
        out.insert(match l {
            Language::Javascript | Language::Jsx | Language::Typescript | Language::Tsx => "js",
            Language::Html => "html",
            Language::Css => "css",
            Language::Scss => "scss",
            Language::Python => "python",
            Language::Json | Language::Yaml | Language::Toml | Language::Markdown => "data",
        });
        if l.is_typescript() {
            out.insert("ts");
        }
    }
    // `sass` (or `sass-embedded`, `node-sass`) in package.json adds the scss pack too, and
    // SCSS files get the `css` pack as well: its rules run on `.scss`.
    if package_json.is_some_and(|j| depends_on(j, "sass") || depends_on(j, "sass-embedded") || depends_on(j, "node-sass")) {
        out.insert("scss");
    }
    if out.contains("scss") {
        out.insert("css");
    }
    if out.contains("js") {
        if package_json.is_some_and(|j| depends_on(j, "react")) {
            out.insert("react");
        }
        if package_json.is_some_and(|j| depends_on(j, "@stencil/core")) {
            out.insert("stencil");
        }
        if package_json.is_some_and(|j| depends_on(j, "@tanstack/react-query") || depends_on(j, "react-query")) {
            out.insert("tanstack-query");
        }
        if package_json.is_some_and(|j| depends_on(j, "@reduxjs/toolkit") || depends_on(j, "redux")) {
            out.insert("redux-toolkit");
        }
    }
    out
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

/// Whether the repo holds a CSS Module: a `*.module.css` or `*.module.scss` file.
fn has_css_modules(root: &Path) -> bool {
    let walk = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_global(false)
        .require_git(false)
        .filter_entry(|e| !codelaw_config::DEFAULT_EXCLUDES.iter().any(|x| e.file_name() == *x))
        .build();
    walk.flatten().any(|e| {
        e.file_type().is_some_and(|t| t.is_file()) && e.file_name().to_str().is_some_and(|n| n.to_ascii_lowercase().ends_with(".module.css") || n.to_ascii_lowercase().ends_with(".module.scss"))
    })
}

fn depends_on(package_json: &str, name: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(package_json) else { return false };
    ["dependencies", "devDependencies", "peerDependencies"].iter().any(|k| v.get(k).and_then(|d| d.get(name)).is_some())
}

/// Each installed pack's `recommended` presets for what the repo holds.
fn presets(packs: &BTreeSet<&str>, languages: &BTreeSet<Language>) -> Vec<String> {
    let mut out = Vec::new();
    for p in packs {
        match *p {
            "js" => {
                out.push("codelaw:js/recommended".into());
                if languages.iter().any(|l| l.is_typescript()) {
                    out.push("codelaw:js/typescript".into());
                }
            }
            "ts" => out.push("codelaw:ts/recommended".into()),
            "react" => out.push("codelaw:react/recommended".into()),
            "stencil" => out.push("codelaw:stencil/recommended".into()),
            "tanstack-query" => out.push("codelaw:tanstack-query/recommended".into()),
            "redux-toolkit" => out.push("codelaw:redux-toolkit/recommended".into()),
            "html" => out.push("codelaw:html/recommended".into()),
            "css" => out.push("codelaw:css/recommended".into()),
            "scss" => out.push("codelaw:scss/recommended".into()),
            "css-modules" => out.push("codelaw:css-modules/recommended".into()),
            "python" => out.push("codelaw:py/recommended".into()),
            "spell" => out.push("codelaw:spell/recommended".into()),
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

impl Registry {
    fn key(self) -> &'static str {
        match self {
            Registry::Npm => "npm",
            Registry::Pypi => "pypi",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Registry::Npm => "npm",
            Registry::Pypi => "PyPI",
        }
    }

    fn other(self) -> Registry {
        match self {
            Registry::Npm => Registry::Pypi,
            Registry::Pypi => Registry::Npm,
        }
    }
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

/// What `init` prints when the repo uses Fux (a `.fux/` folder): the optional
/// index package beside each pack it chose, and how to add it. It is only ever
/// printed, never run, and nothing is printed for a repo without `.fux/`
/// (SR-RULE-PACKS decision 6a; L6 decision 12: never installed by default).
fn fux_index_note(root: &Path, by_registry: &std::collections::BTreeMap<Registry, Vec<&str>>) -> Vec<String> {
    if !root.join(".fux").is_dir() || by_registry.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        "note: this repo uses Fux, and each rule pack has an optional index package that lets `fux_search` find its rules from plain words. `init` does not install them; to add them:".to_string(),
    ];
    for (registry, packs) in by_registry {
        let names: Vec<String> = packs.iter().map(|p| format!("{p}-fux-index")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        lines.push(format!("  {}", install_command(root, *registry, &refs).join(" ")));
    }
    lines.push("  then pin each: fux setup --index <its PyPI name> --index-npm <its npm name> (an npm-only pack: add its `kind = \"index\"` row to .fux/config/plugins.toml)".to_string());
    lines
}

pub fn init(args: InitArgs) -> ExitCode {
    let targets = match codelaw_agents::parse_targets(&args.agents) {
        Ok(t) => t,
        Err(e) => return fail(format!("--agents: {e}")),
    };
    let hooks = match args.git_hooks.as_deref().map(crate::git_hooks::parse).transpose() {
        Ok(h) => h,
        Err(e) => return fail(e),
    };
    // `Some((provider, None))` is the check workflow; with a schedule, the auto-fix one.
    let ci = match (args.ci.as_deref(), args.autofix.as_deref()) {
        (Some(p), w) => match (crate::ci::parse_provider(p), w.map(crate::ci::parse_schedule).transpose()) {
            (Ok(p), Ok(w)) => Some((p, w)),
            (Err(e), _) | (_, Err(e)) => return fail(e),
        },
        _ => None,
    };
    // Refused before anything is written: there is no GitLab check workflow
    // and no Bitbucket auto-fix one.
    let refused = match ci {
        Some((p, None)) => crate::ci::check_workflow(p, codelaw_core::VERSION).err(),
        Some((p, Some(w))) => crate::ci::workflow(p, w, args.unsafe_fixes, codelaw_core::VERSION).err(),
        None => None,
    };
    if let Some(e) = refused {
        return fail(e);
    }
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
        if hooks.is_some() || ci.is_some() {
            if let Some((p, w)) = ci {
                let written = match w {
                    Some(w) => crate::ci::install(&root, p, w, args.unsafe_fixes),
                    None => crate::ci::install_check(&root, p),
                };
                match written {
                    Ok(lines) => lines.iter().for_each(|l| println!("{l}")),
                    Err(e) => return fail(e),
                }
            }
            return match &hooks {
                Some(hooks) => install_hooks(&root, hooks),
                None => ExitCode::SUCCESS,
            };
        }
        return fail(format!("{} already configures codelaw here", codelaw_core::normalize_path(&existing.path)));
    }

    // `--agents --no-install` installs nothing, so the skills package must
    // already be here at this binary's version: refuse before writing any
    // file (W-77; SR-AGENT-SURFACES decision 8).
    if !targets.is_empty()
        && args.no_install
        && let Err(e) = codelaw_agents::package::locate(&root)
    {
        return fail(format!("{e}\nnothing was written; install it, or run `codelaw init` without --no-install"));
    }

    let languages = detect_languages(&root);
    let package_json = std::fs::read_to_string(root.join("package.json")).ok();
    let mut wanted = packs_for(&languages, package_json.as_deref());
    // CSS Modules are a bundler feature: `*.module.*` files add the npm-only pack.
    if has_css_modules(&root) {
        wanted.insert("css-modules");
    }
    // Spelling from day one, on both registries (SR-SPELL decision 1).
    if !args.no_spell {
        wanted.insert("spell");
    }
    let has_npm = package_json.is_some();
    let has_pypi = root.join("pyproject.toml").is_file();

    // Which registry each pack comes from: `--registry`, else the one the
    // repo uses; a repo with both takes the Python pack from PyPI and the
    // rest from npm. A pack the chosen registry does not publish (L6
    // decision 11) comes from the other one when the repo uses it too, or
    // uses neither; otherwise it is left out and init says how to add it.
    let preferred = |pack: &str| -> Registry {
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
                if languages.iter().all(|l| *l == Language::Python || matches!(l, Language::Json | Language::Yaml | Language::Toml | Language::Markdown)) && languages.contains(&Language::Python) {
                    Registry::Pypi
                } else {
                    Registry::Npm
                }
            }
        }
    };
    let in_use = |r: Registry| match r {
        Registry::Npm => has_npm,
        Registry::Pypi => has_pypi,
    };
    let mut by_registry: std::collections::BTreeMap<Registry, Vec<&str>> = Default::default();
    let mut packs: BTreeSet<&str> = BTreeSet::new();
    let mut skipped: Vec<String> = Vec::new();
    for p in &wanted {
        let pack = first_party(p);
        let first = preferred(p);
        let chosen = if pack.on(first.key()).is_some() {
            Some(first)
        } else if forced.is_none() && (in_use(first.other()) || (!has_npm && !has_pypi)) && pack.on(first.other().key()).is_some() {
            Some(first.other())
        } else {
            None
        };
        match chosen {
            Some(r) => {
                by_registry.entry(r).or_default().push(pack.on(r.key()).expect("published there"));
                packs.insert(p);
            }
            None => {
                let (r, name) = if let Some(n) = pack.npm { (Registry::Npm, n) } else { (Registry::Pypi, pack.pypi.expect("on a registry")) };
                skipped.push(format!(
                    "{name} is published on {} only, so it was not installed from {}; to lint with it, install it from {} and add its presets to `extends`",
                    r.label(),
                    first.label(),
                    r.label()
                ));
            }
        }
    }
    let extends = presets(&packs, &languages);

    // codelaw.toml: `extends` and nothing else (SR-CONFIG decision 8); its
    // text is templates/init/ (SR-LAW-10).
    let quoted: Vec<String> = extends.iter().map(|e| format!("\"{e}\"")).collect();
    let agents_section = if targets.is_empty() {
        String::new()
    } else {
        let names: Vec<String> = targets.iter().map(|t| format!("\"{}\"", t.name())).collect();
        match template::render("init/codelaw-agents.toml", &[("targets", &names.join(", "))]) {
            Ok(s) => s,
            Err(e) => return fail(e),
        }
    };
    let config = match template::render("init/codelaw.toml", &[("extends", &quoted.join(", ")), ("agents", &agents_section)]) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    let rules_dir = root.join(".codelaw").join("rules");
    // The words file is never overwritten: `spell learn` only appends to it.
    let words_file = root.join(codelaw_config::SPELL_WORDS_FILE);
    let spell = packs.contains("spell");
    let written = std::fs::write(root.join("codelaw.toml"), &config)
        .and_then(|_| std::fs::create_dir_all(&rules_dir))
        .and_then(|_| std::fs::write(rules_dir.join("README.md"), template::text("init/.codelaw/rules/README.md")))
        .and_then(|_| {
            if !spell || words_file.exists() {
                return Ok(());
            }
            std::fs::create_dir_all(words_file.parent().expect("a folder"))?;
            std::fs::write(&words_file, template::text("init/.codelaw/spell/words.txt"))
        })
        .and_then(|_| {
            let gi = root.join(".gitignore");
            let current = std::fs::read_to_string(&gi).unwrap_or_default();
            let line = template::text("init/gitignore");
            let have = |l: &str| l.trim() == line.trim() || l.trim() == line.trim().trim_end_matches('/');
            if current.lines().any(have) {
                return Ok(());
            }
            let sep = if current.is_empty() || current.ends_with('\n') { "" } else { "\n" };
            std::fs::write(&gi, format!("{current}{sep}{line}"))
        });
    if let Err(e) = written {
        return fail(e);
    }
    let langs: Vec<&str> = languages.iter().map(|l| l.name()).collect();
    println!("languages: {}", if langs.is_empty() { "none found".into() } else { langs.join(", ") });
    println!("wrote codelaw.toml, .codelaw/rules/README.md{} and the .gitignore line", if spell { ", .codelaw/spell/words.txt" } else { "" });
    for s in &skipped {
        println!("note: {s}");
    }

    // With `--agents`, the skills package too, at this binary's version: the
    // agent files are in it, not in codelaw (W-77, SR-AGENT-SURFACES decision 8).
    let skills_spec = match preferred("skills") {
        Registry::Npm => format!("{}@{}", codelaw_agents::package::NPM_NAME, env!("CARGO_PKG_VERSION")),
        Registry::Pypi => format!("{}=={}", codelaw_agents::package::PYPI_NAME, codelaw_agents::package::pep440(env!("CARGO_PKG_VERSION"))),
    };
    let mut install = by_registry.clone();
    if !targets.is_empty() {
        install.entry(preferred("skills")).or_default().push(skills_spec.as_str());
    }
    let commands: Vec<Vec<String>> = install.iter().map(|(r, pkgs)| install_command(&root, *r, pkgs)).collect();
    if args.no_install && !commands.is_empty() {
        println!("install the rule packs{} with:", if targets.is_empty() { "" } else { " and the skills package" });
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
    for line in fux_index_note(&root, &by_registry) {
        println!("{line}");
    }
    // Once the pack is in place: fill the words file (SR-SPELL decision 1).
    if spell {
        match crate::spell_cmd::learn(&root) {
            Ok(learned) => learned.lines().iter().for_each(|l| println!("{l}")),
            Err(e) => println!(
                "note: the words file was not filled ({}); run `codelaw spell learn` once the packs are installed",
                e.lines().next().unwrap_or_default().replace(&format!("{}/", codelaw_core::normalize_path(&root)), "")
            ),
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
            Err(e) => return fail(format!("the agent files were not written: {e}\nthen run `codelaw sync-agents`")),
        }
    }
    if let Some((p, w)) = ci {
        let written = match w {
            Some(w) => crate::ci::install(&root, p, w, args.unsafe_fixes),
            None => crate::ci::install_check(&root, p),
        };
        match written {
            Ok(lines) => lines.iter().for_each(|l| println!("{l}")),
            Err(e) => return fail(e),
        }
    }
    if let Some(hooks) = &hooks {
        return install_hooks(&root, hooks);
    }
    ExitCode::SUCCESS
}

fn install_hooks(root: &Path, hooks: &[&str]) -> ExitCode {
    match crate::git_hooks::install(root, hooks) {
        Ok(lines) => {
            for l in lines {
                println!("{l}");
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
    fn the_fux_index_note_needs_fux_and_names_each_chosen_pack() {
        let root = std::env::temp_dir().join(format!("codelaw-init-fux-note-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let mut by_registry: std::collections::BTreeMap<Registry, Vec<&str>> = Default::default();
        by_registry.entry(Registry::Npm).or_default().extend(["@codelaw/rules-js", "@codelaw/rules-ts"]);
        by_registry.entry(Registry::Pypi).or_default().push("codelaw-rules-python");
        assert!(fux_index_note(&root, &by_registry).is_empty(), "no .fux/, no note");
        std::fs::create_dir_all(root.join(".fux")).unwrap();
        let lines = fux_index_note(&root, &by_registry);
        assert!(lines.contains(&"  npm install -D @codelaw/rules-js-fux-index @codelaw/rules-ts-fux-index".to_string()), "{lines:?}");
        assert!(lines.contains(&"  pip install codelaw-rules-python-fux-index".to_string()), "{lines:?}");
        assert!(fux_index_note(&root, &Default::default()).is_empty(), "no pack chosen, no note");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn presets_follow_the_repo() {
        let langs: BTreeSet<Language> = [Language::Tsx, Language::Markdown, Language::Python].into();
        let pj = r#"{"dependencies":{"react":"^19"}}"#;
        let packs = packs_for(&langs, Some(pj));
        assert_eq!(packs.iter().copied().collect::<Vec<_>>(), ["data", "js", "python", "react", "ts"]);
        assert_eq!(
            presets(&packs, &langs),
            [
                "codelaw:data/recommended",
                "codelaw:md/recommended",
                "codelaw:js/recommended",
                "codelaw:js/typescript",
                "codelaw:py/recommended",
                "codelaw:react/recommended",
                "codelaw:ts/recommended"
            ]
        );
    }

    #[test]
    fn scss_files_or_sass_in_package_json_add_the_scss_pack_and_css() {
        let pj = r#"{"devDependencies":{"sass":"^1.80"}}"#;
        let from_files = packs_for(&[Language::Scss].into(), None);
        assert_eq!(from_files.iter().copied().collect::<Vec<_>>(), ["css", "scss"]);
        let from_dep = packs_for(&[Language::Javascript].into(), Some(pj));
        assert_eq!(from_dep.iter().copied().collect::<Vec<_>>(), ["css", "js", "scss"]);
        let langs: BTreeSet<Language> = [Language::Scss].into();
        assert_eq!(presets(&from_files, &langs), ["codelaw:css/recommended", "codelaw:scss/recommended"]);
    }

    #[test]
    fn module_stylesheets_add_the_css_modules_pack() {
        let dir = std::env::temp_dir().join(format!("codelaw-init-modules-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/a.css"), "a { color: red }\n").unwrap();
        assert!(!has_css_modules(&dir));
        std::fs::write(dir.join("src/Button.module.SCSS"), ".a { color: red }\n").unwrap();
        assert!(has_css_modules(&dir));
        let packs: BTreeSet<&str> = ["css", "css-modules"].into();
        assert_eq!(presets(&packs, &[Language::Css].into()), ["codelaw:css/recommended", "codelaw:css-modules/recommended"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn react_and_stencil_need_javascript() {
        let langs: BTreeSet<Language> = [Language::Python].into();
        let pj = r#"{"dependencies":{"react":"^19","@stencil/core":"^4"}}"#;
        assert_eq!(packs_for(&langs, Some(pj)).into_iter().collect::<Vec<_>>(), ["python"]);
    }

    #[test]
    fn library_packs_follow_package_json() {
        let langs: BTreeSet<Language> = [Language::Tsx].into();
        let pj = r#"{"dependencies":{"@tanstack/react-query":"^5","@reduxjs/toolkit":"^2"}}"#;
        let packs = packs_for(&langs, Some(pj));
        assert_eq!(packs.iter().copied().collect::<Vec<_>>(), ["js", "redux-toolkit", "tanstack-query", "ts"]);
        assert!(presets(&packs, &langs).contains(&"codelaw:tanstack-query/recommended".to_owned()));
        assert!(presets(&packs, &langs).contains(&"codelaw:redux-toolkit/recommended".to_owned()));
        let legacy = r#"{"devDependencies":{"react-query":"^3","redux":"^4"}}"#;
        assert_eq!(packs_for(&langs, Some(legacy)).into_iter().collect::<Vec<_>>(), ["js", "redux-toolkit", "tanstack-query", "ts"]);
        assert_eq!(packs_for(&[Language::Python].into(), Some(pj)).into_iter().collect::<Vec<_>>(), ["python"]);
    }
}
