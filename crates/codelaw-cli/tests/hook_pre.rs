//! The Gate through the binary (SR-AGENT-SURFACES decision 9) and the
//! brief (SR-CLI-MCP decision 14): W-93's definition of done.

use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-pre-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
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

/// A repo with the js pack's recommended rules: `js/no-debugger` an
/// error, `js/no-console` a warning.
fn project(name: &str) -> PathBuf {
    let d = temp(name);
    copy_dir(&repo().join("packs/js"), &d.join("node_modules/@codelaw/rules-js"));
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:js/recommended\"]\npreview = true\n").unwrap();
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/app.ts"), "export function load(): number {\n  return 1;\n}\n").unwrap();
    d
}

fn run(cwd: &Path, args: &[&str], stdin: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_SITE_PACKAGES")
        .env_remove("CODELAW_PROFILE")
        .env_remove("CI")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn pre(cwd: &Path, payload: &Value) -> (i32, String, String) {
    run(cwd, &["hook", "--pre"], &payload.to_string())
}

fn stdout_json(out: &str) -> Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("{e}: {out:?}"))
}

#[test]
fn a_new_error_is_refused_on_every_agent_with_its_rule_line_and_fix() {
    let d = project("deny");
    let cwd = d.to_string_lossy().into_owned();
    let debugger = "export function load(): number {\n  debugger;\n  return 1;\n}\n";
    // Claude Code `Write` and `Edit`.
    let claude_write = json!({ "hook_event_name": "PreToolUse", "cwd": cwd, "tool_name": "Write", "tool_input": { "file_path": "src/app.ts", "content": debugger } });
    let claude_edit = json!({ "hook_event_name": "PreToolUse", "cwd": cwd, "tool_name": "Edit", "tool_input": { "file_path": "src/app.ts", "old_string": "  return 1;", "new_string": "  debugger;\n  return 1;" } });
    for payload in [&claude_write, &claude_edit] {
        let (code, out, err) = pre(&d, payload);
        assert_eq!(code, 0, "{err}");
        let v = stdout_json(&out);
        let h = &v["hookSpecificOutput"];
        assert_eq!(h["permissionDecision"], "deny", "{v}");
        let reason = h["permissionDecisionReason"].as_str().unwrap();
        assert!(reason.contains("src/app.ts:2:3 error js/no-debugger") && reason.contains("fix: auto — remove \"  debugger;"), "{reason}");
    }
    // Codex `apply_patch`.
    let patch = "*** Begin Patch\n*** Update File: src/app.ts\n@@\n export function load(): number {\n+  debugger;\n   return 1;\n*** End Patch";
    let (code, out, _) = run(&d, &["hook", "--pre", "--agent", "codex"], &json!({ "cwd": cwd, "tool_name": "apply_patch", "tool_input": { "command": patch } }).to_string());
    assert_eq!(code, 0);
    assert!(stdout_json(&out)["hookSpecificOutput"]["permissionDecisionReason"].as_str().unwrap().contains("js/no-debugger"), "{out}");
    // Gemini `write_file` and `replace`, its shape read off the payload.
    for tool in [json!({ "tool_name": "write_file", "tool_input": { "file_path": "src/app.ts", "content": debugger } }), json!({ "tool_name": "replace", "tool_input": { "file_path": "src/app.ts", "old_string": "  return 1;", "new_string": "  debugger;\n  return 1;" } })] {
        let mut payload = tool;
        payload["hook_event_name"] = json!("BeforeTool");
        payload["cwd"] = json!(cwd);
        let (code, out, err) = pre(&d, &payload);
        assert_eq!(code, 0, "{err}");
        let v = stdout_json(&out);
        assert_eq!(v["decision"], "deny", "{v}");
        assert!(v["reason"].as_str().unwrap().contains("src/app.ts:2:3 error js/no-debugger"), "{v}");
    }
    // Kiro and Windsurf read no JSON: exit 2 and stderr.
    let (code, out, err) = run(&d, &["hook", "--pre", "--agent", "kiro"], &claude_edit.to_string());
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(err.contains("js/no-debugger"), "{err}");
    // Nothing was written: the gate only judges.
    assert!(!fs::read_to_string(d.join("src/app.ts")).unwrap().contains("debugger"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_warning_is_allowed_with_context_and_an_old_error_adds_nothing() {
    let d = project("warn");
    let cwd = d.to_string_lossy().into_owned();
    let console = json!({ "cwd": cwd, "tool_name": "Edit", "tool_input": { "file_path": "src/app.ts", "old_string": "  return 1;", "new_string": "  console.log(1);\n  return 1;" } });
    let (code, out, err) = pre(&d, &console);
    assert_eq!(code, 0, "{err}");
    let v = stdout_json(&out);
    assert!(v["hookSpecificOutput"].get("permissionDecision").is_none(), "an allow never says allow: {v}");
    assert!(v["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("warn js/no-console"), "{v}");
    // Gemini has no context channel: silent.
    let mut gemini = console.clone();
    gemini["hook_event_name"] = json!("BeforeTool");
    assert_eq!(pre(&d, &gemini), (0, String::new(), String::new()));
    // A file that already fails: an edit on the failing line adds no verdict.
    fs::write(d.join("src/app.ts"), "export function load(): number {\n  debugger;\n  return 1;\n}\n").unwrap();
    let touch = json!({ "cwd": cwd, "tool_name": "Edit", "tool_input": { "file_path": "src/app.ts", "old_string": "  debugger;", "new_string": "  debugger; // still here" } });
    assert_eq!(pre(&d, &touch), (0, String::new(), String::new()));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn codelaws_own_failure_allows_the_write() {
    let d = project("open");
    let cwd = d.to_string_lossy().into_owned();
    // A malformed payload.
    let (code, out, err) = run(&d, &["hook", "--pre"], "{ not json");
    assert_eq!((code, out.as_str()), (0, ""));
    assert!(err.contains("the write is allowed"), "{err}");
    // An edit of a file that is not there: the tool fails on its own.
    let missing = json!({ "cwd": cwd, "tool_name": "Edit", "tool_input": { "file_path": "src/gone.ts", "old_string": "a", "new_string": "debugger;" } });
    assert_eq!(pre(&d, &missing), (0, String::new(), String::new()));
    // A broken config is codelaw's failure, not the agent's.
    fs::write(d.join("codelaw.toml"), "extends = [\n").unwrap();
    let edit = json!({ "cwd": cwd, "tool_name": "Write", "tool_input": { "file_path": "src/app.ts", "content": "debugger;\n" } });
    let (code, out, err) = pre(&d, &edit);
    assert_eq!((code, out.as_str()), (0, ""));
    assert!(err.contains("the write is allowed"), "{err}");
    // The engine panicking is covered in-process by `pre::tests`.
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn the_brief_lists_only_what_is_specific_to_the_repo() {
    let d = project("brief");
    copy_dir(&repo().join("packs/security-js"), &d.join("node_modules/@codelaw/rules-security-js"));
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:js/recommended\", \"codelaw:sec-js/recommended\"]\npreview = true\n\n[rules.\"js/no-console\"]\nseverity = \"warn\"\noptions = { allow = [\"warn\"] }\n").unwrap();
    let rule = fs::read_to_string(repo().join("fixtures/l3/rules/fixture/ts-no-var.md")).unwrap();
    fs::create_dir_all(d.join(".codelaw/rules/team")).unwrap();
    fs::write(d.join(".codelaw/rules/team/no-var.md"), rule.replace("fixture/ts-no-var", "team/no-var")).unwrap();
    fs::write(d.join(".codelaw/rules/team/no-var-2.md"), rule.replace("fixture/ts-no-var", "team/no-var-2")).unwrap();
    fs::write(d.join("src/app.ts"), "export function load(x: number): number {\n  if (x) {}\n  return 1;\n}\n").unwrap();
    let (code, out, err) = run(&d, &["baseline", "add", "--rule", "js/no-empty", "--until", "2099-01-01", "--reason", "adopt"], "");
    assert_eq!(code, 0, "{out}{err}");
    let (code, text, err) = run(&d, &["brief", "--for", "src/app.ts"], "");
    assert_eq!(code, 0, "{err}");
    assert_eq!(run(&d, &["brief", "--for", "src/app.ts"], "").1, text, "byte-identical across runs");
    let (_, out, _) = run(&d, &["brief", "--for", "src/app.ts", "--format", "json"], "");
    let v = stdout_json(&out);
    let ids = |k: &str| v[k].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
    assert_eq!(ids("own"), ["team/no-var", "team/no-var-2"], "{v}");
    assert_eq!(ids("configured"), ["js/no-console"]);
    assert_eq!(v["configured"][0]["options"]["allow"], json!(["warn"]));
    assert_eq!(ids("exceptions"), ["js/no-empty"]);
    assert_eq!(v["exceptions"][0]["kinds"]["baseline"], 1);
    let security = v["security"].as_array().unwrap();
    assert!(!security.is_empty() && security.iter().all(|r| r["fix"] == "manual" && r["security"].is_object()), "{v}");
    assert!(v["standard"]["count"].as_u64().unwrap() > 10 && v["standard"]["ported_from"].as_array().unwrap().contains(&json!("eslint")), "{v}");
    // The groups in order, then the count line.
    let at = |s: &str| text.find(s).unwrap_or_else(|| panic!("{s}: {text}"));
    assert!(at("this repo's own rules:") < at("rules this repo configures:") && at("rules this repo configures:") < at("rules with exceptions here"));
    assert!(at("rules with exceptions here") < at("security rules with a manual fix:") && at("security rules with a manual fix:") < at("the other "));
    assert!(text.trim_end().ends_with("`codelaw hook --pre` refuses a write that breaks one, with its fix"), "{text}");
    // A path codelaw does not check is a usage error.
    assert_eq!(run(&d, &["brief", "--for", "x.rs"], "").0, 2);
    let _ = fs::remove_dir_all(&d);
}
