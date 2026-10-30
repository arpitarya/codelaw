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
//!
//! The dependency rules (W-52) are fenced the same way: `check` and `fix`
//! read the committed registry snapshot and give the same answer with the
//! network denied, while `deps refresh` — the one command that reads a
//! registry — fails inside the fence (L5 decision 3).

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

/// A project with npm and Python manifests, a lockfile, the data pack and
/// a committed snapshot; its `.npmrc` names a registry on `port`.
fn deps_repo(name: &str, port: u16) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-fence-deps-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    let pack = d.join("node_modules/@codelaw/rules-data");
    fs::create_dir_all(&pack).unwrap();
    copy_dir(&repo_root().join("packs/data"), &pack);
    fs::create_dir_all(d.join("node_modules/@codelaw/rules-deps")).unwrap();
    copy_dir(&repo_root().join("packs/deps"), &d.join("node_modules/@codelaw/rules-deps"));
    fs::create_dir_all(d.join(".codelaw")).unwrap();
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:data/package-json\", \"codelaw:data/pyproject\", \"codelaw:deps/recommended\"]\npreview = true\n").unwrap();
    fs::write(d.join("package.json"), "{\n  \"name\": \"app\",\n  \"dependencies\": {\n    \"react\": \"^18.3.1\",\n    \"left-pad\": \"1.3.0\",\n    \"lodash\": \"^4.17.20\"\n  }\n}\n").unwrap();
    fs::write(d.join("package-lock.json"), r#"{"packages":{"node_modules/react":{"version":"18.3.1"},"node_modules/lodash":{"version":"4.17.20"}}}"#).unwrap();
    fs::write(d.join("pyproject.toml"), "[project]\nname = \"app\"\ndependencies = [\"django~=4.2\", \"bad==1.0\"]\n").unwrap();
    fs::write(
        d.join(".codelaw/deps-snapshot.json"),
        r#"{ "fetched": "2026-10-25", "packages": {
  "npm:react": { "latest": "19.2.0", "versions": { "18.3.1": "2024-04-26", "19.2.0": "2025-10-01" } },
  "npm:left-pad": { "latest": "1.3.0", "versions": { "1.3.0": "2018-04-09" }, "deprecated": { "1.3.0": "use padStart" } },
  "npm:lodash": { "latest": "4.17.21", "versions": { "4.17.20": "2020-08-13", "4.17.21": "2021-02-20" },
    "advisories": [ { "id": "GHSA-35jh-r3h4-6jhm", "aliases": ["CVE-2021-23337"], "severity": "high", "affected": ["<4.17.21"], "fixed": ["4.17.21"], "sources": ["osv"] } ],
    "scanned": { "osv": ["*"] } },
  "pypi:django": { "latest": "5.1.2", "versions": { "4.2.0": "2023-04-03", "5.1.2": "2024-10-08" } },
  "pypi:bad": { "latest": "1.1", "versions": { "1.0": "2020-01-01", "1.1": "2021-01-01" }, "yanked": { "1.0": "" } } },
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-25" }, "registry": { "kind": "registry", "state": "unavailable", "on": "2026-10-25", "error": "HTTP 403" } } }
"#,
    )
    .unwrap();
    fs::write(d.join(".npmrc"), format!("registry=http://127.0.0.1:{port}/\n")).unwrap();
    fs::canonicalize(d).unwrap()
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
    let out = cmd
        .args(args)
        .current_dir(cwd)
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_SITE_PACKAGES")
        .env_remove("CODELAW_AS_OF")
        .env("HOME", cwd)
        .env("PIP_INDEX_URL", "http://127.0.0.1:9/simple")
        .output()
        .unwrap();
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
        if runs(&prefix, "true", &[]).is_none() {
            return Err("sandbox-exec refused the deny-network profile".into());
        }
        // Proven, not assumed: a port this test listens on is reachable
        // outside the fence and refused inside it.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port().to_string();
        let probe = ["-z", "-w", "2", "127.0.0.1", port.as_str()];
        let open = Command::new("nc").args(probe).output().map(|o| o.status.success());
        if open.as_ref().is_ok_and(|ok| *ok) {
            assert!(runs(&prefix, "nc", &probe).is_none(), "`{}` let a connection to 127.0.0.1:{port} through", prefix.join(" "));
        } else {
            eprintln!("note: `nc` cannot reach this test's own port, so the fence is assumed, not proven ({open:?})");
        }
        return Ok(prefix);
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

#[test]
fn l5_the_dependency_rules_need_no_network_and_refresh_is_fenced() {
    let prefix = match fence() {
        Ok(p) => p,
        Err(why) => {
            assert!(std::env::var_os("CODELAW_REQUIRE_NETWORK_FENCE").is_none(), "CODELAW_REQUIRE_NETWORK_FENCE is set, but {why}");
            eprintln!("skipped: L5 network fence (deps) — {why}");
            return;
        }
    };
    // A registry address the open run could reach; the fenced one must not.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let repo = deps_repo("check", port);
    for format in ["json", "text", "agent"] {
        let args = ["check", "--no-cache", "--as-of", "2026-11-30", "--format", format];
        let open = output(&[], &repo, &args);
        let fenced = output(&prefix, &repo, &args);
        assert_eq!(open.0, Some(1), "the deprecated and yanked versions are errors: {}{}", open.1, open.2);
        assert_eq!(open, fenced, "`codelaw check --format {format}` differs with networking denied");
    }
    let (_, json, _) = output(&prefix, &repo, &["check", "--no-cache", "--as-of", "2026-11-30", "--format", "json"]);
    for rule in ["deps/outdated", "deps/stale-snapshot", "deps/vulnerable", "deps/incomplete-scan"] {
        assert!(json.contains(&format!("\"{rule}\"")), "{rule} reported nothing, so the comparison proves nothing: {json}");
    }

    let a = deps_repo("fix-open", port);
    let b = deps_repo("fix-fenced", port);
    let args = ["fix", "--no-cache", "--unsafe", "--as-of", "2026-11-30", "--format", "json"];
    let open = output(&[], &a, &args);
    let fenced = output(&prefix, &b, &args);
    assert_eq!(open, fenced, "`codelaw fix --unsafe` differs with networking denied");
    for f in ["package.json", "pyproject.toml"] {
        assert_eq!(fs::read_to_string(a.join(f)).unwrap(), fs::read_to_string(b.join(f)).unwrap(), "fix wrote different bytes to {f} with networking denied");
    }
    let fixed = fs::read_to_string(a.join("package.json")).unwrap();
    assert!(fixed.contains("^19.2.0") && fixed.contains("^4.17.21"), "the suggest fixes did not apply, so the comparison proves nothing: {fixed}");

    // The one networked command fails inside the fence and writes nothing.
    let before = fs::read_to_string(repo.join(".codelaw/deps-snapshot.json")).unwrap();
    let (code, _, err) = output(&prefix, &repo, &["deps", "refresh", "--as-of", "2026-11-30"]);
    assert_eq!(code, Some(2), "`deps refresh` reached a registry with networking denied: {err}");
    assert!(err.contains("the snapshot was not written"), "{err}");
    assert_eq!(fs::read_to_string(repo.join(".codelaw/deps-snapshot.json")).unwrap(), before);
    drop(listener);
    for d in [repo, a, b] {
        let _ = fs::remove_dir_all(d);
    }
}
