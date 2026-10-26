//! SR-CLI-MCP decision 7 through the binary and real git: `init --git-hooks`
//! installs, chains and never overwrites; the pre-commit hook blocks, fixes
//! only fully staged files and re-stages them; the post-commit hook always
//! exits 0.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn have_git() -> bool {
    Command::new("git").arg("--version").output().is_ok_and(|o| o.status.success()) && Command::new("sh").arg("-c").arg("true").status().is_ok_and(|s| s.success())
}

/// A scratch repo with the two fixture rules, a config and one commit.
fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-hooks-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    for r in ["py-no-print.md", "js-eqeqeq.md"] {
        fs::copy(repo().join("fixtures/l3/rules/fixture").join(r), d.join(".codelaw/rules/fixture").join(r)).unwrap();
    }
    fs::write(d.join("codelaw.toml"), "extends = []\n").unwrap();
    fs::write(d.join(".gitignore"), ".codelaw/cache/\n").unwrap();
    git(&d, &["init", "-q", "-b", "main"]);
    git(&d, &["add", "."]);
    git(&d, &["commit", "-q", "-m", "base"]);
    d
}

fn bin_dir() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_codelaw")).parent().unwrap().to_path_buf()
}

fn path_with_codelaw() -> String {
    format!("{}:{}", bin_dir().display(), std::env::var("PATH").unwrap_or_default())
}

fn run(cmd: &mut Command, cwd: &Path) -> (i32, String) {
    let out = cmd
        .current_dir(cwd)
        .env("PATH", path_with_codelaw())
        .env("NO_COLOR", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env_remove("CODELAW_CACHE_DIR")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn git(cwd: &Path, args: &[&str]) -> (i32, String) {
    run(Command::new("git").args(args), cwd)
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    run(Command::new(env!("CARGO_BIN_EXE_codelaw")).args(args), cwd)
}

fn hook(d: &Path, name: &str) -> PathBuf {
    d.join(".git/hooks").join(name)
}

#[test]
fn installs_both_hooks_and_is_idempotent() {
    if !have_git() {
        return;
    }
    let d = temp("install");
    let (code, out) = codelaw(&d, &["init", "--git-hooks"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("pre-commit: installed") && out.contains("post-commit: installed"), "{out}");
    for h in ["pre-commit", "post-commit"] {
        assert!(fs::read_to_string(hook(&d, h)).unwrap().contains("# codelaw-hook:"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(hook(&d, h)).unwrap().permissions().mode() & 0o111, 0o111);
        }
    }
    let (code, out) = codelaw(&d, &["init", "--git-hooks", "pre-commit"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("pre-commit: already installed"), "{out}");
    assert!(!out.contains("post-commit"), "{out}");
    let (code, out) = codelaw(&d, &["init", "--git-hooks", "pre-push"]);
    assert_eq!(code, 2, "{out}");
}

#[test]
fn hooks_path_is_honoured() {
    if !have_git() {
        return;
    }
    let d = temp("hookspath");
    git(&d, &["config", "core.hooksPath", ".githooks"]);
    let (code, out) = codelaw(&d, &["init", "--git-hooks", "post-commit"]);
    assert_eq!(code, 0, "{out}");
    assert!(d.join(".githooks/post-commit").is_file());
    assert!(!hook(&d, "post-commit").exists());
}

/// DoD 3: an existing hook is chained, not replaced — and never overwritten.
#[test]
fn an_existing_hook_is_chained_and_never_overwritten() {
    if !have_git() {
        return;
    }
    let d = temp("chain");
    fs::create_dir_all(d.join(".git/hooks")).unwrap();
    fs::write(hook(&d, "pre-commit"), "#!/bin/sh\necho theirs-ran >&2\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(hook(&d, "pre-commit"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let (code, out) = codelaw(&d, &["init", "--git-hooks", "pre-commit"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("kept as pre-commit.pre-codelaw"), "{out}");
    assert!(fs::read_to_string(hook(&d, "pre-commit.pre-codelaw")).unwrap().contains("theirs-ran"));
    fs::write(d.join("ok.py"), "x = 1\n").unwrap();
    git(&d, &["add", "ok.py"]);
    let (code, out) = git(&d, &["commit", "-m", "ok"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("theirs-ran"), "the earlier hook runs first: {out}");

    // A third hook (the codelaw one removed by hand and a new foreign one) while
    // `.pre-codelaw` is taken: refuse and change nothing.
    fs::write(hook(&d, "pre-commit"), "#!/bin/sh\necho other\n").unwrap();
    let (code, out) = codelaw(&d, &["init", "--git-hooks", "pre-commit"]);
    assert_eq!(code, 2, "{out}");
    assert_eq!(fs::read_to_string(hook(&d, "pre-commit")).unwrap(), "#!/bin/sh\necho other\n");
}

/// DoD 1: a staged violation blocks the commit.
#[test]
fn a_staged_violation_blocks_the_commit() {
    if !have_git() {
        return;
    }
    let d = temp("block");
    assert_eq!(codelaw(&d, &["init", "--git-hooks", "pre-commit"]).0, 0);
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    git(&d, &["add", "a.py"]);
    let (code, out) = git(&d, &["commit", "-m", "bad"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("fixture/py-no-print"), "{out}");
    assert_ne!(git(&d, &["rev-list", "--count", "HEAD"]).1.trim(), "2", "no commit was made");
    // Skipped on request, as git does for every hook.
    let (code, out) = git(&d, &["commit", "--no-verify", "-m", "bad"]);
    assert_eq!(code, 0, "{out}");
}

/// The hook fixes a fully staged file and re-stages it, so the commit holds the fix.
#[test]
fn a_fully_staged_file_is_fixed_and_restaged() {
    if !have_git() {
        return;
    }
    let d = temp("fix");
    assert_eq!(codelaw(&d, &["init", "--git-hooks", "pre-commit"]).0, 0);
    fs::write(d.join("a.js"), "if (a == b) {}\n").unwrap();
    git(&d, &["add", "a.js"]);
    let (code, out) = git(&d, &["commit", "-m", "fixable"]);
    assert_eq!(code, 0, "{out}");
    let committed = git(&d, &["show", "HEAD:a.js"]).1;
    assert!(committed.contains("a === b"), "{committed}");
}

/// DoD 2: a partly staged file is checked but never auto-fixed, so its
/// unstaged hunks are never committed or clobbered.
#[test]
fn a_partly_staged_file_is_checked_but_never_fixed() {
    if !have_git() {
        return;
    }
    let d = temp("partial");
    fs::write(d.join("a.js"), "let x = 1;\nlet y = 2;\n").unwrap();
    git(&d, &["add", "a.js"]);
    git(&d, &["commit", "-q", "-m", "a"]);
    assert_eq!(codelaw(&d, &["init", "--git-hooks", "pre-commit"]).0, 0);
    // Stage one change, leave another in the working tree only.
    fs::write(d.join("a.js"), "let x = 1;\nif (a == b) {}\nlet y = 2;\n").unwrap();
    git(&d, &["add", "a.js"]);
    let worktree = "let x = 1;\nif (a == b) {}\nlet y = 2;\nlet z = 3;\n";
    fs::write(d.join("a.js"), worktree).unwrap();
    let (_, out) = git(&d, &["commit", "-m", "partial"]);
    assert_eq!(fs::read_to_string(d.join("a.js")).unwrap(), worktree, "the working tree is untouched: {out}");
    let committed = git(&d, &["show", "HEAD:a.js"]).1;
    assert!(!committed.contains("==="), "the staged text was not fixed: {committed}");
    assert!(!committed.contains("let z"), "the unstaged hunk was not committed: {committed}");
}

/// DoD 4: post-commit prints what its steps print and always exits 0.
#[test]
fn post_commit_prints_and_never_fails() {
    if !have_git() {
        return;
    }
    let d = temp("post");
    assert_eq!(codelaw(&d, &["init", "--git-hooks", "post-commit"]).0, 0);
    let steps = d.join(".codelaw/hooks/post-commit.d");
    fs::create_dir_all(&steps).unwrap();
    fs::write(steps.join("10-remind"), "#!/bin/sh\necho owed-reminder\nexit 3\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(steps.join("10-remind"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let (code, out) = run(&mut Command::new(hook(&d, "post-commit")), &d);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("owed-reminder"), "{out}");
    // Even with nothing on PATH to run.
    fs::write(d.join("b.txt"), "b\n").unwrap();
    git(&d, &["add", "b.txt"]);
    let (code, out) = git(&d, &["commit", "-m", "b"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("owed-reminder"), "{out}");
}

/// W-46's reminder: the post-commit hook prints the committer's own
/// exceptions that expire within 7 days, judged on the as-of date, and
/// nothing when there are none.
#[test]
fn post_commit_reminds_of_your_expiring_exceptions() {
    if !have_git() {
        return;
    }
    let d = temp("remind");
    assert_eq!(codelaw(&d, &["init", "--git-hooks", "post-commit"]).0, 0);
    git(&d, &["config", "user.email", "t@example.com"]);
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    let entry = |expires: &str, by: &str| format!("[[allow]]\nfile = \"a.py\"\ncount = 1\nexpires = \"{expires}\"\nadded_by = \"{by}\"\nreason = \"legacy\"\n");
    let commit = |name: &str| {
        fs::write(d.join(name), "x\n").unwrap();
        git(&d, &["add", "-A"]);
        let mut c = Command::new("git");
        c.args(["commit", "-q", "-m", name]).env("CODELAW_AS_OF", "2026-10-25");
        run(&mut c, &d)
    };
    // Mine, 5 days left: reminded.
    fs::write(d.join(".codelaw/baseline/fixture--py-no-print.toml"), entry("2026-10-30", "t@example.com")).unwrap();
    let (code, out) = commit("one");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("expire within 7 days") && out.contains("2026-10-30") && out.contains("in 5 days") && out.contains("fixture/py-no-print"), "{out}");
    // Mine but 40 days away, and someone else's 5 days away: silent.
    fs::write(d.join(".codelaw/baseline/fixture--py-no-print.toml"), format!("{}\n{}", entry("2026-12-04", "t@example.com"), entry("2026-10-30", "other@example.com").replace("a.py", "b.py"))).unwrap();
    let (code, out) = commit("two");
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("expire within"), "{out}");
    // The burn-down line: the baseline had 1 entry when it was first committed (commit two), 2 now.
    assert!(out.contains("baseline burn-down, last 30 days"), "{out}");
}

#[test]
fn init_with_hooks_outside_a_repo_fails_cleanly() {
    let d = std::env::temp_dir().join(format!("codelaw-hooks-norepo-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    let d = fs::canonicalize(d).unwrap();
    if !have_git() {
        return;
    }
    fs::write(d.join("codelaw.toml"), "extends = []\n").unwrap();
    let (code, out) = codelaw(&d, &["init", "--git-hooks"]);
    // A temp dir may sit inside some outer repo; either outcome is a clean exit.
    assert!(code == 0 || (code == 2 && out.contains("git repository")), "{code} {out}");
}
