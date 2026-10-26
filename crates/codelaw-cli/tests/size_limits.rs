//! Size limits through the binary (W-58): the options and their types, the
//! measured value in JSON and SARIF, and a baseline that freezes today's
//! long files while a new long file still fails.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-size-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_SITE_PACKAGES")
        .env("CODELAW_CACHE_DIR", cwd.join(".cache"))
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
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

/// A project with the js pack, `codelaw:js/size` and the given `[rules]`.
fn project(name: &str, rules: &str) -> PathBuf {
    let d = temp(name);
    copy_dir(&repo().join("packs/js"), &d.join("node_modules/@codelaw/rules-js"));
    fs::write(d.join("codelaw.toml"), format!("extends = [\"codelaw:js/size\"]\npreview = true\n[rules]\n{rules}")).unwrap();
    fs::write(d.join("long.js"), "a();\nb();\nc();\nd();\n").unwrap();
    fs::write(d.join("many.js"), "function f(a, b, c, d) {\n  return a;\n}\n").unwrap();
    d
}

const SHORT: &str = "\"js/max-lines\" = { severity = \"warn\", options = { max = 3 } }\n";

#[test]
fn the_preset_turns_on_all_three_and_options_change_the_limit() {
    let d = project("preset", SHORT);
    let (code, out, err) = codelaw(&d, &["check", "--format", "agent"]);
    assert_eq!(code, 0, "warnings only: {out}{err}");
    assert!(out.contains("long.js:4:1 js/max-lines 4 lines (max 3; blank lines and comments not counted)"), "{out}");
    assert!(out.contains("many.js:1:10 js/max-params `f` takes 4 parameters (max 3)"), "{out}");
    assert!(!out.contains("max-lines-per-function"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

/// DoD 3: a wrong option type is a load error, exit 2.
#[test]
fn a_wrong_option_type_is_a_load_error() {
    let d = project("wrong", "\"js/max-lines\" = { severity = \"warn\", options = { max = \"300\" } }\n");
    let (code, out, err) = codelaw(&d, &["check"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(err.contains("js/max-lines") && err.contains("option `max` must be a `integer`"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

/// DoD 5: JSON and SARIF carry the measured value and the limit.
#[test]
fn json_and_sarif_carry_the_measured_value() {
    let d = project("data", SHORT);
    let (_, out, err) = codelaw(&d, &["check", "--format", "json", "long.js"]);
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    let f = v["diagnostics"].as_array().unwrap().iter().find(|d| d["rule"] == "js/max-lines").unwrap();
    assert_eq!(f["data"], serde_json::json!({ "lines": 4, "max": 3 }), "{f}");
    let (_, out, _) = codelaw(&d, &["check", "--format", "sarif", "long.js"]);
    let log: Value = serde_json::from_str(&out).unwrap();
    let r = &log["runs"][0]["results"][0];
    assert_eq!(r["ruleId"], "js/max-lines");
    assert_eq!(r["properties"]["data"], serde_json::json!({ "lines": 4, "max": 3 }), "{r}");
    let _ = fs::remove_dir_all(&d);
}

/// DoD 4: `baseline add --rule js/max-lines` freezes today's long files; a
/// new long file is a finding.
#[test]
fn a_baseline_freezes_the_long_files_and_a_new_one_fails() {
    let d = project("baseline", "\"js/max-lines\" = { severity = \"error\", options = { max = 3 } }\n");
    let (code, out, err) = codelaw(&d, &["check", "long.js"]);
    assert_eq!(code, 1, "{out}{err}");
    let (code, out, err) = codelaw(&d, &["baseline", "add", "--rule", "js/max-lines"]);
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = codelaw(&d, &["check", "long.js"]);
    assert_eq!(code, 0, "frozen: {out}{err}");
    fs::write(d.join("newer.js"), "a();\nb();\nc();\nd();\ne();\n").unwrap();
    let (code, out, err) = codelaw(&d, &["check", "long.js", "newer.js"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("newer.js\n  5") || out.contains("error  5 lines (max 3"), "the new file is an error: {out}");
    assert!(out.contains("info  4 lines (max 3") && out.contains("grandfathered"), "the old one is frozen: {out}");
    let _ = fs::remove_dir_all(&d);
}

/// W-61: a project with the css and html packs, the two `size` presets and
/// a scope that gives `_*.scss` partials their own limit.
#[test]
fn css_and_html_size_data_and_a_partials_scope() {
    let d = temp("cssdata");
    copy_dir(&repo().join("packs/css"), &d.join("node_modules/@codelaw/rules-css"));
    copy_dir(&repo().join("packs/html"), &d.join("node_modules/@codelaw/rules-html"));
    fs::write(
        d.join("codelaw.toml"),
        "extends = [\"codelaw:css/size\", \"codelaw:html/size\"]\npreview = true\n[rules]\n\"css/max-lines\" = { severity = \"warn\", options = { max = 3 } }\n\"html/max-lines\" = { severity = \"warn\", options = { max = 3 } }\n\n[scope.partials]\nfiles = [\"**/_*.scss\"]\nrules = { \"css/max-lines\" = { options = { max = 6 } } }\n",
    )
    .unwrap();
    let five = "a {\n  top: 0;\n  left: 0;\n  right: 0;\n}\n";
    fs::write(d.join("site.scss"), five).unwrap();
    fs::write(d.join("_part.scss"), five).unwrap();
    fs::write(d.join("long.css"), five).unwrap();
    fs::write(d.join("page.html"), "<p>\n<b>\n<i>\n<u>\n</p>\n").unwrap();
    let (_, out, err) = codelaw(&d, &["check", "--format", "json"]);
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    let hits: Vec<(&str, &str, &Value)> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["rule"].as_str().is_some_and(|r| r.ends_with("/max-lines")))
        .map(|x| (x["file"].as_str().unwrap(), x["rule"].as_str().unwrap(), &x["data"]))
        .collect();
    let five_over_three = serde_json::json!({ "lines": 5, "max": 3 });
    assert!(hits.contains(&("site.scss", "css/max-lines", &five_over_three)), "{hits:?}");
    assert!(hits.contains(&("long.css", "css/max-lines", &five_over_three)), "{hits:?}");
    assert!(hits.contains(&("page.html", "html/max-lines", &five_over_three)), "{hits:?}");
    assert!(!hits.iter().any(|h| h.0 == "_part.scss"), "the partials scope raises the limit to 6: {hits:?}");
    let (_, out, _) = codelaw(&d, &["check", "--format", "sarif"]);
    let log: Value = serde_json::from_str(&out).unwrap();
    let r = log["runs"][0]["results"].as_array().unwrap().iter().find(|r| r["ruleId"] == "html/max-lines").unwrap();
    assert_eq!(r["properties"]["data"], five_over_three, "{r}");
    let _ = fs::remove_dir_all(&d);
}
