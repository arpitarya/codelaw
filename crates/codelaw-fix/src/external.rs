//! External fixers (SR-ARCHITECTURE decision 18): one JSON object on
//! stdin, `{ "edits": [...] }` on stdout, byte offsets into `source`, a
//! timeout per file, the repo root as the working directory.

use codelaw_core::TextEdit;
use codelaw_core::process::kill_group;
use codelaw_match::Match;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
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

/// How long the fixer's output may stay open after it exits: a process it
/// started and left running can hold the pipe.
const CLOSE_GRACE: Duration = Duration::from_millis(500);

/// Read a pipe to its end on a thread; the result arrives on the channel.
fn drain(mut pipe: impl Read + Send + 'static) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    rx
}

/// The last line of what the fixer wrote to stderr, for the error message.
fn stderr_tail(rx: &mpsc::Receiver<Vec<u8>>) -> String {
    let bytes = rx.recv_timeout(CLOSE_GRACE).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes);
    text.lines().rev().find(|l| !l.trim().is_empty()).map(|l| format!(": {}", l.trim())).unwrap_or_default()
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
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd
        .current_dir(call.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start `{}`: {e}", call.command))?;
    let mut stdin = child.stdin.take().expect("piped");
    std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    // Both pipes are drained as the fixer runs, so a fixer that writes a
    // lot to stderr never blocks on a full pipe.
    let stdout = drain(child.stdout.take().expect("piped"));
    let stderr = drain(child.stderr.take().expect("piped"));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= call.timeout => {
                kill_group(&mut child);
                let _ = child.wait();
                return Err(format!("`{}` timed out after {:?}", call.command, call.timeout));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(2)),
            Err(e) => {
                kill_group(&mut child);
                let _ = child.wait();
                return Err(format!("`{}` failed: {e}", call.command));
            }
        }
    };
    // The fixer is done; whatever it left running goes with it, so its
    // pipes close and nothing waits on them.
    kill_group(&mut child);
    let out = match stdout.recv_timeout(CLOSE_GRACE) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => return Err(format!("`{}` exited but a process it started kept its output open", call.command)),
    };
    if !status.success() {
        return Err(format!("`{}` exited with {status}{}", call.command, stderr_tail(&stderr)));
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
    edits.sort_by_key(|e| (e.start, e.end));
    Ok(edits)
}
