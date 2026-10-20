//! `codelaw format [--check] [--diff] <paths>` (SR-FORMATTER decision 1).

use crate::{Common, color, engine_options, fail, load};
use clap::Args;
use codelaw_engine::{FormatReport, FormatRunOptions};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub struct FormatArgs {
    /// Files and folders; the project root when none.
    paths: Vec<PathBuf>,
    #[command(flatten)]
    common: Common,
    /// Change nothing; exit 1 if any file is not formatted.
    #[arg(long)]
    check: bool,
    /// Change nothing; print what formatting would change, as a unified diff.
    #[arg(long)]
    diff: bool,
}

/// A unified diff of one file's formatting.
pub fn unified_diff(file: &str, before: &str, after: &str) -> String {
    similar::TextDiff::from_lines(before, after).unified_diff().context_radius(3).header(&format!("a/{file}"), &format!("b/{file}")).to_string()
}

/// 0: every file is formatted (or was just formatted); 1: `--check` or
/// `--diff` found an unformatted file, or a file could not be formatted or
/// written; 2: the run could not start.
pub fn exit_code(report: &FormatReport, check: bool) -> u8 {
    if report.check.internal_errors() > 0 {
        2
    } else if (check && !report.files.is_empty()) || !report.check.diagnostics.is_empty() {
        1
    } else {
        0
    }
}

pub fn format(args: FormatArgs) -> ExitCode {
    let options = match engine_options(&args.common) {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let mut project = match load(&args.paths, options) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let check = args.check || args.diff;
    let report = match project.format(&args.paths, &FormatRunOptions { check }) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    for f in &report.files {
        if args.diff {
            print!("{}", unified_diff(&f.file, &f.original, &f.formatted));
        } else if args.check {
            println!("would format {}", f.file);
        } else if f.written {
            println!("formatted {}", f.file);
        }
    }
    if !report.check.diagnostics.is_empty() {
        print!("{}", codelaw_report::check_text(&report.check, color()));
    }
    let n = report.files.len();
    let files = report.check.files_checked;
    match (check, n) {
        (_, 0) => eprintln!("{files} files already formatted"),
        (true, _) => eprintln!("{n} of {files} files would be formatted"),
        (false, _) => eprintln!("formatted {n} of {files} files"),
    }
    ExitCode::from(exit_code(&report, check))
}
