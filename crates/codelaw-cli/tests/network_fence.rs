//! L5's network fence (SR-LAW-5 decision 3, "How L5 is enforced"): `check`
//! and `fix` run with networking denied and give byte-identical output to an
//! ordinary run.
//!
//! The fence is the operating system's, never a flag of codelaw's:
//! - Linux: a new network namespace with only a downed loopback —
//!   `unshare --net --map-root-user`, or, where unprivileged namespaces are
//!   refused (Ubuntu's AppArmor default), `sudo -n unshare --net` dropping
//!   back to this user with `setpriv`. The fence is proven by reading the
//!   interfaces from inside it.
//! - macOS: `sandbox-exec` with a profile that denies `network*`.
//!
//! Where no fence can be raised the test says so and passes — unless
//! `CODELAW_REQUIRE_NETWORK_FENCE=1` (set in CI), which makes it a failure.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn copy_dir(from: &Path, to: &Path) {
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

/// The L3 fixture repo: the fixture rules plus every language's files.
fn fixture_repo(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-fence-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules")).unwrap();
    let fixtures = repo_root().join("fixtures/l3");
    copy_dir(&fixtures.join("rules"), &d.join(".codelaw/rules"));
    for e in fs::read_dir(&fixtures).unwrap().flatten() {
        if e.file_name() != "rules" && e.path().is_dir() {
            copy_dir(&e.path(), &d);
        }
    }
    fs::canonicalize(d).unwrap()
}

fn files(root: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let mut entries: Vec<_> = fs::read_dir(root).unwrap().flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            files(&p, base, out);
        } else {
            out.push((p.strip_prefix(base).unwrap().to_string_lossy().into_owned(), fs::read(&p).unwrap()));
        }
    }
}

fn output(prefix: &[String], cwd: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let bin = env!("CARGO_BIN_EXE_codelaw");
    let mut cmd = match prefix.split_first() {
        Some((first, rest)) => {
            let mut c = Command::new(first);
            c.args(rest).arg(bin);
            c
        }
        None => Command::new(bin),
    };
    let out = cmd.args(args).current_dir(cwd).env_remove("CODELAW_CACHE_DIR").env_remove("VIRTUAL_ENV").env_remove("CODELAW_SITE_PACKAGES").output().unwrap();
    (out.status.code(), String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
}

fn id(flag: &str) -> Option<String> {
    let out = Command::new("id").arg(flag).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn runs(prefix: &[String], program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(&prefix[0]).args(&prefix[1..]).arg(program).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A command prefix that runs its program with networking denied, or why
/// none could be raised here.
fn fence() -> Result<Vec<String>, String> {
    let v = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    if cfg!(target_os = "linux") {
        let mut tried = Vec::new();
        let mut candidates = vec![v(&["unshare", "--net", "--map-root-user", "--"])];
        if let (Some(u), Some(g)) = (id("-u"), id("-g")) {
            candidates.push(v(&["sudo", "-n", "unshare", "--net", "--", "setpriv", &format!("--reuid={u}"), &format!("--regid={g}"), "--clear-groups", "--"]));
        }
        for prefix in candidates {
            // Proven, not assumed: inside the fence the only interface is `lo`.
            match runs(&prefix, "cat", &["/proc/self/net/dev"]) {
                Some(dev) => {
                    let ifaces: Vec<&str> = dev.lines().skip(2).filter_map(|l| l.split(':').next()).map(str::trim).collect();
                    assert_eq!(ifaces, ["lo"], "`{}` left interfaces up: {dev}", prefix.join(" "));
                    return Ok(prefix);
                }
                None => tried.push(prefix.join(" ")),
            }
        }
        return Err(format!("no network namespace could be made (tried `{}`)", tried.join("`, `")));
    }
    if cfg!(target_os = "macos") {
        let prefix = v(&["sandbox-exec", "-p", "(version 1)(allow default)(deny network*)"]);
        return match runs(&prefix, "true", &[]) {
            Some(_) => Ok(prefix),
            None => Err("sandbox-exec refused the deny-network profile".into()),
        };
    }
    Err(format!("{} has no unprivileged way to deny a process the network", std::env::consts::OS))
}

#[test]
fn l5_check_and_fix_are_identical_with_networking_denied() {
    let prefix = match fence() {
        Ok(p) => p,
        Err(why) => {
            assert!(std::env::var_os("CODELAW_REQUIRE_NETWORK_FENCE").is_none(), "CODELAW_REQUIRE_NETWORK_FENCE is set, but {why}");
            eprintln!("skipped: L5 network fence — {why}");
            return;
        }
    };
    eprintln!("L5 network fence: {}", prefix.join(" "));

    // check, in every format, on the same repo.
    let repo = fixture_repo("check");
    for format in ["json", "text", "agent"] {
        let args = ["check", "--no-cache", "--format", format];
        let open = output(&[], &repo, &args);
        let fenced = output(&prefix, &repo, &args);
        assert_eq!(open.0, Some(1), "check found nothing to report: {}{}", open.1, open.2);
        assert_eq!(open, fenced, "`codelaw check --format {format}` differs with networking denied");
    }

    // fix, on two copies of the repo: the same report and the same bytes.
    let a = fixture_repo("fix-open");
    let b = fixture_repo("fix-fenced");
    for args in [&["fix", "--no-cache"][..], &["fix", "--no-cache", "--unsafe", "--format", "json"][..]] {
        let open = output(&[], &a, args);
        let fenced = output(&prefix, &b, args);
        assert_eq!(open, fenced, "`codelaw {}` differs with networking denied", args.join(" "));
    }
    let (mut fa, mut fb) = (Vec::new(), Vec::new());
    files(&a, &a, &mut fa);
    files(&b, &b, &mut fb);
    assert!(!fa.is_empty());
    assert_eq!(fa, fb, "fix wrote different bytes with networking denied");
    let mut original = Vec::new();
    let c = fixture_repo("unfixed");
    files(&c, &c, &mut original);
    assert_ne!(fa, original, "fix changed nothing, so the comparison proves nothing");

    for d in [repo, a, b, c] {
        let _ = fs::remove_dir_all(d);
    }
}
