//! The `git` binary (SR-ARCHITECTURE decision 14): the merge base and diff
//! for `--changed-since`, the index for `--staged`, the `HEAD` commit date
//! for the as-of default, and the baselines of another revision for
//! `baseline check`. No git library is linked; with no `git`, each caller
//! says so and the rest of the run goes on.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Stdio};

/// Run `git -C <dir> <args>`; stdout, or why it could not be had.
fn git(dir: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .arg("--no-optional-locks")
        .args(args)
        .stdin(Stdio::null())
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { "no `git` on PATH".to_owned() } else { format!("`git` could not run: {e}") })?;
    if out.status.success() {
        return Ok(out.stdout);
    }
    let err = String::from_utf8_lossy(&out.stderr);
    let first = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim().trim_start_matches("fatal: ").to_owned();
    if first.contains("not a git repository") {
        return Err("not a git repository".into());
    }
    Err(if first.is_empty() { format!("`git {}` failed", args.join(" ")) } else { first })
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8_lossy(&bytes).trim().to_owned()
}

/// `HEAD`'s commit date, `YYYY-MM-DD`, in the committer's own time zone:
/// the same for a commit on every machine.
pub(crate) fn head_date(dir: &Path) -> Result<String, String> {
    let iso = text(git(dir, &["log", "-1", "--format=%cI", "HEAD"])?);
    iso.get(..10).map(str::to_owned).ok_or_else(|| "`HEAD` has no commit yet".to_owned())
}

/// NUL-separated paths, relative to `dir`.
fn paths(bytes: Vec<u8>) -> BTreeSet<String> {
    bytes.split(|b| *b == 0).filter(|p| !p.is_empty()).map(|p| String::from_utf8_lossy(p).replace('\\', "/")).collect()
}

/// The files changed between the merge base of `reference` and `HEAD`, and
/// the working tree: what a pull request changes.
pub(crate) fn changed_since(dir: &Path, reference: &str) -> Result<BTreeSet<String>, String> {
    let base = text(git(dir, &["merge-base", reference, "HEAD"])?);
    Ok(paths(git(dir, &["diff", "--name-only", "--relative", "--no-renames", "-z", &base, "--"])?))
}

/// The files the index changes against `HEAD`: what a commit would change.
pub(crate) fn staged(dir: &Path) -> Result<BTreeSet<String>, String> {
    Ok(paths(git(dir, &["diff", "--cached", "--name-only", "--relative", "--no-renames", "-z", "--"])?))
}

/// The repository's top folder.
pub(crate) fn toplevel(dir: &Path) -> Result<String, String> {
    Ok(text(git(dir, &["rev-parse", "--show-toplevel"])?))
}

/// The merge base of `reference` and `HEAD`.
pub(crate) fn merge_base(dir: &Path, reference: &str) -> Result<String, String> {
    Ok(text(git(dir, &["merge-base", reference, "HEAD"])?))
}

/// The files of `folder` (relative to `dir`) at `rev`, by name, with their
/// text; no folder at that revision is none.
pub(crate) fn files_at(dir: &Path, rev: &str, folder: &str) -> Result<Vec<(String, String)>, String> {
    let listed = git(dir, &["ls-tree", "-z", "--name-only", rev, "--", &format!("{folder}/")])?;
    let mut out = Vec::new();
    for p in paths(listed) {
        let name = p.rsplit('/').next().unwrap_or(&p).to_owned();
        let body = git(dir, &["show", &format!("{rev}:./{folder}/{name}")])?;
        out.push((name, String::from_utf8_lossy(&body).into_owned()));
    }
    out.sort();
    Ok(out)
}
