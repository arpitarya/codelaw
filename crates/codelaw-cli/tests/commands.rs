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
        fs::write(d.join("codelaw.toml"), "# mine\n[files]\nexclude = []\n").unwrap();
        let (code, out, err) = codelaw(&d, &["format", "--import-config"]);
        assert_eq!(code, 0, "{name}: {out}{err}");
        let o = read(&d);
        assert_eq!((o.line_width, o.quote_style, o.semicolons, o.trailing_commas), (100, QuoteStyle::Single, Semicolons::AsNeeded, TrailingCommas::Es5), "{name}");
        let written = fs::read_to_string(d.join("codelaw.toml")).unwrap();
        assert!(written.starts_with("# mine\n[files]\nexclude = []\n"), "{name}: {written}");
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
