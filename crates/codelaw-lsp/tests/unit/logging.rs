//! `codelaw-lsp`'s unit tests of what the server logs (SR-EDITOR decision
//! 7): why a file gets nothing, a project that does not load, a failed
//! request, and the log level. Nothing is logged that holds source text.

use super::*;
use serde_json::json;
use std::fs;

fn dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("codelaw-lsp-log-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    canonical(&root)
}

fn open(server: &mut Server, u: &str, text: &str) {
    let params = json!({ "textDocument": { "uri": u, "languageId": "javascript", "version": 1, "text": text } });
    server.notification(Notification::new(DidOpenTextDocument::METHOD.into(), params)).unwrap();
}

fn change(server: &mut Server, u: &str, text: &str) {
    let params = json!({ "textDocument": { "uri": u, "version": 2 }, "contentChanges": [{ "text": text }] });
    server.notification(Notification::new(DidChangeTextDocument::METHOD.into(), params)).unwrap();
}

/// Everything the server has sent so far, as (method, params).
fn sent(client: &Connection) -> Vec<(String, serde_json::Value)> {
    client
        .receiver
        .try_iter()
        .filter_map(|m| match m {
            Message::Notification(n) => Some((n.method, n.params)),
            _ => None,
        })
        .collect()
}

fn logs(sent: &[(String, serde_json::Value)]) -> Vec<(i64, String)> {
    sent.iter()
        .filter(|(m, _)| m == LogMessage::METHOD)
        .map(|(_, p)| (p["type"].as_i64().unwrap(), p["message"].as_str().unwrap().to_owned()))
        .collect()
}

#[test]
fn a_file_of_no_covered_language_logs_its_reason_once() {
    let root = dir("none");
    let file = root.join("a.txt");
    fs::write(&file, "SECRET_TOKEN\n").unwrap();
    let u = uri::from_path(&file);
    let (conn, client) = Connection::memory();
    let mut server = Server::new(&conn, vec![]);
    open(&mut server, &u, "SECRET_TOKEN\n");
    change(&mut server, &u, "SECRET_TOKEN 2\n");
    let lines = logs(&sent(&client));
    let why: Vec<_> = lines.iter().filter(|(_, m)| m.contains("no diagnostics for")).collect();
    assert_eq!(why.len(), 1, "once per file, not per keystroke: {lines:?}");
    assert!(why[0].1.contains("a.txt") && why[0].1.contains("language is not one codelaw covers"), "{why:?}");
    assert!(lines.iter().all(|(_, m)| !m.contains("SECRET_TOKEN")), "source text is never logged");
}

#[test]
fn a_file_the_project_excludes_says_so_and_a_reload_says_it_again() {
    let root = dir("excluded");
    fs::write(root.join("codelaw.toml"), "exclude = [\"a.js\"]\n").unwrap();
    let file = root.join("a.js");
    fs::write(&file, "x\n").unwrap();
    let u = uri::from_path(&file);
    let (conn, client) = Connection::memory();
    let mut server = Server::new(&conn, vec![root.clone()]);
    open(&mut server, &u, "x\n");
    let lines = logs(&sent(&client));
    assert!(lines.iter().any(|(_, m)| m.contains("matched by `exclude`")), "{lines:?}");
    assert!(lines.iter().any(|(_, m)| m.contains("loaded: config")), "the project's config is named: {lines:?}");
    server.reload("test").unwrap();
    let again = logs(&sent(&client));
    assert!(again.iter().any(|(_, m)| m.contains("reloading projects: test")));
    assert!(again.iter().any(|(_, m)| m.contains("matched by `exclude`")), "a reload says it again: {again:?}");
}

#[test]
fn a_bad_config_is_logged_and_shown() {
    let root = dir("bad");
    fs::write(root.join("codelaw.toml"), "this is = = not toml\n").unwrap();
    let file = root.join("a.js");
    fs::write(&file, "x\n").unwrap();
    let u = uri::from_path(&file);
    let (conn, client) = Connection::memory();
    let mut server = Server::new(&conn, vec![root.clone()]);
    open(&mut server, &u, "x\n");
    open(&mut server, &u, "x\n");
    let all = sent(&client);
    let errors: Vec<_> = logs(&all).into_iter().filter(|(t, _)| *t == 1).collect();
    assert!(!errors.is_empty(), "the failure is in the log: {all:?}");
    assert_eq!(all.iter().filter(|(m, _)| m == ShowMessage::METHOD).count(), 1, "shown once: {all:?}");
    assert_eq!(errors.len(), 1, "logged once: {errors:?}");
}

#[test]
fn a_failed_request_is_logged_with_its_method_and_file() {
    let root = dir("request");
    let (conn, client) = Connection::memory();
    let mut server = Server::new(&conn, vec![root.clone()]);
    let u = uri::from_path(&root.join("a.js"));
    let req = Request::new(1.into(), HoverRequest::METHOD.into(), json!({ "textDocument": { "uri": u }, "position": "nope" }));
    server.request(req).unwrap();
    let all: Vec<Message> = client.receiver.try_iter().collect();
    let lines: Vec<(i64, String)> = all
        .iter()
        .filter_map(|m| match m {
            Message::Notification(n) if n.method == LogMessage::METHOD => Some((n.params["type"].as_i64().unwrap(), n.params["message"].as_str().unwrap().to_owned())),
            _ => None,
        })
        .collect();
    let line = lines.iter().find(|(t, _)| *t == 1).expect("an error line");
    assert!(line.1.contains("textDocument/hover") && line.1.contains("a.js"), "{line:?}");
    let reply = all.iter().find_map(|m| match m {
        Message::Response(r) => Some(r),
        _ => None,
    });
    assert!(reply.is_some_and(|r| r.response_result.is_err()), "the response is still an error");
}

#[test]
fn the_level_gates_what_is_sent_and_changes_without_a_restart() {
    let root = dir("level");
    let file = root.join("a.js");
    fs::write(&file, "x\n").unwrap();
    let u = uri::from_path(&file);
    let (conn, client) = Connection::memory();
    let mut server = Server::new(&conn, vec![root.clone()]);
    server.level = LogLevel::Warn;
    open(&mut server, &u, "x\n");
    assert!(logs(&sent(&client)).iter().all(|(t, _)| *t <= 2), "warn level sends no info or debug");
    let n = Notification::new(DidChangeConfiguration::METHOD.into(), json!({ "settings": { "codelaw": { "logLevel": "debug" } } }));
    server.notification(n).unwrap();
    assert_eq!(server.level, LogLevel::Debug);
    sent(&client);
    change(&mut server, &u, "y\n");
    let lines = logs(&sent(&client));
    assert!(lines.iter().any(|(t, m)| *t == 4 && m.starts_with("check ") && m.contains(" ms")), "debug adds the per-check line: {lines:?}");
    assert_eq!(LogLevel::parse("loud"), None);
}
