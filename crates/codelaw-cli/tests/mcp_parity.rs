//! SR-CLI-MCP decision 6 (W-08): every fixture through the CLI and through
//! `codelaw mcp`, and the answers must be byte-equal — the same JSON for
//! every tool, the same bytes written by `fix`. Both surfaces are the same
//! binary, so this runs the real one over stdio.

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const LANGUAGES: &[&str] = &["javascript", "typescript", "python", "json", "yaml", "toml", "markdown", "css", "html"];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("fixtures/l3")
}

fn copy_dir(from: &Path, to: &Path) {
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

fn repo(name: &str, languages: &[&str]) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-mcp-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules")).unwrap();
    // The root is the project for every path, as in a real repo.
    fs::write(d.join("codelaw.toml"), "").unwrap();
    copy_dir(&fixtures().join("rules"), &d.join(".codelaw/rules"));
    for l in languages {
        copy_dir(&fixtures().join(l), &d);
    }
    fs::canonicalize(d).unwrap()
}

/// The repo's own files (not `.codelaw/`), relative, sorted.
fn files(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == ".codelaw") {
                continue;
            }
            files(root, &p, out);
        } else {
            out.insert(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"), fs::read(&p).unwrap());
        }
    }
}

fn tree_of(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    files(root, root, &mut out);
    out
}

fn command(cwd: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_SITE_PACKAGES").env("CODELAW_CACHE_DIR", cwd.join(".codelaw/cache"));
    c
}

/// What the CLI prints with `--format json`, without its final newline;
/// for a usage error (exit 2), `error: ` and the message.
fn cli(cwd: &Path, args: &[&str]) -> String {
    let mut c = command(cwd);
    c.args(args);
    // MCP never runs adapters (SR-ARCHITECTURE decision 19); the twins of
    // `check_paths` and `fix_paths` are `check` and `fix` without them.
    if matches!(args.first(), Some(&"check" | &"fix")) {
        c.arg("--no-adapters");
    }
    let out = c.output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    if out.status.code() == Some(2) {
        let stderr = String::from_utf8(out.stderr).unwrap();
        return stderr.strip_suffix('\n').unwrap_or(&stderr).to_owned();
    }
    assert!(out.status.code().is_some_and(|c| c <= 1), "codelaw {args:?} exited {:?}: {}{}", out.status.code(), stdout, String::from_utf8_lossy(&out.stderr));
    stdout.strip_suffix('\n').unwrap_or(&stdout).to_owned()
}

struct Mcp {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next: u64,
}

impl Mcp {
    fn start(cwd: &Path) -> Mcp {
        let mut child = command(cwd).arg("mcp").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut m = Mcp { child, stdin, stdout, next: 0 };
        let init = m.request("initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "parity", "version": "0" } }));
        assert_eq!(init["result"]["serverInfo"]["name"], "codelaw", "{init}");
        writeln!(m.stdin, "{}", json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).unwrap();
        m
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        writeln!(self.stdin, "{}", json!({ "jsonrpc": "2.0", "id": self.next, "method": method, "params": params })).unwrap();
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        let v: Value = serde_json::from_str(&line).unwrap_or_else(|e| panic!("{e}: {line}"));
        assert_eq!(v["id"], self.next, "{v}");
        v
    }

    /// A tool's text, which must also be its structured content; for a
    /// tool error, `error: ` and the message, as the CLI prints it.
    fn call(&mut self, tool: &str, args: Value) -> String {
        let r = self.request("tools/call", json!({ "name": tool, "arguments": args }));
        let text = r["result"]["content"][0]["text"].as_str().unwrap_or_else(|| panic!("{tool} {r}")).to_owned();
        if r["result"]["isError"] == true {
            return format!("error: {text}");
        }
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), r["result"]["structuredContent"], "{tool}");
        text
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn parity(name: &str, languages: &[&str]) {
    let root = repo(name, languages);
    let sources = tree_of(&root);
    let mut mcp = Mcp::start(&root);
    let tools = mcp.request("tools/list", json!({}));
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 9);

    let checked = mcp.call("check_paths", json!({}));
    assert_eq!(checked, cli(&root, &["check", "--format", "json"]), "{name}: check_paths");
    // Not vacuous: every fixture repo has findings.
    assert!(!serde_json::from_str::<Value>(&checked).unwrap()["diagnostics"].as_array().unwrap().is_empty(), "{name}: {checked}");
    let mut traced_a_finding = false;
    let listed = mcp.call("list_rules", json!({}));
    assert_eq!(listed, cli(&root, &["rules", "--format", "json"]), "{name}: list_rules");
    let ids: Vec<String> =
        serde_json::from_str::<Value>(&listed).unwrap()["rules"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap().to_owned()).collect();
    assert!(!ids.is_empty());
    for id in &ids {
        assert_eq!(mcp.call("explain_rule", json!({ "id": id })), cli(&root, &["explain", id, "--format", "json"]), "{name}: explain_rule {id}");
    }
    assert_eq!(
        mcp.call("explain_rule", json!({ "id": "codelaw/no-rules" })),
        cli(&root, &["explain", "codelaw/no-rules", "--format", "json"]),
        "{name}: an engine rule"
    );

    for (file, bytes) in &sources {
        let code = String::from_utf8(bytes.clone()).unwrap();
        assert_eq!(
            mcp.call("rules_for_path", json!({ "path": file })),
            cli(&root, &["rules", "--for", file, "--format", "json"]),
            "{name}: rules_for_path {file}"
        );
        assert_eq!(mcp.call("check_paths", json!({ "paths": [file] })), cli(&root, &["check", file, "--format", "json"]), "{name}: check_paths {file}");
        assert_eq!(
            mcp.call("check_snippet", json!({ "path": file, "code": code })),
            cli(&root, &["check", file, "--format", "json"]),
            "{name}: check_snippet {file}"
        );
        assert_eq!(mcp.call("dump_tree", json!({ "path": file })), cli(&root, &["tree", file, "--format", "json"]), "{name}: dump_tree {file}");
        assert_eq!(
            mcp.call("dump_tree", json!({ "path": file, "line": 1 })),
            cli(&root, &["tree", file, "--line", "1", "--format", "json"]),
            "{name}: dump_tree --line {file}"
        );
        for id in &ids {
            let traced = mcp.call("trace_rule", json!({ "id": id, "path": file }));
            assert_eq!(traced, cli(&root, &["rules", "--why", id, "--for", file, "--format", "json"]), "{name}: trace_rule {id} {file}");
            if let Ok(v) = serde_json::from_str::<Value>(&traced) {
                traced_a_finding |= v["for"]["diagnostics"].as_array().is_some_and(|d| !d.is_empty());
            }
        }
    }
    assert!(traced_a_finding, "{name}: no trace_rule reported a finding");

    // Fixed bytes: `fix` through each door on two copies of the repo, and
    // `format_snippet` against what `fix` wrote.
    let by_cli = repo(&format!("{name}-cli"), languages);
    let by_mcp = repo(&format!("{name}-mcp"), languages);
    let cli_fix = cli(&by_cli, &["fix", "--format", "json"]);
    let mcp_fix = Mcp::start(&by_mcp).call("fix_paths", json!({}));
    assert_eq!(mcp_fix, cli_fix, "{name}: fix_paths");
    let fixed = tree_of(&by_cli);
    assert_eq!(tree_of(&by_mcp), fixed, "{name}: fix_paths wrote different bytes");
    for (file, bytes) in &sources {
        let v: Value = serde_json::from_str(&mcp.call("format_snippet", json!({ "path": file, "code": String::from_utf8(bytes.clone()).unwrap() }))).unwrap();
        assert_eq!(v["code"].as_str().unwrap().as_bytes(), fixed[file].as_slice(), "{name}: format_snippet {file}");
        assert_eq!(v["changed"], json!(fixed[file] != *bytes), "{name}: {file}");
    }
    let dry = repo(&format!("{name}-dry"), languages);
    assert_eq!(
        Mcp::start(&dry).call("fix_paths", json!({ "dry_run": true, "unsafe": true })),
        cli(&root, &["fix", "--dry-run", "--unsafe", "--format", "json"]),
        "{name}: dry run"
    );
    assert_eq!(tree_of(&dry), sources, "{name}: a dry run wrote");

    for d in [root, by_cli, by_mcp, dry] {
        let _ = fs::remove_dir_all(d);
    }
}

#[test]
fn cli_and_mcp_agree_on_every_fixture() {
    for lang in LANGUAGES {
        parity(lang, &[lang]);
    }
    parity("all", LANGUAGES);
}

#[test]
fn mcp_errors_match_the_cli() {
    let root = repo("errors", &["javascript"]);
    let mut mcp = Mcp::start(&root);
    let r = mcp.request("tools/call", json!({ "name": "explain_rule", "arguments": { "id": "fixture/js-eqeqeqq" } }));
    assert_eq!(r["result"]["isError"], true);
    let text = r["result"]["content"][0]["text"].as_str().unwrap();
    let out = command(&root).args(["explain", "fixture/js-eqeqeqq"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stderr), format!("error: {text}\n"));
    assert!(text.contains("did you mean"), "{text}");
    let _ = fs::remove_dir_all(root);
}

/// An unformatted file: `check_snippet` reports `format/unformatted` as
/// `check` does, and `format_snippet` returns the bytes `fix` writes, the
/// format step included (SR-FORMATTER decision 4).
#[test]
fn snippets_format_as_check_and_fix_do() {
    let root = repo("format", &["javascript"]);
    let (file, code) = ("src/messy.js", "if (a == b) { go() }\n");
    fs::write(root.join(file), code).unwrap();
    let mut mcp = Mcp::start(&root);
    let checked = cli(&root, &["check", file, "--format", "json"]);
    assert!(checked.contains("format/unformatted"), "{checked}");
    assert_eq!(mcp.call("check_snippet", json!({ "path": file, "code": code })), checked);
    let v: Value = serde_json::from_str(&mcp.call("format_snippet", json!({ "path": file, "code": code }))).unwrap();
    cli(&root, &["fix", file, "--format", "json"]);
    assert_eq!(v["code"].as_str().unwrap(), fs::read_to_string(root.join(file)).unwrap());
    assert_eq!(v["code"].as_str().unwrap(), "if (a === b) {\n  go();\n}\n");
    let _ = fs::remove_dir_all(root);
}
