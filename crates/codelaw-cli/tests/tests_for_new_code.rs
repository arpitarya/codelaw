//! The test rules of new code (W-71) on real git repos:
//! `graph/untested-function`, `graph/function-coverage` with an lcov
//! report, `codelaw/stale-coverage`, and `.codelawignore` — `forever`,
//! `on_touch`, `on_touch_scope`, the required fields, `codelaw/stale-ignore`
//! and its fix, and the `codelaw exceptions` rows.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const DAY: &str = "2026-10-27";

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-w71-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn git(cwd: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.email=t@example.com", "-c", "user.name=t", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_PROFILE")
        .env_remove("CI")
        .output()
        .unwrap()
}

fn check(cwd: &Path, args: &[&str]) -> Value {
    let mut all = vec!["check", "--format", "json", "--no-cache", "--as-of", DAY];
    all.extend_from_slice(args);
    let out = run(cwd, &all);
    let text = String::from_utf8(out.stdout).unwrap();
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}\n{}", String::from_utf8_lossy(&out.stderr)))
}

/// `rule severity file name` for every finding of the W-71 rules.
fn found(v: &Value) -> Vec<String> {
    let mut out: Vec<String> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| {
            let r = d["rule"].as_str().unwrap();
            r.starts_with("graph/") || r.contains("ignore") || r.contains("coverage") || r.contains("exception")
        })
        .map(|d| {
            let what = d["data"]["function"].as_str().map(str::to_owned).unwrap_or_else(|| format!("line {}", d["range"]["start"]["line"].as_u64().or(d["line"].as_u64()).unwrap_or(0)));
            format!("{} {} {} {}", d["rule"].as_str().unwrap(), d["severity"].as_str().unwrap(), d["file"].as_str().unwrap(), what)
        })
        .collect();
    out.sort();
    out
}

/// A repo on `main` with old, untested code, then a branch that changes it.
fn js_repo(name: &str) -> PathBuf {
    let d = temp(name);
    git(&d, &["init", "-q", "-b", "main"]);
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:tests\"]\n").unwrap();
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/orders.ts"), "export function old(a: number) {\n  return a + 1;\n}\n\nexport function legacy() {\n  return 0;\n}\n").unwrap();
    git(&d, &["add", "."]);
    git(&d, &["commit", "-q", "-m", "base"]);
    git(&d, &["checkout", "-q", "-b", "feature"]);
    fs::write(
        d.join("src/orders.ts"),
        "export function old(a: number) {\n  return a + 2;\n}\n\nexport function legacy() {\n  return 0;\n}\n\nexport function computeTax(x: number) {\n  return x * 0.2;\n}\n\nexport const tested = (x: number) => x;\n\nfunction helper() {\n  return 1;\n}\n",
    )
    .unwrap();
    // A test reaches `tested` through a barrel and an alias; `helper` is not
    // exported, so importing its module reaches it.
    fs::write(d.join("src/index.ts"), "export { tested as t } from './orders';\n").unwrap();
    fs::write(d.join("src/orders.test.ts"), "import { t as alias } from './index';\nimport './orders';\nfunction localHelper() { return alias(1); }\ntest('x', () => { expect(localHelper()).toBe(1); });\n").unwrap();
    git(&d, &["add", "."]);
    git(&d, &["commit", "-q", "-m", "change"]);
    d
}

#[test]
fn only_new_and_edited_functions_no_test_reaches_are_reported() {
    let d = js_repo("js");
    let v = check(&d, &["--changed-since", "main"]);
    assert_eq!(
        found(&v),
        ["graph/untested-function error src/orders.ts computeTax", "graph/untested-function error src/orders.ts old"],
        "{v:#}"
    );
    let d0 = v["diagnostics"].as_array().unwrap().iter().find(|x| x["rule"] == "graph/untested-function").unwrap();
    assert_eq!(d0["fix"]["class"].as_str().or(d0["fix"].as_str()).unwrap_or("manual"), "manual", "{d0:#}");
    assert!(v["new_code"]["change_set"].as_str().unwrap().starts_with("new code: 3 new, 1 edited functions since"), "{v:#}");
    assert_eq!(v["new_code"]["coverage"], "coverage: none; not evaluated");
    // The message names the function and where a test goes.
    let text = String::from_utf8(run(&d, &["check", "--no-cache", "--as-of", DAY, "--changed-since", "main"]).stdout).unwrap();
    assert!(text.contains("`computeTax` is new and no test reaches it (add one in `orders.test.ts`, or exempt it in `.codelawignore`)"), "{text}");
    assert!(text.contains("new code: 3 new, 1 edited functions since"), "{text}");
}

#[test]
fn no_change_set_judges_nothing_and_says_so() {
    let d = js_repo("none");
    let v = check(&d, &[]);
    assert!(found(&v).is_empty(), "{v:#}");
    assert_eq!(v["new_code"]["change_set"], "new code: no change set; test rules not evaluated");
    let text = String::from_utf8(run(&d, &["check", "--no-cache", "--format", "agent"]).stdout).unwrap();
    assert!(text.starts_with("new code: no change set; test rules not evaluated\n"), "{text}");
    // `on_no_change_set = "all"` judges every function, old ones too.
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:tests\"]\n[coverage]\non_no_change_set = \"all\"\nscope = \"public\"\n").unwrap();
    let v = check(&d, &[]);
    assert_eq!(found(&v).len(), 3, "old, legacy, computeTax: {v:#}");
}

#[test]
fn off_unless_the_project_turns_it_on() {
    let d = js_repo("off");
    fs::write(d.join("codelaw.toml"), "").unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    assert!(found(&v).is_empty() && v.get("new_code").is_none(), "{v:#}");
    // Naming one rule turns on that one only.
    fs::write(d.join("codelaw.toml"), "[rules]\n\"graph/untested-function\" = \"warn\"\n").unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    assert_eq!(found(&v), ["graph/untested-function warn src/orders.ts computeTax", "graph/untested-function warn src/orders.ts old"]);
    assert!(v["new_code"]["coverage"].is_null(), "{v:#}");
}

#[test]
fn staged_and_the_hook_are_change_sets_too() {
    let d = js_repo("staged");
    fs::write(d.join("src/new.ts"), "export function fresh() {\n  return 1;\n}\n").unwrap();
    git(&d, &["add", "src/new.ts"]);
    let v = check(&d, &["--staged"]);
    assert_eq!(found(&v), ["graph/untested-function error src/new.ts fresh"], "{v:#}");
    // The agent hook's file is a change set against `HEAD`.
    git(&d, &["commit", "-q", "-m", "fresh"]);
    fs::write(d.join("src/new.ts"), "export function fresh() {\n  return 2;\n}\n\nexport function another() {\n  return 3;\n}\n").unwrap();
    let out = run(&d, &["hook", "src/new.ts"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(err.contains("graph/untested-function") && err.contains("`another` is new") && err.contains("`fresh` was edited"), "{err}");
}

#[test]
fn python_classes_aliases_and_strings() {
    let d = temp("py");
    git(&d, &["init", "-q", "-b", "main"]);
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:tests\"]\n").unwrap();
    fs::create_dir_all(d.join("app")).unwrap();
    fs::create_dir_all(d.join("tests")).unwrap();
    fs::write(d.join("app/__init__.py"), "").unwrap();
    fs::write(d.join("app/billing.py"), "def base():\n    return 1\n").unwrap();
    git(&d, &["add", "."]);
    git(&d, &["commit", "-q", "-m", "base"]);
    fs::write(
        d.join("app/billing.py"),
        "def base():\n    return 1\n\n\nclass Invoice:\n    def total(self):\n        return 2\n\n\ndef lonely():\n    return 3\n\n\ndef aliased():\n    return 4\n\n\ndef patched():\n    return 5\n",
    )
    .unwrap();
    fs::write(
        d.join("tests/test_billing.py"),
        "from unittest import mock\nfrom app.billing import Invoice\nfrom app.billing import aliased as a\n\n\ndef test_it():\n    with mock.patch(\"app.billing.patched\"):\n        assert Invoice().total() == 2 and a() == 4\n",
    )
    .unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    assert_eq!(found(&v), ["graph/untested-function error app/billing.py lonely", "graph/untested-function error app/billing.py patched"], "{v:#}");
}

fn lcov(d: &Path, records: &[(&str, &[(u32, u32)])]) -> PathBuf {
    let mut text = String::new();
    for (sf, das) in records {
        text.push_str(&format!("TN:\nSF:{sf}\n"));
        for (l, h) in *das {
            text.push_str(&format!("DA:{l},{h}\n"));
        }
        text.push_str("end_of_record\n");
    }
    let p = d.join("coverage/lcov.info");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, text).unwrap();
    p
}

#[test]
fn coverage_per_function_from_an_lcov_report() {
    let d = js_repo("cov");
    let abs = d.join("src/orders.ts").display().to_string();
    // old: lines 1-3 (2 of 2 run); computeTax: lines 9-11 (1 of 2 run = 50%).
    let report = lcov(&d, &[(&abs, &[(1, 1), (2, 3), (9, 1), (10, 0), (13, 1)])]);
    let v = check(&d, &["--changed-since", "main", "--coverage", report.to_str().unwrap()]);
    let cov: Vec<&Value> = v["diagnostics"].as_array().unwrap().iter().filter(|x| x["rule"] == "graph/function-coverage").collect();
    assert_eq!(cov.len(), 1, "{v:#}");
    assert_eq!(cov[0]["message"], "`computeTax` is 50% covered (1 of 2 lines; min 60)");
    assert_eq!(cov[0]["data"]["covered"], 1);
    assert_eq!(cov[0]["data"]["lines"], 2);
    assert_eq!(cov[0]["data"]["pct"], 50);
    assert_eq!(cov[0]["data"]["min"], 60);
    assert_eq!(v["new_code"]["coverage"], "coverage: coverage/lcov.info (1 files)");
    // `[coverage] report`, relative `SF:`, and `min`.
    lcov(&d, &[("src/orders.ts", &[(1, 1), (2, 3), (9, 1), (10, 0), (13, 1)])]);
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:tests\"]\n[coverage]\nreport = \"coverage/lcov.info\"\nmin = 50\n").unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    assert!(!found(&v).iter().any(|f| f.starts_with("graph/function-coverage")), "{v:#}");
    // `mode = "patch"`: 4 of 5 lines together, 80%, passes 60.
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:tests\"]\n[coverage]\nreport = \"coverage/lcov.info\"\nmode = \"patch\"\nmin = 90\n").unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    let cov: Vec<&Value> = v["diagnostics"].as_array().unwrap().iter().filter(|x| x["rule"] == "graph/function-coverage").collect();
    assert_eq!(cov.len(), 1, "{v:#}");
    assert!(cov[0]["message"].as_str().unwrap().starts_with("new and edited code is 80% covered (4 of 5 lines; min 90"), "{v:#}");
}

#[test]
fn a_report_that_does_not_match_is_stale_and_judges_nothing() {
    let d = js_repo("stale");
    let report = lcov(&d, &[("src/orders.ts", &[(1, 0), (99, 1)])]);
    let v = check(&d, &["--changed-since", "main", "--coverage", report.to_str().unwrap()]);
    let f = found(&v);
    assert!(f.contains(&"codelaw/stale-coverage warn src/orders.ts line 1".to_owned()), "{f:?}");
    assert!(!f.iter().any(|x| x.starts_with("graph/function-coverage")), "{f:?}");
    // A line that is blank now.
    lcov(&d, &[("src/orders.ts", &[(4, 1)])]);
    assert!(found(&check(&d, &["--changed-since", "main", "--coverage", report.to_str().unwrap()])).iter().any(|x| x.starts_with("codelaw/stale-coverage")));
    // A report that does not name the file.
    lcov(&d, &[("src/other.ts", &[(1, 1)])]);
    assert!(found(&check(&d, &["--changed-since", "main", "--coverage", report.to_str().unwrap()])).iter().any(|x| x.starts_with("codelaw/stale-coverage")));
    // No report: skipped, unless required.
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:tests\"]\n[coverage]\nrequire_report = true\n").unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    assert!(found(&v).contains(&"graph/function-coverage error codelaw.toml line 1".to_owned()), "{v:#}");
}

#[test]
fn codelawignore_exempts_revokes_and_reports() {
    let d = js_repo("ignore");
    fs::write(
        d.join(".codelawignore"),
        "# exemptions\n\
src/orders.ts#computeTax  forever: a thin wrapper\n\
src/orders.ts#legacy      on_touch=error expires=2027-01-31: covered by e2e until ORD-1\n\
src/orders.ts#old         on_touch=warn on_touch_scope=function expires=2027-01-31 by=a@example.com: migrating\n\
src/gone.ts#nothing       forever: deleted long ago\n\
src/orders.ts#helper      expires=2027-01-31: no on_touch\n",
    )
    .unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    assert_eq!(
        found(&v),
        [
            "codelaw/incomplete-ignore error .codelawignore line 6",
            "codelaw/stale-ignore warn .codelawignore line 5",
            // `legacy` is untouched, but its file was edited: file scope revokes it.
            "graph/untested-function error src/orders.ts legacy",
            // `old` was edited: function scope, `on_touch=warn`.
            "graph/untested-function warn src/orders.ts old",
        ],
        "{v:#}"
    );
    let incomplete = v["diagnostics"].as_array().unwrap().iter().find(|x| x["rule"] == "codelaw/incomplete-ignore").unwrap();
    assert!(incomplete["message"].as_str().unwrap().contains("on_touch=error|warn|keep"), "{incomplete:#}");
    // `require_on_touch = false`: the default `on_touch` fills in.
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:tests\"]\n[coverage.ignore]\nrequire_on_touch = false\non_touch = \"keep\"\n").unwrap();
    let v = check(&d, &["--changed-since", "main"]);
    assert!(!found(&v).iter().any(|x| x.starts_with("codelaw/incomplete-ignore")), "{v:#}");
    // `codelaw exceptions` lists the lines, `forever` as such.
    let out = run(&d, &["exceptions", "--format", "json", "--as-of", DAY]);
    let ex: Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows: Vec<String> = ex["exceptions"]
        .as_array()
        .or(ex["rows"].as_array())
        .unwrap_or_else(|| panic!("{ex:#}"))
        .iter()
        .filter(|r| r["kind"] == "codelawignore" && r["rule"] == "graph/untested-function")
        .map(|r| format!("{}#{} {}", r["files"][0].as_str().unwrap(), r["symbol"].as_str().unwrap_or(""), if r["forever"] == true { "forever".to_owned() } else { r["expires"].as_str().unwrap_or("").to_owned() }))
        .collect();
    assert!(rows.contains(&"src/orders.ts#computeTax forever".to_owned()) && rows.contains(&"src/orders.ts#legacy 2027-01-31".to_owned()), "{rows:?}");
    let text = String::from_utf8(run(&d, &["exceptions", "--as-of", DAY]).stdout).unwrap();
    assert!(text.lines().any(|l| l.starts_with("forever ") && l.contains(" never ") && l.contains("src/orders.ts#computeTax  ")), "{text}");
    // `codelaw fix` removes the stale line and nothing else.
    run(&d, &["fix", "--no-cache", "--as-of", DAY, "--changed-since", "main"]);
    let after = fs::read_to_string(d.join(".codelawignore")).unwrap();
    assert!(!after.contains("src/gone.ts") && after.contains("src/orders.ts#legacy"), "{after}");
}

#[test]
fn an_expired_line_exempts_nothing() {
    let d = js_repo("expired");
    fs::write(d.join(".codelawignore"), "src/orders.ts#old  on_touch=keep expires=2026-10-01: until the rewrite\n").unwrap();
    let f = found(&check(&d, &["--changed-since", "main"]));
    assert!(f.contains(&"codelaw/expired-exception error .codelawignore line 1".to_owned()) && f.contains(&"graph/untested-function error src/orders.ts old".to_owned()), "{f:?}");
    fs::write(d.join(".codelawignore"), "src/orders.ts#old  on_touch=keep expires=2026-10-29: until the rewrite\n").unwrap();
    let f = found(&check(&d, &["--changed-since", "main"]));
    assert!(f.contains(&"codelaw/expiring-exception warn .codelawignore line 1".to_owned()) && !f.iter().any(|x| x.ends_with("src/orders.ts old")), "{f:?}");
}

#[test]
fn an_inline_codelaw_ignore_covers_one_function() {
    let d = js_repo("inline");
    let text = fs::read_to_string(d.join("src/orders.ts")).unwrap();
    fs::write(d.join("src/orders.ts"), text.replace("export function computeTax", "// codelaw-ignore graph/untested-function: covered by the e2e suite\nexport function computeTax")).unwrap();
    let f = found(&check(&d, &["--changed-since", "main"]));
    assert_eq!(f, ["graph/untested-function error src/orders.ts old"], "{f:?}");
}
