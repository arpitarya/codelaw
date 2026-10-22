//! `codelaw format [--check] [--diff] <paths>` (SR-FORMATTER decision 1).

use crate::{Common, Ctx, Output, engine_options_in, load};
use clap::Args;
use codelaw_engine::{FormatReport, FormatRunOptions};
use std::path::PathBuf;

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

impl FormatArgs {
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn prefer_warm(&self) -> bool {
        self.common.prefer_warm
    }
}

pub fn format(args: FormatArgs, ctx: &Ctx) -> Output {
    let options = match engine_options_in(&args.common, ctx.env.clone()) {
        Ok(o) => o,
        Err(e) => return Output::fail(e),
    };
    let mut project = match load(&args.paths, options) {
        Ok(p) => p,
        Err(e) => return Output::fail(e),
    };
    let check = args.check || args.diff;
    let report = match project.format(&args.paths, &FormatRunOptions { check }) {
        Ok(r) => r,
        Err(e) => return Output::fail(e),
    };
    let mut out = Output::default();
    for f in &report.files {
        if args.diff {
            out.stdout.push_str(&unified_diff(&f.file, &f.original, &f.formatted));
        } else if args.check {
            out.stdout.push_str(&format!("would format {}\n", f.file));
        } else if f.written {
            out.stdout.push_str(&format!("formatted {}\n", f.file));
        }
    }
    if !report.check.diagnostics.is_empty() {
        out.stdout.push_str(&codelaw_report::check_text(&report.check, ctx.color));
    }
    let n = report.files.len();
    let files = report.check.files_checked;
    out.stderr = match (check, n) {
        (_, 0) => format!("{files} files already formatted\n"),
        (true, _) => format!("{n} of {files} files would be formatted\n"),
        (false, _) => format!("formatted {n} of {files} files\n"),
    };
    out.code = exit_code(&report, check);
    out
}
