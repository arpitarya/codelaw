//! The commands beyond check and fix, through the binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-cmd-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

/// SR-FORMATTER decision 5: `--import-config` writes the `[format]` keys
/// once, for each source format, and `[format]` then reads them back.
#[test]
fn import_config_round_trips_each_source() {
    use codelaw_format::{FormatOptions, IndentStyle, QuoteStyle, Semicolons, TrailingCommas};
    let read = |d: &Path| {
        let t: toml::Table = toml::from_str(&fs::read_to_string(d.join("codelaw.toml")).unwrap()).unwrap();
        FormatOptions::from_table(t.get("format").and_then(|f| f.as_table())).unwrap()
    };
    let sources: &[(&str, &str, &str)] = &[
        ("json", ".prettierrc", r#"{"printWidth": 100, "singleQuote": true, "semi": false, "trailingComma": "es5"}"#),
        ("yaml", ".prettierrc.yaml", "printWidth: 100\nsingleQuote: true\nsemi: false\ntrailingComma: es5\n"),
        ("toml", ".prettierrc.toml", "printWidth = 100\nsingleQuote = true\nsemi = false\ntrailingComma = \"es5\"\n"),
        ("pkg", "package.json", r#"{"name": "x", "prettier": {"printWidth": 100, "singleQuote": true, "semi": false, "trailingComma": "es5"}}"#),
    ];
    for (name, file, text) in sources {
        let d = temp(&format!("import-{name}"));
        fs::write(d.join(file), text).unwrap();
        fs::write(d.join("codelaw.toml"), "# mine\n[files]\nfollow_symlinks = false\n").unwrap();
        let (code, out, err) = codelaw(&d, &["format", "--import-config"]);
        assert_eq!(code, 0, "{name}: {out}{err}");
        let o = read(&d);
        assert_eq!((o.line_width, o.quote_style, o.semicolons, o.trailing_commas), (100, QuoteStyle::Single, Semicolons::AsNeeded, TrailingCommas::Es5), "{name}");
        let written = fs::read_to_string(d.join("codelaw.toml")).unwrap();
        assert!(written.starts_with("# mine\n[files]\nfollow_symlinks = false\n"), "{name}: {written}");
        // Once: a second run writes nothing.
        let (code, out, _) = codelaw(&d, &["format", "--import-config"]);
        assert_eq!(code, 0);
        assert!(!out.contains("wrote") && out.contains("kept [format] line_width"), "{name}: {out}");
        assert_eq!(fs::read_to_string(d.join("codelaw.toml")).unwrap(), written);
        let _ = fs::remove_dir_all(&d);
    }
    // Ruff and Black, into `[format.python]`; codelaw.toml's own key wins.
    for (name, text, width) in [("ruff", "[tool.ruff]\nline-length = 100\nindent-width = 2\n", 100), ("black", "[tool.black]\nline-length = 120\n", 120)] {
        let d = temp(&format!("import-{name}"));
        fs::write(d.join("pyproject.toml"), text).unwrap();
        fs::write(d.join("codelaw.toml"), "[format]\nindent_style = \"tab\"\n").unwrap();
        fs::write(d.join(".prettierrc"), r#"{"useTabs": false}"#).unwrap();
        let (code, out, err) = codelaw(&d, &["format", "--import-config"]);
        assert_eq!(code, 0, "{name}: {out}{err}");
        let o = read(&d);
        assert_eq!((o.python_line_width, o.indent_style), (width, IndentStyle::Tab), "{name}: {out}");
        if name == "ruff" {
            assert_eq!(o.python_indent_width, 2);
        }
        let _ = fs::remove_dir_all(&d);
    }
}

#[test]
fn init_writes_config_rules_folder_and_gitignore() {
    let d = temp("init");
    fs::write(d.join("package.json"), r#"{"name":"x","devDependencies":{"react":"19"}}"#).unwrap();
    fs::write(d.join("pnpm-lock.yaml"), "").unwrap();
    fs::write(d.join("pyproject.toml"), "[project]\nname = \"x\"\n").unwrap();
    fs::write(d.join("uv.lock"), "").unwrap();
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/app.tsx"), "export const A = () => <p />;\n").unwrap();
    fs::write(d.join("src/tool.py"), "x = 1\n").unwrap();
    fs::write(d.join(".gitignore"), "dist/").unwrap();
    let (code, out, err) = codelaw(&d, &["init", "--no-install"]);
    assert_eq!(code, 0, "{out}{err}");
    let config = fs::read_to_string(d.join("codelaw.toml")).unwrap();
    assert!(config.contains(r#"extends = ["codelaw:data/recommended", "codelaw:js/recommended", "codelaw:ts/recommended", "codelaw:react/recommended", "codelaw:py/recommended"]"#), "{config}");
    assert!(out.contains("pnpm add -D @codelaw/rules-data @codelaw/rules-js"), "{out}");
    assert!(out.contains("uv add --dev codelaw-rules-python"), "{out}");
    assert_eq!(fs::read_to_string(d.join(".gitignore")).unwrap(), "dist/\n.codelaw/cache/\n");
    assert!(d.join(".codelaw/rules/README.md").is_file());
    // A second init refuses.
    let (code, _, err) = codelaw(&d, &["init", "--no-install"]);
    assert_eq!(code, 2);
    assert!(err.contains("already configures codelaw"), "{err}");
    // --registry pypi puts every pack on PyPI.
    fs::remove_file(d.join("codelaw.toml")).unwrap();
    let (_, out, _) = codelaw(&d, &["init", "--no-install", "--registry", "pypi"]);
    assert!(out.contains("uv add --dev codelaw-rules-data codelaw-rules-js codelaw-rules-python"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn new_rule_scaffold_passes_rules_lint_and_fails_test() {
    let d = temp("newrule");
    let (code, out, err) = codelaw(&d, &["new-rule", "acme/no-foo", "--lang", "typescript,python", "--fix", "manual"]);
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = codelaw(&d, &["rules", "lint"]);
    assert_eq!(code, 0, "a fresh scaffold passes rules lint (SR-RULE-FORMAT veto): {out}{err}");
    assert!(out.contains("1 rule files · 0 problems"), "{out}");
    // The placeholder Don't (`TODO`) is what the placeholder matcher finds,
    // so the scaffold's own examples pass until the author replaces them.
    let (code, out, _) = codelaw(&d, &["test"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("✔ acme/no-foo"), "{out}");
    let (code, _, err) = codelaw(&d, &["new-rule", "acme/no-foo", "--lang", "python"]);
    assert_eq!(code, 2, "{err}");
    // W-17: an id cannot walk out of the rules folder.
    for bad in ["../x", "acme/..", "../../etc/x"] {
        let (code, _, err) = codelaw(&d, &["new-rule", bad, "--lang", "python"]);
        assert_eq!(code, 2, "{bad}: {err}");
    }
    assert!(!d.join(".codelaw/x.md").exists() && !d.join("x").exists());
    let _ = fs::remove_dir_all(&d);
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// W-17 row 3: with no `target_version`, `strict=` may not exist on the
/// code's Python, so `codelaw fix` must not add it; `--unsafe` may.
#[test]
fn strict_keyword_fixes_are_suggestions() {
    let d = temp("strict");
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("packs/python");
    copy_dir(&packs, &d.join("node_modules/@codelaw/rules-python"));
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:py/recommended\"]\npreview = true\n").unwrap();
    let src = "def f(g, a, b):\n    return list(zip(a, b)), list(map(g, a, b))\n";
    fs::write(d.join("a.py"), src).unwrap();
    let rules = "py/zip-without-explicit-strict,py/map-without-explicit-strict";
    let (_, out, err) = codelaw(&d, &["fix", "--rule", rules]);
    assert_eq!(fs::read_to_string(d.join("a.py")).unwrap(), src, "{out}{err}");
    assert!(out.contains("py/zip-without-explicit-strict") && out.contains("py/map-without-explicit-strict"), "still reported: {out}{err}");
    let (_, out, err) = codelaw(&d, &["fix", "--unsafe", "--rule", rules]);
    assert_eq!(fs::read_to_string(d.join("a.py")).unwrap(), "def f(g, a, b):\n    return list(zip(a, b, strict=False)), list(map(g, a, b, strict=False))\n", "{out}{err}");
    let _ = fs::remove_dir_all(&d);
}

/// W-17 row 1, the panic fixture: a `warn` rule that panics still makes
/// `check` and `fix` exit 2, after every other result is printed
/// (SR-ARCHITECTURE decision 25).
#[test]
fn a_rule_panic_exits_2() {
    let d = temp("panic");
    let pack = d.join("node_modules/@codelaw/rules-js");
    fs::create_dir_all(pack.join("rules/js")).unwrap();
    fs::create_dir_all(pack.join("presets/js")).unwrap();
    fs::write(pack.join("codelaw-pack.toml"), "name = \"@codelaw/rules-js\"\nversion = \"0.1.0\"\nmin_codelaw = \"0.1.0\"\nlanguages = [\"javascript\"]\npresets = [\"js/recommended\"]\n").unwrap();
    fs::write(
        pack.join("rules/js/test-panic.md"),
        "---\nid: js/test-panic\ntitle: panics\nlanguages: [javascript]\nfix: manual\nfix_reason: test\nstability: stable\n---\n\nSummary.\n\n**Why:** test.\n\n```codelaw\nnative: test/panic\n```\n\n## How to fix\n\n1. Nothing.\n\n## Examples\n\n### JavaScript\n\n#### Don't\n\n```js\nx\n```\n\n#### Do\n\n```js\ny\n```\n",
    )
    .unwrap();
    fs::write(pack.join("presets/js/recommended.toml"), "[rules]\n\"js/test-panic\" = \"warn\"\n").unwrap();
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:js/recommended\"]\n").unwrap();
    fs::write(d.join("a.js"), "let a = 1;\n").unwrap();
    for cmd in ["check", "fix"] {
        let (code, out, err) = codelaw(&d, &[cmd]);
        assert_eq!(code, 2, "{cmd}: {out}{err}");
        assert!(out.contains("codelaw/internal-error") && out.contains("1 rule run failed"), "{cmd}: {out}");
        let (code, out, _) = codelaw(&d, &[cmd, "--format", "json"]);
        assert_eq!(code, 2);
        assert!(out.contains("\"internal_errors\": 1"), "{out}");
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn explain_tree_and_rules_why() {
    let d = temp("explain");
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("fixtures/l3/rules/fixture/py-no-print.md");
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    fs::copy(rules, d.join(".codelaw/rules/fixture/py-no-print.md")).unwrap();
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    let (code, out, _) = codelaw(&d, &["explain", "fixture/py-no-print"]);
    assert_eq!(code, 0);
    assert!(out.contains("How to fix:\n  1. Change it by hand."), "{out}");
    let (code, out, _) = codelaw(&d, &["explain", "fixture/py-no-prnt"]);
    assert_eq!(code, 2, "{out}");
    let (_, out, err) = codelaw(&d, &["explain", "fixture/py-no-prnt"]);
    assert!(err.contains("did you mean `fixture/py-no-print`?"), "{out}{err}");
    let (code, out, _) = codelaw(&d, &["tree", "a.py"]);
    assert_eq!(code, 0);
    assert!(out.contains("call  [1:1-1:9]") && out.contains("function: identifier  [1:1-1:6]  \"print\""), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--why", "fixture/py-no-print", "--for", "a.py"]);
    assert!(out.contains("source:     local") && out.contains("for a.py:  error"), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--for", "a.py"]);
    assert!(out.starts_with("fixture/py-no-print"), "{out}");
    let (code, out, _) = codelaw(&d, &["check", "--format", "agent"]);
    assert_eq!(code, 1);
    assert!(out.starts_with("a.py:1:1 fixture/py-no-print print(1) in library code — fix: manual — Change it by hand."), "{out}");
    let _ = fs::remove_dir_all(&d);
}

/// Every file under `dir`, relative to `root`, sorted; `node_modules` aside.
fn all_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "node_modules") {
                continue;
            }
            all_files(root, &p, out);
        } else {
            out.push(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
    out.sort();
}

fn tree(root: &Path) -> Vec<String> {
    let mut v = Vec::new();
    all_files(root, root, &mut v);
    v
}

/// SR-CONSUMER-LAYOUT decisions 4, 5 and 8 with SR-AGENT-SURFACES
/// decisions 1 and 2: `init --agents` writes only the tree's paths,
/// `sync-agents --check` fails on drift inside the markers and on nothing
/// outside them, and with no agents no agent file exists.
#[test]
fn init_agents_writes_the_layout_and_sync_keeps_it() {
    let d = temp("agents");
    fs::write(d.join("package.json"), r#"{"name":"x"}"#).unwrap();
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/a.ts"), "export const a = 1;\n").unwrap();
    fs::write(d.join("CLAUDE.md"), "# Team\n\nOur notes.\n").unwrap();
    // The packs `npm install` would add (`--no-install` here).
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("packs");
    copy_dir(&packs.join("js"), &d.join("node_modules/@codelaw/rules-js"));
    copy_dir(&packs.join("data"), &d.join("node_modules/@codelaw/rules-data"));
    let before = tree(&d);
    let (code, out, err) = codelaw(&d, &["init", "--no-install", "--agents", "kiro,claude,cowork,copilot,codex"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("~/.codex/config.toml") && out.contains("[mcp_servers.codelaw]"), "Codex's user config is printed: {out}");
    let config = fs::read_to_string(d.join("codelaw.toml")).unwrap();
    assert!(config.ends_with("\n[agents]\ntargets = [\"claude\", \"cowork\", \"copilot\", \"codex\", \"kiro\"]\n"), "{config}");
    // SR-CONSUMER-LAYOUT §1: the paths init may add.
    let allowed = ["codelaw.toml", ".gitignore", "AGENTS.md", "CLAUDE.md", ".mcp.json", ".vscode/mcp.json", ".codelaw/", ".claude/", ".kiro/", ".github/", ".agents/"];
    for f in tree(&d).iter().filter(|f| !before.contains(f)) {
        assert!(allowed.iter().any(|a| f == a || (a.ends_with('/') && f.starts_with(a))), "init wrote {f}, which SR-CONSUMER-LAYOUT §1 does not list");
    }
    for skills in [".claude/skills", ".github/skills", ".agents/skills", ".kiro/skills", ".claude/plugins/codelaw/skills"] {
        assert!(d.join(skills).join("codelaw-fix/SKILL.md").is_file(), "{skills}");
    }
    for f in [
        ".claude/settings.json",
        ".claude/agents/codelaw-rule-author.md",
        ".github/agents/codelaw-rule-author.agent.md",
        ".github/hooks/codelaw.json",
        ".vscode/mcp.json",
        ".kiro/steering/codelaw.md",
        ".kiro/hooks/codelaw.json",
        ".kiro/settings/mcp.json",
        ".mcp.json",
        "AGENTS.md",
    ] {
        assert!(d.join(f).is_file(), "{f}");
    }
    let hook = fs::read_to_string(d.join(".claude/settings.json")).unwrap();
    assert!(hook.contains("\"npx --no-install codelaw hook\""), "beside a package.json the hook runs the repo's install: {hook}");

    let (code, out, _) = codelaw(&d, &["sync-agents", "--check"]);
    assert_eq!(code, 0, "init's files are what sync-agents writes: {out}");
    // The team's text outside the markers is theirs.
    let claude = fs::read_to_string(d.join("CLAUDE.md")).unwrap();
    assert!(claude.starts_with("# Team\n\nOur notes.\n\n<!-- codelaw:begin -->\n"), "{claude}");
    let claude = format!("{claude}\nMore team text.\n").replace("Our notes.", "Our notes, edited.");
    fs::write(d.join("CLAUDE.md"), &claude).unwrap();
    let (code, out, _) = codelaw(&d, &["sync-agents", "--check"]);
    assert_eq!(code, 0, "{out}");
    // An edit inside the markers, or to a generated file, is drift.
    fs::write(d.join("CLAUDE.md"), claude.replace("Ask before you write.", "Write first.")).unwrap();
    fs::write(d.join(".github/skills/codelaw-fix/SKILL.md"), "edited").unwrap();
    let (code, out, _) = codelaw(&d, &["sync-agents", "--check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("out of date: CLAUDE.md") && out.contains("out of date: .github/skills/codelaw-fix/SKILL.md"), "{out}");
    let (code, out, err) = codelaw(&d, &["sync-agents"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(fs::read_to_string(d.join("CLAUDE.md")).unwrap(), claude, "sync restores the block and keeps the team's edits");
    assert_eq!(codelaw(&d, &["sync-agents", "--check"]).0, 0);
    // A stale codelaw skill is removed; the team's own skill stays.
    fs::create_dir_all(d.join(".claude/skills/codelaw-gone")).unwrap();
    fs::write(d.join(".claude/skills/codelaw-gone/SKILL.md"), "old").unwrap();
    fs::create_dir_all(d.join(".claude/skills/team-skill")).unwrap();
    fs::write(d.join(".claude/skills/team-skill/SKILL.md"), "ours").unwrap();
    assert_eq!(codelaw(&d, &["sync-agents", "--check"]).0, 1);
    assert_eq!(codelaw(&d, &["sync-agents"]).0, 0);
    assert!(!d.join(".claude/skills/codelaw-gone").exists() && d.join(".claude/skills/team-skill/SKILL.md").is_file());
    // An unknown agent names the nearest one, in init and in the config.
    let (code, _, err) = codelaw(&d, &["init", "--agents", "claud"]);
    assert_eq!(code, 2);
    assert!(err.contains("did you mean `claude`?"), "{err}");
    fs::write(d.join("codelaw.toml"), "[agents]\ntargets = [\"copilt\"]\n").unwrap();
    let (code, _, err) = codelaw(&d, &["sync-agents"]);
    assert_eq!(code, 2);
    assert!(err.contains("did you mean `copilot`?"), "{err}");
    fs::write(d.join("codelaw.toml"), "[agents]\ntarget = [\"claude\"]\n").unwrap();
    assert_eq!(codelaw(&d, &["sync-agents"]).0, 2, "an unknown key in [agents] is a config error");
    let _ = fs::remove_dir_all(&d);

    // L5 decision 4: no `--agents`, no agent file.
    let d = temp("no-agents");
    fs::write(d.join("a.md"), "# A\n").unwrap();
    assert_eq!(codelaw(&d, &["init", "--no-install"]).0, 0);
    assert_eq!(tree(&d), [".codelaw/rules/README.md", ".gitignore", "a.md", "codelaw.toml"]);
    let (code, out, _) = codelaw(&d, &["sync-agents", "--check"]);
    assert_eq!(code, 0, "{out}");
    let _ = fs::remove_dir_all(&d);
}

/// SR-AGENT-SURFACES decision 6: the hook reads the agent's payload, fixes
/// what is `auto`, and sends what is left back with exit 2.
#[test]
fn hook_fixes_the_edited_file_and_returns_what_is_left() {
    use std::io::Write;
    let d = temp("hook");
    let pack = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("packs/js");
    copy_dir(&pack, &d.join("node_modules/@codelaw/rules-js"));
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:js/recommended\"]\npreview = true\n").unwrap();
    fs::write(d.join("a.js"), "let a   = 1\nfor (let i = 0; i < 10; i--) { a++; }\n").unwrap();
    fs::write(d.join("b.js"), "export const b = 1;\n").unwrap();
    let payload = format!(r#"{{"cwd":"{}","tool_name":"Write","tool_input":{{"file_path":"a.js"}}}}"#, d.display());
    let mut child = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .arg("hook")
        .current_dir(&d)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(payload.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    let err = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(err.contains("codelaw auto-fixed a.js: format/unformatted ×1"), "{err}");
    assert!(err.contains("a.js:2:1 error js/for-direction") && err.contains("fix: manual — 1."), "{err}");
    assert!(err.contains("never suppress or disable a rule"), "{err}");
    let fixed = fs::read_to_string(d.join("a.js")).unwrap();
    assert!(fixed.starts_with("let a = 1;\n"), "{fixed}");
    // The same bytes `codelaw fix` writes (L5 decision 5).
    fs::write(d.join("c.js"), "let a   = 1\nfor (let i = 0; i < 10; i--) { a++; }\n").unwrap();
    codelaw(&d, &["fix", "c.js"]);
    assert_eq!(fs::read_to_string(d.join("c.js")).unwrap(), fixed);
    // A clean file: exit 0, nothing to say; a file codelaw does not check: exit 0.
    let (code, out, err) = codelaw(&d, &["hook", "b.js"]);
    assert_eq!((code, out.as_str(), err.as_str()), (0, "", ""));
    fs::write(d.join("x.rs"), "fn main() {}\n").unwrap();
    assert_eq!(codelaw(&d, &["hook", "x.rs"]).0, 0);
    let _ = fs::remove_dir_all(&d);
}
