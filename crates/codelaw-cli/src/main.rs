//! The `codelaw` binary. Every command translates its flags into calls on
//! `codelaw-engine` and prints the result; no logic lives here
//! (SR-ARCHITECTURE decision 2, SR-CLI-MCP decision 6).

mod baseline_cmd;
mod bench;
mod format_cmd;
mod graph_cmd;
mod config_cmd;
mod deps_cmd;
mod init;
mod match_cmd;
mod rules_cmd;
mod tree;
mod warm;

use clap::{Args, Parser, Subcommand};
use codelaw_core::Language;
use codelaw_engine::{EngineError, EngineOptions, FixOptions, Project};
use codelaw_report::Format;
use std::collections::BTreeSet;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "codelaw", version, about = "A linter and formatter whose rules are Markdown files", disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check files and report every finding with its fix.
    Check(CheckArgs),
    /// Apply `auto` fixes (and `suggest` with --unsafe), then report what is left.
    Fix(FixArgs),
    /// Format files in place, or with --check/--diff report what would change.
    Format(format_cmd::FormatArgs),
    /// Run rules against their own examples and cases.
    Test(TestArgs),
    /// List rules, or show which apply to a file and why.
    Rules(rules_cmd::RulesArgs),
    /// Show what a rule requires and how to fix it.
    Explain {
        id: String,
        /// text or json.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Show the syntax tree the engine sees.
    Tree(tree::TreeArgs),
    /// Which clause of a rule matched or failed on a file, and what it captured.
    Match(match_cmd::MatchArgs),
    /// Write a new rule file from the template.
    NewRule(rules_cmd::NewRuleArgs),
    /// Set codelaw up in this repo: config, rule packs, folders.
    Init(init::InitArgs),
    /// Serve the Language Server Protocol over stdio (SR-EDITOR).
    Lsp {
        /// Talk over stdin and stdout: the default and only transport;
        /// accepted because editors pass it.
        #[arg(long)]
        stdio: bool,
    },
    /// Query the project graph: `refs`, `why-unused`.
    Graph(graph_cmd::GraphArgs),
    /// Print the resolved configuration with each value's source, or its JSON Schema.
    Config(config_cmd::ConfigArgs),
    /// Manage the cache.
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
    /// Serve the MCP tools to an agent over stdio (SR-CLI-MCP decision 5).
    Mcp,
    /// Check and fix the files an agent just edited; what is left goes back to it (SR-AGENT-SURFACES).
    Hook(codelaw_agents::HookArgs),
    /// Write every agent's files from the rules and skills, or with --check report drift.
    SyncAgents(codelaw_agents::SyncArgs),
    /// Grandfather a rule's existing findings, lower the counts, or check the ratchet (SR-BASELINE).
    Baseline {
        #[command(subcommand)]
        action: baseline_cmd::BaselineAction,
    },
    /// Deprecate a symbol: write the registry entry and baseline every current use (SR-BASELINE decision 6).
    Deprecate(baseline_cmd::DeprecateArgs),
    /// Read npm and the Python index and write the dependency snapshot the
    /// offline rules read: the one command that uses the network (L5 decision 3).
    Deps(deps_cmd::DepsArgs),
    /// Time every L8 operation on the reference workload and compare each
    /// with `perf/budgets.toml` (SR-LAW-8).
    Bench(bench::BenchArgs),
    /// An opt-in warm process that serves `--prefer-warm` runs (SR-ARCHITECTURE decision 22).
    Daemon {
        #[command(subcommand)]
        action: warm::DaemonAction,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Entries and size.
    Stats,
    /// Remove every entry.
    Clean,
}

#[derive(Args, Clone, Default)]
pub struct Common {
    /// Only these languages (comma-separated, e.g. `typescript,tsx`).
    #[arg(long, value_delimiter = ',')]
    lang: Vec<String>,
    /// Only these rules (comma-separated or repeated).
    #[arg(long, value_delimiter = ',')]
    rule: Vec<String>,
    /// Worker threads; the CPU count by default.
    #[arg(long)]
    threads: Option<usize>,
    /// Neither read nor write the cache.
    #[arg(long)]
    no_cache: bool,
    /// Print where the time went: each phase and the slowest rules.
    #[arg(long)]
    timings: bool,
    /// Run in a warm `lsp`, `mcp` or `daemon` of this version when one is
    /// running for this project; otherwise run here. The output is the same.
    #[arg(long)]
    prefer_warm: bool,
    /// Judge deadlines on this date (YYYY-MM-DD) instead of `HEAD`'s commit date.
    #[arg(long, value_name = "DATE")]
    as_of: Option<String>,
    /// Files changed since the merge base with this ref count as touched (needs git).
    #[arg(long, value_name = "REF")]
    changed_since: Option<String>,
    /// Files in the index count as touched (needs git).
    #[arg(long)]
    staged: bool,
}

#[derive(Args)]
struct CheckArgs {
    /// Files and folders; the project root when none.
    paths: Vec<PathBuf>,
    #[command(flatten)]
    common: Common,
    /// text, json, sarif or agent.
    #[arg(long, default_value = "text")]
    format: String,
    /// Fail when there are more warnings than this.
    #[arg(long)]
    max_warnings: Option<usize>,
    /// Don't run the installed external linters (oxlint, Ruff, stylelint,
    /// html-validate) this command runs otherwise.
    #[arg(long)]
    no_adapters: bool,
}

#[derive(Args)]
struct FixArgs {
    paths: Vec<PathBuf>,
    #[command(flatten)]
    common: Common,
    /// Apply `suggest` fixes too.
    #[arg(long = "unsafe")]
    unsafe_fixes: bool,
    /// Compute the fixes but write nothing.
    #[arg(long)]
    dry_run: bool,
    /// Show each pass of the fix loop: what was offered, applied or refused, and why.
    #[arg(long)]
    trace: bool,
    /// text, json, sarif or agent.
    #[arg(long, default_value = "text")]
    format: String,
    #[arg(long)]
    max_warnings: Option<usize>,
    /// Don't run the installed external linters (oxlint, Ruff, stylelint,
    /// html-validate) this command runs otherwise.
    #[arg(long)]
    no_adapters: bool,
}

#[derive(Args)]
struct TestArgs {
    /// A rule id, a rule file, a rules folder or a pack folder; every rule
    /// of the project when none.
    target: Option<String>,
    /// Only the case with this title.
    #[arg(long)]
    case: Option<String>,
    /// Level 4 as well: run the rules over every file under this folder (a
    /// pinned corpus checkout) and hold each file to the invariants.
    #[arg(long, value_name = "DIR")]
    corpus: Option<PathBuf>,
    /// The finding counts the corpus run must give (a TOML file).
    #[arg(long, value_name = "FILE", requires = "corpus")]
    snapshot: Option<PathBuf>,
    /// Write the counts to --snapshot instead of comparing them.
    #[arg(long, requires = "snapshot")]
    update_snapshot: bool,
}

pub fn color() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

pub fn fail(e: impl std::fmt::Display) -> ExitCode {
    eprintln!("error: {e}");
    ExitCode::from(2)
}

/// What a command takes from the process it runs for: the environment
/// (SR-CONFIG decision 4) and whether its stdout is a terminal. A warm
/// process runs a command with its client's, never its own
/// (SR-ARCHITECTURE decision 21).
#[derive(Clone, Debug, Default)]
pub struct Ctx {
    pub color: bool,
    pub env: EngineOptions,
    /// Running in a warm process for a client.
    pub warm: bool,
}

impl Ctx {
    pub fn here() -> Ctx {
        Ctx { color: color(), env: EngineOptions::from_env(), warm: false }
    }
}

/// What a command prints and its exit code: the same bytes whether it ran
/// here or in a warm process.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub code: u8,
}

impl Output {
    pub fn fail(e: impl std::fmt::Display) -> Output {
        Output { stderr: format!("error: {e}\n"), code: 2, ..Default::default() }
    }

    pub fn emit(self) -> ExitCode {
        use std::io::Write;
        let _ = std::io::stdout().write_all(self.stdout.as_bytes());
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().write_all(self.stderr.as_bytes());
        ExitCode::from(self.code)
    }
}

/// Run a command here, or with `--prefer-warm` in a warm process when one
/// serves it (SR-ARCHITECTURE decision 21).
fn run(prefer_warm: bool, paths: &[PathBuf], local: impl FnOnce(&Ctx) -> Output) -> ExitCode {
    let ctx = Ctx::here();
    if prefer_warm && let Some(out) = warm::client(paths, &ctx) {
        return out.emit();
    }
    local(&ctx).emit()
}

/// A command line a warm process was sent, run as this binary runs it, or
/// `None` when it runs only in its own process.
pub fn serve_command(argv: &[String], ctx: &Ctx) -> Option<Output> {
    let cli = Cli::try_parse_from(argv).ok()?;
    match cli.command {
        Command::Check(a) => Some(check(a, ctx)),
        Command::Fix(a) => fix(a, ctx),
        Command::Format(a) => Some(format_cmd::format(a, ctx)),
        _ => None,
    }
}

pub fn engine_options(common: &Common) -> Result<EngineOptions, String> {
    engine_options_in(common, EngineOptions::from_env())
}

/// The flags of `common` over the options the environment gives.
pub fn engine_options_in(common: &Common, env: EngineOptions) -> Result<EngineOptions, String> {
    let mut o = env;
    if !common.lang.is_empty() {
        let mut set = BTreeSet::new();
        for l in &common.lang {
            match Language::from_name(l) {
                Some(lang) => {
                    set.insert(lang);
                }
                None => {
                    let near = codelaw_config::did_you_mean(l, Language::ALL.iter().map(|x| x.name()))
                        .map(|n| format!("; did you mean `{n}`?"))
                        .unwrap_or_default();
                    return Err(format!("unknown language `{l}` in --lang{near}"));
                }
            }
        }
        o.languages = Some(set);
    }
    if !common.rule.is_empty() {
        o.rules = Some(common.rule.clone());
    }
    o.threads = common.threads;
    o.no_cache = common.no_cache;
    o.timings = common.timings;
    // The time and the change set are inputs (L5 decision 2).
    o.as_of = common.as_of.clone().or(o.as_of);
    o.changed_since = common.changed_since.clone();
    o.staged = common.staged;
    Ok(o)
}

/// The project that governs the first path, or the current folder.
pub fn load(paths: &[PathBuf], options: EngineOptions) -> Result<Project, EngineError> {
    let start = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
    let start = if start.is_file() { start.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")) } else { start };
    let start = if start.as_os_str().is_empty() { PathBuf::from(".") } else { start };
    Project::load(&start, options)
}

pub fn format(s: &str) -> Result<Format, String> {
    Format::parse(s).ok_or_else(|| format!("unknown --format `{s}` (text, json, sarif or agent)"))
}

fn check(args: CheckArgs, ctx: &Ctx) -> Output {
    let fmt = match format(&args.format) {
        Ok(f) => f,
        Err(e) => return Output::fail(e),
    };
    let mut options = match engine_options_in(&args.common, ctx.env.clone()) {
        Ok(o) => o,
        Err(e) => return Output::fail(e),
    };
    // The CLI is the one surface that runs adapters (SR-ARCHITECTURE decision 19).
    options.adapters = !args.no_adapters;
    let mut project = match load(&args.paths, options) {
        Ok(p) => p,
        Err(e) => return Output::fail(e),
    };
    let report = match project.check(&args.paths) {
        Ok(r) => r,
        Err(e) => return Output::fail(e),
    };
    let mut out = Output::default();
    match fmt {
        Format::Text => out.stdout.push_str(&codelaw_report::check_text(&report, ctx.color)),
        Format::Json => out.stdout = json_line(&with_timings(codelaw_report::check_json(&report), &project)),
        Format::Sarif => out.stdout = json_line(&codelaw_report::check_sarif(&report, &project)),
        Format::Agent => out.stdout.push_str(&codelaw_report::check_agent(&report)),
    }
    timings(&project, fmt, &mut out);
    out.code = codelaw_report::exit_code(&report, args.max_warnings) as u8;
    out
}

/// A JSON value as `println!` of its pretty form prints it.
fn json_line(v: &serde_json::Value) -> String {
    serde_json::to_string_pretty(v).expect("json") + "\n"
}

/// `None` in a warm process when the run may start an external fixer, which
/// would inherit the warm process's environment rather than the client's
/// (SR-ARCHITECTURE decisions 18 and 21): the client runs it itself.
fn fix(args: FixArgs, ctx: &Ctx) -> Option<Output> {
    let fmt = match format(&args.format) {
        Ok(f) => f,
        Err(e) => return Some(Output::fail(e)),
    };
    let mut options = match engine_options_in(&args.common, ctx.env.clone()) {
        Ok(o) => o,
        Err(e) => return Some(Output::fail(e)),
    };
    // The CLI is the one surface that runs adapters (SR-ARCHITECTURE decision 19).
    options.adapters = !args.no_adapters;
    let mut project = match load(&args.paths, options) {
        Ok(p) => p,
        Err(e) => return Some(Output::fail(e)),
    };
    if ctx.warm && project.runs_external_fixers() {
        return None;
    }
    // One run: `--trace` records the loop `fix` runs, and writes what it
    // writes (nothing with `--dry-run`).
    let report = match project.fix(&args.paths, &FixOptions { unsafe_fixes: args.unsafe_fixes, dry_run: args.dry_run, trace: args.trace }) {
        Ok(r) => r,
        Err(e) => return Some(Output::fail(e)),
    };
    let trace = args.trace.then_some(&report.traces);
    let mut out = Output::default();
    match fmt {
        Format::Text => {
            for f in &report.files {
                // Only a file the disk now holds fixed is "fixed"; a failed
                // write is reported as `codelaw/write-failed` below.
                let verb = if args.dry_run {
                    "would fix"
                } else if f.written {
                    "fixed"
                } else {
                    "not written"
                };
                let applied: Vec<String> = f.applied.iter().map(|(r, n)| format!("{r} ×{n}")).collect();
                out.stdout.push_str(&format!("{verb} {} ({})\n", f.file, applied.join(", ")));
            }
            for b in &report.pruned {
                out.stdout.push_str(&format!("pruned {b}\n"));
            }
            out.stdout.push_str(&codelaw_report::fix_text(&report, ctx.color));
        }
        Format::Json => {
            let mut v = with_timings(codelaw_report::fix_json(&report), &project);
            if let Some(t) = trace {
                v["trace"] = codelaw_report::trace_json(t);
            }
            out.stdout = json_line(&v);
        }
        Format::Sarif => out.stdout = json_line(&codelaw_report::check_sarif(&report.check, &project)),
        Format::Agent => out.stdout.push_str(&codelaw_report::check_agent(&report.check)),
    }
    // Text: on stdout after the report; SARIF and agent: on stderr.
    match (trace, fmt) {
        (Some(t), Format::Text) => out.stdout.push_str(&codelaw_report::trace_text(t)),
        (Some(t), Format::Sarif | Format::Agent) => out.stderr.push_str(&codelaw_report::trace_text(t)),
        _ => {}
    }
    timings(&project, fmt, &mut out);
    out.code = codelaw_report::exit_code(&report.check, args.max_warnings) as u8;
    Some(out)
}

/// `--timings` in JSON output: a `timings` field.
pub fn with_timings(mut v: serde_json::Value, project: &Project) -> serde_json::Value {
    if let Some(t) = project.timings() {
        v["timings"] = codelaw_report::timings_json(&t);
    }
    v
}

/// `--timings` beside the other formats: on stderr, so stdout stays parseable.
fn timings(project: &Project, fmt: Format, out: &mut Output) {
    if let Some(t) = project.timings().filter(|_| fmt != Format::Json) {
        out.stderr.push_str(&codelaw_report::timings_text(&t));
    }
}

fn test(args: TestArgs) -> ExitCode {
    let rules = match rules_cmd::rules_for_test(args.target.as_deref()) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    if rules.is_empty() {
        return fail("no rules to test");
    }
    let mut reports = codelaw_test::test_rules(rules.iter(), args.case.as_deref());
    if args.case.is_some() && reports.iter().all(|r| r.checks == 0) {
        return fail(format!("no case titled `{}`", args.case.unwrap_or_default()));
    }
    if let Some(dir) = &args.corpus {
        // Level 4 (SR-RULE-AUTHORING decision 4), after levels 1–3.
        let files = match codelaw_test::corpus_files(dir) {
            Ok(f) if f.is_empty() => return fail(format!("{}: no file the engine reads", dir.display())),
            Ok(f) => f,
            Err(e) => return fail(format!("{}: {e}", dir.display())),
        };
        let mut corpus: Vec<codelaw_test::CorpusReport> = rules.iter().map(|r| codelaw_test::corpus_rule(r, &files)).collect();
        corpus.sort_by(|a, b| a.report.id.cmp(&b.report.id));
        match (&args.snapshot, args.update_snapshot) {
            (Some(path), true) => {
                if let Err(e) = std::fs::write(path, codelaw_test::write_snapshot(&corpus)) {
                    return fail(format!("{}: {e}", path.display()));
                }
            }
            (Some(path), false) => match std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|t| codelaw_test::read_snapshot(&t)) {
                Ok(snapshot) => codelaw_test::compare_snapshot(&mut corpus, &snapshot),
                Err(e) => return fail(format!("{}: {e}", path.display())),
            },
            (None, _) => {}
        }
        for (r, c) in reports.iter_mut().zip(&corpus) {
            r.checks += c.report.checks;
            r.failures.extend(c.report.failures.iter().cloned());
        }
        let findings: usize = corpus.iter().map(|c| c.findings).sum();
        println!("corpus: {} files · {findings} findings", files.len());
    }
    print!("{}", codelaw_test::render(&reports, color()));
    if reports.iter().all(|r| r.passed()) { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

fn cache(action: CacheAction) -> ExitCode {
    let project = match load(&[], EngineOptions::from_env()) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let dir = project.config.cache_dir(std::env::var("CODELAW_CACHE_DIR").ok().as_deref());
    match action {
        CacheAction::Stats => {
            let (mut files, mut bytes) = (0u64, 0u64);
            let mut stack = vec![dir.join("entries")];
            while let Some(d) = stack.pop() {
                for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
                    match e.metadata() {
                        Ok(m) if m.is_dir() => stack.push(e.path()),
                        Ok(m) => {
                            files += 1;
                            bytes += m.len();
                        }
                        Err(_) => {}
                    }
                }
            }
            println!("{}: {files} entries, {bytes} bytes (limit {} bytes)", codelaw_core::normalize_path(&dir), project.config.cache_max_size());
            ExitCode::SUCCESS
        }
        CacheAction::Clean => match project.cache.clean() {
            Ok(()) => {
                println!("cleaned {}", codelaw_core::normalize_path(&dir));
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Check(a) => run(a.common.prefer_warm, &a.paths.clone(), |ctx| check(a, ctx)),
        // Only a warm process declines a fix; here it always runs.
        Command::Fix(a) => run(a.common.prefer_warm, &a.paths.clone(), |ctx| fix(a, ctx).unwrap_or_default()),
        Command::Format(a) => run(a.prefer_warm(), &a.paths.clone(), |ctx| format_cmd::format(a, ctx)),
        Command::Test(a) => test(a),
        Command::Rules(a) => rules_cmd::rules(a),
        Command::Explain { id, format } => rules_cmd::explain(&id, &format),
        Command::Tree(a) => tree::tree(a),
        Command::Match(a) => match_cmd::run(a),
        Command::NewRule(a) => rules_cmd::new_rule(a),
        Command::Init(a) => init::init(a),
        Command::Lsp { .. } => {
            // A warm process for `--prefer-warm` runs for as long as the
            // editor's session (SR-ARCHITECTURE decision 21).
            let _warm = warm::serve_in_background(warm::Kind::Lsp);
            codelaw_lsp::run_stdio().map_or_else(fail, |()| ExitCode::SUCCESS)
        }
        Command::Graph(a) => graph_cmd::graph(a),
        Command::Config(a) => config_cmd::run(a),
        Command::Cache { action } => cache(action),
        Command::Mcp => {
            let _warm = warm::serve_in_background(warm::Kind::Mcp);
            match codelaw_mcp::serve(std::io::stdin().lock(), std::io::stdout().lock()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => fail(e),
            }
        }
        Command::Baseline { action } => baseline_cmd::baseline(action),
        Command::Deprecate(a) => baseline_cmd::deprecate(a),
        Command::Daemon { action } => warm::daemon(action),
        Command::Bench(a) => bench::bench(a),
        Command::Deps(a) => deps_cmd::deps(a),
        Command::Hook(a) => codelaw_agents::hook(a),
        Command::SyncAgents(a) => codelaw_agents::sync_agents(a),
    }
}
