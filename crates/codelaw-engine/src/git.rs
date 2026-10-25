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

/// Whether the clone is shallow: its history stops at the clone's depth.
pub(crate) fn is_shallow(dir: &Path) -> bool {
    git(dir, &["rev-parse", "--is-shallow-repository"]).map(|b| text(b) == "true").unwrap_or(false)
}

/// `rev` as a full commit id, or why it is none.
pub(crate) fn resolve(dir: &Path, rev: &str) -> Result<String, String> {
    git(dir, &["rev-parse", "--verify", "--quiet", &format!("{rev}^{{commit}}")]).map(text).map_err(|_| format!("`{rev}` is not a date, a duration like 90d, or a commit"))
}

/// The commits that changed `folder` (relative to `dir`), oldest first, as
/// `(id, committer date YYYY-MM-DD)`; `range` limits them (`ref..HEAD`),
/// `HEAD` when none.
pub(crate) fn commits_touching(dir: &Path, range: Option<&str>, folder: &str) -> Result<Vec<(String, String)>, String> {
    let out = text(git(dir, &["log", "--reverse", "--topo-order", "--format=%H %cI", range.unwrap_or("HEAD"), "--", folder])?);
    Ok(out
        .lines()
        .filter_map(|l| {
            let (id, iso) = l.split_once(' ')?;
            Some((id.to_owned(), iso.get(..10)?.to_owned()))
        })
        .collect())
}

/// A commit's date, `YYYY-MM-DD`, in the committer's own time zone.
pub(crate) fn commit_date(dir: &Path, rev: &str) -> Result<String, String> {
    let iso = text(git(dir, &["log", "-1", "--format=%cI", rev])?);
    iso.get(..10).map(str::to_owned).ok_or_else(|| format!("`{rev}` has no commit date"))
}

/// `git config user.email`: who a commit made here would name.
pub(crate) fn user_email(dir: &Path) -> Result<String, String> {
    let e = text(git(dir, &["config", "user.email"]).map_err(|e| if e.contains("config") || e.is_empty() { "no `user.email` is set".to_owned() } else { e })?);
    if e.is_empty() { Err("no `user.email` is set".into()) } else { Ok(e) }
}

/// The lines each file has after a change, by file (relative to `dir`)
/// and 1-based line: the `+` side of `git diff -U0`.
pub(crate) type Lines = std::collections::BTreeMap<String, BTreeSet<u32>>;

fn added_lines(bytes: Vec<u8>) -> Lines {
    let mut out = Lines::new();
    let mut file: Option<String> = None;
    for line in String::from_utf8_lossy(&bytes).lines() {
        if let Some(p) = line.strip_prefix("+++ ") {
            file = p.strip_prefix("b/").map(str::to_owned);
            continue;
        }
        let (Some(f), Some(hunk)) = (&file, line.strip_prefix("@@ ")) else { continue };
        // `@@ -a,b +c,d @@`: `d` lines from `c`; no `,d` is one line.
        let Some(plus) = hunk.split_whitespace().find_map(|w| w.strip_prefix('+')) else { continue };
        let (start, count) = match plus.split_once(',') {
            Some((s, c)) => (s.parse::<u32>().unwrap_or(0), c.parse::<u32>().unwrap_or(0)),
            None => (plus.parse::<u32>().unwrap_or(0), 1),
        };
        if count > 0 && start > 0 {
            out.entry(f.clone()).or_default().extend(start..start + count);
        }
    }
    out
}

const DIFF_LINES: &[&str] = &["diff", "-U0", "--relative", "--no-renames", "--no-color", "--no-ext-diff", "--src-prefix=a/", "--dst-prefix=b/"];

/// The lines the index adds or changes against `HEAD`.
pub(crate) fn staged_lines(dir: &Path) -> Result<Lines, String> {
    let mut args = DIFF_LINES.to_vec();
    args.extend(["--cached", "--"]);
    Ok(added_lines(git(dir, &args)?))
}

/// The lines the working tree adds or changes since the merge base of
/// `reference` and `HEAD`.
pub(crate) fn lines_since(dir: &Path, reference: &str) -> Result<Lines, String> {
    let base = merge_base(dir, reference)?;
    let mut args = DIFF_LINES.to_vec();
    args.extend([base.as_str(), "--"]);
    Ok(added_lines(git(dir, &args)?))
}

/// The author email of each of `lines` of `file` (relative to `dir`), as
/// `git blame --porcelain` gives it; a line not committed yet has none.
pub(crate) fn blame_authors(dir: &Path, file: &str, lines: &BTreeSet<u32>) -> Result<std::collections::BTreeMap<u32, String>, String> {
    let ranges: Vec<String> = lines.iter().map(|l| format!("-L{l},{l}")).collect();
    let mut args = vec!["blame", "--porcelain"];
    args.extend(ranges.iter().map(String::as_str));
    args.extend(["--", file]);
    let out = git(dir, &args)?;
    let mut mails: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut by_line = std::collections::BTreeMap::new();
    let mut current: Option<(String, u32)> = None;
    for l in String::from_utf8_lossy(&out).lines() {
        if l.starts_with('\t') {
            if let Some((sha, line)) = current.take() {
                by_line.insert(line, sha);
            }
            continue;
        }
        if let Some(m) = l.strip_prefix("author-mail ") {
            if let Some((sha, _)) = &current {
                mails.insert(sha.clone(), m.trim().trim_start_matches('<').trim_end_matches('>').to_owned());
            }
            continue;
        }
        let mut parts = l.split(' ');
        if let (Some(sha), Some(_), Some(fin)) = (parts.next(), parts.next(), parts.next())
            && sha.len() >= 40
            && sha.bytes().all(|b| b.is_ascii_hexdigit())
            && let Ok(n) = fin.parse::<u32>()
        {
            current = Some((sha.to_owned(), n));
        }
    }
    Ok(by_line
        .into_iter()
        .filter_map(|(line, sha)| {
            let mail = mails.get(&sha)?;
            let uncommitted = sha.bytes().all(|b| b == b'0') || mail == "not.committed.yet";
            (!uncommitted).then(|| (line, mail.clone()))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_plus_side_of_a_zero_context_diff() {
        let diff = b"diff --git a/x.py b/x.py\n--- a/x.py\n+++ b/x.py\n@@ -1,0 +2,3 @@\n+a\n+b\n+c\n@@ -9 +12 @@\n-x\n+y\n@@ -20,2 +22,0 @@\n-gone\n-gone\n--- a/gone.py\n+++ /dev/null\n@@ -1 +0,0 @@\n-z\n".to_vec();
        let l = super::added_lines(diff);
        assert_eq!(l.len(), 1);
        assert_eq!(l["x.py"].iter().copied().collect::<Vec<_>>(), vec![2, 3, 4, 12]);
    }
}
