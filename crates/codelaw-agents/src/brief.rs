//! `codelaw brief --for <path>…` (SR-CLI-MCP decision 14): what is specific
//! to this repo for the files an agent is about to write. The list is
//! `codelaw_report::brief`'s, the same call MCP `brief_for_task` makes.

use clap::Args;
use codelaw_engine::{EngineOptions, Project};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Clone, Debug, Default)]
pub struct BriefArgs {
    /// The files about to be written; they need not exist yet.
    #[arg(long = "for", value_name = "PATH", required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,
    /// text or json.
    #[arg(long, default_value = "text")]
    pub format: String,
    /// The profile whose severities to list (SR-CONFIG decision 10).
    #[arg(long)]
    pub profile: Option<String>,
}

/// `0` with the brief; `2` on a usage, config or rule error.
pub fn brief(args: BriefArgs) -> ExitCode {
    let fail = |e: String| {
        eprintln!("error: {e}");
        ExitCode::from(2)
    };
    let json = match args.format.as_str() {
        "text" => false,
        "json" => true,
        f => return fail(format!("`--format {f}`: `brief` prints text or json")),
    };
    let mut project = match Project::load_for(&args.paths, EngineOptions { profile: args.profile.clone(), ..EngineOptions::from_env() }) {
        Ok(p) => p,
        Err(e) => return fail(e.to_string()),
    };
    match codelaw_report::brief(&mut project, &args.paths) {
        Ok(b) if json => {
            println!("{}", serde_json::to_string_pretty(&codelaw_report::brief_json(&b)).expect("json"));
            ExitCode::SUCCESS
        }
        Ok(b) => {
            print!("{}", codelaw_report::brief_text(&b));
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}
