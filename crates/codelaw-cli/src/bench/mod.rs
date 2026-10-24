//! `codelaw bench` (L8 decision 3, SR-CLI-MCP decision 10): generate the
//! reference workload, time every L8 operation end to end by running this
//! very binary as a user, an editor or an agent would, and compare each p95
//! with `perf/budgets.toml`.
//!
//! - Each operation runs `--runs` times (10 by default); the first run is
//!   discarded and the median and p95 (nearest rank) of the rest reported.
//! - **Cold** is a fresh, empty cache folder per run and no warm process.
//!   **Warm** is a primed cache, and for `check` and `format` the W-15
//!   daemon serving `--prefer-warm` (the bench refuses a warm number the
//!   daemon did not serve).
//! - External adapters are off (`--no-adapters`): their time is not codelaw's.
//! - Caches live beside the workload, never inside it, so they are never
//!   walked as project files.

pub mod budgets;
pub mod workload;

use clap::Args;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitCode, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// One measured operation.
pub struct Op {
    pub name: &'static str,
    pub unit: &'static str,
    /// An L8 operation (decision 1); the others are reported for context only.
    pub budgeted: bool,
    pub what: &'static str,
}

/// L8 decision 1's operations, then the context rows.
pub const OPERATIONS: &[Op] = &[
    Op { name: "startup", unit: "ms", budgeted: true, what: "`codelaw --version`, spawn to exit" },
    Op { name: "check_warm_10k", unit: "ms", budgeted: true, what: "`check --prefer-warm`, served by the daemon, primed cache" },
    Op { name: "check_cold_10k", unit: "ms", budgeted: true, what: "`check`, empty cache, no warm process" },
    Op { name: "format_warm_10k", unit: "ms", budgeted: true, what: "`format --check --prefer-warm`, served by the daemon, primed cache" },
    Op { name: "fix_one_file", unit: "ms", budgeted: true, what: "`codelaw hook <file>` on one just-edited file, primed cache" },
    Op { name: "lsp_after_edit", unit: "ms", budgeted: true, what: "`didChange` to `publishDiagnostics` in `codelaw lsp`" },
    Op { name: "mcp_rules_for_path", unit: "ms", budgeted: true, what: "one `rules_for_path` call to `codelaw mcp`" },
    Op { name: "peak_memory_10k", unit: "MiB", budgeted: true, what: "peak resident memory of `check_cold_10k`" },
];

/// Context rows: not L8 operations, never gated.
pub const CONTEXT: &[Op] = &[
    Op { name: "check_warm_10k_no_daemon", unit: "ms", budgeted: false, what: "`check` in its own process, primed cache" },
    Op { name: "fix_one_file_cold", unit: "ms", budgeted: false, what: "`codelaw hook <file>`, empty cache" },
];

#[derive(Args)]
pub struct BenchArgs {
    /// Where the reference workload is generated; reused when it matches.
    /// A temporary folder by default.
    #[arg(long, value_name = "DIR")]
    workload: Option<PathBuf>,
    /// The first-party packs to install into the workload (the repo's
    /// `packs/`); found above the current folder by default.
    #[arg(long, value_name = "DIR")]
    packs: Option<PathBuf>,
    /// Files in the workload.
    #[arg(long, default_value_t = workload::FILES)]
    files: usize,
    /// The workload's seed.
    #[arg(long, default_value_t = workload::SEED)]
    seed: u64,
    /// Runs per operation; the first is discarded.
    #[arg(long, default_value_t = 10)]
    runs: usize,
    /// Only these operations (comma-separated).
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,
    /// Compare each p95 with this budgets file: exit 1 when one is over a
    /// budget the decider set.
    #[arg(long, value_name = "FILE")]
    budgets: Option<PathBuf>,
    /// text or json.
    #[arg(long, default_value = "text")]
    format: String,
    /// Also write the JSON report to this file.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Generate the workload, print where, and stop.
    #[arg(long)]
    generate_only: bool,
}

/// One operation's samples, after the discarded first run.
struct Measured {
    op: &'static Op,
    samples: Vec<f64>,
}

impl Measured {
    fn sorted(&self) -> Vec<f64> {
        let mut s = self.samples.clone();
        s.sort_by(f64::total_cmp);
        s
    }

    fn median(&self) -> f64 {
        let s = self.sorted();
        let n = s.len();
        if n % 2 == 1 { s[n / 2] } else { (s[n / 2 - 1] + s[n / 2]) / 2.0 }
    }

    /// Nearest rank.
    fn p95(&self) -> f64 {
        let s = self.sorted();
        let rank = ((0.95 * s.len() as f64).ceil() as usize).clamp(1, s.len());
        s[rank - 1]
    }
}

fn find_packs() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok();
    while let Some(d) = dir {
        if d.join("packs/js/codelaw-pack.toml").is_file() {
            return Some(d.join("packs"));
        }
        dir = d.parent().map(Path::to_path_buf);
    }
    None
}

pub fn bench(args: BenchArgs) -> ExitCode {
    match run(args) {
        Ok(code) => code,
        Err(e) => crate::fail(e),
    }
}

fn run(args: BenchArgs) -> Result<ExitCode, String> {
    if args.format != "text" && args.format != "json" {
        return Err(format!("unknown --format `{}` (text or json)", args.format));
    }
    if args.runs < 2 {
        return Err("--runs is at least 2: the first run is discarded".into());
    }
    if args.files < 100 {
        return Err("--files is at least 100".into());
    }
    let all: Vec<&'static Op> = OPERATIONS.iter().chain(CONTEXT).collect();
    for o in &args.only {
        if !all.iter().any(|op| op.name == o) {
            let near = codelaw_config::did_you_mean(o, all.iter().map(|op| op.name)).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
            return Err(format!("unknown operation `{o}` in --only{near}"));
        }
    }
    let wanted = |name: &str| args.only.is_empty() || args.only.iter().any(|o| o == name);
    let budgets = args.budgets.as_deref().map(budgets::load).transpose()?;

    let packs = args.packs.clone().or_else(find_packs).ok_or("no `packs/` folder above the current one; name it with --packs")?;
    let dir = args.workload.clone().unwrap_or_else(|| std::env::temp_dir().join("codelaw-bench").join(format!("{}-{}", args.seed, args.files)));
    let digest = workload::write(&dir, &packs, args.seed, args.files)?;
    let dir = std::fs::canonicalize(&dir).map_err(|e| e.to_string())?;
    if args.generate_only {
        println!("{} ({} files, seed {}, digest {digest})", codelaw_core::normalize_path(&dir), args.files, args.seed);
        return Ok(ExitCode::SUCCESS);
    }
    let caches = dir.with_file_name(format!("{}.caches", dir.file_name().and_then(|n| n.to_str()).unwrap_or("workload")));
    let _ = std::fs::remove_dir_all(&caches);
    std::fs::create_dir_all(&caches).map_err(|e| e.to_string())?;
    let exe = std::env::current_exe().map_err(|e| format!("cannot find the codelaw binary: {e}"))?;
    let b = Bench { exe, dir: dir.clone(), caches: caches.clone(), runs: args.runs };
    let target = target_file(args.seed, args.files);

    let mut results: Vec<Measured> = Vec::new();
    let mut served_warm: Option<u64> = None;
    let op = |name: &str| all.iter().copied().find(|o| o.name == name).expect("known operation");

    if wanted("startup") {
        results.push(b.repeat(op("startup"), |_| b.cli(&["--version"], None).map(|r| r.ms))?);
    }
    if wanted("check_cold_10k") || wanted("peak_memory_10k") {
        let mut peaks = Vec::new();
        let cold = b.repeat(op("check_cold_10k"), |i| {
            let cache = caches.join(format!("cold-{i}"));
            let r = b.cli(&["check", "--no-adapters"], Some(&cache))?;
            let _ = std::fs::remove_dir_all(&cache);
            peaks.push(r.peak_mib);
            Ok(r.ms)
        })?;
        if wanted("check_cold_10k") {
            results.push(cold);
        }
        // The first run is discarded here too.
        let peaks: Option<Vec<f64>> = peaks.into_iter().skip(1).collect();
        if wanted("peak_memory_10k")
            && let Some(samples) = peaks
        {
            results.push(Measured { op: op("peak_memory_10k"), samples });
        }
    }
    let warm = caches.join("warm");
    if wanted("check_warm_10k_no_daemon") {
        b.cli(&["check", "--no-adapters"], Some(&warm))?;
        results.push(b.repeat(op("check_warm_10k_no_daemon"), |_| b.cli(&["check", "--no-adapters"], Some(&warm)).map(|r| r.ms))?);
    }
    if wanted("check_warm_10k") || wanted("format_warm_10k") {
        b.daemon("start", &warm)?;
        let measured = (|| -> Result<(), String> {
            if wanted("check_warm_10k") {
                b.cli(&["check", "--no-adapters", "--prefer-warm"], Some(&warm))?;
                results.push(b.repeat(op("check_warm_10k"), |_| b.cli(&["check", "--no-adapters", "--prefer-warm"], Some(&warm)).map(|r| r.ms))?);
            }
            if wanted("format_warm_10k") {
                b.cli(&["format", "--check", "--prefer-warm"], Some(&warm))?;
                results.push(b.repeat(op("format_warm_10k"), |_| b.cli(&["format", "--check", "--prefer-warm"], Some(&warm)).map(|r| r.ms))?);
            }
            Ok(())
        })();
        let status = b.cli(&["daemon", "status"], Some(&warm));
        b.daemon("stop", &warm)?;
        measured?;
        let served = status?.stdout.lines().filter(|l| l.starts_with("daemon ")).filter_map(|l| l.split(" served ").nth(1)?.split_whitespace().next()?.parse::<u64>().ok()).sum();
        let expected = (args.runs as u64 + 1) * [wanted("check_warm_10k"), wanted("format_warm_10k")].iter().filter(|w| **w).count() as u64;
        if served < expected {
            return Err(format!("the daemon served {served} of the {expected} warm runs; a warm number it did not serve is not a warm number"));
        }
        served_warm = Some(served);
    }
    if wanted("fix_one_file") || wanted("fix_one_file_cold") {
        let path = dir.join(&target.path);
        let dirty = format!("{}\nexport const  unformatted =  1 ;\n", target.text);
        let p = path.to_string_lossy().into_owned();
        let measured = (|| -> Result<(), String> {
            if wanted("fix_one_file") {
                b.cli(&["check", "--no-adapters"], Some(&warm))?;
                results.push(b.repeat(op("fix_one_file"), |_| {
                    std::fs::write(&path, &dirty).map_err(|e| e.to_string())?;
                    b.cli(&["hook", &p], Some(&warm)).map(|r| r.ms)
                })?);
            }
            if wanted("fix_one_file_cold") {
                results.push(b.repeat(op("fix_one_file_cold"), |i| {
                    std::fs::write(&path, &dirty).map_err(|e| e.to_string())?;
                    let cache = caches.join(format!("hook-cold-{i}"));
                    let r = b.cli(&["hook", &p], Some(&cache));
                    let _ = std::fs::remove_dir_all(&cache);
                    r.map(|r| r.ms)
                })?);
            }
            Ok(())
        })();
        std::fs::write(&path, &target.text).map_err(|e| e.to_string())?;
        measured?;
    }
    if wanted("lsp_after_edit") {
        results.push(b.lsp(op("lsp_after_edit"), &target, &warm)?);
    }
    if wanted("mcp_rules_for_path") {
        results.push(b.mcp(op("mcp_rules_for_path"), &target, &warm)?);
    }
    let _ = std::fs::remove_dir_all(&caches);

    // The report, in the order of OPERATIONS then CONTEXT.
    results.sort_by_key(|m| all.iter().position(|o| o.name == m.op.name));
    let mut over = false;
    let rows: Vec<Value> = results
        .iter()
        .map(|m| {
            let entry = budgets.as_ref().and_then(|b| b.operations.get(m.op.name));
            let (median, p95) = (m.median(), m.p95());
            let sorted = m.sorted();
            let v = if m.op.budgeted && budgets.is_some() { Some(budgets::verdict(entry, p95)) } else { None };
            over |= v == Some(budgets::Verdict::Over);
            json!({
                "name": m.op.name,
                "unit": m.op.unit,
                "budgeted": m.op.budgeted,
                "what": m.op.what,
                "samples": m.samples.iter().map(|x| round(*x)).collect::<Vec<_>>(),
                "median": round(median),
                "p95": round(p95),
                "min": round(sorted[0]),
                "max": round(sorted[sorted.len() - 1]),
                "budget": entry.and_then(|e| e.budget),
                "proposed": entry.and_then(|e| e.proposed),
                "verdict": v.map(|v| match v {
                    budgets::Verdict::Within => "within",
                    budgets::Verdict::Over => "over",
                    budgets::Verdict::Unset => "unset",
                }),
            })
        })
        .collect();
    let report = json!({
        "schema_version": 1,
        "codelaw_version": env!("CARGO_PKG_VERSION"),
        "workload": { "seed": args.seed, "files": args.files, "digest": digest, "reference": digest == workload::REFERENCE_DIGEST, "path": codelaw_core::normalize_path(&dir) },
        "runs": args.runs,
        "discarded": 1,
        "threads": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        "os": format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        "warm_runs_served_by_daemon": served_warm,
        "reference_machine": budgets.as_ref().map(|b| b.machine.clone()),
        "operations": rows,
    });
    let pretty = serde_json::to_string_pretty(&report).expect("json");
    if let Some(out) = &args.output {
        std::fs::write(out, format!("{pretty}\n")).map_err(|e| format!("cannot write {}: {e}", codelaw_core::normalize_path(out)))?;
    }
    if args.format == "json" {
        println!("{pretty}");
    } else {
        print!("{}", text(&report));
    }
    Ok(if over { ExitCode::from(1) } else { ExitCode::SUCCESS })
}

fn round(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn text(r: &Value) -> String {
    let mut s = format!(
        "codelaw bench {} — workload {}{} ({} files, seed {}), {} runs per operation, first discarded, on {} with {} threads\n\n",
        r["codelaw_version"].as_str().unwrap_or(""),
        r["workload"]["digest"].as_str().unwrap_or(""),
        if r["workload"]["reference"] == true { ", the reference workload" } else { ", NOT the reference workload: not comparable with the budgets" },
        r["workload"]["files"],
        r["workload"]["seed"],
        r["runs"],
        r["os"].as_str().unwrap_or(""),
        r["threads"],
    );
    s.push_str(&format!("{:<26} {:>10} {:>10} {:>21}  {}\n", "operation", "median", "p95", "min – max", "budget"));
    let num = |v: &Value| v.as_f64().map(|x| format!("{x:.1}")).unwrap_or_else(|| "-".into());
    for o in r["operations"].as_array().into_iter().flatten() {
        let unit = o["unit"].as_str().unwrap_or("");
        let budget = if !o["budgeted"].as_bool().unwrap_or(false) {
            "context, not an L8 operation".to_owned()
        } else {
            match o["verdict"].as_str() {
                None => "not compared (no --budgets)".to_owned(),
                Some("unset") => match o["proposed"].as_f64() {
                    Some(p) => format!("unset (proposed {p} {unit}, awaiting the decider): reported, not gated"),
                    None => "unset: reported, not gated".to_owned(),
                },
                Some(v) => format!("{} {unit}: {}", num(&o["budget"]), if v == "over" { "OVER BUDGET" } else { "within" }),
            }
        };
        s.push_str(&format!(
            "{:<26} {:>7} {:<3}{:>6} {:<3} {:>8} – {:<8} {}\n",
            o["name"].as_str().unwrap_or(""),
            num(&o["median"]),
            unit,
            num(&o["p95"]),
            unit,
            num(&o["min"]),
            num(&o["max"]),
            budget
        ));
    }
    if let Some(n) = r["warm_runs_served_by_daemon"].as_u64() {
        s.push_str(&format!("\nwarm runs served by the daemon: {n}\n"));
    }
    s
}

/// The file the one-file operations use: a mid-folder TypeScript file.
fn target_file(seed: u64, files: usize) -> workload::File {
    let set = workload::generate(seed, files);
    let pick = set.iter().position(|f| f.path.ends_with("/m5.ts")).or_else(|| set.iter().position(|f| f.path.ends_with(".ts"))).unwrap_or(0);
    set.into_iter().nth(pick).expect("a workload has files")
}

struct Bench {
    exe: PathBuf,
    dir: PathBuf,
    caches: PathBuf,
    runs: usize,
}

struct Ran {
    ms: f64,
    peak_mib: Option<f64>,
    stdout: String,
}

impl Bench {
    /// `runs` samples of `f(i)`, the first discarded.
    fn repeat(&self, op: &'static Op, mut f: impl FnMut(usize) -> Result<f64, String>) -> Result<Measured, String> {
        let mut samples = Vec::with_capacity(self.runs);
        for i in 0..self.runs {
            samples.push(f(i).map_err(|e| format!("{}: {e}", op.name))?);
        }
        samples.remove(0);
        Ok(Measured { op, samples })
    }

    fn command(&self, cache: Option<&Path>) -> Command {
        let mut c = Command::new(&self.exe);
        c.current_dir(&self.dir).env("NO_COLOR", "1");
        if let Some(cache) = cache {
            c.env("CODELAW_CACHE_DIR", cache);
        }
        c
    }

    /// Run `codelaw <args>` in the workload and time it, spawn to exit. A
    /// finding (exit 1, or the hook's 2) is the workload's; `error:` is not.
    fn cli(&self, args: &[&str], cache: Option<&Path>) -> Result<Ran, String> {
        let out = self.caches.join("stdout");
        let err = self.caches.join("stderr");
        let file = |p: &Path| std::fs::File::create(p).map_err(|e| e.to_string());
        let mut c = self.command(cache);
        c.args(args).stdout(file(&out)?).stderr(file(&err)?).stdin(Stdio::null());
        let started = Instant::now();
        let mut child = c.spawn().map_err(|e| format!("cannot run {}: {e}", codelaw_core::normalize_path(&self.exe)))?;
        let (code, peak) = wait(&mut child)?;
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        let stderr = std::fs::read_to_string(&err).unwrap_or_default();
        if code > 2 || stderr.lines().any(|l| l.starts_with("error:")) {
            return Err(format!("`codelaw {}` exited {code}: {}", args.join(" "), stderr.trim()));
        }
        Ok(Ran { ms, peak_mib: peak.map(|b| b as f64 / (1024.0 * 1024.0)), stdout: std::fs::read_to_string(&out).unwrap_or_default() })
    }

    fn daemon(&self, action: &str, cache: &Path) -> Result<(), String> {
        let out = self.command(Some(cache)).args(["daemon", action]).stdin(Stdio::null()).output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(format!("`codelaw daemon {action}`: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(())
    }

    fn spawn_server(&self, args: &[&str], cache: &Path) -> Result<(Child, ChildStdin, std::process::ChildStdout), String> {
        let mut child = self.command(Some(cache)).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|e| e.to_string())?;
        let stdin = child.stdin.take().ok_or("no stdin")?;
        let stdout = child.stdout.take().ok_or("no stdout")?;
        Ok((child, stdin, stdout))
    }

    /// `didChange` → `publishDiagnostics` for the target, in one `codelaw
    /// lsp` session: each run inserts a comment line, the next removes it.
    fn lsp(&self, op: &'static Op, target: &workload::File, cache: &Path) -> Result<Measured, String> {
        let (mut child, mut stdin, stdout) = self.spawn_server(&["lsp", "--stdio"], cache)?;
        let rx = read_lsp(stdout);
        let root = file_uri(&self.dir);
        let uri = file_uri(&self.dir.join(&target.path));
        let send = |stdin: &mut ChildStdin, v: Value| -> Result<(), String> {
            let body = v.to_string();
            stdin.write_all(format!("Content-Length: {}\r\n\r\n{body}", body.len()).as_bytes()).and_then(|()| stdin.flush()).map_err(|e| format!("lsp: {e}"))
        };
        let published = |rx: &Receiver<Value>| wait_for(rx, |m| m["method"] == "textDocument/publishDiagnostics" && m["params"]["uri"] == uri.as_str());
        let result = (|| {
            send(&mut stdin, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"processId": null, "rootUri": root, "capabilities": {}, "workspaceFolders": [{"uri": root, "name": "workload"}]}}))?;
            wait_for(&rx, |m| m["id"] == 1)?;
            send(&mut stdin, json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}))?;
            send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": uri, "languageId": "typescript", "version": 1, "text": target.text}}}))?;
            published(&rx)?;
            let mut version = 1;
            self.repeat(op, |i| {
                version += 1;
                let change = if i % 2 == 0 {
                    json!({"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}, "text": "// edited\n"})
                } else {
                    json!({"range": {"start": {"line": 0, "character": 0}, "end": {"line": 1, "character": 0}}, "text": ""})
                };
                let started = Instant::now();
                send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {"textDocument": {"uri": uri, "version": version}, "contentChanges": [change]}}))?;
                published(&rx)?;
                Ok(started.elapsed().as_secs_f64() * 1000.0)
            })
        })();
        let _ = send(&mut stdin, json!({"jsonrpc": "2.0", "id": 99, "method": "shutdown"}));
        let _ = wait_for(&rx, |m| m["id"] == 99);
        let _ = send(&mut stdin, json!({"jsonrpc": "2.0", "method": "exit"}));
        drop(stdin);
        finish(&mut child);
        result
    }

    /// One `tools/call rules_for_path` round trip to a running `codelaw mcp`.
    fn mcp(&self, op: &'static Op, target: &workload::File, cache: &Path) -> Result<Measured, String> {
        let (mut child, mut stdin, stdout) = self.spawn_server(&["mcp"], cache)?;
        let rx = read_lines(stdout);
        let send = |stdin: &mut ChildStdin, v: Value| -> Result<(), String> { writeln!(stdin, "{v}").and_then(|()| stdin.flush()).map_err(|e| format!("mcp: {e}")) };
        let path = self.dir.join(&target.path).to_string_lossy().into_owned();
        let result = (|| {
            send(&mut stdin, json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "codelaw-bench", "version": env!("CARGO_PKG_VERSION")}}}))?;
            wait_for(&rx, |m| m["id"] == 0)?;
            send(&mut stdin, json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))?;
            let mut id = 0;
            self.repeat(op, |_| {
                id += 1;
                let started = Instant::now();
                send(&mut stdin, json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": "rules_for_path", "arguments": {"path": path}}}))?;
                let answer = wait_for(&rx, |m| m["id"] == id)?;
                let ms = started.elapsed().as_secs_f64() * 1000.0;
                if answer.get("error").is_some() || answer["result"]["isError"] == true {
                    return Err(format!("rules_for_path failed: {answer}"));
                }
                Ok(ms)
            })
        })();
        drop(stdin);
        finish(&mut child);
        result
    }
}

fn file_uri(p: &Path) -> String {
    format!("file://{}", codelaw_core::normalize_path(p))
}

/// Messages from an LSP server's stdout, framed by `Content-Length`.
fn read_lsp(stdout: std::process::ChildStdout) -> Receiver<Value> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let mut r = BufReader::new(stdout);
        loop {
            let mut len = None;
            loop {
                let mut line = String::new();
                if r.read_line(&mut line).unwrap_or(0) == 0 {
                    return;
                }
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some(n) = line.strip_prefix("Content-Length:") {
                    len = n.trim().parse::<usize>().ok();
                }
            }
            let Some(len) = len else { return };
            let mut body = vec![0; len];
            if r.read_exact(&mut body).is_err() {
                return;
            }
            if let Ok(v) = serde_json::from_slice(&body)
                && tx.send(v).is_err()
            {
                return;
            }
        }
    });
    rx
}

/// Messages from an MCP server's stdout, one per line.
fn read_lines(stdout: std::process::ChildStdout) -> Receiver<Value> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { return };
            if let Ok(v) = serde_json::from_str(&line)
                && tx.send(v).is_err()
            {
                return;
            }
        }
    });
    rx
}

fn wait_for(rx: &Receiver<Value>, want: impl Fn(&Value) -> bool) -> Result<Value, String> {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(left) {
            Ok(m) if want(&m) => return Ok(m),
            Ok(_) => {}
            Err(_) => return Err("the server did not answer within 120 s".into()),
        }
    }
}

fn finish(child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Wait for `child`: its exit code and, on Unix, its peak resident memory
/// in bytes (`wait4`'s `ru_maxrss`).
#[cfg(unix)]
fn wait(child: &mut Child) -> Result<(i32, Option<u64>), String> {
    let pid = libc::pid_t::try_from(child.id()).map_err(|e| e.to_string())?;
    let mut status: libc::c_int = 0;
    // SAFETY: `rusage` is plain data that `wait4` fills; `pid` is our own
    // child, which nothing else waits on.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::wait4(pid, &mut status, 0, &mut usage) };
    if r != pid {
        return Err(format!("wait4: {}", std::io::Error::last_os_error()));
    }
    let code = if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { 128 + libc::WTERMSIG(status) };
    // Bytes on macOS, KiB on Linux and the BSDs.
    let rss = u64::try_from(usage.ru_maxrss).unwrap_or(0);
    let bytes = if cfg!(target_os = "macos") { rss } else { rss * 1024 };
    Ok((code, Some(bytes)))
}

#[cfg(not(unix))]
fn wait(child: &mut Child) -> Result<(i32, Option<u64>), String> {
    let status = child.wait().map_err(|e| e.to_string())?;
    // No `rusage` here: the peak memory is not measured on this platform.
    Ok((status.code().unwrap_or(128), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_and_p95_are_nearest_rank() {
        let m = Measured { op: &OPERATIONS[0], samples: vec![5.0, 1.0, 9.0, 3.0, 7.0, 2.0, 8.0, 4.0, 6.0] };
        assert_eq!(m.median(), 5.0);
        assert_eq!(m.p95(), 9.0);
        let even = Measured { op: &OPERATIONS[0], samples: vec![1.0, 2.0, 3.0, 4.0] };
        assert_eq!(even.median(), 2.5);
        assert_eq!(even.p95(), 4.0);
    }

    #[test]
    fn wait_returns_the_exit_code_and_peak_memory_only_where_measured() {
        // W-24: the not(unix) arm once returned a bare code; both arms return
        // (code, peak). The test binary lists its tests and exits 0.
        let mut child = Command::new(std::env::current_exe().unwrap()).arg("--list").stdout(Stdio::null()).spawn().unwrap();
        let (code, peak): (i32, Option<u64>) = wait(&mut child).unwrap();
        assert_eq!(code, 0);
        assert_eq!(peak.is_some_and(|b| b > 0), cfg!(unix), "{peak:?}");
    }

    #[test]
    fn every_l8_operation_is_measured() {
        // L8 decision 1's table, in its order.
        let names: Vec<&str> = OPERATIONS.iter().map(|o| o.name).collect();
        assert_eq!(names, ["startup", "check_warm_10k", "check_cold_10k", "format_warm_10k", "fix_one_file", "lsp_after_edit", "mcp_rules_for_path", "peak_memory_10k"]);
        assert!(OPERATIONS.iter().all(|o| o.budgeted) && CONTEXT.iter().all(|o| !o.budgeted));
    }
}
