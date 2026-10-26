//! `codelaw debug changed-functions` (W-71 step 1): which functions the
//! change set adds, edits or leaves alone, by token stream against the base
//! revision. What the test-for-new-code rules will judge, made inspectable.

use crate::{Common, engine_options, fail, load};
use clap::{Args, Subcommand};
use std::process::ExitCode;

#[derive(Args)]
pub struct DebugArgs {
    #[command(subcommand)]
    action: DebugAction,
}

#[derive(Subcommand)]
enum DebugAction {
    /// Each changed file's functions as new, edited or untouched (needs `--changed-since` or `--staged`).
    ChangedFunctions {
        /// text or json.
        #[arg(long, default_value = "text")]
        format: String,
        #[command(flatten)]
        common: Common,
    },
}

pub fn debug(args: DebugArgs) -> ExitCode {
    let DebugAction::ChangedFunctions { format, common } = args.action;
    if !matches!(format.as_str(), "text" | "json") {
        return fail(format!("unknown --format `{format}` (text or json)"));
    }
    let options = match engine_options(&common) {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let project = match load(&[], options) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    match project.changed_functions() {
        Ok(c) if format == "json" => {
            println!("{}", serde_json::to_string_pretty(&c.to_json()).expect("json"));
            ExitCode::SUCCESS
        }
        Ok(c) => {
            print!("{}", c.to_text());
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}
