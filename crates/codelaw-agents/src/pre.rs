//! `codelaw hook --pre` (SR-AGENT-SURFACES decision 9, the Gate): an
//! agent's proposed write, judged before it reaches disk.
//!
//! 1. **Rebuild** each file the write touches in memory, from the file on
//!    disk and the payload, exactly as the agent's tool would build it.
//!    When the tool would fail (old text missing or not unique), the file
//!    gets no verdict: the tool fails on its own.
//! 2. **Judge only what the write adds:** both texts through MCP
//!    `check_snippet`'s engine call, with the `ci` profile; a finding is new
//!    when its rule and message occur more often after than before.
//! 3. **Answer** in the agent's own shape: a new `error` refuses the write
//!    with each fix; new warnings ride along as context where the agent has
//!    a channel for it.
//!
//! Every failure of codelaw's own — a payload it cannot read, a config or
//! rule error, a panic, a timeout — **allows** the write with one line on
//! stderr and exit 0: Copilot treats any non-zero exit as a deny.

use clap::ValueEnum;
use codelaw_core::{Diagnostic, FixClass, Language, Severity};
use codelaw_engine::{EngineOptions, Project};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

/// The agents whose pre-write payload and answer `hook --pre` knows
/// (decision 2's pre-write table).
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum PreAgent {
    Claude,
    Codex,
    Copilot,
    Vscode,
    Kiro,
    Cursor,
    Gemini,
    Windsurf,
}

impl PreAgent {
    /// The agent a payload comes from, when `--agent` does not say.
    pub fn detect(payload: &Value) -> PreAgent {
        let event = |key: &str| payload.get(key).and_then(Value::as_str).unwrap_or_default();
        if event("hook_event_name") == "BeforeTool" {
            PreAgent::Gemini
        } else if event("hook_event_name") == "pre_write_code" || event("agent_action_name") == "pre_write_code" {
            PreAgent::Windsurf
        } else if payload.get("toolArgs").is_some() {
            PreAgent::Copilot
        } else {
            PreAgent::Claude
        }
    }
}

/// One file as the write would leave it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rebuilt {
    pub path: PathBuf,
    /// The file on disk; `None` when the write creates it.
    pub before: Option<String>,
    pub after: String,
}

/// One replacement of an edit tool.
#[derive(Clone, Debug, Default)]
struct Replace {
    old: String,
    new: String,
    all: bool,
    expected: Option<usize>,
}

fn str_of<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| v.get(*k).and_then(Value::as_str))
}

fn replace_of(v: &Value) -> Option<Replace> {
    let old = str_of(v, &["old_string", "old_str"])?;
    Some(Replace {
        old: old.to_owned(),
        new: str_of(v, &["new_string", "new_str"]).unwrap_or_default().to_owned(),
        all: v.get("replace_all").or_else(|| v.get("allow_multiple")).and_then(Value::as_bool).unwrap_or(false),
        expected: v.get("expected_replacements").and_then(Value::as_u64).map(|n| n as usize),
    })
}

fn crlf(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\n', "\r\n")
}

/// `text` with one replacement made as the tool makes it; `None` when the
/// tool would refuse it (no match, or not the one match it needs).
fn apply_replace(text: &str, r: &Replace) -> Option<String> {
    let once = |old: &str, new: &str| -> Option<String> {
        if old.is_empty() {
            return None;
        }
        let n = text.matches(old).count();
        let ok = match r.expected {
            Some(k) => n == k && n > 0,
            None => n == 1 || (n > 1 && r.all),
        };
        ok.then(|| text.replace(old, new))
    };
    once(&r.old, &r.new).or_else(|| {
        // CRLF kept: an LF `old` against a CRLF file matches and inserts
        // with CRLF.
        (text.contains("\r\n") && r.old.contains('\n') && !r.old.contains("\r\n")).then(|| once(&crlf(&r.old), &crlf(&r.new))).flatten()
    })
}

/// A path from a payload, against its `cwd`.
fn resolve(name: &str, cwd: Option<&Path>) -> PathBuf {
    let p = PathBuf::from(name);
    match cwd {
        Some(c) if !p.is_absolute() => c.join(p),
        _ => p,
    }
}

/// The tool's arguments: `tool_input` (Claude Code, Codex, Gemini, Cursor,
/// Kiro), `toolArgs` as an object or a JSON string (Copilot, VS Code), or
/// `tool_info` (Windsurf).
fn arguments(payload: &Value) -> Option<Value> {
    if let Some(a) = payload.get("tool_input").filter(|a| a.is_object()) {
        return Some(a.clone());
    }
    match payload.get("toolArgs") {
        Some(Value::String(s)) => return serde_json::from_str(s).ok(),
        Some(a @ Value::Object(_)) => return Some(a.clone()),
        _ => {}
    }
    payload.get("tool_info").filter(|a| a.is_object()).cloned()
}

/// The `apply_patch` envelope in the arguments, if there is one: a string
/// under `command`, `input` or `patch`, or the last word of a `command` list.
fn patch_text(args: &Value) -> Option<String> {
    let text = str_of(args, &["command", "input", "patch"])
        .map(str::to_owned)
        .or_else(|| args.get("command").and_then(Value::as_array).and_then(|a| a.last()).and_then(Value::as_str).map(str::to_owned))?;
    text.contains("*** Begin Patch").then_some(text)
}

/// Rebuild every file the payload's write touches. `Err` is a payload
/// codelaw cannot read (the caller allows the write and says why); a file
/// the tool itself would fail on is left out.
pub fn rebuild(payload: &Value) -> Result<Vec<Rebuilt>, String> {
    let args = arguments(payload).ok_or("the payload has no tool arguments")?;
    let cwd = payload.get("cwd").and_then(Value::as_str).map(PathBuf::from);
    if let Some(patch) = patch_text(&args) {
        return rebuild_patch(&patch, cwd.as_deref());
    }
    let name = str_of(&args, &["file_path", "path", "filePath", "absolute_path"]).ok_or("the payload names no file")?;
    let path = resolve(name, cwd.as_deref());
    let before = match std::fs::read(&path) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(t) => Some(t),
            Err(_) => return Ok(Vec::new()), // not text: not codelaw's
        },
        Err(_) => None,
    };
    let after = if let Some(whole) = str_of(&args, &["content", "file_text", "text"]) {
        Some(whole.to_owned())
    } else if let Some(edits) = args.get("edits").and_then(Value::as_array) {
        let mut text = before.clone().unwrap_or_default();
        let mut ok = true;
        for e in edits {
            match replace_of(e).and_then(|r| step(before.is_some(), &text, &r)) {
                Some(t) => text = t,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        ok.then_some(text)
    } else if let Some(r) = replace_of(&args) {
        step(before.is_some(), before.as_deref().unwrap_or_default(), &r)
    } else {
        return Err("the payload's tool is neither a whole-file write nor a replacement".into());
    };
    Ok(after.map(|after| Rebuilt { path, before, after }).into_iter().collect())
}

/// One replacement on the text so far; an empty `old` on a file that does
/// not exist yet creates it (Gemini `replace`).
fn step(exists: bool, text: &str, r: &Replace) -> Option<String> {
    if r.old.is_empty() && !exists && text.is_empty() {
        return Some(r.new.clone());
    }
    apply_replace(text, r)
}

/// One `*** Update File` chunk: the lines it expects and the lines it
/// writes, and the `@@` context line it is anchored after.
#[derive(Debug, Default)]
struct Chunk {
    anchor: Option<String>,
    old: Vec<String>,
    new: Vec<String>,
    eof: bool,
}

/// The files of an `apply_patch` envelope, rebuilt.
fn rebuild_patch(patch: &str, cwd: Option<&Path>) -> Result<Vec<Rebuilt>, String> {
    let mut out = Vec::new();
    let mut lines = patch.lines().map(|l| l.strip_suffix('\r').unwrap_or(l)).peekable();
    while let Some(l) = lines.next() {
        if let Some(name) = l.strip_prefix("*** Add File: ") {
            let mut text = String::new();
            while let Some(next) = lines.peek().filter(|n| !n.starts_with("*** ")) {
                text.push_str(next.strip_prefix('+').unwrap_or(next));
                text.push('\n');
                lines.next();
            }
            out.push(Rebuilt { path: resolve(name.trim(), cwd), before: None, after: text });
        } else if let Some(name) = l.strip_prefix("*** Update File: ") {
            let path = resolve(name.trim(), cwd);
            let mut target = path.clone();
            let mut chunks: Vec<Chunk> = Vec::new();
            while let Some(next) = lines.peek().copied() {
                if let Some(to) = next.strip_prefix("*** Move to: ") {
                    target = resolve(to.trim(), cwd);
                } else if next == "*** End of File" {
                    if let Some(c) = chunks.last_mut() {
                        c.eof = true;
                    }
                } else if next.starts_with("*** ") {
                    break;
                } else if let Some(anchor) = next.strip_prefix("@@") {
                    let anchor = anchor.trim();
                    chunks.push(Chunk { anchor: (!anchor.is_empty()).then(|| anchor.to_owned()), ..Chunk::default() });
                } else {
                    if chunks.is_empty() {
                        chunks.push(Chunk::default());
                    }
                    let c = chunks.last_mut().expect("a chunk");
                    match next.split_at(next.chars().next().map_or(0, char::len_utf8)) {
                        ("+", rest) => c.new.push(rest.to_owned()),
                        ("-", rest) => c.old.push(rest.to_owned()),
                        (" ", rest) => {
                            c.old.push(rest.to_owned());
                            c.new.push(rest.to_owned());
                        }
                        ("", _) => {
                            c.old.push(String::new());
                            c.new.push(String::new());
                        }
                        _ => return Err(format!("the patch has a line codelaw cannot read: {next:?}")),
                    }
                }
                lines.next();
            }
            let Ok(before) = std::fs::read_to_string(&path) else { continue };
            if let Some(after) = apply_chunks(&before, &chunks) {
                out.push(Rebuilt { path: target, before: Some(before), after });
            }
        }
        // `*** Delete File`, `*** Begin Patch`, `*** End Patch`: nothing
        // to judge.
    }
    Ok(out)
}

/// Where `old` occurs in `lines` at or after `from`: exactly, then
/// ignoring trailing whitespace, then surrounding whitespace (as Codex
/// seeks).
fn seek(lines: &[String], old: &[String], from: usize, eof: bool) -> Option<usize> {
    if old.len() > lines.len() {
        return None;
    }
    let last = lines.len() - old.len();
    let starts: Vec<usize> = if eof { (from..=last).rev().collect() } else { (from..=last).collect() };
    let eq: [fn(&str, &str) -> bool; 3] = [|a, b| a == b, |a, b| a.trim_end() == b.trim_end(), |a, b| a.trim() == b.trim()];
    eq.iter().find_map(|f| starts.iter().copied().find(|&i| old.iter().enumerate().all(|(j, o)| f(&lines[i + j], o))))
}

/// The text with every chunk applied in order; `None` when one does not
/// match (Codex would refuse the patch).
fn apply_chunks(before: &str, chunks: &[Chunk]) -> Option<String> {
    let crlf = before.contains("\r\n");
    let body = before.strip_suffix('\n').unwrap_or(before);
    let mut lines: Vec<String> = if before.is_empty() { Vec::new() } else { body.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).to_owned()).collect() };
    let mut from = 0;
    let mut edits: Vec<(usize, usize, Vec<String>)> = Vec::new();
    for c in chunks {
        if let Some(a) = &c.anchor {
            from = (from..lines.len()).find(|&i| lines[i].trim() == a.trim()).map(|i| i + 1)?;
        }
        let at = if c.old.is_empty() { lines.len() } else { seek(&lines, &c.old, from, c.eof)? };
        edits.push((at, c.old.len(), c.new.clone()));
        from = at + c.old.len();
    }
    for (at, len, new) in edits.into_iter().rev() {
        lines.splice(at..at + len, new);
    }
    let nl = if crlf { "\r\n" } else { "\n" };
    let mut text = lines.join(nl);
    text.push_str(nl);
    Some(text)
}

/// A path as the config sees it: through the real folders, even when the
/// file does not exist yet (macOS `/var` is `/private/var`).
fn real(path: &Path) -> PathBuf {
    if let Ok(p) = std::fs::canonicalize(path) {
        return p;
    }
    let abs = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().map(|d| d.join(path)).unwrap_or_else(|_| path.to_path_buf()) };
    match (abs.parent().and_then(|d| std::fs::canonicalize(d).ok()), abs.file_name()) {
        (Some(dir), Some(name)) => dir.join(name),
        _ => abs,
    }
}

/// The nearest folder that exists, to load the project from.
fn existing_dir(path: &Path) -> PathBuf {
    let mut dir = path.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    while !dir.is_dir() {
        match dir.parent() {
            Some(p) if !p.as_os_str().is_empty() => dir = p.to_path_buf(),
            _ => return PathBuf::from("."),
        }
    }
    dir
}

/// One finding the write adds, with the text it was found in.
#[derive(Clone, Debug)]
pub struct Added {
    pub diagnostic: Diagnostic,
    /// The rebuilt buffer, for the fix's old text.
    text: String,
}

/// What the write adds, over every file it touches.
#[derive(Clone, Debug, Default)]
pub struct Verdict {
    pub errors: Vec<Added>,
    pub warnings: Vec<Added>,
}

fn line_text(text: &str, byte: usize) -> &str {
    let byte = byte.min(text.len());
    let start = text[..byte].rfind('\n').map_or(0, |i| i + 1);
    let end = text[byte..].find('\n').map_or(text.len(), |i| byte + i);
    text[start..end].trim()
}

/// The findings of `after` the write adds over `before`: per rule and
/// message, as many as `after` has more of; among equal ones, those whose
/// line's text `before` does not have.
pub fn added(before: &[Diagnostic], before_text: &str, after: &[Diagnostic], after_text: &str) -> Vec<Diagnostic> {
    let counts = |ds: &[Diagnostic]| {
        let mut m: std::collections::BTreeMap<(String, String), Vec<usize>> = Default::default();
        for (i, d) in ds.iter().enumerate().filter(|(_, d)| !d.rule.starts_with("format/") && !d.rule.starts_with("codelaw/")) {
            m.entry((d.rule.clone(), d.message.clone())).or_default().push(i);
        }
        m
    };
    let was = counts(before);
    let mut out: Vec<Diagnostic> = Vec::new();
    for (key, now) in counts(after) {
        let had = was.get(&key).map(Vec::as_slice).unwrap_or_default();
        let more = now.len().saturating_sub(had.len());
        if more == 0 {
            continue;
        }
        let old_lines: Vec<&str> = had.iter().map(|&i| line_text(before_text, before[i].start_byte)).collect();
        let mut picked: Vec<usize> = now.iter().copied().filter(|&i| !old_lines.contains(&line_text(after_text, after[i].start_byte))).take(more).collect();
        for &i in &now {
            if picked.len() == more {
                break;
            }
            if !picked.contains(&i) {
                picked.push(i);
            }
        }
        out.extend(picked.into_iter().map(|i| after[i].clone()));
    }
    out.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    out
}

/// Judge the rebuilt files with one project load, the `ci` profile unless
/// `profile` names another.
pub fn judge(files: &[Rebuilt], profile: Option<String>) -> Result<Verdict, String> {
    let files: Vec<(PathBuf, &Rebuilt)> = files.iter().filter(|f| Language::from_path(&f.path).is_some()).map(|f| (real(&f.path), f)).collect();
    let Some((first, _)) = files.first() else { return Ok(Verdict::default()) };
    let touched: Vec<PathBuf> = files.iter().map(|(p, _)| p.clone()).collect();
    let options = EngineOptions { touched_paths: touched, profile, profile_surface: Some(("ci", "codelaw hook --pre")), ..EngineOptions::from_env() };
    let project = Project::load(&existing_dir(first), options).map_err(|e| e.to_string())?;
    let mut verdict = Verdict::default();
    for (path, f) in &files {
        let before = match &f.before {
            Some(t) => project.check_snippet(path, t).map_err(|e| e.to_string())?.diagnostics,
            None => Vec::new(),
        };
        let after = project.check_snippet(path, &f.after).map_err(|e| e.to_string())?.diagnostics;
        for d in added(&before, f.before.as_deref().unwrap_or_default(), &after, &f.after) {
            let a = Added { diagnostic: d, text: f.after.clone() };
            match a.diagnostic.severity {
                Severity::Error => verdict.errors.push(a),
                Severity::Warn => verdict.warnings.push(a),
                Severity::Info => {}
            }
        }
    }
    Ok(verdict)
}

/// At most this many findings are spelled out.
const SHOWN: usize = 20;

fn one_line(s: &str, max: usize) -> String {
    let flat = s.replace('\r', "").replace('\n', "\\n");
    if flat.chars().count() <= max { flat } else { format!("{}…", flat.chars().take(max).collect::<String>()) }
}

/// The fix, as the agent can apply it to what it writes (L4): the edit's
/// old and new text, or the manual steps.
fn fix_text(a: &Added) -> String {
    let d = &a.diagnostic;
    if !d.edits.is_empty() && matches!(d.fix, FixClass::Auto | FixClass::Suggest) {
        let edits: Vec<String> = d
            .edits
            .iter()
            .take(3)
            .map(|e| {
                let old = a.text.get(e.start..e.end).unwrap_or_default();
                match (old.is_empty(), e.text.is_empty()) {
                    (true, _) => format!("insert \"{}\"", one_line(&e.text, 60)),
                    (false, true) => format!("remove \"{}\"", one_line(old, 60)),
                    (false, false) => format!("replace \"{}\" with \"{}\"", one_line(old, 60), one_line(&e.text, 60)),
                }
            })
            .collect();
        return format!("fix: {} — {}", d.fix.name(), edits.join("; "));
    }
    if d.steps.is_empty() {
        return format!("fix: manual — see `codelaw explain {}` for the steps", d.rule);
    }
    format!("fix: manual — {}", d.steps.iter().enumerate().map(|(i, s)| format!("{}. {s}", i + 1)).collect::<Vec<_>>().join(" "))
}

fn lines(found: &[Added]) -> String {
    let mut out = String::new();
    for a in found.iter().take(SHOWN) {
        let d = &a.diagnostic;
        out.push_str(&format!("{}:{}:{} {} {} {} — {}\n", d.file, d.start.line, d.start.column, d.severity.name(), d.rule, d.message, fix_text(a)));
    }
    if found.len() > SHOWN {
        out.push_str(&format!("… and {} more\n", found.len() - SHOWN));
    }
    out
}

fn plural(n: usize, one: &str) -> String {
    if n == 1 { format!("1 {one}") } else { format!("{n} {one}s") }
}

/// Why the write is refused, for the agent.
pub fn deny_reason(v: &Verdict) -> String {
    let mut out = format!(
        "codelaw refused this write: it adds {} that CI would fail. Make the change again without {}; never suppress or disable a rule to pass.\n",
        plural(v.errors.len(), "error"),
        if v.errors.len() == 1 { "it" } else { "them" }
    );
    out.push_str(&lines(&v.errors));
    if !v.warnings.is_empty() {
        out.push_str(&format!("It also adds {}:\n", plural(v.warnings.len(), "warning")));
        out.push_str(&lines(&v.warnings));
    }
    out
}

/// The warnings an allowed write adds, as context for the agent.
pub fn warning_context(v: &Verdict) -> String {
    format!("codelaw: this write adds {}; fix them now, or the post-write hook reports them again.\n{}", plural(v.warnings.len(), "warning"), lines(&v.warnings))
}

/// What `hook --pre` prints and exits with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub stdout: String,
    pub stderr: String,
    pub code: u8,
}

impl Answer {
    fn allow() -> Answer {
        Answer { stdout: String::new(), stderr: String::new(), code: 0 }
    }
    fn json(v: Value) -> Answer {
        Answer { stdout: format!("{v}\n"), stderr: String::new(), code: 0 }
    }
}

/// The verdict in the agent's own shape. An allow never says `allow`: on
/// Claude Code that would skip the user's permission prompt.
pub fn answer(agent: PreAgent, v: &Verdict) -> Answer {
    if !v.errors.is_empty() {
        let reason = deny_reason(v);
        return match agent {
            PreAgent::Claude | PreAgent::Codex => Answer::json(
                json!({ "hookSpecificOutput": { "hookEventName": "PreToolUse", "permissionDecision": "deny", "permissionDecisionReason": reason } }),
            ),
            PreAgent::Copilot | PreAgent::Vscode => Answer::json(json!({ "permissionDecision": "deny", "permissionDecisionReason": reason })),
            PreAgent::Gemini => Answer::json(json!({ "decision": "deny", "reason": reason })),
            PreAgent::Cursor => Answer::json(json!({ "permission": "deny", "agent_message": reason, "user_message": format!("codelaw refused a write that adds {}", plural(v.errors.len(), "error")) })),
            PreAgent::Kiro | PreAgent::Windsurf => Answer { stdout: String::new(), stderr: reason, code: 2 },
        };
    }
    match agent {
        PreAgent::Claude | PreAgent::Codex if !v.warnings.is_empty() => {
            Answer::json(json!({ "hookSpecificOutput": { "hookEventName": "PreToolUse", "additionalContext": warning_context(v) } }))
        }
        _ => Answer::allow(),
    }
}

/// The whole decision for one payload: `Err` is codelaw's own failure,
/// which the caller turns into an allow.
pub fn decide(payload: &str, agent: Option<PreAgent>, profile: Option<String>) -> Result<Answer, String> {
    let v: Value = serde_json::from_str(payload).map_err(|_| "the hook payload on stdin is not JSON".to_owned())?;
    let agent = agent.unwrap_or_else(|| PreAgent::detect(&v));
    let files = rebuild(&v)?;
    Ok(answer(agent, &judge(&files, profile)?))
}

/// Run `f` on a thread of its own and wait at most `timeout`; a panic, an
/// error and a timeout are each an allow with the reason on stderr.
pub fn guarded(timeout: Duration, f: impl FnOnce() -> Result<Answer, String> + Send + 'static) -> Answer {
    let (tx, rx) = channel();
    let spawned = std::thread::Builder::new().stack_size(64 << 20).spawn(move || {
        let _ = tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)));
    });
    let why = match spawned {
        Err(e) => format!("could not start ({e})"),
        Ok(_) => match rx.recv_timeout(timeout) {
            Ok(Ok(Ok(answer))) => return answer,
            Ok(Ok(Err(e))) => e,
            Ok(Err(_)) => "the engine panicked".to_owned(),
            Err(RecvTimeoutError::Timeout) => format!("no verdict within {} ms", timeout.as_millis()),
            Err(RecvTimeoutError::Disconnected) => "the engine stopped".to_owned(),
        },
    };
    Answer { stdout: String::new(), stderr: format!("codelaw hook --pre: {why}; the write is allowed\n"), code: 0 }
}

/// `codelaw hook --pre`: read the payload, answer, exit as the answer says.
pub fn hook_pre(payload: String, agent: Option<PreAgent>, profile: Option<String>, timeout_ms: u64) -> ExitCode {
    let a = guarded(Duration::from_millis(timeout_ms), move || decide(&payload, agent, profile));
    print!("{}", a.stdout);
    eprint!("{}", a.stderr);
    ExitCode::from(a.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(old: &str, new: &str) -> Replace {
        Replace { old: old.into(), new: new.into(), ..Replace::default() }
    }

    #[test]
    fn a_replacement_matches_as_the_tool_matches() {
        assert_eq!(apply_replace("a b a", &r("b", "c")).as_deref(), Some("a c a"));
        assert_eq!(apply_replace("a b a", &r("a", "c")), None, "not unique");
        assert_eq!(apply_replace("a b a", &Replace { all: true, ..r("a", "c") }).as_deref(), Some("c b c"));
        assert_eq!(apply_replace("a b a", &Replace { expected: Some(2), ..r("a", "c") }).as_deref(), Some("c b c"));
        assert_eq!(apply_replace("a b a", &Replace { expected: Some(3), ..r("a", "c") }), None);
        assert_eq!(apply_replace("x", &r("y", "z")), None, "not found");
        assert_eq!(apply_replace("one\r\ntwo\r\n", &r("one\ntwo", "1\n2")).as_deref(), Some("1\r\n2\r\n"), "CRLF kept");
    }

    #[test]
    fn every_agent_shape_rebuilds() {
        let dir = std::env::temp_dir().join(format!("codelaw-pre-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ts"), "const a = 1;\nconst b = 2;\n").unwrap();
        let cwd = dir.to_string_lossy().into_owned();
        let one = |payload: Value| rebuild(&payload).unwrap();
        // Claude Code `Write`, `Edit`, `MultiEdit`.
        let w = one(json!({ "cwd": cwd, "tool_name": "Write", "tool_input": { "file_path": "new.ts", "content": "x;\n" } }));
        assert_eq!((w[0].before.as_deref(), w[0].after.as_str()), (None, "x;\n"));
        let e = one(json!({ "cwd": cwd, "tool_name": "Edit", "tool_input": { "file_path": "a.ts", "old_string": "= 2", "new_string": "= 3" } }));
        assert_eq!(e[0].after, "const a = 1;\nconst b = 3;\n");
        let m = one(json!({ "cwd": cwd, "tool_input": { "file_path": "a.ts", "edits": [{ "old_string": "a = 1", "new_string": "a = 9" }, { "old_string": "a = 9", "new_string": "a = 8" }] } }));
        assert_eq!(m[0].after, "const a = 8;\nconst b = 2;\n");
        assert!(one(json!({ "cwd": cwd, "tool_input": { "file_path": "a.ts", "old_string": "nope", "new_string": "x" } })).is_empty(), "the tool would fail");
        // Gemini `replace` with `allow_multiple`; `replace` creating a file.
        let g = one(json!({ "cwd": cwd, "hook_event_name": "BeforeTool", "tool_name": "replace", "tool_input": { "file_path": "a.ts", "old_string": "const", "new_string": "let", "allow_multiple": true } }));
        assert_eq!(g[0].after, "let a = 1;\nlet b = 2;\n");
        let g = one(json!({ "cwd": cwd, "tool_input": { "file_path": "made.ts", "old_string": "", "new_string": "y;\n" } }));
        assert_eq!(g[0].after, "y;\n");
        // Copilot: `toolArgs` as a JSON string.
        let c = one(json!({ "cwd": cwd, "toolName": "edit", "toolArgs": "{\"path\":\"a.ts\",\"old_str\":\"b = 2\",\"new_str\":\"b = 0\"}" }));
        assert_eq!(c[0].after, "const a = 1;\nconst b = 0;\n");
        // Windsurf: `tool_info.edits`.
        let ws = one(json!({ "agent_action_name": "pre_write_code", "tool_info": { "file_path": dir.join("a.ts"), "edits": [{ "old_string": "1", "new_string": "4" }] } }));
        assert_eq!(ws[0].after, "const a = 4;\nconst b = 2;\n");
        // Codex `apply_patch`: update with context, move, add, delete.
        let patch = "*** Begin Patch\n*** Update File: a.ts\n*** Move to: moved.ts\n@@\n const a = 1;\n-const b = 2;\n+const b = 5;\n*** Add File: added.ts\n+z;\n*** Delete File: gone.ts\n*** End Patch";
        let x = one(json!({ "cwd": cwd, "tool_name": "apply_patch", "tool_input": { "command": patch } }));
        assert_eq!(x.len(), 2);
        assert_eq!((x[0].path.file_name().unwrap().to_str(), x[0].after.as_str()), (Some("moved.ts"), "const a = 1;\nconst b = 5;\n"));
        assert_eq!((x[1].before.as_deref(), x[1].after.as_str()), (None, "z;\n"));
        let listed = one(json!({ "cwd": cwd, "tool_input": { "command": ["apply_patch", "*** Begin Patch\n*** Update File: a.ts\n@@\n-  const a = 1;  \n+const a = 7;\n*** End Patch"] } }));
        assert_eq!(listed[0].after, "const a = 7;\nconst b = 2;\n", "matched ignoring whitespace, as Codex seeks");
        // Payloads codelaw cannot read.
        assert!(rebuild(&json!({ "tool_input": { "command": "ls" } })).is_err());
        assert!(rebuild(&json!({ "hook_event_name": "PreToolUse" })).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_shape_is_read_off_the_payload() {
        assert_eq!(PreAgent::detect(&json!({ "hook_event_name": "BeforeTool" })), PreAgent::Gemini);
        assert_eq!(PreAgent::detect(&json!({ "agent_action_name": "pre_write_code" })), PreAgent::Windsurf);
        assert_eq!(PreAgent::detect(&json!({ "toolName": "edit", "toolArgs": "{}" })), PreAgent::Copilot);
        assert_eq!(PreAgent::detect(&json!({ "hook_event_name": "PreToolUse" })), PreAgent::Claude);
    }

    #[test]
    fn codelaws_own_failure_allows_the_write() {
        let a = guarded(Duration::from_secs(5), || panic!("boom"));
        assert_eq!((a.code, a.stdout.as_str()), (0, ""));
        assert!(a.stderr.contains("panicked; the write is allowed"), "{}", a.stderr);
        let a = guarded(Duration::from_millis(20), || {
            std::thread::sleep(Duration::from_millis(500));
            Ok(Answer::allow())
        });
        assert!(a.stderr.contains("no verdict within 20 ms"), "{}", a.stderr);
        let a = guarded(Duration::from_secs(5), || decide("not json", None, None));
        assert_eq!(a.code, 0);
        assert!(a.stderr.contains("not JSON"), "{}", a.stderr);
    }

    fn diag(rule: &str, at: usize, line: u32) -> Diagnostic {
        Diagnostic {
            rule: rule.into(),
            severity: Severity::Error,
            file: "a.ts".into(),
            start_byte: at,
            end_byte: at,
            start: codelaw_core::Position { line, column: 1 },
            end: codelaw_core::Position { line, column: 1 },
            message: "m".into(),
            fix: FixClass::Auto,
            edits: Vec::new(),
            steps: Vec::new(),
            source: "p".into(),
            note: None,
            data: None,
        }
    }

    #[test]
    fn only_what_the_write_adds_counts() {
        let before = "debugger;\nx;\n";
        let after = "y;\ndebugger;\nx;\ndebugger; // new\n";
        let was = [diag("js/no-debugger", 0, 1)];
        let now = [diag("js/no-debugger", 3, 2), diag("js/no-debugger", 16, 4), diag("format/unformatted", 0, 1)];
        let new = added(&was, before, &now, after);
        assert_eq!(new.len(), 1, "{new:?}");
        assert_eq!(new[0].start.line, 4, "the moved one is the old one");
        assert!(added(&was, before, &was, before).is_empty());
    }
}
