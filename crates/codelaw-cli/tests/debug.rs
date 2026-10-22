//! The W-16 surface through the binary: `config --print/--schema`,
//! `rules --for --format json`, `match --explain`, `fix --trace`,
//! `--timings` and `--format sarif` (validated against the SARIF 2.1.0
//! schema, vendored in `crates/codelaw-report/tests/`).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-debug-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_SITE_PACKAGES")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn json(cwd: &Path, args: &[&str]) -> Value {
    let (_, out, err) = codelaw(cwd, args);
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}\n{err}"))
}

/// A repo with two fixture rules: one `auto` (JS), one `manual` (Python).
fn project(name: &str) -> PathBuf {
    let d = temp(name);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    for r in ["js-eqeqeq.md", "py-no-print.md"] {
        fs::copy(repo().join("fixtures/l3/rules/fixture").join(r), d.join(".codelaw/rules/fixture").join(r)).unwrap();
    }
    fs::write(d.join("codelaw.toml"), "[rules]\n\"fixture/py-no-print\" = \"warn\"\n\n[[overrides]]\nfiles = [\"scripts/**\"]\nrules = { \"fixture/py-no-print\" = \"off\" }\n").unwrap();
    fs::create_dir_all(d.join("scripts")).unwrap();
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    fs::write(d.join("scripts/b.py"), "print(2)\n").unwrap();
    fs::write(d.join("a.js"), "if (a == b) { f(c == d); }\n").unwrap();
    d
}

#[test]
fn config_print_names_every_source_and_schema_is_the_shipped_one() {
    let d = project("config");
    let (code, out, err) = codelaw(&d, &["config", "--print"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("# config files: codelaw.toml\n"), "{out}");
    assert!(out.lines().any(|l| l.starts_with("cache.max_size = 512000000") && l.ends_with("# default")), "{out}");
    assert!(out.lines().any(|l| l.starts_with("rules.\"fixture/py-no-print\" = \"warn\"") && l.ends_with("# codelaw.toml:2")), "{out}");
    let v = json(&d, &["config", "--print", "--for", "scripts/b.py", "--format", "json"]);
    let rule = v["values"].as_array().unwrap().iter().find(|x| x["key"] == "rules.\"fixture/py-no-print\"").unwrap();
    assert_eq!(rule["value"], "off");
    assert_eq!(rule["source"], "codelaw.toml:6 override #1");
    let js = v["values"].as_array().unwrap().iter().find(|x| x["key"] == "rules.\"fixture/js-eqeqeq\"").unwrap();
    assert_eq!(js["source"], "preset .codelaw/rules", "{js}");
    let (code, out, _) = codelaw(&d, &["config", "--schema"]);
    assert_eq!(code, 0);
    assert_eq!(out, codelaw_config::SCHEMA);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rules_for_json_is_the_api_shape() {
    let d = project("rules");
    let v = json(&d, &["rules", "--for", "a.py", "--format", "json"]);
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["file"], "a.py");
    let rules = v["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 1, "{v}");
    assert_eq!(rules[0]["id"], "fixture/py-no-print");
    assert_eq!(rules[0]["severity"], "warn");
    assert_eq!(rules[0]["fix"], "manual");
    let v = json(&d, &["rules", "--for", "scripts/b.py", "--format", "json"]);
    assert_eq!(v["rules"].as_array().unwrap().len(), 0, "off by override: {v}");
    let v = json(&d, &["rules", "--format", "json"]);
    assert_eq!(v["rules"].as_array().unwrap().len(), 2);
    let (_, out, _) = codelaw(&d, &["rules", "--for", "a.py"]);
    assert!(out.starts_with("fixture/py-no-print"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn match_explain_names_each_clause() {
    let d = project("match");
    let (code, out, err) = codelaw(&d, &["match", "fixture/js-eqeqeq", "a.js", "--explain"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("a.js:1:5-1:11  binary_expression  match\n  ✔ pattern: \"$A == $B\"\n  $A = \"a\"\n  $B = \"b\""), "{out}");
    assert!(out.ends_with("fixture/js-eqeqeq on a.js: 2 matches among 2 candidate nodes\n"), "{out}");
    let v = json(&d, &["match", "fixture/js-eqeqeq", "a.js", "--format", "json", "--line", "1"]);
    assert_eq!(v["nodes"][1]["captures"]["$A"], "c");
    let (code, _, err) = codelaw(&d, &["match", "fixture/js-eqeqq", "a.js"]);
    assert_eq!(code, 2);
    assert!(err.contains("did you mean `fixture/js-eqeqeq`?"), "{err}");
    let (code, _, err) = codelaw(&d, &["match", "fixture/js-eqeqeq", "a.py"]);
    assert_eq!(code, 2);
    assert!(err.contains("does not apply to python"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn fix_trace_shows_each_pass_and_writes_only_without_dry_run() {
    let d = project("trace");
    let (code, out, err) = codelaw(&d, &["fix", "--dry-run", "--trace", "a.js"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("trace a.js\n  pass 1\n    offered   fixture/js-eqeqeq (1 edits)\n"), "{out}");
    assert!(out.contains("    applied   fixture/js-eqeqeq (1 edits), verified\n"), "{out}");
    assert!(out.contains("  fixpoint: nothing more to fix\n"), "{out}");
    assert_eq!(fs::read_to_string(d.join("a.js")).unwrap(), "if (a == b) { f(c == d); }\n", "--dry-run writes nothing");
    let v = json(&d, &["fix", "--trace", "--format", "json", "a.js"]);
    assert_eq!(v["trace"][0]["file"], "a.js");
    assert_eq!(v["trace"][0]["end"], "fixpoint");
    let events: Vec<&str> = v["trace"][0]["passes"][0].as_array().unwrap().iter().map(|e| e["event"].as_str().unwrap()).collect();
    assert_eq!(events, ["offered", "offered", "applied", "applied"], "{v}");
    assert_eq!(fs::read_to_string(d.join("a.js")).unwrap(), "if (a === b) { f(c === d); }\n");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn timings_show_one_parse_per_file() {
    let d = project("timings");
    let (_, out, err) = codelaw(&d, &["check", "--timings", "--no-cache", "--threads", "1"]);
    assert!(out.contains("3 files checked") || out.contains("files checked"), "{out}");
    assert!(err.starts_with("timings: "), "{err}");
    let v = json(&d, &["check", "--timings", "--no-cache", "--format", "json"]);
    let t = &v["timings"];
    let parse = t["phases"].as_array().unwrap().iter().find(|p| p["phase"] == "parse").unwrap();
    // At most one parse per file checked (the SR-ARCHITECTURE veto's check).
    assert!(parse["count"].as_u64().unwrap() <= v["summary"]["files_checked"].as_u64().unwrap(), "{t}");
    assert!(parse["count"].as_u64().unwrap() >= 3, "a.py, scripts/b.py and a.js are parsed: {t}");
    let rules: Vec<&str> = t["rules"].as_array().unwrap().iter().map(|r| r["rule"].as_str().unwrap()).collect();
    assert!(rules.contains(&"fixture/js-eqeqeq") && rules.contains(&"fixture/py-no-print"), "{t}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn sarif_is_valid_sarif_2_1_0() {
    let d = project("sarif");
    fs::write(d.join("big.py"), "x = 1\n".repeat(10)).unwrap();
    fs::write(d.join("codelaw.toml"), "[files]\nmax_file_size = 40\n[rules]\n\"fixture/py-no-print\" = \"warn\"\n").unwrap();
    let (code, out, err) = codelaw(&d, &["check", "--format", "sarif", "--no-cache"]);
    assert_eq!(code, 1, "the js-eqeqeq errors remain: {err}");
    let log: Value = serde_json::from_str(&out).unwrap();
    let schema: Value = serde_json::from_str(&fs::read_to_string(repo().join("crates/codelaw-report/tests/sarif-schema-2.1.0.json")).unwrap()).unwrap();
    let mut schemas = boon::Schemas::new();
    let mut c = boon::Compiler::new();
    c.add_resource("file:///sarif-schema-2.1.0.json", schema).unwrap();
    let idx = c.compile("file:///sarif-schema-2.1.0.json", &mut schemas).unwrap();
    assert!(schemas.validate(&serde_json::json!({ "version": "2.1.0" }), idx).is_err(), "the schema check bites: `runs` is required");
    if let Err(e) = schemas.validate(&log, idx) {
        panic!("not valid SARIF 2.1.0: {e:#}\n{out}");
    }
    let run = &log["runs"][0];
    let results = run["results"].as_array().unwrap();
    let eq = results.iter().find(|r| r["ruleId"] == "fixture/js-eqeqeq").unwrap();
    assert_eq!(eq["level"], "error");
    assert_eq!(eq["properties"]["fixClass"], "auto");
    assert_eq!(eq["fixes"][0]["artifactChanges"][0]["replacements"][0]["insertedContent"]["text"], "a === b");
    let print = results.iter().find(|r| r["ruleId"] == "fixture/py-no-print").unwrap();
    assert_eq!(print["properties"]["steps"][0], "Change it by hand.");
    assert!(results.iter().any(|r| r["ruleId"] == "codelaw/file-too-large"), "{out}");
    assert_eq!(run["tool"]["driver"]["rules"][eq["ruleIndex"].as_u64().unwrap() as usize]["id"], "fixture/js-eqeqeq");
    // fix --format sarif: what is left after fixing, valid too.
    let (_, out, _) = codelaw(&d, &["fix", "--dry-run", "--format", "sarif"]);
    assert!(schemas.validate(&serde_json::from_str(&out).unwrap(), idx).is_ok(), "{out}");
    let _ = fs::remove_dir_all(&d);
}
