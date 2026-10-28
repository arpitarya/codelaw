//! Type-aware TypeScript (SR-TYPES): the project's own TypeScript (≥ 7.1)
//! found, and the `codelaw-types` helper that asks it.
//!
//! - **The helper** is codelaw's own JavaScript ([`HELPER`]), carried in the
//!   binary — the same bytes in the npm and the PyPI package — and run as
//!   `node --input-type=module --eval <helper> -- <typescript dir>`, so
//!   nothing is written to disk and stdin stays free for the protocol
//!   (decision 2).
//! - **The protocol** is codelaw's: one JSON object per line each way, the
//!   protocol version in the first exchange; a helper that answers another
//!   version is refused (decision 3).
//! - **TypeScript** is the `typescript` package resolved from the file's
//!   folder up to the project root; codelaw never installs, downloads or
//!   pins one (decision 5). Older than 7.1, the type rules are skipped
//!   (decision 6); outside [`TESTED`], they run with a warning (decision 7).

use codelaw_core::process::kill_group;
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// The helper's source (decision 2).
pub const HELPER: &str = include_str!("helper.mjs");

/// The protocol version both sides speak (decision 3).
pub const PROTOCOL: u32 = 1;

/// SR-TYPES decision 4's vocabulary: the only questions a `type:` clause
/// may ask. A new one is an amendment of the record.
pub const QUESTIONS: &[&str] = &["promise-like", "thenable", "any", "nullable", "union", "deprecated", "never"];

/// The TypeScript versions the fixtures pass on (decision 7).
pub const TESTED: &str = ">=7.1.0-beta, <7.2.0";

/// The `native:` id of `ts/type-error` (decision 9): TypeScript's own
/// diagnostics, which the helper answers, not a Rust rule.
pub const TYPE_ERROR_NATIVE: &str = "ts-type-error";

/// How long one answer may take. The first loads the program.
const TIMEOUT: Duration = Duration::from_secs(300);

/// The `typescript` package that governs `dir`: `node_modules/typescript`
/// in `dir` or a folder above it, up to `root` (decision 5).
pub fn find_typescript(dir: &Path, root: &Path) -> Option<PathBuf> {
    let mut d = Some(dir);
    while let Some(here) = d {
        let candidate = here.join("node_modules").join("typescript");
        if candidate.join("package.json").is_file() {
            return Some(candidate);
        }
        if here == root || !here.starts_with(root) {
            break;
        }
        d = here.parent();
    }
    None
}

/// The `version` a package's `package.json` names.
pub fn package_version(package: &Path) -> Option<String> {
    let text = std::fs::read_to_string(package.join("package.json")).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    v["version"].as_str().map(str::to_owned)
}

/// Whether a TypeScript version has the 7.1 API: 7.1 or later, its
/// pre-releases (`7.1.0-beta`, `7.1.0-rc`, `7.1.0-dev.…`) included.
pub fn has_api(version: &str) -> bool {
    semver::Version::parse(version).is_ok_and(|v| (v.major, v.minor) >= (7, 1))
}

/// Whether a version is inside [`TESTED`].
pub fn tested(version: &str) -> bool {
    let req = semver::VersionReq::parse(TESTED).expect("TESTED parses");
    semver::Version::parse(version).is_ok_and(|v| req.matches(&v))
}

/// One question about the node at `start..end` (UTF-8 byte offsets).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Question {
    pub start: usize,
    pub end: usize,
    pub question: &'static str,
}

/// One of TypeScript's own diagnostics, offsets in UTF-8 bytes.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TsDiagnostic {
    pub start: usize,
    pub end: usize,
    pub code: u32,
    /// `error` or `warning`.
    pub category: String,
    pub message: String,
}

/// What the helper answered about one file.
#[derive(Clone, Debug, Default)]
pub struct Answers {
    /// One per question, in order; `None` when no node spans it.
    pub answers: Vec<Option<bool>>,
    pub diagnostics: Vec<TsDiagnostic>,
}

/// A running helper.
pub struct Helper {
    child: Child,
    stdin: ChildStdin,
    lines: mpsc::Receiver<String>,
    /// The TypeScript version it loaded.
    pub typescript: String,
}

impl Helper {
    /// Start the helper with `node` on the TypeScript at `typescript`, in
    /// `cwd`, and greet it.
    pub fn start(node: &Path, typescript: &Path, cwd: &Path) -> Result<Helper, String> {
        let mut cmd = Command::new(node);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd
            .args(["--input-type=module", "--eval", HELPER, "--"])
            .arg(typescript)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start Node: {e}"))?;
        let stdin = child.stdin.take().expect("piped");
        let stdout = child.stdout.take().expect("piped");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let mut helper = Helper { child, stdin, lines: rx, typescript: String::new() };
        let hello = helper.ask(&json!({ "op": "hello", "protocol": PROTOCOL, "cwd": cwd }))?;
        if hello["protocol"].as_u64() != Some(u64::from(PROTOCOL)) {
            return Err(format!("the type helper speaks protocol {}, codelaw {PROTOCOL}", hello["protocol"]));
        }
        helper.typescript = hello["typescript"].as_str().unwrap_or_default().to_owned();
        Ok(helper)
    }

    fn ask(&mut self, request: &Value) -> Result<Value, String> {
        let mut line = serde_json::to_string(request).map_err(|e| e.to_string())?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).and_then(|_| self.stdin.flush()).map_err(|e| format!("the type helper stopped: {e}"))?;
        let answer = match self.lines.recv_timeout(TIMEOUT) {
            Ok(l) => l,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                kill_group(&mut self.child);
                return Err(format!("the type helper did not answer within {TIMEOUT:?}"));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err("the type helper exited".into()),
        };
        let v: Value = serde_json::from_str(&answer).map_err(|e| format!("the type helper answered something that is not JSON: {e}"))?;
        match v["error"].as_str() {
            Some(e) => Err(e.to_owned()),
            None => Ok(v),
        }
    }

    /// Ask about one file: `text` is what the engine checks, which wins
    /// over the file on disk.
    pub fn check(&mut self, file: &Path, text: &str, questions: &[Question], diagnostics: bool) -> Result<Answers, String> {
        let qs: Vec<Value> = questions.iter().map(|q| json!({ "start": q.start, "end": q.end, "q": q.question })).collect();
        let v = self.ask(&json!({ "op": "check", "file": file, "text": text, "questions": qs, "diagnostics": diagnostics }))?;
        let answers: Vec<Option<bool>> = v["answers"].as_array().map(|a| a.iter().map(Value::as_bool).collect()).unwrap_or_default();
        if answers.len() != questions.len() {
            return Err(format!("the type helper answered {} of {} questions", answers.len(), questions.len()));
        }
        let diagnostics = match v.get("diagnostics") {
            Some(d) => serde_json::from_value(d.clone()).map_err(|e| format!("the type helper's diagnostics: {e}"))?,
            None => Vec::new(),
        };
        Ok(Answers { answers, diagnostics })
    }

    /// Tell a warm program what changed on disk (decision 8).
    pub fn changed(&mut self, changed: &[PathBuf], created: &[PathBuf], deleted: &[PathBuf]) -> Result<(), String> {
        self.ask(&json!({ "op": "changed", "files": changed, "created": created, "deleted": deleted })).map(|_| ())
    }

    /// Have a warm program re-read every file of it that changed on disk
    /// since it last looked (decision 8).
    pub fn refresh(&mut self) -> Result<(), String> {
        self.ask(&json!({ "op": "refresh" })).map(|_| ())
    }
}

impl Drop for Helper {
    fn drop(&mut self) {
        let _ = self.stdin.write_all(b"{\"op\":\"close\"}\n");
        let _ = self.stdin.flush();
        // It disposes its snapshot and closes the API, then exits.
        for _ in 0..50 {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        kill_group(&mut self.child);
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_api_starts_at_7_1_pre_releases_included() {
        for v in ["7.1.0", "7.1.0-beta", "7.1.0-rc", "7.1.0-dev.20261027.1", "7.2.3", "8.0.0"] {
            assert!(has_api(v), "{v}");
        }
        for v in ["7.0.2", "5.9.3", "7.0.1-rc", "not a version"] {
            assert!(!has_api(v), "{v}");
        }
    }

    #[test]
    fn the_tested_range_holds_7_1() {
        assert!(tested("7.1.0-beta"));
        assert!(tested("7.1.0"));
        assert!(tested("7.1.4"));
        assert!(!tested("7.2.0"));
        assert!(!tested("7.0.2"));
    }

    #[test]
    fn typescript_is_found_from_the_file_up_to_the_root_and_no_further() {
        let tmp = std::env::temp_dir().join(format!("codelaw-types-find-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let root = tmp.join("repo");
        let ts = root.join("node_modules").join("typescript");
        std::fs::create_dir_all(&ts).unwrap();
        std::fs::write(ts.join("package.json"), r#"{"version":"7.1.0-beta"}"#).unwrap();
        let deep = root.join("packages").join("a").join("src");
        std::fs::create_dir_all(&deep).unwrap();
        assert_eq!(find_typescript(&deep, &root), Some(ts.clone()));
        assert_eq!(package_version(&ts).as_deref(), Some("7.1.0-beta"));
        // Above the root is not the project's.
        assert_eq!(find_typescript(&deep, &root.join("packages")), None);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn the_helper_names_the_protocol_and_every_question() {
        assert!(HELPER.contains(&format!("const PROTOCOL = {PROTOCOL};")));
        for q in QUESTIONS {
            assert!(HELPER.contains(&format!("\"{q}\"")), "the helper answers `{q}`");
        }
    }
}
