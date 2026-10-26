//! SR-CONFIG decision 3a through the binary: `[scope.<name>]`, the scopes a
//! preset ships, the deprecated spellings and `config --scopes` (W-59).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-scopes-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() { copy_dir(&e.path(), &dest) } else { fs::copy(e.path(), dest).map(|_| ()).unwrap() }
    }
}

/// A project that has the first-party js pack installed.
fn with_js_pack(d: &Path) {
    let pack = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("packs/js");
    copy_dir(&pack, &d.join("node_modules/@codelaw/rules-js"));
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_AS_OF")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_PROFILE")
        .env_remove("CI")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn diagnostics(d: &Path) -> Vec<(String, String, String)> {
    let (_, out, err) = codelaw(d, &["check", "--format", "json", "--as-of", "2026-10-18"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| (x["file"].as_str().unwrap().to_owned(), x["rule"].as_str().unwrap().to_owned(), x["severity"].as_str().unwrap().to_owned()))
        .collect()
}

const CONSOLE: &str = "console.log(1);\n";

#[test]
fn a_project_scope_extends_the_preset_scope_of_its_name() {
    let d = temp("preset");
    with_js_pack(&d);
    fs::write(
        d.join("codelaw.toml"),
        "extends = [\"codelaw:js/recommended\"]\npreview = true\n\n[scope.tests]\nrules = { \"js/no-console\" = \"off\" }\n\n[scope.utils]\nfiles = [\"**/*.utils.js\"]\nrules = { \"js/no-console\" = \"error\" }\n",
    )
    .unwrap();
    for f in ["a.js", "a.test.js", "b.spec.js", "date.utils.js"] {
        fs::write(d.join(f), CONSOLE).unwrap();
    }
    let got: Vec<(String, String)> = diagnostics(&d).into_iter().filter(|x| x.1 == "js/no-console").map(|x| (x.0, x.2)).collect();
    assert_eq!(got, [("a.js".to_owned(), "warn".to_owned()), ("date.utils.js".to_owned(), "error".to_owned())], "the preset's test files stay, off by the project's scope");

    // DoD 4: preset and project scopes, in precedence order.
    let (code, out, err) = codelaw(&d, &["config", "--scopes"]);
    assert_eq!(code, 0, "{err}");
    let names: Vec<&str> = out.lines().filter(|l| !l.starts_with(' ')).map(|l| l.split(':').next().unwrap()).collect();
    assert_eq!(names, ["config", "stories", "tests", "utils"], "{out}");
    assert!(out.contains("tests: **/*.test.*, **/*.spec.*\n    codelaw.toml:4, extends codelaw:js/recommended"), "{out}");
    assert!(out.contains("stories: **/*.stories.*\n    codelaw:js/recommended"), "{out}");

    // `config --print` names the scope.
    let (_, out, _) = codelaw(&d, &["config", "--print", "--for", "a.test.js"]);
    assert!(out.lines().any(|l| l.starts_with("rules.\"js/no-console\" = \"off\"") && l.contains("scope tests")), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn the_old_spellings_load_with_a_warning_and_bad_names_are_exit_2() {
    let d = temp("alias");
    with_js_pack(&d);
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:js/recommended\"]\npreview = true\n\n[[overrides]]\nfiles = [\"scripts/**\"]\nrules = { \"js/no-console\" = \"off\" }\n\n[profiles.ci]\nrules = { \"js/no-console\" = \"error\" }\n").unwrap();
    fs::create_dir_all(d.join("scripts")).unwrap();
    fs::write(d.join("scripts/a.js"), CONSOLE).unwrap();
    fs::write(d.join("a.js"), CONSOLE).unwrap();
    let (code, out, err) = codelaw(&d, &["check", "--format", "json", "--as-of", "2026-10-18"]);
    assert_eq!(code, 0, "{out}{err}");
    let all = format!("{out}{err}");
    assert_eq!(all.matches("`[[overrides]]` is deprecated").count(), 1, "{all}");
    assert_eq!(all.matches("`[profiles.<name>]` is deprecated").count(), 1, "{all}");
    assert!(all.contains("[scope.<name>]") && all.contains("[profile.<name>]"), "{all}");
    let got: Vec<String> = diagnostics(&d).into_iter().filter(|x| x.1 == "js/no-console").map(|x| x.0).collect();
    assert_eq!(got, ["a.js"]);

    for bad in ["default", "a b"] {
        fs::write(d.join("codelaw.toml"), format!("[scope.\"{bad}\"]\nfiles = [\"a\"]\n")).unwrap();
        let (code, _, err) = codelaw(&d, &["check"]);
        assert_eq!(code, 2, "{bad}: {err}");
        assert!(err.contains("scope name"), "{bad}: {err}");
    }
    let _ = fs::remove_dir_all(&d);
}
