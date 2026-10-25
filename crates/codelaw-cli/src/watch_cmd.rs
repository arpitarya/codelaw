//! `codelaw check --watch` and `fix --watch` (SR-CLI-MCP decision 2a,
//! SR-ARCHITECTURE decision 22a): the engine's `Watch` does the work; this
//! loop polls it, waits for the changes to settle, and prints each run.
//! It runs in this process on every platform, the warm socket aside.

use crate::{CheckArgs, Common, FixArgs, engine_options};
use codelaw_engine::{Changes, CheckReport, EngineOptions, FixOptions, FixReport, Rerun, Watch};
use codelaw_report::Format;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// How often the files are looked at while nothing changes.
const POLL: Duration = Duration::from_millis(100);
/// A change is acted on once a look this long after it saw nothing new.
const SETTLE: Duration = Duration::from_millis(100);
/// Files that never stop changing are checked at least this often.
const SETTLE_MAX: Duration = Duration::from_secs(2);

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

extern "C" fn on_interrupt(_: libc::c_int) {
    // The first Ctrl-C ends the watch once the run in flight is printed;
    // a second one ends it at once.
    if INTERRUPTED.swap(true, Ordering::SeqCst) {
        #[cfg(unix)]
        unsafe {
            libc::_exit(0)
        };
        #[cfg(not(unix))]
        std::process::exit(0);
    }
    #[cfg(not(unix))]
    unsafe {
        // The C runtime resets the handler after each signal on Windows.
        libc::signal(libc::SIGINT, on_interrupt as *const () as libc::sighandler_t);
    }
}

fn catch_interrupt() {
    unsafe {
        libc::signal(libc::SIGINT, on_interrupt as *const () as libc::sighandler_t);
    }
}

fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// What every run prints with.
struct Printer {
    format: Format,
    color: bool,
    /// Clear the screen before each text run: stdout is a terminal.
    clear: bool,
    run: usize,
}

impl Printer {
    fn new(format: Format) -> Printer {
        let tty = std::io::stdout().is_terminal();
        Printer { format, color: crate::color(), clear: tty && format == Format::Text, run: 0 }
    }

    fn dim(&self, s: &str) -> String {
        if self.color { format!("\x1b[2m{s}\x1b[0m") } else { s.to_owned() }
    }

    /// One run: the report as `check` prints it, then the watch's line
    /// (text), or one JSON object on one line (json, sarif).
    fn report(&mut self, watch: &Watch, report: &CheckReport, rerun: Option<&Rerun>, fixed: Option<&FixReport>) {
        self.run += 1;
        let changed: &[String] = rerun.map(|r| r.changed.as_slice()).unwrap_or_default();
        let reloaded = rerun.is_some_and(|r| r.reloaded);
        let mut out = String::new();
        match self.format {
            Format::Text => {
                if self.clear {
                    out.push_str("\x1b[2J\x1b[3J\x1b[H");
                }
                for f in fixed.map(|f| f.files.as_slice()).unwrap_or_default() {
                    let verb = if f.written { "fixed" } else { "not written" };
                    let applied: Vec<String> = f.applied.iter().map(|(r, n)| format!("{r} ×{n}")).collect();
                    out.push_str(&format!("{verb} {} ({})\n", f.file, applied.join(", ")));
                }
                out.push_str(&codelaw_report::check_text(report, self.color));
                out.push_str(&self.dim(&watch_line(self.run, watch.watched(), changed, reloaded)));
                out.push('\n');
            }
            Format::Json => {
                let mut v = codelaw_report::check_json(report);
                v["watch"] = watch_json(self.run, watch.watched(), changed, reloaded);
                if let Some(f) = fixed {
                    v["fixed"] = codelaw_report::fix_json(f)["fixed"].clone();
                }
                out = serde_json::to_string(&v).expect("json") + "\n";
            }
            Format::Sarif => out = serde_json::to_string(&codelaw_report::check_sarif(report, watch.project())).expect("json") + "\n",
            Format::Agent => out.push_str(&codelaw_report::check_agent(report)),
        }
        let mut stdout = std::io::stdout().lock();
        let _ = stdout.write_all(out.as_bytes());
        let _ = stdout.flush();
    }

    /// A run that could not be made: the error, and the watch goes on.
    fn error(&mut self, watch: &Watch, e: &dyn std::fmt::Display, changed: &[String]) {
        self.run += 1;
        eprintln!("error: {e}");
        match self.format {
            Format::Json => {
                let v = serde_json::json!({
                    "schema_version": codelaw_report::SCHEMA_VERSION,
                    "codelaw_version": codelaw_core::VERSION,
                    "watch": watch_json(self.run, watch.watched(), changed, true),
                    "error": e.to_string(),
                });
                let mut stdout = std::io::stdout().lock();
                let _ = writeln!(stdout, "{}", serde_json::to_string(&v).expect("json"));
                let _ = stdout.flush();
            }
            Format::Text => eprintln!("{}", self.dim("waiting for a change · Ctrl-C to stop")),
            _ => {}
        }
    }
}

/// `watching 1,204 files · run 3 · src/a.ts changed · Ctrl-C to stop`.
fn watch_line(run: usize, watched: usize, changed: &[String], reloaded: bool) -> String {
    let mut parts = vec![format!("watching {} {}", watched, if watched == 1 { "file" } else { "files" }), format!("run {run}")];
    match changed {
        [] => {}
        [one] => parts.push(format!("{one} changed")),
        [first, rest @ ..] => parts.push(format!("{first} and {} more changed", rest.len())),
    }
    if reloaded {
        parts.push("project reloaded".into());
    }
    parts.push("Ctrl-C to stop".into());
    parts.join(" · ")
}

fn watch_json(run: usize, watched: usize, changed: &[String], reloaded: bool) -> serde_json::Value {
    serde_json::json!({ "run": run, "watched": watched, "changed": changed, "reloaded": reloaded })
}

/// The options a watch runs with: the command's, with `local` as the
/// profile unless `--profile` or `CODELAW_PROFILE` names another (the
/// person at the keyboard, not CI; SR-CONFIG decision 10).
fn options(common: &Common, adapters: bool) -> Result<EngineOptions, String> {
    let mut o = engine_options(common)?;
    o.adapters = adapters;
    o.profile_surface = Some(("local", "--watch"));
    Ok(o)
}

/// Where the project is found from: the first path's folder, or here.
fn start(paths: &[PathBuf]) -> PathBuf {
    let first = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
    let dir = if first.is_file() { first.parent().map(Path::to_path_buf).unwrap_or_default() } else { first };
    if dir.as_os_str().is_empty() { PathBuf::from(".") } else { dir }
}

/// Wait for a change and for it to settle; `None` once interrupted.
fn next_change(watch: &mut Watch) -> Option<Changes> {
    loop {
        if interrupted() {
            return None;
        }
        std::thread::sleep(POLL);
        let mut changes = watch.poll();
        if changes.is_empty() {
            continue;
        }
        let began = Instant::now();
        loop {
            std::thread::sleep(SETTLE);
            let more = watch.poll();
            let quiet = more.is_empty();
            changes.merge(more);
            if quiet || began.elapsed() >= SETTLE_MAX || interrupted() {
                break;
            }
        }
        return Some(changes);
    }
}

pub fn check(args: CheckArgs) -> ExitCode {
    let format = match crate::format(&args.format) {
        Ok(f) => f,
        Err(e) => return crate::fail(e),
    };
    let options = match options(&args.common, !args.no_adapters) {
        Ok(o) => o,
        Err(e) => return crate::fail(e),
    };
    let (mut watch, report) = match Watch::start(&start(&args.paths), args.paths.clone(), options) {
        Ok(w) => w,
        Err(e) => return crate::fail(e),
    };
    catch_interrupt();
    let mut printer = Printer::new(format);
    printer.report(&watch, &report, None, None);
    while let Some(changes) = next_change(&mut watch) {
        match watch.rerun(&changes) {
            Ok(Some(r)) => printer.report(&watch, &r.report, Some(&r), None),
            Ok(None) => {}
            Err(e) => printer.error(&watch, &e, &rels(&watch, &changes)),
        }
    }
    ExitCode::SUCCESS
}

pub fn fix(args: FixArgs) -> ExitCode {
    let format = match crate::format(&args.format) {
        Ok(f) => f,
        Err(e) => return crate::fail(e),
    };
    let options = match options(&args.common, !args.no_adapters) {
        Ok(o) => o,
        Err(e) => return crate::fail(e),
    };
    eprintln!("warning: `fix --watch` writes each file you save; an editor that also fixes or formats on save can fight it");
    let fix = FixOptions { unsafe_fixes: args.unsafe_fixes, dry_run: args.dry_run, trace: false };
    let (mut watch, report, fixed) = match Watch::start_fixing(&start(&args.paths), args.paths.clone(), options, &fix) {
        Ok(w) => w,
        Err(e) => return crate::fail(e),
    };
    catch_interrupt();
    let mut printer = Printer::new(format);
    printer.report(&watch, &report, None, Some(&fixed));
    while let Some(changes) = next_change(&mut watch) {
        match watch.fix(&changes, &fix) {
            Ok(Some((r, fixed))) => printer.report(&watch, &r.report, Some(&r), Some(&fixed)),
            Ok(None) => {}
            Err(e) => printer.error(&watch, &e, &rels(&watch, &changes)),
        }
    }
    ExitCode::SUCCESS
}

fn rels(watch: &Watch, changes: &Changes) -> Vec<String> {
    let root = watch.project().root();
    changes.paths.iter().map(|p| codelaw_core::normalize_path(p.strip_prefix(root).unwrap_or(p))).collect()
}
