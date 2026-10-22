//! Warm processes (SR-ARCHITECTURE decisions 21 and 22, W-15): a command run
//! with `--prefer-warm` in a warm `daemon`, `mcp` or `lsp` prints the bytes
//! the same command prints run cold, on every L3 fixture; discovery picks
//! the newest live socket of its own version and removes stale ones; the
//! daemon is opt-in, exits when idle and reloads what changed.
//!
//! Unix only: elsewhere every command runs in its own process.
#![cfg(unix)]

use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const LANGUAGES: &[&str] = &["javascript", "typescript", "python", "json", "yaml", "toml", "markdown", "css", "html"];
const CONFIG: &str = "[daemon]\nidle_timeout = \"120s\"\n";

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

/// The fixture rules plus every language's files, with a config (a warm
/// process serves only a codelaw project).
fn repo(name: &str, config: &str) -> PathBuf {
    repo_in(&std::env::temp_dir(), name, config)
}

fn repo_in(base: &Path, name: &str, config: &str) -> PathBuf {
    let d = base.join(format!("codelaw-warm-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules")).unwrap();
    fs::write(d.join("codelaw.toml"), config).unwrap();
    copy_dir(&fixtures().join("rules"), &d.join(".codelaw/rules"));
    for l in LANGUAGES {
        copy_dir(&fixtures().join(l), &d);
    }
    fs::canonicalize(d).unwrap()
}

/// The repo's own files (not `.codelaw/`), relative, sorted, with bytes.
fn tree(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for e in fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != ".codelaw") {
                    walk(root, &p, out);
                }
            } else {
                out.push((p.strip_prefix(root).unwrap().to_string_lossy().into_owned(), fs::read(&p).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

fn command(cwd: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.current_dir(cwd).env_remove("CODELAW_CACHE_DIR").env_remove("VIRTUAL_ENV").env_remove("CODELAW_SITE_PACKAGES").env_remove("NO_COLOR");
    c
}

type Ran = (Option<i32>, String, String);

fn run(cwd: &Path, args: &[&str]) -> Ran {
    let out = command(cwd).args(args).stdin(Stdio::null()).output().unwrap();
    (out.status.code(), String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
}

fn run_dir(root: &Path) -> PathBuf {
    root.join(".codelaw/cache/run")
}

/// The `served` count of each warm process `daemon status` lists, by pid.
fn served(root: &Path) -> Vec<(u32, String, u64)> {
    let (_, out, _) = run(root, &["daemon", "status"]);
    out.lines()
        .filter_map(|l| {
            let w: Vec<&str> = l.split_whitespace().collect();
            (w.len() >= 7 && w[1] == "pid").then(|| (w[2].parse().unwrap(), w[0].to_owned(), w[6].parse().unwrap()))
        })
        .collect()
}

fn total_served(root: &Path) -> u64 {
    served(root).iter().map(|s| s.2).sum()
}

fn wait_for(what: &str, mut ok: impl FnMut() -> bool) {
    let start = Instant::now();
    while !ok() {
        assert!(start.elapsed() < Duration::from_secs(20), "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Stops the project's daemons however the test ends.
struct Daemon(PathBuf);

impl Daemon {
    fn start(root: &Path) -> Daemon {
        let (code, out, err) = run(root, &["daemon", "start"]);
        assert_eq!(code, Some(0), "daemon start: {out}{err}");
        assert!(out.starts_with("codelaw daemon started for "), "{out}");
        Daemon(root.to_path_buf())
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        // A test that removed its folder stopped the daemon first.
        if self.0.exists() {
            let _ = run(&self.0, &["daemon", "stop"]);
        }
    }
}

/// A child killed however the test ends.
struct Kill(Child);

impl Drop for Kill {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Cold, then with `--prefer-warm`: the same exit code, stdout and stderr,
/// and the warm process served it.
fn same(root: &Path, cwd: &Path, args: &[&str]) -> Ran {
    let cold = run(cwd, args);
    let before = total_served(root);
    let mut warm_args = args.to_vec();
    warm_args.push("--prefer-warm");
    let warm = run(cwd, &warm_args);
    assert_eq!(total_served(root), before + 1, "`codelaw {}` was not served warm: {warm:?}", args.join(" "));
    assert_eq!(cold, warm, "`codelaw {}` differs warm", args.join(" "));
    cold
}

#[test]
fn warm_runs_print_what_cold_runs_print_on_every_fixture() {
    let root = repo("parity", CONFIG);
    let _daemon = Daemon::start(&root);
    let (_, list, _) = run(&root, &["rules", "--for", "src/a.js"]);
    assert!(!list.is_empty());

    // The whole repo in every format, with and without the cache.
    for format in ["text", "json", "sarif", "agent"] {
        let (code, out, _) = same(&root, &root, &["check", "--format", format]);
        assert_eq!(code, Some(1), "the fixtures report findings: {out}");
        same(&root, &root, &["check", "--no-cache", "--format", format]);
    }
    // Every file alone, every language alone.
    let files: Vec<String> = tree(&root).into_iter().map(|(f, _)| f).filter(|f| f != "codelaw.toml").collect();
    assert!(files.len() >= 11, "{files:?}");
    for f in &files {
        same(&root, &root, &["check", f, "--format", "json"]);
    }
    for l in LANGUAGES {
        same(&root, &root, &["check", "--lang", l, "--format", "json"]);
    }
    // From a sub-folder: the daemon runs in the client's folder.
    let sub = root.join("src");
    same(&root, &sub, &["check", "--format", "json"]);
    same(&root, &sub, &["check", "a.js", "--format", "text"]);
    // Errors are the same bytes too.
    let (code, _, err) = same(&root, &root, &["check", "no-such-file.js"]);
    assert_eq!(code, Some(2), "{err}");
    same(&root, &root, &["check", "--rule", "fixture/no-such-rule"]);
    same(&root, &root, &["format", "--check"]);
    same(&root, &root, &["format", "--diff"]);
    same(&root, &root, &["fix", "--dry-run", "--format", "json"]);
    drop(_daemon);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn warm_fix_and_format_write_the_bytes_cold_ones_write() {
    let cold = repo("fix-cold", CONFIG);
    let warm = repo("fix-warm", CONFIG);
    let _daemon = Daemon::start(&warm);
    let mut served = 0;
    for args in [&["fix"][..], &["fix", "--unsafe", "--format", "json"][..], &["format"][..]] {
        let c = run(&cold, args);
        let mut warm_args = args.to_vec();
        warm_args.push("--prefer-warm");
        let w = run(&warm, &warm_args);
        served += 1;
        assert_eq!(total_served(&warm), served, "`codelaw {}` was not served warm", args.join(" "));
        assert_eq!(c, w, "`codelaw {}` differs warm", args.join(" "));
        assert_eq!(tree(&cold), tree(&warm), "`codelaw {}` wrote other bytes warm", args.join(" "));
    }
    assert_ne!(tree(&cold), tree(&repo("fix-unfixed", CONFIG)), "fix changed nothing, so the comparison proves nothing");
    drop(_daemon);
    for d in [cold, warm] {
        let _ = fs::remove_dir_all(d);
    }
    let _ = fs::remove_dir_all(std::env::temp_dir().join(format!("codelaw-warm-fix-unfixed-{}", std::process::id())));
}

#[test]
fn the_daemon_reloads_a_changed_rule_and_config() {
    let root = repo("reload", CONFIG);
    let _daemon = Daemon::start(&root);
    let before = same(&root, &root, &["check", "--format", "json"]);
    // A rule switched off in the config, then a rule file removed.
    fs::write(root.join("codelaw.toml"), format!("{CONFIG}[rules]\n\"fixture/js-eqeqeq\" = \"off\"\n")).unwrap();
    let off = same(&root, &root, &["check", "--format", "json"]);
    assert_ne!(before, off, "the config change shows");
    let rule = fs::read_dir(root.join(".codelaw/rules/fixture")).unwrap().flatten().map(|e| e.path()).find(|p| p.to_string_lossy().contains("py")).unwrap();
    fs::remove_file(rule).unwrap();
    let removed = same(&root, &root, &["check", "--format", "json"]);
    assert_ne!(off, removed, "the rule change shows");
    drop(_daemon);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn discovery_takes_the_newest_live_socket_of_its_version_and_removes_stale_ones() {
    // The test binds sockets itself, by absolute path, which the platform
    // limits to about a hundred bytes: a short folder. (codelaw binds
    // relative to the run folder, so its own sockets have no such limit.)
    let root = repo_in(Path::new("/tmp"), "discover", CONFIG);
    let dir = run_dir(&root);
    let spawn = || Kill(command(&root).args(["daemon", "run"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap());
    let older = spawn();
    let older_pid = older.0.id();
    wait_for("the first daemon", || served(&root).iter().any(|s| s.0 == older_pid));
    std::thread::sleep(Duration::from_millis(50));
    let newer = spawn();
    let newer_pid = newer.0.id();
    wait_for("the second daemon", || served(&root).iter().any(|s| s.0 == newer_pid));

    // A stale registration: its process is gone, its socket file left.
    drop(UnixListener::bind(dir.join("4000001.sock")).unwrap());
    fs::write(dir.join("4000001.json"), format!(r#"{{"pid":4000001,"kind":"daemon","version":"{}","cwd":"/","any_cwd":true}}"#, env!("CARGO_PKG_VERSION"))).unwrap();
    // A live process of another version, registered newest: it is never sent a command.
    std::thread::sleep(Duration::from_millis(50));
    let other = UnixListener::bind(dir.join("4000002.sock")).unwrap();
    other.set_nonblocking(true).unwrap();
    fs::write(dir.join("4000002.json"), r#"{"pid":4000002,"kind":"daemon","version":"0.0.0-other","cwd":"/","any_cwd":true}"#).unwrap();

    let cold = run(&root, &["check", "--format", "json"]);
    let warm = run(&root, &["check", "--format", "json", "--prefer-warm"]);
    assert_eq!(cold, warm);
    let mut sent = Vec::new();
    while let Ok((mut s, _)) = other.accept() {
        // macOS answers EINVAL here when the client has already closed.
        let _ = s.set_nonblocking(false);
        let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
        let mut buf = Vec::new();
        let _ = s.read_to_end(&mut buf);
        sent.extend(buf);
    }
    assert!(sent.is_empty(), "another version was sent a command: {}", String::from_utf8_lossy(&sent));
    assert!(!dir.join("4000001.json").exists() && !dir.join("4000001.sock").exists(), "the stale socket was not removed");
    let counts = served(&root);
    let of = |pid: u32| counts.iter().find(|s| s.0 == pid).map(|s| s.2);
    assert_eq!((of(newer_pid), of(older_pid)), (Some(1), Some(0)), "the newest daemon serves: {counts:?}");
    drop(other);
    let _ = fs::remove_file(dir.join("4000002.sock"));
    let _ = fs::remove_file(dir.join("4000002.json"));

    // `stop` stops both, and each removes its own registration.
    let (_, out, _) = run(&root, &["daemon", "stop"]);
    assert_eq!(out.lines().count(), 2, "{out}");
    let names: Vec<String> = fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    assert!(names.is_empty(), "{names:?}");
    drop((older, newer));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn the_daemon_exits_when_idle_and_is_never_started_for_you() {
    let root = repo("idle", "[daemon]\nidle_timeout = \"1s\"\n");
    // Opt-in: nothing runs until asked, and a warm run with none is cold.
    let (code, out, _) = run(&root, &["daemon", "status"]);
    assert_eq!((code, out.trim_end()), (Some(1), format!("no warm codelaw process for {}", root.display()).as_str()));
    let cold = run(&root, &["check", "--format", "json"]);
    assert_eq!(cold, run(&root, &["check", "--format", "json", "--prefer-warm"]));
    assert!(!run_dir(&root).exists() || fs::read_dir(run_dir(&root)).unwrap().count() == 0);

    let mut daemon = Kill(command(&root).args(["daemon", "run"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().unwrap());
    let started = Instant::now();
    wait_for("the daemon to exit when idle", || matches!(daemon.0.try_wait(), Ok(Some(_))));
    assert!(started.elapsed() >= Duration::from_secs(1));
    assert!(daemon.0.try_wait().unwrap().unwrap().success());
    assert_eq!(fs::read_dir(run_dir(&root)).unwrap().count(), 0, "the idle daemon left its registration");

    // A second `start` finds the first; outside a project it refuses.
    fs::write(root.join("codelaw.toml"), CONFIG).unwrap();
    let _d = Daemon::start(&root);
    let (code, out, _) = run(&root, &["daemon", "start"]);
    assert_eq!(code, Some(0));
    assert!(out.starts_with("codelaw daemon already running"), "{out}");
    let bare = std::env::temp_dir().join(format!("codelaw-warm-bare-{}", std::process::id()));
    fs::create_dir_all(&bare).unwrap();
    let (code, _, err) = run(&bare, &["daemon", "run"]);
    assert_eq!(code, Some(2), "{err}");
    assert!(err.contains("no codelaw config"), "{err}");
    let _ = fs::remove_dir_all(&bare);
    drop(_d);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn mcp_and_lsp_are_warm_processes_for_their_folder() {
    let root = repo("servers", CONFIG);
    for kind in ["mcp", "lsp"] {
        let mut server = Kill(command(&root).arg(kind).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap());
        let pid = server.0.id();
        wait_for("the server's socket", || served(&root).iter().any(|s| s.0 == pid && s.1 == kind));
        same(&root, &root, &["check", "--format", "json"]);
        // Another folder is not its folder: that run is cold.
        let sub = root.join("src");
        let cold = run(&sub, &["check", "--format", "json"]);
        assert_eq!(cold, run(&sub, &["check", "--format", "json", "--prefer-warm"]));
        assert_eq!(served(&root).iter().find(|s| s.0 == pid).map(|s| s.2), Some(1));
        // `daemon stop` stops daemons only.
        let (_, out, _) = run(&root, &["daemon", "stop"]);
        assert!(out.starts_with("no codelaw daemon"), "{out}");
        // The session ends: the registration goes with it.
        if kind == "lsp" {
            let msg = |v: &str| format!("Content-Length: {}\r\n\r\n{v}", v.len());
            let stdin = server.0.stdin.as_mut().unwrap();
            stdin.write_all(msg(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#).as_bytes()).unwrap();
            stdin.write_all(msg(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#).as_bytes()).unwrap();
            stdin.write_all(msg(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#).as_bytes()).unwrap();
            stdin.write_all(msg(r#"{"jsonrpc":"2.0","method":"exit"}"#).as_bytes()).unwrap();
        }
        drop(server.0.stdin.take());
        wait_for("the server to exit", || matches!(server.0.try_wait(), Ok(Some(_))));
        assert!(!run_dir(&root).join(format!("{pid}.json")).exists(), "{kind} left its registration");
    }
    let _ = fs::remove_dir_all(&root);
}
