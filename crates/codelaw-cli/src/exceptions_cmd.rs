//! `codelaw exceptions` (SR-BASELINE decision 11): every exception, what
//! expires, who owns it. A read-only report; the MCP twin is
//! `list_exceptions`. Flags in, one engine call, one rendering.

use crate::{engine_options, fail, load};
use clap::Args;
use codelaw_report::exceptions::{Filter, GroupBy, any_within, build, parse_days};
use std::process::ExitCode;

#[derive(Args)]
pub struct ExceptionsArgs {
    /// Expires within this many days after the as-of date: `30d`, `4w` or a number.
    #[arg(long, value_name = "DURATION")]
    expiring: Option<String>,
    /// Already expired on the as-of date.
    #[arg(long)]
    expired: bool,
    /// Only the exceptions whose `added_by` is `git config user.email`.
    #[arg(long)]
    mine: bool,
    /// Only these rule ids (comma-separated or repeated).
    #[arg(long, value_delimiter = ',')]
    rule: Vec<String>,
    /// Only exceptions covering a file at or under this path.
    #[arg(long)]
    path: Option<String>,
    /// added_by, rule, file or expires-month.
    #[arg(long, value_name = "KEY")]
    group_by: Option<String>,
    /// text, json, csv or markdown.
    #[arg(long, default_value = "text")]
    format: String,
    /// Exit 1 when any listed exception expires within this long, or already has.
    #[arg(long, value_name = "DURATION")]
    fail_if_expiring: Option<String>,
    /// Judge deadlines on this date (YYYY-MM-DD) instead of `HEAD`'s commit date.
    #[arg(long, value_name = "DATE")]
    as_of: Option<String>,
}

pub fn exceptions(a: ExceptionsArgs) -> ExitCode {
    let run = || -> Result<(String, bool), String> {
        if !["text", "json", "csv", "markdown"].contains(&a.format.as_str()) {
            return Err(format!("unknown --format `{}` (text, json, csv or markdown)", a.format));
        }
        let filter = Filter {
            expiring: a.expiring.as_deref().map(|s| parse_days("expiring", s)).transpose()?,
            expired: a.expired,
            mine: a.mine,
            rules: a.rule.clone(),
            path: a.path.clone(),
        };
        let group_by = a.group_by.as_deref().map(GroupBy::parse).transpose()?;
        let fail_at = a.fail_if_expiring.as_deref().map(|s| parse_days("fail-if-expiring", s)).transpose()?;
        let common = crate::Common { as_of: a.as_of.clone(), ..Default::default() };
        let mut p = load(&[], engine_options(&common)?).map_err(|e| e.to_string())?;
        let all = p.exceptions().map_err(|e| e.to_string())?;
        let report = build(&all, &filter, group_by)?;
        if all.as_of.date.is_none() && a.format == "text" {
            eprintln!("warning: no as-of date ({}); days left are not shown", all.as_of.source);
        }
        let out = match a.format.as_str() {
            "json" => serde_json::to_string_pretty(&report.json()).unwrap_or_default() + "\n",
            "csv" => report.csv(),
            "markdown" => report.markdown(),
            _ => report.text(),
        };
        Ok((out, fail_at.is_some_and(|n| any_within(&report, n))))
    };
    match run() {
        Ok((out, fail_now)) => {
            print!("{out}");
            if fail_now { ExitCode::from(1) } else { ExitCode::SUCCESS }
        }
        Err(e) => fail(e),
    }
}
