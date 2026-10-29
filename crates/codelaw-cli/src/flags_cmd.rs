//! `codelaw flags accessors` (SR-CLI-MCP decision 1; SR-PROJECT-GRAPH
//! decision 11): every call the flag rules read a flag through — the
//! patterns of `[flags] accessors`, those of its presets, and the wrappers
//! found through the project graph, each with where it comes from.

use crate::{Common, engine_options, fail, load};
use clap::{Args, Subcommand};
use std::process::ExitCode;

#[derive(Args)]
pub struct FlagsArgs {
    #[command(subcommand)]
    action: FlagsAction,
}

#[derive(Subcommand)]
enum FlagsAction {
    /// Every accessor in force: config, presets, and the wrappers over them.
    Accessors {
        /// text or json.
        #[arg(long, default_value = "text")]
        format: String,
        #[command(flatten)]
        common: Common,
    },
}

pub fn flags(args: FlagsArgs) -> ExitCode {
    let FlagsAction::Accessors { format, common } = args.action;
    if format != "text" && format != "json" {
        return fail(format!("--format {format}: one of text, json"));
    }
    let options = match engine_options(&common) {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let mut project = match load(&[], options) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let accessors = match project.flag_accessors() {
        Ok(a) => a,
        Err(e) => return fail(e),
    };
    if format == "json" {
        let rows: Vec<serde_json::Value> = accessors
            .iter()
            .map(|a| serde_json::json!({ "pattern": a.pattern, "origin": a.origin, "file": a.at.as_ref().map(|x| &x.0), "line": a.at.as_ref().map(|x| x.1) }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows).unwrap_or_default());
        return ExitCode::SUCCESS;
    }
    if accessors.is_empty() {
        println!("no accessors: `[flags]` names no `presets` and no `accessors`");
        return ExitCode::SUCCESS;
    }
    let width = accessors.iter().map(|a| a.pattern.chars().count()).max().unwrap_or(0);
    for a in &accessors {
        let at = a.at.as_ref().map(|(f, l)| format!("  {f}:{l}")).unwrap_or_default();
        println!("{:width$}  {}{at}", a.pattern, a.origin);
    }
    ExitCode::SUCCESS
}
