//! `codelaw init --git-hooks` (SR-CLI-MCP decision 7): write a blocking
//! `pre-commit` and a non-blocking `post-commit` into the hooks folder git
//! uses, chain a hook that was already there, never overwrite one.

use std::path::{Path, PathBuf};
use std::process::Command;

pub const PRE_COMMIT: &str = include_str!("hooks/pre-commit.sh");
pub const POST_COMMIT: &str = include_str!("hooks/post-commit.sh");

/// The line that marks a hook as ours.
const MARKER: &str = "# codelaw-hook:";
/// What an existing hook is renamed to, so the new one can run it first.
const CHAINED: &str = ".pre-codelaw";

pub const HOOKS: [&str; 2] = ["pre-commit", "post-commit"];

/// `pre-commit,post-commit` or any one of them; both when none is named.
pub fn parse(names: &[String]) -> Result<Vec<&'static str>, String> {
    if names.is_empty() {
        return Ok(HOOKS.to_vec());
    }
    let mut out = Vec::new();
    for n in names {
        match HOOKS.iter().find(|h| **h == n) {
            Some(h) if !out.contains(h) => out.push(*h),
            Some(_) => {}
            None => return Err(format!("--git-hooks takes `pre-commit` and `post-commit`, not `{n}`")),
        }
    }
    Ok(out)
}

fn script(hook: &str) -> &'static str {
    if hook == "pre-commit" { PRE_COMMIT } else { POST_COMMIT }
}

/// Where git looks for hooks: `core.hooksPath` or `.git/hooks`, as git itself
/// resolves it (a worktree and a relative path included).
fn hooks_dir(root: &Path) -> Result<PathBuf, String> {
    let out = Command::new("git")
        .args(["rev-parse", "--git-path", "hooks"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err("not inside a git repository, so there is nowhere to install hooks".into());
    }
    let p = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    Ok(if p.is_absolute() { p } else { root.join(p) })
}

fn make_executable(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Install one hook into `dir`; the line to print says what happened.
fn install_one(dir: &Path, hook: &str) -> Result<String, String> {
    let target = dir.join(hook);
    let text = script(hook);
    let mut chained = false;
    if target.symlink_metadata().is_ok() {
        let current = std::fs::read_to_string(&target).unwrap_or_default();
        if current.contains(MARKER) {
            if current == text {
                return Ok(format!("{hook}: already installed ({})", target.display()));
            }
        } else {
            let kept = dir.join(format!("{hook}{CHAINED}"));
            if kept.symlink_metadata().is_ok() {
                return Err(format!(
                    "{hook}: {} already holds a hook and so does {}; move one away, nothing was changed",
                    target.display(),
                    kept.display()
                ));
            }
            std::fs::rename(&target, &kept).map_err(|e| format!("{hook}: cannot keep the existing hook: {e}"))?;
            chained = true;
        }
    }
    std::fs::write(&target, text).map_err(|e| format!("{hook}: cannot write {}: {e}", target.display()))?;
    make_executable(&target).map_err(|e| format!("{hook}: cannot make {} executable: {e}", target.display()))?;
    Ok(if chained {
        format!("{hook}: installed ({}); the hook that was there is kept as {hook}{CHAINED} and runs first", target.display())
    } else {
        format!("{hook}: installed ({})", target.display())
    })
}

/// Install `hooks` for the repo at `root`; each result is one printed line.
pub fn install(root: &Path, hooks: &[&str]) -> Result<Vec<String>, String> {
    let dir = hooks_dir(root)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    hooks.iter().map(|h| install_one(&dir, h)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_parse() {
        assert_eq!(parse(&[]).unwrap(), ["pre-commit", "post-commit"]);
        assert_eq!(parse(&["post-commit".into(), "post-commit".into()]).unwrap(), ["post-commit"]);
        assert!(parse(&["pre-push".into()]).is_err());
    }

    #[test]
    fn scripts_carry_the_marker_and_are_posix_sh() {
        for s in [PRE_COMMIT, POST_COMMIT] {
            assert!(s.starts_with("#!/bin/sh\n"));
            assert!(s.contains(MARKER));
            assert!(!s.contains("[["), "bashisms break dash");
        }
    }
}
