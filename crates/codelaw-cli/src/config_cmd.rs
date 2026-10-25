//! `codelaw config --print [--for <file>]` and `codelaw config --schema`
//! (SR-CONFIG decisions 6 and 7).

use crate::{fail, load};
use clap::Args;
use codelaw_config::TraceEnv;
use codelaw_engine::EngineOptions;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub struct ConfigArgs {
    /// Print every resolved value and where it came from.
    #[arg(long)]
    print: bool,
    /// Print `codelaw.schema.json`, the JSON Schema of the config.
    #[arg(long, conflicts_with = "print")]
    schema: bool,
    /// Resolve for this file: nested configs, `[[overrides]]` and `languages` apply.
    #[arg(long = "for", requires = "print")]
    for_path: Option<PathBuf>,
    /// text or json.
    #[arg(long, default_value = "text")]
    format: String,
    /// Show the cache as `--no-cache` would leave it.
    #[arg(long)]
    no_cache: bool,
    /// Resolve with this profile, as `check --profile` would.
    #[arg(long, value_name = "NAME")]
    profile: Option<String>,
}

pub fn run(args: ConfigArgs) -> ExitCode {
    if args.schema {
        print!("{}", codelaw_config::SCHEMA);
        return ExitCode::SUCCESS;
    }
    if !args.print {
        return fail("`codelaw config` takes --print [--for <file>] or --schema");
    }
    let json = match args.format.as_str() {
        "text" => false,
        "json" => true,
        other => return fail(format!("unknown --format `{other}` (text or json)")),
    };
    let options = EngineOptions { no_cache: true, profile: args.profile.clone(), ..EngineOptions::from_env() };
    let project = match load(&args.for_path.iter().cloned().collect::<Vec<_>>(), options) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let env = TraceEnv { cache_dir: std::env::var("CODELAW_CACHE_DIR").ok(), no_cache: args.no_cache };
    let file = args.for_path.as_deref();
    let values = project.config_trace(file, &env);
    let abs = file.map(|f| std::fs::canonicalize(f).unwrap_or_else(|_| f.to_path_buf()));
    let files = project.config.governing_files(abs.as_deref());
    if json {
        let shown = file.map(codelaw_core::normalize_path);
        println!("{}", serde_json::to_string_pretty(&codelaw_report::config_json(&values, &files, shown.as_deref())).expect("json"));
    } else {
        print!("{}", codelaw_report::config_text(&values, &files));
    }
    ExitCode::SUCCESS
}
