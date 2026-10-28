//! W-45: `--format github|gitlab|junit|checkstyle` through the binary (W-109 adds `bitbucket` to the shared checks). GitLab
//! is validated with `boon` against GitLab's own Code Quality schema, JUnit
//! and Checkstyle with `xmllint --schema` (libxml2) against the JUnit XSD and
//! a Checkstyle XSD, all vendored in `crates/codelaw-report/tests/`. The
//! XML checks are skipped, loudly, on a machine without `xmllint`.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn schema(name: &str) -> PathBuf {
    repo().join("crates/codelaw-report/tests").join(name)
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-ci-{name}-{}", std::process::id()));
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

/// Two fixture rules (an `auto` JS error, a `manual` Python warning), an
/// oversize file for an engine finding, and a JSON file the formatter would
/// change. The JS file has the same rule twice with the same message, and the
/// message carries characters every format must escape.
fn project(name: &str) -> PathBuf {
    let d = temp(name);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    for r in ["js-eqeqeq.md", "py-no-print.md"] {
        fs::copy(repo().join("fixtures/l3/rules/fixture").join(r), d.join(".codelaw/rules/fixture").join(r)).unwrap();
    }
    fs::write(d.join("codelaw.toml"), "[files]\nmax_file_size = 40\n[rules]\n\"fixture/py-no-print\" = \"warn\"\n").unwrap();
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    fs::write(d.join("a.js"), "if (a == b) { f(c == d); }\n").unwrap();
    fs::write(d.join("big.py"), "x = 1\n".repeat(10)).unwrap();
    d
}

fn xmllint(schema_file: &Path, xml: &str) -> Option<Result<(), String>> {
    let mut child = match Command::new("xmllint").arg("--noout").arg("--schema").arg(schema_file).arg("-").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("SKIPPED: xmllint is not installed, so {} was not checked", schema_file.display());
            return None;
        }
    };
    use std::io::Write;
    child.stdin.take().unwrap().write_all(xml.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    Some(if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).into_owned()) })
}

fn assert_valid_xml(schema_file: &str, xml: &str) {
    if let Some(r) = xmllint(&schema(schema_file), xml) {
        if let Err(e) = r {
            panic!("not valid against {schema_file}: {e}\n{xml}");
        }
        // The check bites: a broken document fails the same schema.
        let broken = xml.replacen("<", "<bogus/><", 2);
        assert!(xmllint(&schema(schema_file), &broken).unwrap().is_err(), "the schema check bites");
    }
}

#[test]
fn gitlab_is_valid_code_quality_with_stable_fingerprints() {
    let d = project("gitlab");
    let (code, out, err) = codelaw(&d, &["check", "--format", "gitlab", "--no-cache"]);
    assert_eq!(code, 1, "the js-eqeqeq errors remain: {err}");
    let report: Value = serde_json::from_str(&out).unwrap();

    let schema_json: Value = serde_json::from_str(&fs::read_to_string(schema("gitlab-codequality-schema.json")).unwrap()).unwrap();
    let mut schemas = boon::Schemas::new();
    let mut c = boon::Compiler::new();
    c.add_resource("file:///codequality.json", schema_json).unwrap();
    let idx = c.compile("file:///codequality.json", &mut schemas).unwrap();
    assert!(schemas.validate(&serde_json::json!({ "description": "x" }), idx).is_err(), "the schema check bites");
    let items = report.as_array().expect("the report is an array");
    assert!(items.len() >= 4, "{out}");
    let mut prints = std::collections::BTreeSet::new();
    for item in items {
        if let Err(e) = schemas.validate(item, idx) {
            panic!("not valid Code Quality: {e:#}\n{item}");
        }
        // What GitLab's documentation adds to the schema's required set.
        assert!(item["check_name"].is_string() && item["location"]["lines"]["begin"].is_u64(), "{item}");
        assert!(["info", "minor", "major", "critical", "blocker"].contains(&item["severity"].as_str().unwrap()), "{item}");
        assert!(prints.insert(item["fingerprint"].as_str().unwrap().to_owned()), "fingerprints are unique: {out}");
    }
    let eq: Vec<&Value> = items.iter().filter(|i| i["check_name"] == "fixture/js-eqeqeq").collect();
    assert_eq!(eq.len(), 2);
    assert_eq!(eq[0]["severity"], "major");
    assert!(eq[0]["content"]["body"].as_str().unwrap().starts_with("fix: auto"), "{}", eq[0]);
    let print = items.iter().find(|i| i["check_name"] == "fixture/py-no-print").unwrap();
    assert_eq!(print["severity"], "minor");
    assert!(print["content"]["body"].as_str().unwrap().contains("1. Change it by hand."), "{print}");

    // Moving the code does not change a fingerprint.
    fs::write(d.join("a.js"), "\n\n\nif (a == b) { f(c == d); }\n").unwrap();
    let (_, moved, _) = codelaw(&d, &["check", "--format", "gitlab", "--no-cache"]);
    let moved: Value = serde_json::from_str(&moved).unwrap();
    let ids = |v: &Value| v.as_array().unwrap().iter().map(|i| (i["check_name"].to_string(), i["fingerprint"].to_string())).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids(&report), ids(&moved));
    let line_of = |v: &Value| v.as_array().unwrap().iter().find(|i| i["check_name"] == "fixture/js-eqeqeq").unwrap()["location"]["lines"]["begin"].clone();
    assert_ne!(line_of(&report), line_of(&moved), "the lines did move");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn junit_and_checkstyle_are_valid_against_their_schemas() {
    let d = project("xml");
    let (code, junit, err) = codelaw(&d, &["check", "--format", "junit", "--no-cache"]);
    assert_eq!(code, 1, "{err}");
    assert!(junit.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites>\n"), "{junit}");
    assert!(junit.contains("<failure message=") && junit.contains("type=\"fixture/js-eqeqeq\""), "{junit}");
    assert!(junit.contains("1. Change it by hand."), "the steps ride along: {junit}");
    assert_valid_xml("junit-windyroad.xsd", &junit);

    let (code, cs, err) = codelaw(&d, &["check", "--format", "checkstyle", "--no-cache"]);
    assert_eq!(code, 1, "{err}");
    assert!(cs.contains("<file name=\"a.js\">") && cs.contains("severity=\"error\"") && cs.contains("source=\"fixture/js-eqeqeq\""), "{cs}");
    assert!(cs.contains("severity=\"warning\"") && cs.contains("[fix: auto]"), "{cs}");
    assert_valid_xml("checkstyle.xsd", &cs);

    // A clean project is still a valid, empty report.
    let clean = temp("xml-clean");
    fs::write(clean.join("ok.js"), "const a = 1;\n").unwrap();
    for (format, schema) in [("junit", "junit-windyroad.xsd"), ("checkstyle", "checkstyle.xsd")] {
        let (code, out, err) = codelaw(&clean, &["check", "--format", format, "--no-cache"]);
        assert_eq!(code, 0, "{err}");
        assert_valid_xml(schema, &out);
    }
    let _ = fs::remove_dir_all(&d);
    let _ = fs::remove_dir_all(&clean);
}

#[test]
fn github_prints_workflow_commands_and_a_summary() {
    let d = project("github");
    let (code, out, err) = codelaw(&d, &["check", "--format", "github", "--no-cache"]);
    assert_eq!(code, 1, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    let first = lines.iter().find(|l| l.contains("title=fixture/js-eqeqeq")).unwrap();
    assert!(first.starts_with("::error file=a.js,line=1,col=5,"), "{first}");
    assert!(first.contains("[fix: auto]"), "{first}");
    assert!(lines.iter().any(|l| l.starts_with("::warning file=a.py,") && l.contains("title=fixture/py-no-print::")), "{out}");
    // Every annotation is one line; the summary is the last, and plain.
    for l in &lines[..lines.len() - 1] {
        assert!(l.starts_with("::"), "{l}");
    }
    assert!(lines.last().unwrap().starts_with("codelaw: 2 errors, "), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn the_same_input_gives_byte_identical_output_and_the_exit_code_is_the_text_run() {
    let d = project("same");
    let (text_code, _, _) = codelaw(&d, &["check", "--no-cache"]);
    let (text_fix_code, _, _) = codelaw(&d, &["fix", "--dry-run"]);
    for format in ["github", "gitlab", "junit", "checkstyle", "bitbucket"] {
        let (code, a, _) = codelaw(&d, &["check", "--format", format, "--no-cache"]);
        let (_, b, _) = codelaw(&d, &["check", "--format", format, "--no-cache"]);
        assert_eq!(a, b, "{format} is deterministic");
        assert_eq!(code, text_code, "{format}: exit codes unchanged");
        let (code, _, _) = codelaw(&d, &["check", "--format", format, "--max-warnings", "0", "--no-cache"]);
        assert_eq!(code, 1);
        let (code, fixed, _) = codelaw(&d, &["fix", "--dry-run", "--format", format]);
        assert_eq!(code, text_fix_code, "{format}: fix exit codes unchanged");
        assert!(!fixed.is_empty());
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn format_check_reports_unformatted_files_in_the_ci_formats() {
    let d = temp("format");
    fs::write(d.join("a.json"), "{\"a\":1,\n\"b\":[1,2]}\n").unwrap();
    let (text_code, _, _) = codelaw(&d, &["format", "--check"]);
    assert_eq!(text_code, 1);
    for format in ["github", "gitlab", "junit", "checkstyle", "bitbucket"] {
        let (code, out, err) = codelaw(&d, &["format", "--check", "--format", format]);
        assert_eq!(code, 1, "{format}: {err}");
        assert!(out.contains("format/unformatted") && out.contains("a.json"), "{format}: {out}");
    }
    let (_, out, _) = codelaw(&d, &["format", "--check", "--format", "junit"]);
    assert_valid_xml("junit-windyroad.xsd", &out);
    let (code, _, err) = codelaw(&d, &["format", "--diff", "--format", "github"]);
    assert_eq!(code, 2, "{err}");
    assert_eq!(fs::read_to_string(d.join("a.json")).unwrap(), "{\"a\":1,\n\"b\":[1,2]}\n", "--check wrote nothing");
    let _ = fs::remove_dir_all(&d);
}
