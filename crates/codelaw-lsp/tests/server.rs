//! `codelaw lsp` end to end over an in-memory connection (W-09): UTF-16
//! positions on multi-byte and astral text (SR-ARCHITECTURE decision 13),
//! fixes as `WorkspaceEdit`s with nothing written (decision 12), and the
//! same diagnostics as `check` on every L3 fixture (SR-EDITOR decision 3).

use codelaw_engine::{EngineOptions, Project};
use lsp_server::{Connection, Message, Notification, Request, RequestId};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
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

/// A repo of the L3 fixture rules plus every language's fixture files.
fn repo(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-lsp-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    let fixtures = repo_root().join("fixtures/l3");
    copy_dir(&fixtures.join("rules"), &d.join(".codelaw/rules"));
    for e in fs::read_dir(&fixtures).unwrap().flatten() {
        if e.path().is_dir() && e.file_name() != "rules" {
            copy_dir(&e.path(), &d);
        }
    }
    d
}

/// Every file under `root` and its bytes.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(&dir).unwrap().flatten() {
            if e.path().is_dir() {
                stack.push(e.path());
            } else {
                out.insert(e.path(), fs::read(e.path()).unwrap());
            }
        }
    }
    out
}

fn uri(path: &Path) -> String {
    codelaw_lsp::uri::from_path(path)
}

struct Client {
    conn: Connection,
    server: Option<JoinHandle<()>>,
    next: i32,
    /// Notifications received while waiting for something else.
    inbox: Vec<Notification>,
}

impl Client {
    fn start(root: &Path) -> Client {
        let (server, conn) = Connection::memory();
        let handle = std::thread::spawn(move || codelaw_lsp::serve(&server).unwrap());
        let mut c = Client { conn, server: Some(handle), next: 0, inbox: vec![] };
        let init = c.request("initialize", json!({ "capabilities": {}, "processId": null, "workspaceFolders": [{ "uri": uri(root), "name": "repo" }] }));
        assert_eq!(init["capabilities"]["positionEncoding"], "utf-16");
        c.notify("initialized", json!({}));
        c
    }

    fn notify(&self, method: &str, params: Value) {
        self.conn.sender.send(Message::Notification(Notification::new(method.into(), params))).unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        let id = RequestId::from(self.next);
        self.conn.sender.send(Message::Request(Request::new(id.clone(), method.into(), params))).unwrap();
        loop {
            match self.conn.receiver.recv_timeout(Duration::from_secs(60)).expect("a response") {
                Message::Response(r) if r.id == id => return r.response_result.unwrap_or_else(|e| panic!("{method}: {e:?}")),
                Message::Notification(n) => self.inbox.push(n),
                _ => {}
            }
        }
    }

    /// The next diagnostics published for `uri`.
    fn diagnostics(&mut self, uri: &str) -> Value {
        if let Some(i) = self.inbox.iter().position(|n| n.method == "textDocument/publishDiagnostics" && n.params["uri"] == uri) {
            return self.inbox.remove(i).params;
        }
        loop {
            match self.conn.receiver.recv_timeout(Duration::from_secs(60)).expect("diagnostics") {
                Message::Notification(n) if n.method == "textDocument/publishDiagnostics" && n.params["uri"] == uri => return n.params,
                Message::Notification(n) => self.inbox.push(n),
                _ => {}
            }
        }
    }

    fn open(&mut self, uri: &str, language: &str, text: &str) -> Value {
        self.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": language, "version": 1, "text": text } }));
        self.diagnostics(uri)
    }

    fn shutdown(mut self) {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.server.take().unwrap().join().unwrap();
    }
}

/// UTF-16 positions as a client counts them: code units since the line's start.
fn utf16(text: &str, byte: usize) -> Value {
    let line_start = text[..byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = text[..byte].matches('\n').count();
    json!({ "line": line, "character": text[line_start..byte].encode_utf16().count() })
}

/// Apply LSP edits the way a UTF-16 editor does.
fn apply(text: &str, edits: &[Value]) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut starts = vec![0usize];
    starts.extend(units.iter().enumerate().filter(|(_, u)| **u == u16::from(b'\n')).map(|(i, _)| i + 1));
    let at = |p: &Value| starts[p["line"].as_u64().unwrap() as usize] + p["character"].as_u64().unwrap() as usize;
    let mut spans: Vec<(usize, usize, Vec<u16>)> =
        edits.iter().map(|e| (at(&e["range"]["start"]), at(&e["range"]["end"]), e["newText"].as_str().unwrap().encode_utf16().collect())).collect();
    spans.sort_by_key(|a| std::cmp::Reverse(a.0));
    let mut out = units;
    for (s, e, t) in spans {
        out.splice(s..e, t);
    }
    String::from_utf16(&out).unwrap()
}

/// Multi-byte (é, 中) and astral (𝄞, 😀) characters before the finding,
/// on the same line and on earlier lines.
/// Formatted, so the only findings are the rule's.
const ASTRAL: &str = "const s = \"é𝄞😀\"; // 中文\nif (s == \"𝄞\") go(\"😀\" == t);\n";

/// The same, unformatted: the block is on one line.
const UNFORMATTED: &str = "const s = \"é𝄞😀\"; // 中文\nif (s == \"𝄞\") { go(\"😀\" == t); }\n";

#[test]
fn diagnostics_use_utf16_positions_on_astral_text() {
    let root = repo("utf16");
    let path = root.join("src/astral.js");
    fs::write(&path, ASTRAL).unwrap();
    let mut c = Client::start(&root);
    let published = c.open(&uri(&path), "javascript", ASTRAL);
    let diags = published["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 2, "{published}");
    assert_eq!(published["version"], 1);
    let first = ASTRAL.find("s == \"𝄞\"").unwrap();
    let second = ASTRAL.find("\"😀\" == t").unwrap();
    for (d, start, len) in [(&diags[0], first, "s == \"𝄞\"".len()), (&diags[1], second, "\"😀\" == t".len())] {
        assert_eq!(d["code"], "fixture/js-eqeqeq");
        assert_eq!(d["source"], "codelaw");
        assert_eq!(d["range"]["start"], utf16(ASTRAL, start), "{d}");
        assert_eq!(d["range"]["end"], utf16(ASTRAL, start + len), "{d}");
    }
    c.shutdown();
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn fixes_are_workspace_edits_and_nothing_is_written() {
    let root = repo("fixes");
    let path = root.join("src/astral.js");
    fs::write(&path, ASTRAL).unwrap();
    let before = snapshot(&root);
    let u = uri(&path);
    let mut c = Client::start(&root);
    let published = c.open(&u, "javascript", ASTRAL);
    let diag = published["diagnostics"][1].clone();

    // A quick fix for the diagnostic under the cursor.
    let actions = c.request("textDocument/codeAction", json!({ "textDocument": { "uri": u }, "range": diag["range"], "context": { "diagnostics": [diag] } }));
    let quick: Vec<&Value> = actions.as_array().unwrap().iter().filter(|a| a["kind"] == "quickfix").collect();
    assert_eq!(quick.len(), 1, "{actions}");
    assert_eq!(quick[0]["isPreferred"], true);
    let edits = quick[0]["edit"]["changes"][&u].as_array().unwrap();
    let once = apply(ASTRAL, edits);
    assert_eq!(once, ASTRAL.replace("\"😀\" == t", "\"😀\" === t"));

    // Fix all, as on save: the same text `fix` would write.
    let all = c.request(
        "textDocument/codeAction",
        json!({ "textDocument": { "uri": u }, "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                "context": { "diagnostics": [], "only": ["source.fixAll"] } }),
    );
    let all = all.as_array().unwrap();
    assert_eq!(all.len(), 1, "{all:?}");
    assert_eq!(all[0]["kind"], codelaw_lsp::FIX_ALL);
    let fixed = apply(ASTRAL, all[0]["edit"]["changes"][&u].as_array().unwrap());
    let project = Project::load(&root, EngineOptions { no_cache: true, ..Default::default() }).unwrap();
    let entry = project.entry_for(&path).unwrap();
    assert_eq!(fixed, project.fix_source(&entry.rel, entry.language, ASTRAL, false).fixed);
    assert_eq!(fixed.matches("===").count(), 2);

    // The editor applies the edit; the server wrote nothing.
    assert_eq!(snapshot(&root), before, "the language server wrote to disk");
    c.shutdown();
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn formatting_is_one_edit_to_what_format_writes_and_nothing_is_written() {
    let root = repo("format");
    let path = root.join("src/astral.js");
    fs::write(&path, UNFORMATTED).unwrap();
    let before = snapshot(&root);
    let u = uri(&path);
    let mut c = Client::start(&root);
    let published = c.open(&u, "javascript", UNFORMATTED);
    assert!(published["diagnostics"].as_array().unwrap().iter().any(|d| d["code"] == "format/unformatted"), "{published}");

    let edits = c.request("textDocument/formatting", json!({ "textDocument": { "uri": u }, "options": { "tabSize": 2, "insertSpaces": true } }));
    let edits = edits.as_array().unwrap();
    assert_eq!(edits.len(), 1, "{edits:?}");
    let formatted = apply(UNFORMATTED, edits);
    let project = Project::load(&root, EngineOptions { no_cache: true, ..Default::default() }).unwrap();
    let entry = project.entry_for(&path).unwrap();
    assert_eq!(formatted, project.format_source(&entry.rel, entry.language, UNFORMATTED).unwrap());
    assert_ne!(formatted, UNFORMATTED);

    // Formatted text formats to no edit at all.
    c.notify("textDocument/didChange", json!({ "textDocument": { "uri": u, "version": 2 }, "contentChanges": [{ "text": formatted }] }));
    let again = c.request("textDocument/formatting", json!({ "textDocument": { "uri": u }, "options": { "tabSize": 2, "insertSpaces": true } }));
    assert_eq!(again, json!([]));

    assert_eq!(snapshot(&root), before, "the language server wrote to disk");
    c.shutdown();
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn edits_recheck_and_close_clears() {
    let root = repo("edits");
    let path = root.join("src/a.js");
    let u = uri(&path);
    let mut c = Client::start(&root);
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(c.open(&u, "javascript", &text)["diagnostics"].as_array().unwrap().len(), 1);
    // An unsaved edit is checked as typed.
    c.notify("textDocument/didChange", json!({ "textDocument": { "uri": u, "version": 2 }, "contentChanges": [{ "text": "x == y;\nz == w;\n" }] }));
    let p = c.diagnostics(&u);
    assert_eq!(p["version"], 2);
    assert_eq!(p["diagnostics"].as_array().unwrap().len(), 2);
    // Hover names the rule.
    let hover = c.request("textDocument/hover", json!({ "textDocument": { "uri": u }, "position": { "line": 1, "character": 1 } }));
    assert!(hover["contents"]["value"].as_str().unwrap().contains("fixture/js-eqeqeq"), "{hover}");
    c.notify("textDocument/didClose", json!({ "textDocument": { "uri": u } }));
    assert_eq!(c.diagnostics(&u)["diagnostics"], json!([]));
    // Not a file: nothing is checked.
    assert_eq!(c.open("untitled:Untitled-1", "javascript", "a == b;")["diagnostics"], json!([]));
    c.shutdown();
    let _ = fs::remove_dir_all(&root);
}

/// The LSP shows exactly what `check` shows, file by file, on every L3
/// fixture (SR-EDITOR decision 3; its veto's parity test).
#[test]
fn same_diagnostics_as_check_on_every_fixture() {
    let root = repo("parity");
    let mut project = Project::load(&root, EngineOptions { no_cache: true, ..Default::default() }).unwrap();
    let files = project.files(&[]).unwrap();
    let report = project.check(&[]).unwrap();
    assert!(files.len() >= 11, "{} files", files.len());
    let mut c = Client::start(&root);
    let mut compared = 0;
    for f in &files {
        let text = fs::read_to_string(&f.abs).unwrap();
        let published = c.open(&uri(&f.abs), f.language.name(), &text);
        let got: Vec<(String, String, Value, Value)> = published["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| {
                (d["code"].as_str().unwrap().to_owned(), d["message"].as_str().unwrap().to_owned(), d["range"]["start"].clone(), d["range"]["end"].clone())
            })
            .collect();
        let want: Vec<(String, String, Value, Value)> = report
            .diagnostics
            .iter()
            .filter(|d| d.file == f.rel)
            .map(|d| (d.rule.clone(), d.message.clone(), utf16(&text, d.start_byte), utf16(&text, d.end_byte)))
            .collect();
        assert_eq!(got, want, "{}", f.rel);
        compared += want.len();
    }
    assert!(compared >= 9, "every fixture rule fires: {compared}");
    c.shutdown();
    let _ = fs::remove_dir_all(&root);
}

/// SR-ARCHITECTURE decision 12: nothing in the server opens a file for
/// writing; the editor applies every edit.
#[test]
fn the_server_has_no_write_path() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for e in fs::read_dir(src).unwrap().flatten() {
        let text = fs::read_to_string(e.path()).unwrap();
        for banned in ["fs::write", "File::create", "OpenOptions", "fs::rename", "remove_file", "create_dir", ".fix(&"] {
            assert!(!text.contains(banned), "{} contains `{banned}`", e.path().display());
        }
    }
}
