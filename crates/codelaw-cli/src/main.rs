//! The `codelaw` binary. Every command translates its flags into calls on
//! `codelaw-engine` and prints the result; no logic lives here
//! (SR-ARCHITECTURE decision 2, SR-CLI-MCP decision 6).

mod init;
mod rules_cmd;
mod tree;

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
    /// Run rules against their own examples and cases.
    Test(TestArgs),
    /// List rules, or show which apply to a file and why.
    Rules(rules_cmd::RulesArgs),
    /// Show what a rule requires and how to fix it.
    Explain { id: String },
    /// Show the syntax tree the engine sees.
    Tree(tree::TreeArgs),
    /// Write a new rule file from the template.
    NewRule(rules_cmd::NewRuleArgs),
    /// Set codelaw up in this repo: config, rule packs, folders.
    Init(init::InitArgs),
    /// Manage the cache.
    Cache {
        #[command(subcommand)]
        action: CacheAction,
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
}

#[derive(Args)]
struct CheckArgs {
    /// Files and folders; the project root when none.
    paths: Vec<PathBuf>,
    #[command(flatten)]
    common: Common,
    /// text, json or agent.
    #[arg(long, default_value = "text")]
    format: String,
    /// Fail when there are more warnings than this.
    #[arg(long)]
    max_warnings: Option<usize>,
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
    #[arg(long, default_value = "text")]
    format: String,
    #[arg(long)]
    max_warnings: Option<usize>,
}

#[derive(Args)]
struct TestArgs {
    /// A rule id, a rule file, a rules folder or a pack folder; every rule
    /// of the project when none.
    target: Option<String>,
    /// Only the case with this title.
    #[arg(long)]
    case: Option<String>,
}

pub fn color() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

pub fn fail(e: impl std::fmt::Display) -> ExitCode {
    eprintln!("error: {e}");
    ExitCode::from(2)
}

pub fn engine_options(common: &Common) -> Result<EngineOptions, String> {
    let mut o = EngineOptions::from_env();
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
    Ok(o)
}

/// The project that governs the first path, or the current folder.
pub fn load(paths: &[PathBuf], options: EngineOptions) -> Result<Project, EngineError> {
    let start = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
    let start = if start.is_file() { start.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")) } else { start };
    let start = if start.as_os_str().is_empty() { PathBuf::from(".") } else { start };
    Project::load(&start, options)
}

fn format(s: &str) -> Result<Format, String> {
    Format::parse(s).ok_or_else(|| format!("unknown --format `{s}` (text, json or agent)"))
}

fn check(args: CheckArgs) -> ExitCode {
    let fmt = match format(&args.format) {
        Ok(f) => f,
        Err(e) => return fail(e),
    };
    let options = match engine_options(&args.common) {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let mut project = match load(&args.paths, options) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let report = match project.check(&args.paths) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    match fmt {
        Format::Text => print!("{}", codelaw_report::check_text(&report, color())),
        Format::Json => println!("{}", serde_json::to_string_pretty(&codelaw_report::check_json(&report)).expect("json")),
        Format::Agent => print!("{}", codelaw_report::check_agent(&report)),
    }
    ExitCode::from(codelaw_report::exit_code(&report, args.max_warnings) as u8)
}

fn fix(args: FixArgs) -> ExitCode {
    let fmt = match format(&args.format) {
        Ok(f) => f,
        Err(e) => return fail(e),
    };
    let options = match engine_options(&args.common) {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let mut project = match load(&args.paths, options) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let report = match project.fix(&args.paths, &FixOptions { unsafe_fixes: args.unsafe_fixes, dry_run: args.dry_run }) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
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
                println!("{verb} {} ({})", f.file, applied.join(", "));
            }
            print!("{}", codelaw_report::fix_text(&report, color()));
        }
        Format::Json => println!("{}", serde_json::to_string_pretty(&codelaw_report::fix_json(&report)).expect("json")),
        Format::Agent => print!("{}", codelaw_report::check_agent(&report.check)),
    }
    ExitCode::from(codelaw_report::exit_code(&report.check, args.max_warnings) as u8)
}

fn test(args: TestArgs) -> ExitCode {
    let rules = match rules_cmd::rules_for_test(args.target.as_deref()) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    if rules.is_empty() {
        return fail("no rules to test");
    }
    let reports = codelaw_test::test_rules(rules.iter(), args.case.as_deref());
    if args.case.is_some() && reports.iter().all(|r| r.checks == 0) {
        return fail(format!("no case titled `{}`", args.case.unwrap_or_default()));
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
        Command::Check(a) => check(a),
        Command::Fix(a) => fix(a),
        Command::Test(a) => test(a),
        Command::Rules(a) => rules_cmd::rules(a),
        Command::Explain { id } => rules_cmd::explain(&id),
        Command::Tree(a) => tree::tree(a),
        Command::NewRule(a) => rules_cmd::new_rule(a),
        Command::Init(a) => init::init(a),
        Command::Cache { action } => cache(action),
    }
}
