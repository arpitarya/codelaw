//! Warm processes (SR-ARCHITECTURE decisions 21 and 22).
//!
//! A long-lived `codelaw lsp`, `codelaw mcp` or `codelaw daemon` registers a
//! local socket at `<cache>/run/<pid>.sock`, with `<pid>.json` beside it
//! naming its version, kind and folder. A command run with `--prefer-warm`
//! sends its command line, folder and environment to the newest live socket
//! of its own version and prints what comes back; with none, or when the
//! warm process declines, it runs in its own process. Either way the bytes
//! are the same: the warm process runs the very function the command runs
//! here, with the client's environment and terminal, never its own.
//!
//! - **Local only.** A Unix domain socket in the project's cache folder;
//!   nothing listens on a network port (L5 decision 3).
//! - **Stale sockets** — no process accepts on them, because it died or
//!   was killed — are removed by whoever discovers them.
//! - **Windows** has no Unix sockets here: every command runs in its own
//!   process and `codelaw daemon` says so.

#[cfg(not(unix))]
use crate::{Ctx, Output};
use clap::Subcommand;
use std::path::{Path, PathBuf};
#[cfg(not(unix))]
use std::process::ExitCode;

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start a warm daemon for this project in the background.
    Start,
    /// Stop this project's daemon.
    Stop,
    /// List this project's warm processes (`lsp`, `mcp`, `daemon`).
    Status,
    /// Run the daemon in the foreground (`start` runs this).
    #[command(hide = true)]
    Run,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Lsp,
    Mcp,
    Daemon,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Lsp => "lsp",
            Kind::Mcp => "mcp",
            Kind::Daemon => "daemon",
        }
    }
}

/// The run folder of the project that governs `start`: `None` without a
/// config, so a warm process is only ever registered inside a codelaw project.
fn run_dir(start: &Path, cache_dir_env: Option<&str>) -> Option<(PathBuf, PathBuf)> {
    let config = codelaw_config::find_config(start).ok().filter(|c| !c.files.is_empty())?;
    Some((config.cache_dir(cache_dir_env).join("run"), config.root_dir.clone()))
}

/// Where `codelaw <command> <paths>` finds its project: as `crate::load`.
fn start_of(paths: &[PathBuf]) -> PathBuf {
    let start = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
    let start = if start.is_file() { start.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")) } else { start };
    if start.as_os_str().is_empty() { PathBuf::from(".") } else { start }
}

#[cfg(unix)]
pub use unix::{client, daemon, serve_in_background};

#[cfg(not(unix))]
pub fn client(_paths: &[PathBuf], _ctx: &Ctx) -> Option<Output> {
    None
}

#[cfg(not(unix))]
pub struct Unregister;

#[cfg(not(unix))]
pub fn serve_in_background(_kind: Kind) -> Option<Unregister> {
    None
}

#[cfg(not(unix))]
pub fn daemon(_action: DaemonAction) -> ExitCode {
    crate::fail("`codelaw daemon` needs Unix domain sockets; on this system every command runs in its own process")
}

#[cfg(unix)]
mod unix {
    use super::{DaemonAction, Kind, run_dir, start_of};
    use crate::{Ctx, Output};
    use codelaw_engine::EngineOptions;
    use serde_json::{Value, json};
    use std::fs;
    use std::io::{BufRead, BufReader, ErrorKind, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::process::{ExitCode, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant, SystemTime};

    const VERSION: &str = codelaw_core::VERSION;

    /// One warm process, as its `<pid>.json` names it.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Registration {
        pub pid: u32,
        pub kind: String,
        pub version: String,
        /// The folder it runs commands in.
        pub cwd: PathBuf,
        /// It runs a command in the client's folder (the daemon); `lsp`
        /// and `mcp` serve only clients in their own folder.
        pub any_cwd: bool,
    }

    impl Registration {
        fn json(&self) -> Value {
            json!({ "pid": self.pid, "kind": self.kind, "version": self.version, "cwd": self.cwd.to_string_lossy(), "any_cwd": self.any_cwd })
        }

        fn parse(v: &Value) -> Option<Registration> {
            Some(Registration {
                pid: u32::try_from(v["pid"].as_u64()?).ok()?,
                kind: v["kind"].as_str()?.to_owned(),
                version: v["version"].as_str()?.to_owned(),
                cwd: PathBuf::from(v["cwd"].as_str()?),
                any_cwd: v["any_cwd"].as_bool()?,
            })
        }
    }

    fn sock_name(pid: u32) -> String {
        format!("{pid}.sock")
    }

    fn json_name(pid: u32) -> String {
        format!("{pid}.json")
    }

    /// Run `f` with `dir` as the working folder. A socket path is limited
    /// to about a hundred bytes, which a project's cache folder can exceed;
    /// relative to its folder the name is short. Only called where no other
    /// thread of this process is running yet, or where none uses the folder.
    fn in_dir<T>(dir: &Path, f: impl FnOnce() -> std::io::Result<T>) -> std::io::Result<T> {
        let back = std::env::current_dir()?;
        std::env::set_current_dir(dir)?;
        let out = f();
        std::env::set_current_dir(back)?;
        out
    }

    fn connect(dir: &Path, pid: u32) -> std::io::Result<UnixStream> {
        match UnixStream::connect(dir.join(sock_name(pid))) {
            Err(e) if e.kind() == ErrorKind::InvalidInput => in_dir(dir, || UnixStream::connect(sock_name(pid))),
            other => other,
        }
    }

    fn bind(dir: &Path, pid: u32) -> std::io::Result<UnixListener> {
        match UnixListener::bind(dir.join(sock_name(pid))) {
            Err(e) if e.kind() == ErrorKind::InvalidInput => in_dir(dir, || UnixListener::bind(sock_name(pid))),
            other => other,
        }
    }

    fn remove(dir: &Path, pid: u32) {
        let _ = fs::remove_file(dir.join(sock_name(pid)));
        let _ = fs::remove_file(dir.join(json_name(pid)));
    }

    /// No process accepts on the socket: it is stale.
    fn refused(e: &std::io::Error) -> bool {
        matches!(e.kind(), ErrorKind::ConnectionRefused | ErrorKind::NotFound)
    }

    /// The live warm processes registered in `dir`, newest first, each with
    /// an open connection. Every stale socket found on the way is removed,
    /// with its registration (SR-ARCHITECTURE decision 21).
    pub fn discover(dir: &Path) -> Vec<(Registration, UnixStream)> {
        let mut found: Vec<(SystemTime, Registration, UnixStream)> = Vec::new();
        let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };
        let mut names: Vec<String> = entries.flatten().filter_map(|e| e.file_name().to_str().map(str::to_owned)).collect();
        names.sort();
        for name in &names {
            let Some(pid) = name.strip_suffix(".json").and_then(|p| p.parse::<u32>().ok()) else {
                // A socket whose registration is gone: removed once no one
                // accepts on it.
                if let Some(pid) = name.strip_suffix(".sock").and_then(|p| p.parse::<u32>().ok())
                    && !names.contains(&json_name(pid))
                    && connect(dir, pid).as_ref().err().is_some_and(refused)
                {
                    let _ = fs::remove_file(dir.join(name));
                }
                continue;
            };
            let path = dir.join(name);
            let registration = fs::read(&path).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()).and_then(|v| Registration::parse(&v));
            let Some(registration) = registration.filter(|r| r.pid == pid) else {
                // Registrations are written whole (a rename), so one that
                // does not parse is left over from another codelaw.
                remove(dir, pid);
                continue;
            };
            match connect(dir, pid) {
                Ok(stream) => {
                    let modified = fs::metadata(&path).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
                    found.push((modified, registration, stream));
                }
                Err(e) if refused(&e) => remove(dir, pid),
                Err(_) => {}
            }
        }
        found.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.pid.cmp(&a.1.pid)));
        found.into_iter().map(|(_, r, s)| (r, s)).collect()
    }

    /// Send one request line and read one answer line.
    fn exchange(stream: &mut UnixStream, request: &Value) -> Option<Value> {
        let mut line = serde_json::to_string(request).ok()?;
        line.push('\n');
        stream.write_all(line.as_bytes()).ok()?;
        let mut answer = String::new();
        BufReader::new(stream).read_line(&mut answer).ok()?;
        serde_json::from_str(&answer).ok()
    }

    fn env_json(env: &EngineOptions) -> Value {
        json!({
            "cache_dir": env.cache_dir_env,
            "site_packages": env.site_packages_env,
            "virtual_env": env.virtual_env,
            "as_of": env.as_of_env,
            "profile": env.profile_env,
            "ci": env.ci_env,
            // Where the type helper finds Node and TypeScript (SR-TYPES
            // decision 5): the client's, not the warm process's.
            "path": env.path_env.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "typescript": env.typescript_env,
        })
    }

    fn format_env_json(env: &codelaw_report::FormatEnv) -> Value {
        json!({ "format": env.format, "agent_vars": env.agent_vars, "tty": env.tty })
    }

    /// The client's format inputs (SR-CLI-MCP decision 4a).
    fn format_env_from(v: &Value) -> codelaw_report::FormatEnv {
        let pairs = v["agent_vars"].as_array().into_iter().flatten();
        codelaw_report::FormatEnv {
            format: v["format"].as_str().map(str::to_owned),
            agent_vars: pairs.filter_map(|p| Some((p[0].as_str()?.to_owned(), p[1].as_str()?.to_owned()))).collect(),
            tty: v["tty"].as_bool().unwrap_or(false),
        }
    }

    fn env_from(v: &Value) -> EngineOptions {
        let s = |k: &str| v[k].as_str().map(str::to_owned);
        // The client's profile inputs, never the warm process's own
        // (SR-CONFIG decision 10).
        EngineOptions {
            cache_dir_env: s("cache_dir"),
            site_packages_env: s("site_packages"),
            virtual_env: s("virtual_env"),
            as_of_env: s("as_of"),
            profile_env: s("profile"),
            ci_env: s("ci"),
            path_env: s("path").map(std::ffi::OsString::from),
            typescript_env: s("typescript"),
            ..Default::default()
        }
    }

    fn canonical_cwd() -> Option<PathBuf> {
        std::env::current_dir().and_then(fs::canonicalize).ok()
    }

    /// `--prefer-warm`: this command line run by the newest live warm
    /// process of this version that serves this folder, or `None` to run it
    /// here (SR-ARCHITECTURE decision 21).
    pub fn client(paths: &[PathBuf], ctx: &Ctx) -> Option<Output> {
        let (dir, _) = run_dir(&start_of(paths), ctx.env.cache_dir_env.as_deref())?;
        let cwd = canonical_cwd()?;
        let argv: Vec<String> = std::env::args_os().map(|a| a.into_string()).collect::<Result<_, _>>().ok()?;
        let request = json!({ "op": "run", "version": VERSION, "argv": argv, "cwd": cwd.to_string_lossy(), "color": ctx.color, "env": env_json(&ctx.env), "format_env": format_env_json(&ctx.format_env) });
        for (registration, mut stream) in discover(&dir) {
            if registration.version != VERSION || !(registration.any_cwd || registration.cwd == cwd) {
                continue;
            }
            // A warm process that declines, or dies mid-request, leaves the
            // command to the next one, or to this process.
            let Some(answer) = exchange(&mut stream, &request) else { continue };
            if let (Some(stdout), Some(stderr), Some(code)) = (answer["stdout"].as_str(), answer["stderr"].as_str(), answer["code"].as_u64()) {
                return Some(Output { stdout: stdout.to_owned(), stderr: stderr.to_owned(), code: u8::try_from(code).unwrap_or(2) });
            }
        }
        None
    }

    /// A registered socket; dropping it removes the socket and its
    /// registration.
    pub struct Registered {
        dir: PathBuf,
        pid: u32,
        listener: UnixListener,
        registration: Registration,
        root: PathBuf,
        served: AtomicU64,
    }

    impl Drop for Registered {
        fn drop(&mut self) {
            remove(&self.dir, self.pid);
        }
    }

    /// Bind this process's socket and write its registration, last, so a
    /// registration always has a socket behind it. `None` outside a codelaw
    /// project, or when the socket cannot be made.
    fn register(kind: Kind) -> Result<Registered, String> {
        let cwd = canonical_cwd().ok_or("the current folder cannot be read")?;
        let env = EngineOptions::from_env();
        let (dir, root) = run_dir(&cwd, env.cache_dir_env.as_deref()).ok_or_else(|| format!("no codelaw config in {} or above", codelaw_core::normalize_path(&cwd)))?;
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", codelaw_core::normalize_path(&dir)))?;
        let pid = std::process::id();
        // A file of a dead process that had this pid.
        remove(&dir, pid);
        let listener = bind(&dir, pid).map_err(|e| format!("cannot listen on {}: {e}", codelaw_core::normalize_path(&dir.join(sock_name(pid)))))?;
        let registration = Registration { pid, kind: kind.name().into(), version: VERSION.into(), cwd, any_cwd: kind == Kind::Daemon };
        let tmp = dir.join(format!(".{pid}.json.tmp"));
        let written = fs::write(&tmp, registration.json().to_string()).and_then(|()| fs::rename(&tmp, dir.join(json_name(pid))));
        if let Err(e) = written {
            let _ = fs::remove_file(&tmp);
            let _ = fs::remove_file(dir.join(sock_name(pid)));
            return Err(format!("cannot register in {}: {e}", codelaw_core::normalize_path(&dir)));
        }
        Ok(Registered { dir, pid, listener, registration, root, served: AtomicU64::new(0) })
    }

    /// What one connection asked for, once answered.
    enum Asked {
        Nothing,
        Served,
        Stop,
    }

    /// Answer one connection: a command to run, `status` or `stop`. An
    /// empty connection is a liveness probe of discovery.
    fn answer(reg: &Registered, stream: UnixStream) -> Asked {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let mut line = String::new();
        let mut reader = BufReader::new(&stream);
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return Asked::Nothing;
        }
        let _ = stream.set_read_timeout(None);
        let request: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
        let mut writer = &stream;
        let mut reply = |v: Value| {
            let _ = writer.write_all(format!("{v}\n").as_bytes());
        };
        match request["op"].as_str() {
            Some("status") => {
                let mut v = reg.registration.json();
                v["served"] = json!(reg.served.load(Ordering::Relaxed));
                reply(v);
                Asked::Nothing
            }
            Some("stop") if reg.registration.kind == Kind::Daemon.name() => {
                reply(json!({ "stopping": reg.pid }));
                Asked::Stop
            }
            Some("run") => match run(reg, &request) {
                Ok(out) => {
                    reg.served.fetch_add(1, Ordering::Relaxed);
                    reply(json!({ "stdout": out.stdout, "stderr": out.stderr, "code": out.code }));
                    Asked::Served
                }
                Err(why) => {
                    reply(json!({ "declined": why }));
                    Asked::Nothing
                }
            },
            _ => {
                reply(json!({ "declined": "unknown request" }));
                Asked::Nothing
            }
        }
    }

    /// Run a client's command line, or say why not.
    fn run(reg: &Registered, request: &Value) -> Result<Output, String> {
        if request["version"].as_str() != Some(VERSION) {
            return Err(format!("this is codelaw {VERSION}"));
        }
        let cwd = PathBuf::from(request["cwd"].as_str().ok_or("no folder")?);
        let argv: Vec<String> = request["argv"].as_array().ok_or("no command line")?.iter().map(|a| a.as_str().map(str::to_owned)).collect::<Option<_>>().ok_or("a command line is strings")?;
        let ctx = Ctx { color: request["color"].as_bool().unwrap_or(false), env: env_from(&request["env"]), format_env: format_env_from(&request["format_env"]), warm: true };
        let go = || std::panic::catch_unwind(|| crate::serve_command(&argv, &ctx)).ok().flatten().ok_or_else(|| "not served".to_owned());
        if cwd == reg.registration.cwd {
            go()
        } else if reg.registration.any_cwd {
            // The daemon answers one request at a time on one thread, so
            // the folder it runs in is the client's for that request.
            in_dir(&cwd, || Ok(go())).map_err(|e| e.to_string())?
        } else {
            Err("another folder".into())
        }
    }

    /// Removes a registration when the session it belongs to ends.
    pub struct Unregister {
        dir: PathBuf,
        pid: u32,
    }

    impl Drop for Unregister {
        fn drop(&mut self) {
            remove(&self.dir, self.pid);
        }
    }

    /// Serve the socket on a thread of its own while this process (`lsp`,
    /// `mcp`) does its work; the registration is removed when the returned
    /// guard is dropped. Never writes to stdout, which is the protocol's.
    pub fn serve_in_background(kind: Kind) -> Option<Unregister> {
        let reg = register(kind).ok()?;
        let guard = Unregister { dir: reg.dir.clone(), pid: reg.pid };
        std::thread::Builder::new()
            .name("codelaw-warm".into())
            .spawn(move || {
                for stream in reg.listener.incoming().flatten() {
                    answer(&reg, stream);
                }
            })
            .ok()?;
        Some(guard)
    }

    pub fn daemon(action: DaemonAction) -> ExitCode {
        match action {
            DaemonAction::Run => run_daemon(),
            DaemonAction::Start => start(),
            DaemonAction::Stop => stop(),
            DaemonAction::Status => status(),
        }
    }

    fn here() -> Result<(PathBuf, PathBuf), String> {
        let cwd = canonical_cwd().ok_or("the current folder cannot be read")?;
        run_dir(&cwd, EngineOptions::from_env().cache_dir_env.as_deref())
            .ok_or_else(|| format!("no codelaw config in {} or above; a warm process serves one project", codelaw_core::normalize_path(&cwd)))
    }

    /// The version a binary on disk reports.
    fn version_of(exe: &Path) -> Option<String> {
        let out = std::process::Command::new(exe).arg("--version").stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
        String::from_utf8(out.stdout).ok()?.trim().strip_prefix("codelaw ").map(str::to_owned)
    }

    /// The daemon itself: serve until idle for `[daemon] idle_timeout`,
    /// until the binary on disk is another version, or until `stop`
    /// (SR-ARCHITECTURE decision 22). Config, rules and baseline are read
    /// afresh for every request, so a change to any of them is served at
    /// once; the compiled rule set is reused while they are unchanged.
    fn run_daemon() -> ExitCode {
        let reg = match register(Kind::Daemon) {
            Ok(r) => r,
            Err(e) => return crate::fail(e),
        };
        if reg.listener.set_nonblocking(true).is_err() {
            return crate::fail("cannot serve the daemon's socket");
        }
        let idle = || codelaw_config::find_config(&reg.root).map(|c| c.daemon_idle_timeout()).unwrap_or(Duration::from_secs(30 * 60));
        let mut timeout = idle();
        let exe = std::env::current_exe().ok();
        let stamp = |p: &Path| fs::metadata(p).ok().map(|m| (m.len(), m.modified().ok()));
        let exe_stamp = exe.as_deref().and_then(stamp);
        let mut last = Instant::now();
        let mut checked = Instant::now();
        loop {
            match reg.listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    match answer(&reg, stream) {
                        Asked::Stop => return ExitCode::SUCCESS,
                        Asked::Served => {
                            last = Instant::now();
                            timeout = idle();
                        }
                        Asked::Nothing => {}
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(20)),
                Err(_) => std::thread::sleep(Duration::from_millis(20)),
            }
            if last.elapsed() >= timeout {
                return ExitCode::SUCCESS;
            }
            // A binary replaced on disk (an upgrade) that reports another
            // version ends the daemon; one that is gone ends it too.
            if checked.elapsed() >= Duration::from_secs(2) {
                checked = Instant::now();
                if let Some(exe) = exe.as_deref()
                    && stamp(exe) != exe_stamp
                    && version_of(exe).as_deref() != Some(VERSION)
                {
                    return ExitCode::SUCCESS;
                }
            }
        }
    }

    fn start() -> ExitCode {
        let (dir, root) = match here() {
            Ok(h) => h,
            Err(e) => return crate::fail(e),
        };
        let root = codelaw_core::normalize_path(&root);
        if let Some((r, _)) = discover(&dir).into_iter().find(|(r, _)| r.kind == Kind::Daemon.name() && r.version == VERSION) {
            println!("codelaw daemon already running for {root} (pid {})", r.pid);
            return ExitCode::SUCCESS;
        }
        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => return crate::fail(format!("cannot find the codelaw binary: {e}")),
        };
        use std::os::unix::process::CommandExt;
        // Its own process group, so the terminal's Ctrl-C does not reach it.
        let child = std::process::Command::new(exe).args(["daemon", "run"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).process_group(0).spawn();
        let mut child = match child {
            Ok(c) => c,
            Err(e) => return crate::fail(format!("cannot start the daemon: {e}")),
        };
        let pid = child.id();
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(10) {
            if discover(&dir).iter().any(|(r, _)| r.pid == pid) {
                println!("codelaw daemon started for {root} (pid {pid})");
                return ExitCode::SUCCESS;
            }
            if let Ok(Some(status)) = child.try_wait() {
                return crate::fail(format!("the daemon exited at start ({status}); `codelaw daemon run` shows why"));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = child.kill();
        crate::fail("the daemon did not start within 10 s")
    }

    fn stop() -> ExitCode {
        let (dir, root) = match here() {
            Ok(h) => h,
            Err(e) => return crate::fail(e),
        };
        let root = codelaw_core::normalize_path(&root);
        let mut stopped = 0;
        for (r, mut stream) in discover(&dir).into_iter().filter(|(r, _)| r.kind == Kind::Daemon.name()) {
            if exchange(&mut stream, &json!({ "op": "stop" })).is_none() {
                continue;
            }
            // Gone once its socket is: it removes its own on the way out.
            let asked = Instant::now();
            while dir.join(sock_name(r.pid)).exists() && asked.elapsed() < Duration::from_secs(5) {
                std::thread::sleep(Duration::from_millis(20));
            }
            println!("codelaw daemon stopped for {root} (pid {})", r.pid);
            stopped += 1;
        }
        if stopped == 0 {
            println!("no codelaw daemon is running for {root}");
        }
        ExitCode::SUCCESS
    }

    fn status() -> ExitCode {
        let (dir, root) = match here() {
            Ok(h) => h,
            Err(e) => return crate::fail(e),
        };
        let root = codelaw_core::normalize_path(&root);
        let mut any = false;
        for (r, mut stream) in discover(&dir) {
            // A process that accepts and never answers is listed, not waited on.
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let served = exchange(&mut stream, &json!({ "op": "status" })).and_then(|v| v["served"].as_u64()).unwrap_or(0);
            let serves = if r.any_cwd { "any folder".to_owned() } else { codelaw_core::normalize_path(&r.cwd) };
            println!("{} pid {} codelaw {} served {served} (serves {serves})", r.kind, r.pid, r.version);
            any = true;
        }
        if !any {
            println!("no warm codelaw process for {root}");
            return ExitCode::from(1);
        }
        ExitCode::SUCCESS
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn temp(name: &str) -> PathBuf {
            let d = std::env::temp_dir().join(format!("codelaw-warm-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&d);
            fs::create_dir_all(&d).unwrap();
            fs::canonicalize(d).unwrap()
        }

        fn write(dir: &Path, r: &Registration) {
            fs::write(dir.join(json_name(r.pid)), r.json().to_string()).unwrap();
        }

        fn reg(pid: u32, version: &str) -> Registration {
            Registration { pid, kind: "daemon".into(), version: version.into(), cwd: "/x".into(), any_cwd: true }
        }

        #[test]
        fn stale_sockets_are_removed_and_live_ones_kept_newest_first() {
            let dir = temp("discover");
            // A dead process: its socket file is left, nobody accepts.
            let dead = bind(&dir, 4_000_001).unwrap();
            drop(dead);
            write(&dir, &reg(4_000_001, VERSION));
            // A registration without a socket, and a socket without one.
            write(&dir, &reg(4_000_002, VERSION));
            drop(bind(&dir, 4_000_003).unwrap());
            // A registration that is not one.
            fs::write(dir.join("4000004.json"), "{").unwrap();
            // Two live ones, the second newer.
            let older = bind(&dir, 4_000_005).unwrap();
            write(&dir, &reg(4_000_005, VERSION));
            std::thread::sleep(Duration::from_millis(20));
            let newer = bind(&dir, 4_000_006).unwrap();
            write(&dir, &reg(4_000_006, "0.0.0-other"));
            let live: Vec<u32> = discover(&dir).into_iter().map(|(r, _)| r.pid).collect();
            assert_eq!(live, [4_000_006, 4_000_005]);
            let mut left: Vec<String> = fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
            left.sort();
            assert_eq!(left, ["4000005.json", "4000005.sock", "4000006.json", "4000006.sock"]);
            drop((older, newer));
            assert!(discover(&dir).is_empty());
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 0, "every stale socket is removed");
            let _ = fs::remove_dir_all(&dir);
        }

        #[test]
        fn a_long_folder_still_binds_and_connects() {
            let mut dir = temp("long");
            while dir.as_os_str().len() < 140 {
                dir = dir.join("a-rather-long-folder-name");
            }
            fs::create_dir_all(&dir).unwrap();
            let listener = bind(&dir, 4_000_007).unwrap();
            write(&dir, &reg(4_000_007, VERSION));
            assert_eq!(discover(&dir).len(), 1);
            drop(listener);
            assert!(discover(&dir).is_empty());
            let _ = fs::remove_dir_all(temp("long"));
        }
    }
}
