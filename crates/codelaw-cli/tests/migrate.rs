//! `codelaw migrate` through the binary (SR-CLI-MCP decision 15, W-94): on
//! the three repos of `fixtures/migrate/`, the written config makes `check`
//! report what the source tool reports on the rules codelaw ports, the
//! report names every rule that did not map, the same input writes the
//! same bytes, and no config that is a program is run.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-migrate-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("{}: {e}", to.display()));
    for e in fs::read_dir(from).unwrap_or_else(|e| panic!("{}: {e}", from.display())).flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() { copy_dir(&e.path(), &dest) } else { fs::copy(e.path(), dest).map(|_| ()).unwrap() }
    }
}

/// A copy of one fixture with the js pack (npm) and the python pack (a
/// virtualenv's site-packages) installed.
fn fixture(name: &str, copy: &str) -> PathBuf {
    let d = temp(copy);
    copy_dir(&repo().join("fixtures/migrate").join(name), &d);
    copy_dir(&repo().join("packs/js"), &d.join("node_modules/@codelaw/rules-js"));
    copy_dir(&repo().join("packs/python"), &d.join(".venv/lib/python3.12/site-packages/codelaw_rules_python"));
    d
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_SITE_PACKAGES")
        .env_remove("CODELAW_AS_OF")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_PROFILE")
        .env_remove("CI")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

/// `file:line codelaw-rule` of every lint finding (`format/*` is the formatter's).
fn findings(d: &Path) -> BTreeSet<String> {
    let (code, out, err) = codelaw(d, &["check", "--format", "json", "--as-of", "2026-10-27", "."]);
    assert!(code == 0 || code == 1, "{err}{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| !x["rule"].as_str().unwrap().starts_with("format/"))
        .map(|x| format!("{}:{} {}", x["file"].as_str().unwrap(), x["range"]["start"]["line"], x["rule"].as_str().unwrap()))
        .collect()
}

/// `expected.txt`'s lines as `file:line codelaw-rule`.
fn expected(name: &str) -> BTreeSet<String> {
    let text = fs::read_to_string(repo().join("fixtures/migrate").join(name).join("expected.txt")).unwrap();
    text.lines().filter(|l| !l.trim().is_empty()).map(|l| {
        let parts: Vec<&str> = l.split_whitespace().collect();
        format!("{} {}", parts[0], parts[2])
    }).collect()
}

fn unmapped(report: &str) -> BTreeSet<String> {
    let v: serde_json::Value = serde_json::from_str(report).unwrap_or_else(|e| panic!("{e}: {report}"));
    v["unmapped"].as_array().unwrap().iter().map(|l| l["source"].as_str().unwrap().to_owned()).collect()
}

fn expected_unmapped(name: &str) -> BTreeSet<String> {
    fs::read_to_string(repo().join("fixtures/migrate").join(name).join("unmapped.txt")).unwrap().lines().filter(|l| !l.is_empty()).map(str::to_owned).collect()
}

/// DoD 1 and 2 on one fixture: the findings, the unmapped rules, and the
/// same bytes from the same input.
fn migrates(name: &str, tools: &[&str]) -> PathBuf {
    let d = fixture(name, name);
    let (code, dry, err) = codelaw(&d, &["migrate", tools[0], "--format", "json"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(unmapped(&dry), expected_unmapped(name), "{name}: every rule that did not map is named");
    assert!(!d.join("codelaw.toml").exists(), "without --write nothing is written");
    for t in tools {
        let (code, out, err) = codelaw(&d, &["migrate", t, "--write"]);
        assert_eq!(code, 0, "{t}: {out}{err}");
    }
    let first = fs::read_to_string(d.join("codelaw.toml")).unwrap();
    assert_eq!(findings(&d), expected(name), "{name}: codelaw.toml:\n{first}");

    // The same input writes the same bytes (L5).
    let again = fixture(name, &format!("{name}-again"));
    for t in tools {
        codelaw(&again, &["migrate", t, "--write"]);
    }
    assert_eq!(fs::read_to_string(again.join("codelaw.toml")).unwrap(), first, "{name}: byte-identical");
    let _ = fs::remove_dir_all(&again);
    d
}

#[test]
fn an_eslint_flat_and_prettier_repo() {
    let d = migrates("eslint-prettier", &["eslint", "prettier"]);
    let config = fs::read_to_string(d.join("codelaw.toml")).unwrap();
    assert!(config.contains("extends = [\"codelaw:js/recommended\"]"), "{config}");
    assert!(config.contains("\"js/no-console\" = { severity = \"warn\", options = { allow = [\"error\"] } }"), "options carried: {config}");
    assert!(config.contains("[scope.eslint-1]\nfiles = [\"**/*.test.js\"]"), "{config}");
    assert!(config.contains("ignore_aliases = [\"eslint\"]"), "{config}");
    // The second tool added to the first's config.
    assert!(config.contains("[format]\nline_width = 100\nquote_style = \"single\""), "{config}");
    let (_, out, _) = codelaw(&d, &["migrate", "prettier", "--write"]);
    assert!(out.contains("kept      [format] line_width: codelaw.toml already sets it"), "{out}");
    assert!(out.contains("ignored   .prettierrc: proseWrap = \"always\""), "{out}");
}

#[test]
fn a_biome_repo() {
    let d = migrates("biome", &["biome"]);
    let config = fs::read_to_string(d.join("codelaw.toml")).unwrap();
    assert!(config.contains("ignore_aliases = [\"biome\"]") && config.contains("indent_width = 2"), "{config}");
    let (_, out, _) = codelaw(&d, &["migrate", "biome"]);
    assert!(out.contains("unmapped  biome/noAccumulatingSpread (Biome names no ESLint"), "{out}");
}

#[test]
fn a_ruff_pyproject_repo() {
    let d = migrates("ruff", &["ruff"]);
    let config = fs::read_to_string(d.join("codelaw.toml")).unwrap();
    assert!(config.contains("[scope.ruff-1]\nfiles = [\"**/__init__.py\"]\nrules = { \"py/unused-import\" = \"off\" }"), "{config}");
    assert!(config.contains("[format.python]\nline_width = 100"), "{config}");
    assert!(!config.contains("graph/"), "no whole-repo preset is pulled in: {config}");
}

#[test]
fn a_config_that_is_a_program_is_never_run() {
    let d = temp("program");
    fs::write(d.join("eslint.config.js"), "import base from './base.js';\nexport default makeConfig(base);\n").unwrap();
    let (code, _, err) = codelaw(&d, &["migrate", "eslint"]);
    assert_eq!(code, 2);
    assert!(err.contains("convert it to JSON first, or list the rules by hand"), "{err}");
    fs::remove_file(d.join("eslint.config.js")).unwrap();
    fs::write(d.join(".eslintrc.js"), "module.exports = { rules: {} };\n").unwrap();
    let (code, _, err) = codelaw(&d, &["migrate", "eslint"]);
    assert_eq!(code, 2);
    assert!(err.contains(".eslintrc.js is a program") && err.contains("convert it to JSON first"), "{err}");
    let (code, _, err) = codelaw(&d, &["migrate", "stylelint"]);
    assert_eq!(code, 2);
    assert!(err.contains("no stylelint config here"), "{err}");
}

#[test]
fn ignore_aliases_name_only_known_tools() {
    let d = temp("aliases");
    fs::write(d.join("codelaw.toml"), "extends = []\n[lint]\nignore_aliases = [\"tslint\"]\n").unwrap();
    fs::write(d.join("a.js"), "a();\n").unwrap();
    let (code, _, err) = codelaw(&d, &["check", "."]);
    assert_eq!(code, 2);
    assert!(err.contains("`[lint] ignore_aliases`: `tslint` is not one of `eslint`, `ruff`, `biome`"), "{err}");
}
