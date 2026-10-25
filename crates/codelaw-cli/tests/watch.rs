//! `codelaw check --watch` and `fix --watch` (W-44; SR-CLI-MCP decision
//! 2a, SR-ARCHITECTURE decision 22a), through the binary: a save re-checks
//! the file and the files that depend on it, a config edit changes the next
//! verdict, every run is one JSON object on one line and is the very object
//! `check --format json` gives for the same files, and Ctrl-C exits 0.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

const CONFIG: &str = "extends = [\"codelaw:js/project\"]\npreview = true\n";

/// A project with the js pack installed: `main.js` uses both of `a.js`'s
/// exports.
fn project(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-watch-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("src")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    copy_dir(&repo().join("packs/js"), &d.join("node_modules/@codelaw/rules-js"));
    fs::write(d.join("codelaw.toml"), CONFIG).unwrap();
    fs::write(d.join("package.json"), "{ \"name\": \"w\", \"main\": \"src/main.js\" }\n").unwrap();
    fs::write(d.join("src/main.js"), "import { a, c } from \"./a.js\";\n\nconsole.log(a, c);\n").unwrap();
    fs::write(d.join("src/a.js"), "export const a = 1;\nexport const c = 2;\n").unwrap();
    d
}

fn command(cwd: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_AS_OF")
        .env_remove("CODELAW_PROFILE")
        .env_remove("CI")
        .env("CODELAW_CACHE_DIR", cwd.with_extension("cache"))
        .env("NO_COLOR", "1");
    c
}

/// A running watch and the lines it has printed.
struct Running {
    child: Child,
    lines: Receiver<String>,
}

impl Running {
    fn start(cwd: &Path, args: &[&str]) -> Running {
        let mut child = command(cwd, args).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let out = child.stdout.take().unwrap();
        let (tx, lines) = channel();
        std::thread::spawn(move || {
            for l in BufReader::new(out).lines().map_while(Result::ok) {
                if tx.send(l).is_err() {
                    break;
                }
            }
        });
        Running { child, lines }
    }

    /// The next line, within 20 s.
    fn line(&self) -> String {
        self.lines.recv_timeout(Duration::from_secs(20)).expect("the watch printed nothing in 20 s")
    }

    /// The next run's object: each line is one valid JSON object (DoD 4).
    fn run(&self) -> serde_json::Value {
        let l = self.line();
        serde_json::from_str(&l).unwrap_or_else(|e| panic!("not one JSON object per line: {e}: {l}"))
    }

    /// Nothing more within `wait`.
    fn quiet(&self, wait: Duration) -> Option<String> {
        self.lines.recv_timeout(wait).ok()
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn findings(v: &serde_json::Value) -> Vec<(String, String)> {
    v["diagnostics"].as_array().unwrap().iter().map(|d| (d["file"].as_str().unwrap().to_owned(), d["rule"].as_str().unwrap().to_owned())).collect()
}

fn has(v: &serde_json::Value, file: &str, rule: &str) -> bool {
    findings(v).iter().any(|(f, r)| f == file && r == rule)
}

/// `check --format json` on the same files: what a watch's run must be,
/// its `watch` field aside.
fn check(cwd: &Path) -> serde_json::Value {
    let out = command(cwd, &["check", "--no-adapters", "--format", "json"]).output().unwrap();
    serde_json::from_slice(&out.stdout).unwrap()
}

fn same_as_check(run: &serde_json::Value, cwd: &Path) {
    let mut run = run.clone();
    run.as_object_mut().unwrap().remove("watch");
    assert_eq!(run, check(cwd), "a watch's run is `check`'s object");
}

/// DoD 2 and 4: deleting the last use of an export in one file reports
/// `graph/no-unused-exports` in the file that exports it.
#[test]
fn a_save_rechecks_the_files_that_depend_on_it() {
    let d = project("graph");
    let w = Running::start(&d, &["check", "--watch", "--no-adapters", "--format", "json"]);
    let first = w.run();
    assert_eq!(first["watch"]["run"], 1, "{first}");
    assert!(!has(&first, "src/a.js", "graph/no-unused-exports"), "{first}");
    same_as_check(&first, &d);

    fs::write(d.join("src/main.js"), "import { a } from \"./a.js\";\n\nconsole.log(a);\n").unwrap();
    let second = w.run();
    assert_eq!(second["watch"]["run"], 2);
    assert_eq!(second["watch"]["changed"], serde_json::json!(["src/main.js"]), "{second}");
    assert_eq!(second["watch"]["reloaded"], false);
    assert!(has(&second, "src/a.js", "graph/no-unused-exports"), "the export main.js stopped using: {second}");
    same_as_check(&second, &d);

    // An editor's save: a new file renamed over the old one.
    fs::write(d.join("src/.main.js.tmp"), "import { a, c } from \"./a.js\";\n\nconsole.log(a, c);\n").unwrap();
    fs::rename(d.join("src/.main.js.tmp"), d.join("src/main.js")).unwrap();
    let third = w.run();
    assert!(!has(&third, "src/a.js", "graph/no-unused-exports"), "{third}");
    same_as_check(&third, &d);

    // A file added is checked; removed, it is gone from the verdict.
    fs::write(d.join("src/b.js"), "export const b = 1\n").unwrap();
    let fourth = w.run();
    assert!(has(&fourth, "src/b.js", "format/unformatted"), "{fourth}");
    same_as_check(&fourth, &d);
    fs::remove_file(d.join("src/b.js")).unwrap();
    let fifth = w.run();
    assert!(!findings(&fifth).iter().any(|(f, _)| f == "src/b.js"), "{fifth}");
    same_as_check(&fifth, &d);
    let _ = fs::remove_dir_all(&d);
}

/// DoD 3: editing `codelaw.toml` changes the next run's verdict; a config
/// that does not load is reported and the watch goes on.
#[test]
fn a_config_edit_changes_the_next_verdict() {
    let d = project("config");
    fs::write(d.join("src/main.js"), "import { a } from \"./a.js\";\n\nconsole.log(a);\n").unwrap();
    let w = Running::start(&d, &["check", "--watch", "--no-adapters", "--format", "json"]);
    let first = w.run();
    assert!(has(&first, "src/a.js", "graph/no-unused-exports"), "{first}");

    fs::write(d.join("codelaw.toml"), format!("{CONFIG}[rules]\n\"graph/no-unused-exports\" = \"off\"\n")).unwrap();
    let second = w.run();
    assert_eq!(second["watch"]["reloaded"], true, "{second}");
    assert!(!has(&second, "src/a.js", "graph/no-unused-exports"), "{second}");
    same_as_check(&second, &d);

    fs::write(d.join("codelaw.toml"), "extends = [\n").unwrap();
    let broken = w.run();
    assert!(broken["error"].as_str().is_some_and(|e| e.contains("codelaw.toml")), "{broken}");

    fs::write(d.join("codelaw.toml"), CONFIG).unwrap();
    let fixed = w.run();
    assert!(fixed.get("error").is_none() && has(&fixed, "src/a.js", "graph/no-unused-exports"), "{fixed}");
    let _ = fs::remove_dir_all(&d);
}

/// The text format: the report as `check` prints it, then the watch's
/// line; no screen-clearing codes when stdout is not a terminal.
#[test]
fn text_runs_end_with_the_watch_line() {
    let d = project("text");
    let w = Running::start(&d, &["check", "--watch", "--no-adapters"]);
    let mut lines = Vec::new();
    loop {
        let l = w.line();
        let end = l.starts_with("watching ");
        lines.push(l);
        if end {
            break;
        }
    }
    let last = lines.last().unwrap();
    assert!(last.contains(" · run 1 · Ctrl-C to stop"), "{last}");
    assert!(lines.iter().any(|l| l.starts_with("✔ ") && l.contains("files checked")), "{lines:?}");
    assert!(!lines.concat().contains('\x1b'), "{lines:?}");
    fs::write(d.join("src/a.js"), "export const a = 1;\nexport const c = 3;\n").unwrap();
    loop {
        let l = w.line();
        if l.starts_with("watching ") {
            assert!(l.contains(" · run 2 · src/a.js changed · "), "{l}");
            break;
        }
    }
    let _ = fs::remove_dir_all(&d);
}

/// `fix --watch` fixes a saved file, and its own write does not start
/// another run.
#[test]
fn fix_watch_fixes_a_save_once() {
    let d = project("fix");
    let w = Running::start(&d, &["fix", "--watch", "--no-adapters", "--format", "json"]);
    let first = w.run();
    assert_eq!(first["watch"]["run"], 1, "{first}");
    fs::write(d.join("src/a.js"), "export const a = 1\nexport const c = 2;\n").unwrap();
    let second = w.run();
    assert_eq!(second["fixed"][0]["file"], "src/a.js", "{second}");
    assert_eq!(second["fixed"][0]["written"], true, "{second}");
    assert_eq!(fs::read_to_string(d.join("src/a.js")).unwrap(), "export const a = 1;\nexport const c = 2;\n");
    assert!(!has(&second, "src/a.js", "format/unformatted"), "{second}");
    assert_eq!(w.quiet(Duration::from_millis(1500)), None, "codelaw's own write is not a change");
    let _ = fs::remove_dir_all(&d);
}

/// Ctrl-C ends the watch with exit 0.
#[cfg(unix)]
#[test]
fn ctrl_c_exits_0() {
    let d = project("interrupt");
    let mut w = Running::start(&d, &["check", "--watch", "--no-adapters", "--format", "json"]);
    w.run();
    unsafe {
        libc::kill(w.child.id() as libc::pid_t, libc::SIGINT);
    }
    let status = w.child.wait().unwrap();
    assert_eq!(status.code(), Some(0), "{status:?}");
    let _ = fs::remove_dir_all(&d);
}

/// A warm process does not serve a watch: it runs in its own process.
#[test]
fn watch_flags_parse() {
    let d = project("flags");
    let out = command(&d, &["fix", "--watch", "--trace"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2), "--watch and --trace conflict");
    let _ = fs::remove_dir_all(&d);
}
