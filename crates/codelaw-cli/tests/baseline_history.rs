//! SR-BASELINE decision 10 through the binary: `codelaw baseline history`
//! reads the burn-down from the commits that touched `.codelaw/baseline/`,
//! with no lint run.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-history-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn codelaw_env(cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_AS_OF").env_remove("CODELAW_CACHE_DIR").env("NO_COLOR", "1");
    c.env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1");
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    codelaw_env(cwd, args, &[])
}

fn have_git() -> bool {
    Command::new("git").arg("--version").output().is_ok_and(|o| o.status.success())
}

/// `git <args>` in `cwd`, with both dates `day` when one is given.
fn git(cwd: &Path, args: &[&str], day: Option<&str>) {
    let date = format!("{}T12:00:00+00:00", day.unwrap_or("2026-01-01"));
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_AUTHOR_DATE", &date)
        .env("GIT_COMMITTER_DATE", &date)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

fn entries(files: &[(&str, u32)]) -> String {
    files.iter().map(|(f, n)| format!("[[allow]]\nfile = \"{f}\"\ncount = {n}\n")).collect()
}

/// Five baseline commits, hand-counted. The third adds a second rule, the
/// fourth renames the first rule's file (one rule gone, a new one), the
/// fifth deletes a rule's file.
#[test]
fn the_burn_down_is_read_from_the_commits() {
    if !have_git() {
        eprintln!("skipped: no git on this host");
        return;
    }
    let d = temp("five");
    let dir = d.join(".codelaw/baseline");
    git(&d, &["init", "-q", "-b", "main"], None);
    let step = |day: &str, files: &[(&str, &[(&str, u32)])]| {
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for (name, es) in files {
            fs::write(dir.join(name), entries(es)).unwrap();
        }
        git(&d, &["add", "-A"], Some(day));
        git(&d, &["commit", "-q", "-m", day], Some(day));
    };
    let (a, b, r) = ("fixture--py-no-print.toml", "fixture--other.toml", "fixture--renamed.toml");
    step("2026-01-05", &[(a, &[("a.py", 5), ("b.py", 3)])]);
    step("2026-01-06", &[(a, &[("a.py", 4), ("b.py", 3)])]);
    step("2026-01-13", &[(a, &[("a.py", 5)]), (b, &[("x.py", 2)])]);
    step("2026-01-14", &[(r, &[("a.py", 3)]), (b, &[("x.py", 2)])]);
    step("2026-01-20", &[(r, &[("a.py", 1)])]);

    let counts = |v: &serde_json::Value, rule: &str| -> Vec<u64> {
        let row = if rule == "total" { &v["total"] } else { v["rules"].as_array().unwrap().iter().find(|r| r["rule"] == rule).unwrap_or_else(|| panic!("{rule} in {v}")) };
        row["counts"].as_array().unwrap().iter().map(|n| n.as_u64().unwrap()).collect()
    };
    let hist = |args: &[&str]| -> serde_json::Value {
        let mut a = vec!["baseline", "history", "--format", "json"];
        a.extend(args);
        let (code, out, err) = codelaw(&d, &a);
        assert_eq!(code, 0, "{out}{err}");
        serde_json::from_str(&out).unwrap()
    };

    // DoD 1 and 2: hand-counted, through a rename and a deletion.
    let v = hist(&[]);
    assert_eq!(counts(&v, "total"), [8, 7, 7, 5, 1]);
    assert_eq!(counts(&v, "fixture/py-no-print"), [8, 7, 5, 0, 0]);
    assert_eq!(counts(&v, "fixture/other"), [0, 0, 2, 2, 0]);
    assert_eq!(counts(&v, "fixture/renamed"), [0, 0, 0, 3, 1]);
    assert_eq!(v["rules"][0]["rule"], "fixture/other", "rules are sorted by id");
    assert_eq!(v["points"].as_array().unwrap().len(), 5);
    assert_eq!(v["points"][4]["date"], "2026-01-20");

    // Buckets keep the last commit of the day or week (Monday first).
    assert_eq!(counts(&hist(&["--by", "week"]), "total"), [7, 5, 1]);
    assert_eq!(counts(&hist(&["--by", "day"]), "total"), [8, 7, 7, 5, 1]);
    // The window: the state before it opens the series; durations count from `HEAD`'s date.
    assert_eq!(counts(&hist(&["--since", "2026-01-13"]), "total"), [7, 7, 5, 1]);
    assert_eq!(counts(&hist(&["--since", "7d"]), "total"), [7, 7, 5, 1]);
    assert_eq!(counts(&hist(&["--since", "1w"]), "total"), [7, 7, 5, 1]);
    assert_eq!(counts(&hist(&["--since", "HEAD~2"]), "total"), [7, 5, 1]);
    assert_eq!(counts(&hist(&["--since", "2026-02-01"]), "total"), [1]);
    // One rule.
    let v = hist(&["--rule", "fixture/other"]);
    assert_eq!(v["rules"].as_array().unwrap().len(), 1);
    assert_eq!(counts(&v, "total"), [0, 0, 2, 2, 0]);

    // Text, markdown and csv.
    let (code, text, _) = codelaw(&d, &["baseline", "history", "--since", "2026-01-13"]);
    assert_eq!(code, 0);
    assert!(text.contains("rule") && text.contains("fixture/py-no-print") && text.contains("−7"), "{text}");
    let last = text.lines().last().unwrap();
    assert!(last.starts_with("total") && last.contains("−6"), "{text}");
    let (_, md, _) = codelaw(&d, &["baseline", "history", "--format", "markdown"]);
    assert!(md.contains("| `fixture/other` | 0 | 0 | 0 |") && md.contains("| `total` | 8 | 1 | −7 |"), "{md}");
    let (_, csv, _) = codelaw(&d, &["baseline", "history", "--format", "csv", "--by", "week"]);
    assert!(csv.starts_with("date,commit,rule,count\n2026-01-06,") && csv.contains(",total,7\n") && csv.contains(",fixture/renamed,3\n"), "{csv}");

    // DoD 3: the same range, byte for byte, whatever the time zone.
    for fmt in ["text", "json", "csv", "markdown"] {
        let run = codelaw_env(&d, &["baseline", "history", "--format", fmt], &[("TZ", "Pacific/Kiritimati")]);
        assert_eq!(run, codelaw(&d, &["baseline", "history", "--format", fmt]), "{fmt}");
    }

    // Usage errors.
    let (code, _, err) = codelaw(&d, &["baseline", "history", "--by", "year"]);
    assert!(code == 2 && err.contains("unknown --by"), "{err}");
    let (code, _, err) = codelaw(&d, &["baseline", "history", "--since", "nonesuch"]);
    assert!(code == 2 && err.contains("nonesuch"), "{err}");

    // A shallow clone says so and suggests the whole history.
    let shallow = temp("shallow");
    git(&d, &["clone", "-q", "--depth", "1", "--no-local", d.to_str().unwrap(), shallow.join("c").to_str().unwrap()], None);
    let (code, out, err) = codelaw(&shallow.join("c"), &["baseline", "history"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(err.contains("shallow clone") && err.contains("fetch-depth: 0"), "{err}");
    let _ = fs::remove_dir_all(&shallow);

    // DoD 4: no `git` is a usage error.
    let (code, _, err) = codelaw_env(&d, &["baseline", "history"], &[("PATH", "")]);
    assert!(code == 2 && err.contains("needs git"), "{err}");
    let _ = fs::remove_dir_all(&d);

    // So is no repository.
    let bare = temp("norepo");
    let (code, _, err) = codelaw(&bare, &["baseline", "history"]);
    assert!(code == 2 && err.contains("needs git") && err.contains("not a git repository"), "{err}");
    let _ = fs::remove_dir_all(&bare);
}
