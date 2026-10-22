//! `codelaw baseline add|prune|check` and `codelaw deprecate`
//! (SR-CLI-MCP decision 1; SR-BASELINE decisions 5 and 6). Each translates
//! its flags into one engine call and prints the result.

use crate::{engine_options, fail, load};
use clap::{Args, Subcommand};
use codelaw_baseline::{Date, Deprecation, OnTouch, Policy};
use codelaw_engine::BaselineAdd;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Subcommand)]
pub enum BaselineAction {
    /// Record a rule's current findings: the one command that may raise a
    /// count, add an entry or extend a deadline. A person runs it.
    Add(AddArgs),
    /// Lower every entry to what the files now hold (always safe).
    Prune(PruneArgs),
    /// Fail if the baseline got looser since the merge base with a ref.
    Check(CheckArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// The rule whose findings to record.
    #[arg(long)]
    rule: String,
    /// Only these files (repeatable); every file when none.
    #[arg(long = "file")]
    files: Vec<PathBuf>,
    /// The deadline, YYYY-MM-DD: after it every grandfathered finding is an error.
    #[arg(long)]
    until: Option<String>,
    /// From this date, YYYY-MM-DD, a grandfathered finding is a warning.
    #[arg(long)]
    warn_from: Option<String>,
    /// What a grandfathered finding becomes when its file is touched: error, warn or keep.
    #[arg(long)]
    on_touch: Option<String>,
    /// Why, for the reviewer.
    #[arg(long)]
    reason: Option<String>,
}

#[derive(Args)]
pub struct PruneArgs {
    /// Files and folders; the project root when none.
    paths: Vec<PathBuf>,
}

#[derive(Args)]
pub struct CheckArgs {
    /// The ref whose merge base with `HEAD` is the baseline to compare with (e.g. `origin/main`).
    #[arg(long)]
    since: String,
}

#[derive(Args)]
pub struct DeprecateArgs {
    /// `<package or path>#<export>` (JS/TS) or `<module>:<name>` (Python).
    symbol: String,
    /// What to use instead, named the same way.
    #[arg(long)]
    replacement: Option<String>,
    #[arg(long)]
    until: Option<String>,
    #[arg(long)]
    warn_from: Option<String>,
    #[arg(long)]
    on_touch: Option<String>,
    #[arg(long)]
    reason: Option<String>,
}

fn date(flag: &str, v: &Option<String>) -> Result<Option<Date>, String> {
    v.as_deref().map(|s| Date::parse(s).map_err(|e| format!("--{flag}: {e}"))).transpose()
}

fn on_touch(v: &Option<String>) -> Result<Option<OnTouch>, String> {
    v.as_deref().map(|s| OnTouch::parse(s).map_err(|e| format!("--on-touch: {e}"))).transpose()
}

fn project(paths: &[PathBuf]) -> Result<codelaw_engine::Project, String> {
    let options = engine_options(&crate::Common::default())?;
    load(paths, options).map_err(|e| e.to_string())
}

pub fn baseline(action: BaselineAction) -> ExitCode {
    match action {
        BaselineAction::Add(a) => add(a),
        BaselineAction::Prune(a) => prune(a),
        BaselineAction::Check(a) => check(a),
    }
}

fn add(a: AddArgs) -> ExitCode {
    let parsed = (|| -> Result<BaselineAdd, String> {
        Ok(BaselineAdd {
            rule: a.rule.clone(),
            symbol: None,
            until: date("until", &a.until)?,
            warn_from: date("warn-from", &a.warn_from)?,
            on_touch: on_touch(&a.on_touch)?,
            reason: a.reason.clone(),
        })
    })();
    let options = match parsed {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let mut p = match project(&a.files) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    match p.baseline_add(&a.files, &options) {
        Ok(b) => {
            let total: usize = b.entries.iter().map(|e| e.count).sum();
            println!("{}: {} entries, {total} findings allowed", codelaw_baseline::file::rel_path(&b.rule), b.entries.len());
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}

fn prune(a: PruneArgs) -> ExitCode {
    let mut p = match project(&a.paths) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    match p.baseline_prune(&a.paths) {
        Ok(changed) if changed.is_empty() => {
            println!("nothing to prune");
            ExitCode::SUCCESS
        }
        Ok(changed) => {
            for c in changed {
                println!("pruned {c}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}

fn check(a: CheckArgs) -> ExitCode {
    let p = match project(&[]) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    match p.baseline_ratchet(&a.since) {
        Ok(r) => {
            if let Some(w) = &r.codeowners {
                eprintln!("warning: {w}");
            }
            if r.loosened.is_empty() {
                println!("the baseline only shrank since {} (merge base with {})", &r.base[..r.base.len().min(12)], a.since);
                return ExitCode::SUCCESS;
            }
            for l in &r.loosened {
                println!("{l}");
            }
            println!("the baseline got looser since {}: only `codelaw baseline add`, reviewed by the baseline's CODEOWNERS, may do that", a.since);
            ExitCode::from(1)
        }
        Err(e) => fail(e),
    }
}

pub fn deprecate(a: DeprecateArgs) -> ExitCode {
    let parsed = (|| -> Result<Deprecation, String> {
        let policy = Policy { until: date("until", &a.until)?, warn_from: date("warn-from", &a.warn_from)?, on_touch: on_touch(&a.on_touch)? };
        let text = codelaw_baseline::registry::entry_text(&Deprecation { symbol: a.symbol.clone(), replacement: a.replacement.clone(), policy, reason: a.reason.clone(), rewrite: None });
        // Through the parser, so a flag the registry would refuse is refused here.
        codelaw_baseline::Registry::parse(&text).map(|r| r.deprecated.into_iter().next().expect("one entry"))
    })();
    let d = match parsed {
        Ok(d) => d,
        Err(e) => return fail(e),
    };
    let mut p = match project(&[]) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    match p.deprecate(&d, None) {
        Ok(b) => {
            let n: usize = b.entries.iter().filter(|e| e.symbol.as_deref() == Some(d.symbol.as_str())).map(|e| e.count).sum();
            println!("deprecated `{}` in {}", d.symbol, codelaw_baseline::REGISTRY_REL);
            let uses = if n == 1 { "1 current use".to_owned() } else { format!("{n} current uses") };
            println!("{}: {uses} grandfathered", codelaw_baseline::file::rel_path(&b.rule));
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}
