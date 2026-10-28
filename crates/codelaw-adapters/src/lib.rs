//! The adapters (SR-LINTER decision 2, SR-ARCHITECTURE decision 19):
//! external linters the user installed, run as programs from `check` and
//! `fix` only, their JSON mapped onto codelaw diagnostics and the three fix
//! classes (L4 decision 9).
//!
//! None is bundled and none is needed: a repo without them gets codelaw's
//! own rules (L6). The engine decides when they run and caches their
//! results; this crate knows the manifests, finds the programs, runs them
//! and reads what they print.

mod manifest;
mod parse;
mod position;
mod run;

pub use manifest::{Manifest, Output, manifest, manifests, parse_manifest};
pub use parse::{Finding, Parsed, RawEdit, Span, parse};
pub use position::Unit;
pub use run::{locate, locate_program, parse_version, stderr_tail, version};

use semver::Version;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Files per run of a program, so a command line stays under every
/// platform's limit.
const BATCH: usize = 200;

/// An adapter found installed, ready to run.
#[derive(Clone, Debug)]
pub struct Ready {
    pub manifest: &'static Manifest,
    pub program: PathBuf,
    pub version: Version,
}

impl Ready {
    /// `<name>@<version>`: a diagnostic's `source`, and part of every cache
    /// key (SR-ARCHITECTURE decision 19).
    pub fn source(&self) -> String {
        format!("{}@{}", self.manifest.name, self.version)
    }
}

/// The adapters a run uses, and what it has to say about the others.
#[derive(Clone, Debug, Default)]
pub struct Resolved {
    pub ready: Vec<Ready>,
    pub warnings: Vec<String>,
}

/// The adapters `[adapters] enabled` names, or the default ones when it is
/// not set; an unknown name is a config error (SR-CONFIG decision 5).
pub fn selected(enabled: Option<&[String]>) -> Result<Vec<&'static Manifest>, String> {
    let Some(names) = enabled else {
        return Ok(manifests().iter().filter(|m| m.default).collect());
    };
    let mut out = Vec::new();
    for name in names {
        match manifest(name) {
            Some(m) if !out.iter().any(|o: &&Manifest| o.name == m.name) => out.push(m),
            Some(_) => {}
            None => {
                let known: Vec<String> = manifests().iter().map(|m| format!("`{}`", m.name)).collect();
                return Err(format!("[adapters] enabled: unknown adapter `{name}`; codelaw has {}", known.join(", ")));
            }
        }
    }
    Ok(out)
}

/// Find the selected adapters' programs and their versions. A default
/// adapter that is not installed is skipped quietly (it runs "when
/// installed"); one the config names is a warning, never an error; one
/// outside its tested range runs with a warning (SR-ARCHITECTURE decision
/// 19).
pub fn resolve(enabled: Option<&[String]>, root: &Path, path_env: Option<&std::ffi::OsStr>) -> Result<Resolved, String> {
    let named = enabled.is_some();
    let mut out = Resolved::default();
    for m in selected(enabled)? {
        let Some(program) = locate(m, root, path_env) else {
            if named {
                out.warnings.push(format!("adapter `{}` is enabled in [adapters] but `{}` is not installed, so it did not run", m.name, m.binary));
            }
            continue;
        };
        match version(m, &program, root) {
            Ok(v) => {
                if !m.tested.matches(&v) {
                    out.warnings.push(format!("adapter `{}` is {v}, outside its tested range {}; its findings may be mapped wrongly", m.name, m.tested));
                }
                out.ready.push(Ready { manifest: m, program, version: v });
            }
            Err(e) => out.warnings.push(format!("adapter `{}` did not run: {e}", m.name)),
        }
    }
    Ok(out)
}

/// What one adapter found in a set of files, by the file's path relative
/// to the project root.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub findings: BTreeMap<String, Vec<Finding>>,
    pub warnings: Vec<String>,
}

/// The project-relative path a tool's file name stands for, among `files`.
fn match_file<'a>(reported: &str, root: &Path, files: &'a [String], canonical: &BTreeMap<PathBuf, &'a String>) -> Option<&'a String> {
    let reported = reported.strip_prefix("./").unwrap_or(reported);
    if let Some(f) = files.iter().find(|f| f.as_str() == reported) {
        return Some(f);
    }
    let path = Path::new(reported);
    let abs = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
    canonical.get(&abs).or_else(|| std::fs::canonicalize(&abs).ok().and_then(|c| canonical.get(&c))).copied()
}

/// Run `ready` over `files` (project-relative, forward slashes) from
/// `root`. Every file is in the result, with no findings when it has none,
/// so the caller can cache each. A program that fails, times out or prints
/// a report this adapter can't read is an `Err`.
pub fn check_files(ready: &Ready, root: &Path, files: &[String], timeout: Duration) -> Result<Report, String> {
    let mut report = Report::default();
    let mut canonical = BTreeMap::new();
    for f in files {
        report.findings.insert(f.clone(), Vec::new());
        let abs = root.join(f);
        if let Ok(c) = std::fs::canonicalize(&abs) {
            canonical.insert(c, f);
        }
        canonical.insert(abs, f);
    }
    let m = ready.manifest;
    for batch in files.chunks(BATCH) {
        let mut args = m.args.clone();
        args.extend(batch.iter().cloned());
        let ran = run::run(&ready.program, &args, root, timeout)?;
        let parsed = parse(m.output, &ran.stdout, &ran.stderr).map_err(|e| format!("`{}` exited with {}{}; {e}", m.binary, ran.status, stderr_tail(&ran.stderr)))?;
        for w in parsed.warnings {
            if !report.warnings.contains(&w) {
                report.warnings.push(w);
            }
        }
        for finding in parsed.findings {
            if let Some(rel) = match_file(&finding.file, root, files, &canonical) {
                report.findings.entry(rel.clone()).or_default().push(finding);
            }
        }
    }
    Ok(report)
}
