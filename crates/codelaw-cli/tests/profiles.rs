//! SR-CONFIG decisions 10 and 11 through the binary: the `ci` and `local`
//! profiles and how each surface picks one, and `remap`, applied last and
//! never to `codelaw/*`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-profiles-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    fs::copy(repo().join("fixtures/l3/rules/fixture/py-no-print.md"), d.join(".codelaw/rules/fixture/py-no-print.md")).unwrap();
    d
}

/// The binary with no profile input from the test's own environment: CI
/// runners set `CI`.
fn codelaw_env(cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_AS_OF")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_PROFILE")
        .env_remove("CI")
        .env("NO_COLOR", "1");
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    codelaw_env(cwd, args, &[])
}

fn json(cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> serde_json::Value {
    let (_, out, err) = codelaw_env(cwd, args, env);
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"))
}

/// `(file, rule, severity)` of every finding.
fn findings(v: &serde_json::Value) -> Vec<(String, String, String)> {
    v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["file"].as_str().unwrap().to_owned(), d["rule"].as_str().unwrap().to_owned(), d["severity"].as_str().unwrap().to_owned()))
        .collect()
}

const RULE: &str = "fixture/py-no-print";

const PROFILES: &str = "[profiles.local.rules]\n\"fixture/py-no-print\" = \"warn\"\n\n[profiles.ci.rules]\n\"fixture/py-no-print\" = \"error\"\n";

#[test]
fn warn_on_a_laptop_fail_in_ci() {
    let d = temp("ci-local");
    fs::write(d.join("codelaw.toml"), PROFILES).unwrap();
    fs::write(d.join("a.py"), "print(1)\n").unwrap();

    // DoD 1: without CI, a warning and exit 0; with CI=true, exit 1;
    // `--profile local` wins over CI=true.
    let (code, out, err) = codelaw(&d, &["check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.starts_with("profile local (default)\n"), "{out}");
    assert!(out.contains("  warn  "), "{out}");
    let (code, out, _) = codelaw_env(&d, &["check"], &[("CI", "true")]);
    assert_eq!(code, 1, "{out}");
    assert!(out.starts_with("profile ci (CI=true)\n"), "{out}");
    let (code, out, _) = codelaw_env(&d, &["check", "--profile", "local"], &[("CI", "true")]);
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("profile local (--profile)\n"), "{out}");
    // `CI` set to false, 0 or nothing is not CI.
    for v in ["false", "FALSE", "0", ""] {
        let (code, out, _) = codelaw_env(&d, &["check"], &[("CI", v)]);
        assert_eq!(code, 0, "CI={v}: {out}");
    }
    // CODELAW_PROFILE, below the flag.
    let v = json(&d, &["check", "--format", "json"], &[("CODELAW_PROFILE", "ci")]);
    assert_eq!(v["profile"], serde_json::json!({ "name": "ci", "source": "CODELAW_PROFILE" }));
    assert_eq!(findings(&v), [("a.py".to_owned(), RULE.to_owned(), "error".to_owned())]);
    let (code, out, _) = codelaw_env(&d, &["check", "--format", "agent", "--profile", "local"], &[("CODELAW_PROFILE", "ci")]);
    assert_eq!(code, 0);
    assert!(out.starts_with("profile local (--profile)\n"), "{out}");

    // A name the config does not define is a usage error; `ci` and
    // `local` may be absent.
    let (code, _, err) = codelaw(&d, &["check", "--profile", "lcoal"]);
    assert_eq!(code, 2);
    assert!(err.contains("profile `lcoal` (from --profile) is not defined; did you mean `local`?"), "{err}");

    // DoD 2: `codelaw hook` judges with `ci` when CI is not set.
    let (code, _, err) = codelaw(&d, &["hook", "a.py"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("a.py:1:1 error fixture/py-no-print"), "{err}");
    let (_, _, err) = codelaw(&d, &["hook", "--profile", "local", "a.py"]);
    assert!(err.contains("a.py:1:1 warn fixture/py-no-print"), "{err}");

    // DoD 4: `config --print` names the profile and its source.
    let (code, out, _) = codelaw_env(&d, &["config", "--print", "--for", "a.py"], &[("CI", "1")]);
    assert_eq!(code, 0);
    let line = out.lines().find(|l| l.starts_with("profile = ")).unwrap_or_else(|| panic!("{out}"));
    assert!(line.contains("\"ci\"") && line.contains("# CI=1 (codelaw.toml:4)"), "{line}");
    let (_, out, _) = codelaw(&d, &["config", "--print", "--profile", "local"]);
    assert!(out.lines().any(|l| l.starts_with("profile = \"local\"") && l.contains("# --profile (codelaw.toml:1)")), "{out}");

    // A config without profiles: no header, and `ci` changes nothing.
    fs::write(d.join("codelaw.toml"), "").unwrap();
    let (code, out, _) = codelaw_env(&d, &["check", "--profile", "ci"], &[("CI", "true")]);
    assert_eq!(code, 1, "{out}");
    assert!(!out.contains("profile"), "{out}");
    let (code, _, err) = codelaw(&d, &["check", "--profile", "staging"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("the config defines no profiles"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn remap_never_errors_never_warns_or_only_errors() {
    let d = temp("remap");
    fs::write(
        d.join("codelaw.toml"),
        r#"[rules]
"fixture/py-no-print" = "warn"

[[overrides]]
files = ["scripts/**"]
remap = { error = "warn" }

[[overrides]]
files = ["gen/**"]
remap = { error = "off", warn = "off", info = "off" }

[[overrides]]
files = ["payments/**"]
remap = { warn = "error" }

[profiles.ci]
rules = { "fixture/py-no-print" = "error" }
overrides = [{ files = ["legacy/**"], remap = { error = "info" } }]
"#,
    )
    .unwrap();
    for dir in ["scripts", "gen", "payments", "legacy", "src"] {
        fs::create_dir_all(d.join(dir)).unwrap();
        fs::write(d.join(dir).join("a.py"), "print(1)\n").unwrap();
    }
    let sev = |v: &serde_json::Value| -> Vec<(String, String)> { findings(v).into_iter().filter(|f| f.1 == RULE).map(|f| (f.0, f.2)).collect() };
    let pairs = |p: &[(&str, &str)]| -> Vec<(String, String)> { p.iter().map(|(a, b)| ((*a).to_owned(), (*b).to_owned())).collect() };

    // DoD 3, locally: the rule warns; payments/ makes it an error, gen/ is silent.
    let v = json(&d, &["check", "--format", "json", "--as-of", "2026-10-18"], &[]);
    assert_eq!(sev(&v), pairs(&[("legacy/a.py", "warn"), ("payments/a.py", "error"), ("scripts/a.py", "warn"), ("src/a.py", "warn")]));
    // Under CI: the rule errors; scripts/ never fails the run, and the
    // profile's own override applies too.
    let v = json(&d, &["check", "--format", "json", "--as-of", "2026-10-18"], &[("CI", "true")]);
    assert_eq!(sev(&v), pairs(&[("legacy/a.py", "info"), ("payments/a.py", "error"), ("scripts/a.py", "warn"), ("src/a.py", "error")]));

    // Last, after the baseline: an expired entry in scripts/ is a warning;
    // a stale entry's `codelaw/stale-baseline` is never remapped.
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    fs::write(
        d.join(".codelaw/baseline/fixture--py-no-print.toml"),
        "[[allow]]\nfile = \"scripts/a.py\"\ncount = 1\nuntil = \"2026-01-01\"\non_touch = \"error\"\n\n[[allow]]\nfile = \"gen/a.py\"\ncount = 3\non_touch = \"error\"\n",
    )
    .unwrap();
    let v = json(&d, &["check", "--format", "json", "--as-of", "2026-10-18"], &[("CI", "true")]);
    let all = findings(&v);
    assert!(all.contains(&("scripts/a.py".into(), RULE.into(), "warn".into())), "{all:?}");
    assert!(all.iter().any(|f| f.0 == "gen/a.py" && f.1 == "codelaw/stale-baseline" && f.2 == "error"), "{all:?}");
    assert!(!all.iter().any(|f| f.0 == "gen/a.py" && f.1 == RULE), "{all:?}");

    // `config --print --for` shows the remap and where it came from.
    let (_, out, _) = codelaw_env(&d, &["config", "--print", "--for", "legacy/a.py"], &[("CI", "true")]);
    assert!(out.lines().any(|l| l.starts_with("remap.error = \"info\"") && l.contains("profile ci override #1")), "{out}");

    // A bad remap is a config error naming the nearest value.
    fs::write(d.join("codelaw.toml"), "[[overrides]]\nfiles = [\"a/**\"]\nremap = { error = \"wran\" }\n").unwrap();
    let (code, _, err) = codelaw(&d, &["check"]);
    assert_eq!(code, 2);
    assert!(err.contains("override #1: `remap` maps `error` to `wran`; did you mean `warn`?"), "{err}");
    let _ = fs::remove_dir_all(&d);
}
