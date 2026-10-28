//! W-105: type-aware TypeScript rules on `fixtures/types/` (SR-TYPES), run
//! through the binary and the engine with the TypeScript the fixture pins
//! (L11 §3 D-45). Each first rule fires once; an oxlint finding a rule ports
//! is shown once, as codelaw's; without TypeScript the type rules are
//! skipped with a note; a hook never loads a cold program but uses a warm
//! one.
//!
//! Needs `npm ci` in `fixtures/types` and Node on `PATH`; without them each
//! test says so and passes, unless `CODELAW_TYPES_REQUIRED=1` (CI).
#![cfg(unix)]

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The fixture's TypeScript, or `None` (with a note) when it or Node is
/// missing and CI does not require them.
fn typescript() -> Option<PathBuf> {
    let ts = repo().join("fixtures/types/node_modules/typescript");
    let node = std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join("node").is_file()));
    if ts.join("package.json").is_file() && node {
        return Some(fs::canonicalize(ts).unwrap());
    }
    assert!(
        std::env::var("CODELAW_TYPES_REQUIRED").as_deref() != Ok("1"),
        "CODELAW_TYPES_REQUIRED=1, but TypeScript (`npm ci` in fixtures/types) or Node is missing"
    );
    eprintln!("skipped: run `npm ci` in fixtures/types and put Node on PATH to run the type-aware tests");
    None
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if e.file_name() == "node_modules" {
            continue;
        }
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A copy of the fixture with the `ts` pack, and the fixture's TypeScript
/// linked in when `ts` is given.
fn project(name: &str, ts: Option<&Path>) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-types-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    copy_dir(&repo().join("fixtures/types"), &d);
    copy_dir(&repo().join("packs/ts"), &d.join("node_modules/@codelaw/rules-ts"));
    if let Some(ts) = ts {
        std::os::unix::fs::symlink(ts, d.join("node_modules/typescript")).unwrap();
    }
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_TYPESCRIPT")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap() + &String::from_utf8(out.stderr).unwrap())
}

fn json(out: &str) -> Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("{e}: {out}"))
}

fn found(v: &Value) -> Vec<String> {
    v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| format!("{}:{}:{} {}", d["file"].as_str().unwrap(), d["range"]["start"]["line"], d["range"]["start"]["column"], d["rule"].as_str().unwrap()))
        .collect()
}

const EXPECTED: &[&str] = &[
    "src/await.ts:2:10 ts/await-thenable",
    "src/error.ts:1:14 ts/type-error",
    "src/floating.ts:2:1 ts/no-floating-promises",
    "src/unsafe.ts:2:21 ts/no-unsafe-member-access",
    "src/unsafe.ts:3:1 ts/no-unsafe-call",
    "src/unsafe.ts:5:3 ts/no-unsafe-return",
];

/// A stand-in `oxlint` that prints, in oxlint-tsgolint's `--format=json`,
/// the type-aware findings typescript-eslint names for the fixture, and one
/// no `ts` rule ports.
fn fake_oxlint(d: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let at = |file: &str, needle: &str| -> (usize, usize) {
        let text = fs::read_to_string(d.join(file)).unwrap();
        (text.find(needle).unwrap(), needle.len())
    };
    let finding = |file: &str, needle: &str, rule: &str| {
        let (offset, length) = at(file, needle);
        format!(
            r#"{{"message": "m","code": "typescript({rule})","severity": "error","causes": [],"url": "https://oxc.rs/docs/guide/usage/linter/rules/typescript/{rule}.html","help": "h","filename": "{file}","labels": [{{"span": {{"offset": {offset},"length": {length}}}}}],"related": []}}"#
        )
    };
    let report = format!(
        r#"{{ "diagnostics": [{}, {}, {}, {}], "number_of_files": 4, "number_of_rules": 98, "threads_count": 1, "start_time": 0.1 }}"#,
        finding("src/floating.ts", "save();", "no-floating-promises"),
        finding("src/await.ts", "await 42", "await-thenable"),
        finding("src/unsafe.ts", "loose(1)", "no-unsafe-call"),
        finding("src/clean.ts", "n;", "no-unnecessary-type-assertion"),
    );
    fs::write(d.join("report.json"), report).unwrap();
    fs::write(d.join(".oxlintrc.json"), "{ \"options\": { \"typeAware\": true } }\n").unwrap();
    let oxlint = d.join("node_modules/.bin/oxlint");
    fs::create_dir_all(oxlint.parent().unwrap()).unwrap();
    fs::write(&oxlint, "#!/bin/sh\ncase \"$1\" in --version) echo 'Version: 1.89.0'; exit 0;; esac\ncat report.json\nexit 1\n").unwrap();
    fs::set_permissions(&oxlint, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn each_first_rule_fires_once_and_an_oxlint_port_is_shown_once() {
    let Some(ts) = typescript() else { return };
    let d = project("fixture", Some(&ts));
    fake_oxlint(&d);
    let (code, out) = codelaw(&d, &["check", "--format", "json", "--no-cache", "src"]);
    let v = json(&out);
    let mut got = found(&v);
    assert_eq!(code, 1, "{out}");
    // The one oxlint finding no `ts` rule ports passes through; the three
    // a type rule ports are shown once, as codelaw's.
    let adapter: Vec<String> = got.iter().filter(|f| f.contains("adapter/")).cloned().collect();
    assert_eq!(adapter, ["src/clean.ts:6:10 adapter/oxlint/typescript/no-unnecessary-type-assertion"], "{got:?}");
    got.retain(|f| !f.contains("adapter/"));
    assert_eq!(got, EXPECTED, "{out}");
    // The report names the TypeScript and the protocol (decision 12).
    let version = codelaw_types::package_version(&ts).unwrap();
    assert_eq!(v["types"]["typescript"], serde_json::json!([version]), "{}", v["types"]);
    assert_eq!(v["types"]["protocol"], codelaw_types::PROTOCOL);
    assert_eq!(v["types"]["notes"], serde_json::json!([]));
    // `ts/type-error` is TypeScript's own message, with its code.
    let error = v["diagnostics"].as_array().unwrap().iter().find(|d| d["rule"] == "ts/type-error").unwrap();
    assert!(error["message"].as_str().unwrap().starts_with("TS2322: "), "{error}");
    assert_eq!(error["fix"]["class"], "manual");
    // The text summary names it too.
    let (_, text) = codelaw(&d, &["check", "--format", "text", "--no-cache", "--no-adapters", "src"]);
    assert!(text.contains(&format!("· TypeScript {version}")), "{text}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_suggested_fix_is_checked_against_types_again() {
    let Some(ts) = typescript() else { return };
    let d = project("fix", Some(&ts));
    let (_, out) = codelaw(&d, &["fix", "--unsafe", "--format", "json", "--no-adapters", "src"]);
    let v = json(&out);
    assert_eq!(fs::read_to_string(d.join("src/floating.ts")).unwrap(), "export async function save(): Promise<void> {}\nvoid save();\n");
    assert_eq!(fs::read_to_string(d.join("src/await.ts")).unwrap(), "export async function main(): Promise<number> {\n  return 42;\n}\n");
    let left = found(&v);
    assert!(!left.iter().any(|f| f.contains("no-floating-promises") || f.contains("await-thenable")), "{left:?}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn without_typescript_the_type_rules_are_skipped_with_a_note() {
    if typescript().is_none() {
        return;
    }
    let d = project("none", None);
    let (code, out) = codelaw(&d, &["check", "--format", "json", "--no-adapters", "src"]);
    let v = json(&out);
    assert_eq!(code, 0, "{out}");
    assert_eq!(found(&v), Vec::<String>::new());
    let notes = v["types"]["notes"].as_array().unwrap();
    assert!(notes.iter().any(|n| n.as_str().unwrap().contains("no `typescript` package")), "{notes:?}");
    // The summary line says so too: never a silent gap (decision 6).
    let (_, text) = codelaw(&d, &["check", "--format", "text", "--no-adapters", "src"]);
    assert!(text.contains("the type rules were skipped: no `typescript` package"), "{text}");
    let _ = fs::remove_dir_all(&d);
}

/// L8 decision 8: a hook (`no_cold_types`) never starts a program; once a
/// warm one exists in the process (the daemon's), it answers the hook.
#[test]
fn a_hook_uses_a_warm_program_and_never_a_cold_one() {
    use codelaw_engine::{EngineOptions, Project};
    let Some(ts) = typescript() else { return };
    let d = project("hook", Some(&ts));
    let options = |no_cold_types: bool| EngineOptions { no_cache: true, no_cold_types, ..EngineOptions::from_env() };
    let types_of = |no_cold_types: bool| {
        let mut p = Project::load(&d, options(no_cold_types)).unwrap();
        let report = p.check(&[d.join("src")]).unwrap();
        let fired = report.diagnostics.iter().filter(|x| x.rule.starts_with("ts/")).count();
        (fired, report.types.unwrap())
    };
    let (fired, cold) = types_of(true);
    assert_eq!(fired, 0);
    assert!(cold.notes.iter().any(|n| n.contains("hooks use only a warm type program")), "{cold:?}");
    assert!(cold.typescript.is_empty());
    // A run that may load one does; the hook after it reuses it.
    let (fired, _) = types_of(false);
    assert_eq!(fired, EXPECTED.len());
    let (fired, warm) = types_of(true);
    assert_eq!(fired, EXPECTED.len(), "{warm:?}");
    assert!(warm.notes.is_empty(), "{warm:?}");
    // A warm program sees a change on disk.
    fs::write(d.join("src/error.ts"), "export const count: number = 1;\n").unwrap();
    let (fired, _) = types_of(true);
    assert_eq!(fired, EXPECTED.len() - 1);
    let _ = fs::remove_dir_all(&d);
}
