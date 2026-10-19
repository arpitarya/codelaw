//! External fixers (SR-ARCHITECTURE decision 18): one JSON object on
//! stdin, `{ "edits": [...] }` on stdout, byte offsets into `source`, a
//! timeout per file, the repo root as the working directory.

use codelaw_core::TextEdit;
use codelaw_match::Match;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Serialize)]
struct Range {
    start: usize,
    end: usize,
}

#[derive(Serialize)]
struct Capture<'a> {
    range: Range,
    text: &'a str,
}

#[derive(Serialize)]
struct MatchOut<'a> {
    range: Range,
    captures: BTreeMap<&'a str, Capture<'a>>,
}

#[derive(Serialize)]
struct Request<'a> {
    schema: u32,
    rule: &'a str,
    file: &'a str,
    language: &'a str,
    source: &'a str,
    matches: Vec<MatchOut<'a>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RangeIn {
    Pair([usize; 2]),
    Object { start: usize, end: usize },
}

#[derive(Deserialize)]
struct EditIn {
    range: RangeIn,
    text: String,
}

#[derive(Deserialize)]
struct Response {
    edits: Vec<EditIn>,
}

pub struct ExternalCall<'a> {
    pub command: &'a str,
    pub cwd: &'a Path,
    pub timeout: Duration,
    pub rule: &'a str,
    pub file: &'a str,
    pub language: &'a str,
    pub source: &'a str,
    pub matches: &'a [Match],
}

/// The request JSON, exactly as the fixer receives it.
pub fn request_json(call: &ExternalCall<'_>) -> String {
    let matches = call
        .matches
        .iter()
        .map(|m| MatchOut {
            range: Range { start: m.start, end: m.end },
            captures: m
                .single
                .iter()
                .map(|(k, c)| (k.as_str(), Capture { range: Range { start: c.start, end: c.end }, text: c.text.as_str() }))
                .collect(),
        })
        .collect();
    let req = Request { schema: 1, rule: call.rule, file: call.file, language: call.language, source: call.source, matches };
    serde_json::to_string(&req).expect("serialisable")
}

/// Run an external fixer. Any failure is an `Err`; the caller reports the
/// instances as `manual` with the rule's steps.
pub fn run_external(call: &ExternalCall<'_>) -> Result<Vec<TextEdit>, String> {
    let input = request_json(call);
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(call.command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(call.command);
        c
    };
    let mut child = cmd
        .current_dir(call.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start `{}`: {e}", call.command))?;
    let mut stdin = child.stdin.take().expect("piped");
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let mut stdout = child.stdout.take().expect("piped");
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= call.timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("`{}` timed out after {:?}", call.command, call.timeout));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(2)),
            Err(e) => return Err(format!("`{}` failed: {e}", call.command)),
        }
    };
    let _ = writer.join();
    let out = reader.join().unwrap_or_default();
    if !status.success() {
        return Err(format!("`{}` exited with {status}", call.command));
    }
    let resp: Response = serde_json::from_str(&out).map_err(|e| format!("`{}` printed invalid JSON: {e}", call.command))?;
    let mut edits = Vec::new();
    for e in resp.edits {
        let (start, end) = match e.range {
            RangeIn::Pair([s, e]) => (s, e),
            RangeIn::Object { start, end } => (start, end),
        };
        if start > end || end > call.source.len() {
            return Err(format!("`{}` returned an edit outside the file", call.command));
        }
        edits.push(TextEdit::new(start, end, e.text));
    }
    edits.sort_by(|a, b| (a.start, a.end).cmp(&(b.start, b.end)));
    Ok(edits)
}
