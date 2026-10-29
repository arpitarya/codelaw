//! L3 decision 10 (W-136): a repo whose `.nvmrc` disagrees with
//! `package.json`'s `engines.node` shows the finding on both files in every
//! output — `check` text, JSON, SARIF and GitHub, the LSP and the post-edit
//! hook. The rule is the test-only `test/node-version` of `codelaw-native`'s
//! `fault` module, a stand-in for W-129's runtime rule.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repo(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-text-config-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    let write = |rel: &str, text: &str| {
        let p = d.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    };
    let pack = "node_modules/@codelaw/rules-data";
    write(&format!("{pack}/codelaw-pack.toml"), "name = \"@codelaw/rules-data\"\nversion = \"0.1.0\"\nmin_codelaw = \"0.1.0\"\nlanguages = [\"json\"]\npresets = [\"data/recommended\"]\n");
    write(
        &format!("{pack}/rules/data/test-node-version.md"),
        "---\nid: data/test-node-version\ntitle: node versions agree\nlanguages: [json]\nfix: manual\nfix_reason: test\nstability: stable\n---\n\nSummary.\n\n**Why:** test.\n\n```codelaw\nnative: test/node-version\n```\n\n## How to fix\n\n1. Make them agree.\n\n## Examples\n\n### JSON\n\n#### Don't\n\n```json\n{}\n```\n\n#### Do\n\n```json\n{}\n```\n",
    );
    write(&format!("{pack}/presets/data/recommended.toml"), "[rules]\n\"data/test-node-version\" = \"warn\"\n");
    write("codelaw.toml", "extends = [\"codelaw:data/recommended\"]\n");
    write("package.json", "{\n  \"name\": \"a\",\n  \"engines\": { \"node\": \">=18\" }\n}\n");
    write(".nvmrc", "20\n");
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_SITE_PACKAGES").env_remove("CI").output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

#[test]
fn check_shows_both_places_in_every_format() {
    let d = repo("check");
    let (_, out, err) = codelaw(&d, &["check"]);
    assert!(out.contains(".nvmrc\n  1:1  warn") && out.contains("package.json\n  3:24  warn"), "text: {out}{err}");

    let (_, out, _) = codelaw(&d, &["check", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let found: Vec<(String, u64)> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["rule"] == "data/test-node-version")
        .map(|x| (x["file"].as_str().unwrap().to_owned(), x["range"]["start"]["line"].as_u64().unwrap()))
        .collect();
    assert_eq!(found, [(".nvmrc".to_owned(), 1), ("package.json".to_owned(), 3)], "json: {out}");
    let copy = v["diagnostics"].as_array().unwrap().iter().find(|x| x["file"] == ".nvmrc").unwrap();
    assert_eq!(copy["data"]["primary"]["file"], "package.json", "json: {out}");

    let (_, out, _) = codelaw(&d, &["check", "--format", "sarif"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let uris: Vec<&str> = v["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&".nvmrc") && uris.contains(&"package.json"), "sarif: {uris:?}");

    let (_, out, _) = codelaw(&d, &["check", "--format", "github"]);
    assert!(out.contains("::warning file=.nvmrc,line=1,") && out.contains("::warning file=package.json,line=3,"), "github: {out}");

    // `fix` writes no text config file.
    codelaw(&d, &["fix"]);
    assert_eq!(fs::read_to_string(d.join(".nvmrc")).unwrap(), "20\n");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn the_hook_reports_both_places_whichever_file_was_edited() {
    let d = repo("hook");
    for edited in ["package.json", ".nvmrc"] {
        let (code, _, err) = codelaw(&d, &["hook", d.join(edited).to_str().unwrap()]);
        assert_eq!(code, 2, "{edited}: {err}");
        assert!(err.contains(".nvmrc:1:1"), "{edited}: {err}");
        if edited == "package.json" {
            assert!(err.contains("package.json:3:24"), "{edited}: {err}");
        }
    }
    let _ = fs::remove_dir_all(&d);
}

fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

/// The publishDiagnostics notifications in an LSP transcript, by URI: the
/// last one for each.
fn published(out: &str) -> Vec<(String, serde_json::Value)> {
    let mut last: Vec<(String, serde_json::Value)> = Vec::new();
    for part in out.split("Content-Length: ").skip(1) {
        let body = &part[part.find("\r\n\r\n").unwrap() + 4..];
        let Ok(v) = serde_json::from_str::<serde_json::Value>(body) else { continue };
        if v["method"] == "textDocument/publishDiagnostics" {
            let uri = v["params"]["uri"].as_str().unwrap().to_owned();
            last.retain(|(u, _)| *u != uri);
            last.push((uri, v["params"]["diagnostics"].clone()));
        }
    }
    last
}

#[test]
fn the_editor_shows_the_copy_on_the_text_file() {
    let d = repo("lsp");
    let root = format!("file://{}", d.to_string_lossy().replace('\\', "/"));
    let open = |name: &str, lang: &str| {
        let text = fs::read_to_string(d.join(name)).unwrap();
        serde_json::json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": format!("{root}/{name}"), "languageId": lang, "version": 1, "text": text } } }).to_string()
    };
    // Each run opens one of the two files: the text file shows the copy
    // whether its manifest is open or not.
    for (name, lang) in [("package.json", "json"), (".nvmrc", "plaintext")] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(["lsp", "--stdio"]).current_dir(&d).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let mut stdin = child.stdin.take().unwrap();
        for body in [
            format!(r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"capabilities":{{}},"rootUri":"{root}"}}}}"#),
            r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#.to_owned(),
            open(name, lang),
            r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#.to_owned(),
            r#"{"jsonrpc":"2.0","method":"exit"}"#.to_owned(),
        ] {
            stdin.write_all(frame(&body).as_bytes()).unwrap();
        }
        drop(stdin);
        let mut out = String::new();
        child.stdout.take().unwrap().read_to_string(&mut out).unwrap();
        assert!(child.wait().unwrap().success(), "{out}");
        let p = published(&out);
        let on = |file: &str| p.iter().find(|(u, _)| u.ends_with(&format!("/{file}"))).map(|(_, d)| d.clone());
        let nvmrc = on(".nvmrc").unwrap_or_else(|| panic!("{name}: nothing published on .nvmrc: {out}"));
        assert_eq!(nvmrc.as_array().unwrap().len(), 1, "{name}: {nvmrc}");
        assert_eq!(nvmrc[0]["code"], "data/test-node-version");
        assert_eq!(nvmrc[0]["range"]["start"], serde_json::json!({ "line": 0, "character": 0 }));
        assert_eq!(nvmrc[0]["range"]["end"], serde_json::json!({ "line": 0, "character": 2 }));
        if name == "package.json" {
            let pkg = on("package.json").unwrap();
            let codes: Vec<&str> = pkg.as_array().unwrap().iter().map(|x| x["code"].as_str().unwrap()).collect();
            assert_eq!(codes, ["data/test-node-version"], "the copy is not on the manifest: {pkg}");
            assert_eq!(pkg[0]["range"]["start"]["line"], 2);
        }
    }
    let _ = fs::remove_dir_all(&d);
}
