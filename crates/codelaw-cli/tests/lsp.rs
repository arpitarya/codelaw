//! `codelaw lsp` over real stdio (SR-EDITOR decision 1): the handshake,
//! one diagnostic, a clean shutdown. The protocol itself is tested in
//! `crates/codelaw-lsp/tests/server.rs`.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

#[test]
fn lsp_speaks_over_stdio() {
    let dir = std::env::temp_dir().join(format!("codelaw-lsp-stdio-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = std::fs::canonicalize(&dir).unwrap();
    let file = format!("file://{}/a.js", dir.to_string_lossy().replace('\\', "/"));
    let mut child =
        Command::new(env!("CARGO_BIN_EXE_codelaw")).args(["lsp", "--stdio"]).current_dir(&dir).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for body in [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#.to_owned(),
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#.to_owned(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{file}","languageId":"javascript","version":1,"text":"a == b;\n"}}}}}}"#
        ),
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#.to_owned(),
        r#"{"jsonrpc":"2.0","method":"exit"}"#.to_owned(),
    ] {
        stdin.write_all(frame(&body).as_bytes()).unwrap();
    }
    drop(stdin);
    let mut out = String::new();
    child.stdout.take().unwrap().read_to_string(&mut out).unwrap();
    assert!(child.wait().unwrap().success(), "{out}");
    assert!(out.contains(r#""positionEncoding":"utf-16""#), "{out}");
    assert!(out.contains(r#""serverInfo":{"name":"codelaw""#), "{out}");
    // No rules in this folder: the editor says so, as `check` does.
    assert!(out.contains("textDocument/publishDiagnostics") && out.contains("codelaw/no-rules"), "{out}");
    assert!(out.contains(r#""id":2"#), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}
