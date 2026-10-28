//! SR-CLI-MCP decision 12b (W-109) through the binary: codelaw writes no
//! Bitbucket file (`init --ci bitbucket` is an unknown value), and the step
//! USAGE.md shows to copy — its script taken from the doc and run here with
//! `curl` and `npm` stubbed — checks, fails the build on an error, sends the
//! Code Insights report and its annotation through the pipeline proxy, and
//! writes SARIF only when `CODELAW_SARIF` is on. What it cannot prove is
//! Bitbucket itself: the live run is the decider's (W-109).

use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-bitbucket-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

#[test]
fn init_writes_no_bitbucket_file() {
    let d = temp("init");
    fs::write(d.join("codelaw.toml"), "extends = []\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(["init", "--ci", "bitbucket"]).current_dir(&d).env("NO_COLOR", "1").output().unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(text.contains("--ci is `github` or `gitlab`, not `bitbucket`"), "{text}");
    assert!(!d.join("bitbucket-pipelines.yml").exists());
    let _ = fs::remove_dir_all(d);
}

/// The YAML block under USAGE.md's `### Bitbucket Pipelines`.
fn usage_step() -> String {
    let usage = fs::read_to_string(repo().join("USAGE.md")).unwrap();
    let section = &usage[usage.find("\n### Bitbucket Pipelines\n").expect("USAGE.md has a Bitbucket Pipelines section")..];
    let block = &section[section.find("```yaml\n").unwrap() + 8..];
    block[..block.find("\n```").unwrap()].to_string()
}

/// The step's `script:` items, as Bitbucket runs them: one shell, in order.
fn step_script(yml: &str) -> String {
    let mut out = String::new();
    let mut lines = yml.lines().skip_while(|l| l.trim() != "script:").skip(1).peekable();
    while let Some(l) = lines.next() {
        let Some(item) = l.strip_prefix("          - ") else { break };
        if item == "|" {
            while let Some(body) = lines.peek().and_then(|b| b.strip_prefix("            ").or((b.is_empty()).then_some(""))) {
                out.push_str(body);
                out.push('\n');
                lines.next();
            }
        } else {
            out.push_str(item);
            out.push('\n');
        }
    }
    out
}

fn stub(dir: &Path, name: &str, body: &str) {
    let p = dir.join(name);
    fs::write(&p, format!("#!/usr/bin/env bash\n{body}")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Runs the step in a clone with the project in `proj/`; returns the exit
/// code, the output and each `curl` call (its arguments, then its body).
fn run_step(name: &str, vars: &[(&str, &str)]) -> (i32, String, Vec<(String, String)>, PathBuf) {
    let clone = temp(name);
    let proj = clone.join("proj");
    fs::create_dir_all(proj.join("src")).unwrap();
    let rules = repo().join("fixtures/github-action/.codelaw/rules/fixture");
    fs::create_dir_all(proj.join(".codelaw/rules/fixture")).unwrap();
    for e in fs::read_dir(&rules).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), proj.join(".codelaw/rules/fixture").join(e.file_name())).unwrap();
    }
    fs::copy(repo().join("fixtures/github-action/codelaw.toml"), proj.join("codelaw.toml")).unwrap();
    fs::write(proj.join("src/app.js"), "export function run() {\n  debugger;\n  return 1;\n}\n").unwrap();
    let bin = clone.join(".stub-bin");
    fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_codelaw"), bin.join("codelaw")).unwrap();
    stub(&bin, "npm", "exit 0\n");
    let log = clone.join(".curl-log");
    fs::create_dir_all(&log).unwrap();
    stub(&bin, "curl", &format!("n=$(ls '{l}' | grep -c 'args$'); printf '%s\\n' \"$*\" > '{l}'/$n.args; case \"$*\" in *'@-'*) cat > '{l}'/$n.body;; esac; exit 0\n", l = log.display()));
    let script = step_script(&usage_step());
    assert!(script.contains("npm install --global --no-audit --no-fund codelaw@") && script.contains("codelaw check --format bitbucket"), "{script}");
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(&script)
        .current_dir(&clone)
        .env("PATH", format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()))
        .env("BITBUCKET_CLONE_DIR", &clone)
        .env("BITBUCKET_WORKSPACE", "ws")
        .env("BITBUCKET_REPO_SLUG", "repo")
        .env("BITBUCKET_COMMIT", "abc123")
        .env("CODELAW_DIR", "proj")
        .env("CODELAW_CHANGED_SINCE", "none")
        .env("NO_COLOR", "1")
        .env_remove("CODELAW_FORMAT")
        .env_remove("BITBUCKET_PR_DESTINATION_BRANCH");
    for k in ["CODELAW_SARIF", "CODELAW_CODE_INSIGHTS", "CODELAW_FAIL_ON_ERROR", "CODELAW_ARGS"] {
        cmd.env_remove(k);
    }
    for (k, v) in vars {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    let mut calls = Vec::new();
    for n in 0..fs::read_dir(&log).unwrap().filter(|e| e.as_ref().unwrap().path().extension().is_some_and(|x| x == "args")).count() {
        let args = fs::read_to_string(log.join(format!("{n}.args"))).unwrap();
        let body = fs::read_to_string(log.join(format!("{n}.body"))).unwrap_or_default();
        calls.push((args, body));
    }
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)), calls, clone)
}

#[test]
fn the_step_fails_on_a_debugger_and_sends_the_report_and_its_annotation() {
    let (code, out, calls, clone) = run_step("insights", &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("fixture/no-debugger"), "the log shows the finding: {out}");
    assert!(!clone.join("codelaw.sarif").exists(), "no SARIF unless switched on");
    let api = "http://api.bitbucket.org/2.0/repositories/ws/repo/commit/abc123/reports/codelaw";
    assert_eq!(calls.len(), 3, "{calls:?}");
    assert!(calls.iter().all(|(a, _)| a.contains("--proxy http://localhost:29418")), "{calls:?}");
    assert!(calls[0].0.contains("--request DELETE") && calls[0].0.trim_end().ends_with(api), "{calls:?}");
    assert!(calls[1].0.contains("--request PUT") && calls[1].0.contains(api), "{calls:?}");
    let report: Value = serde_json::from_str(&calls[1].1).unwrap();
    assert_eq!((report["title"].as_str(), report["result"].as_str(), report["report_type"].as_str()), (Some("codelaw"), Some("FAILED"), Some("BUG")));
    assert!(calls[2].0.contains("--request POST") && calls[2].0.contains(&format!("{api}/annotations")), "{calls:?}");
    let batch: Value = serde_json::from_str(&calls[2].1).unwrap();
    let a = &batch.as_array().unwrap()[0];
    assert_eq!(a["path"], "proj/src/app.js", "the path is from the repository root");
    assert_eq!((a["line"].as_u64(), a["severity"].as_str(), a["result"].as_str()), (Some(2), Some("HIGH"), Some("FAILED")));
    assert!(a["summary"].as_str().unwrap().starts_with("fixture/no-debugger: "), "{a}");
    let _ = fs::remove_dir_all(clone);
}

#[test]
fn sarif_is_a_switch_and_the_step_still_fails_with_the_upload_off() {
    let (code, out, calls, clone) = run_step("sarif", &[("CODELAW_SARIF", "true"), ("CODELAW_CODE_INSIGHTS", "false")]);
    assert_eq!(code, 1, "{out}");
    assert!(calls.is_empty(), "no upload: {calls:?}");
    let sarif: Value = serde_json::from_str(&fs::read_to_string(clone.join("codelaw.sarif")).unwrap()).unwrap();
    assert_eq!(sarif["runs"][0]["results"].as_array().unwrap().len(), 1);
    let _ = fs::remove_dir_all(clone);

    let (code, out, calls, clone) = run_step("passing", &[("CODELAW_CODE_INSIGHTS", "false"), ("CODELAW_FAIL_ON_ERROR", "false")]);
    assert_eq!((code, calls.len()), (0, 0), "{out}");
    let _ = fs::remove_dir_all(clone);
}
