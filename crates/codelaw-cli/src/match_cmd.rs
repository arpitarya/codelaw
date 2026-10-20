//! `codelaw match <id> <file> [--explain]` (SR-RULE-AUTHORING decision 3):
//! where a `match:` rule matches a file, and with `--explain` which clause
//! held or failed on every candidate node and what it captured.

use crate::{fail, load};
use clap::Args;
use codelaw_engine::EngineOptions;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub struct MatchArgs {
    /// The rule id.
    id: String,
    /// The file to match against.
    file: PathBuf,
    /// Show every candidate node and each clause: held, failed, captured.
    #[arg(long)]
    explain: bool,
    /// Only the nodes that start on this line.
    #[arg(long)]
    line: Option<u32>,
    /// text or json.
    #[arg(long, default_value = "text")]
    format: String,
}

pub fn run(args: MatchArgs) -> ExitCode {
    let json = match args.format.as_str() {
        "text" => false,
        "json" => true,
        other => return fail(format!("unknown --format `{other}` (text or json)")),
    };
    let project = match load(std::slice::from_ref(&args.file), EngineOptions { no_cache: true, ..EngineOptions::from_env() }) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let mut report = match project.explain_match(&args.id, &args.file) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    if let Some(l) = args.line {
        report.nodes.retain(|(start, _, _)| start.line == l);
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&codelaw_report::match_json(&report)).expect("json"));
    } else {
        print!("{}", codelaw_report::match_text(&report, args.explain));
    }
    ExitCode::SUCCESS
}
