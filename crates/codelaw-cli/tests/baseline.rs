//! SR-BASELINE through the binary: the as-of date and the change set as
//! explicit inputs (L5 decision 2), count-based baselines with `until`,
//! `on_touch` and the ratchet, and the deprecations registry with
//! `graph/no-deprecated-usage`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-baseline-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    fs::copy(repo().join("fixtures/l3/rules/fixture/py-no-print.md"), d.join(".codelaw/rules/fixture/py-no-print.md")).unwrap();
    d
}

fn codelaw_env(cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_AS_OF").env_remove("CODELAW_CACHE_DIR").env("NO_COLOR", "1");
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    codelaw_env(cwd, args, &[])
}

fn json(cwd: &Path, args: &[&str]) -> serde_json::Value {
    let (_, out, err) = codelaw(cwd, args);
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"))
}

fn severities(v: &serde_json::Value, rule: &str) -> Vec<String> {
    v["diagnostics"].as_array().unwrap().iter().filter(|d| d["rule"] == rule).map(|d| d["severity"].as_str().unwrap().to_owned()).collect()
}

fn have_git() -> bool {
    Command::new("git").arg("--version").output().is_ok_and(|o| o.status.success())
}

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_AUTHOR_DATE", "2026-01-15T12:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-01-15T12:00:00+00:00")
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

const RULE: &str = "fixture/py-no-print";
const BASELINE: &str = ".codelaw/baseline/fixture--py-no-print.toml";

#[test]
fn grandfather_deadline_count_stale_and_prune() {
    let d = temp("lifecycle");
    fs::write(d.join("a.py"), "print(1)\nprint(2)\n").unwrap();
    fs::write(d.join("b.py"), "print(3)\n").unwrap();
    let before = json(&d, &["check", "--as-of", "2026-10-18", "--format", "json"]);
    assert_eq!(severities(&before, RULE), vec!["error"; 3]);
    assert!(before.get("as_of").is_none(), "no baseline, no as-of: {before}");

    let (code, out, err) = codelaw(&d, &["baseline", "add", "--rule", RULE, "--until", "2027-03-31", "--warn-from", "2027-03-01", "--reason", "adopt"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("2 entries, 3 findings allowed"), "{out}");
    let written = fs::read_to_string(d.join(BASELINE)).unwrap();
    assert!(written.contains("[[allow]]\nfile = \"a.py\"\ncount = 2\nuntil = \"2027-03-31\"\nwarn_from = \"2027-03-01\"\non_touch = \"error\"\nreason = \"adopt\"\n"), "{written}");

    // Within allowance: info, exit 0, the as-of date in the header.
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("as-of 2026-10-18 (--as-of)\n"), "{out}");
    assert!(out.contains("grandfathered until 2027-03-31 (2 left in this file)"), "{out}");
    // CODELAW_AS_OF works the same.
    let (_, out, _) = codelaw_env(&d, &["check"], &[("CODELAW_AS_OF", "2026-10-18")]);
    assert!(out.starts_with("as-of 2026-10-18 (CODELAW_AS_OF)\n"), "{out}");
    // In the warning window, then past the deadline.
    let v = json(&d, &["check", "--as-of", "2027-03-19", "--format", "json"]);
    assert_eq!(severities(&v, RULE), vec!["warn"; 3]);
    assert_eq!(v["as_of"], serde_json::json!({ "date": "2027-03-19", "source": "--as-of" }));
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2027-04-01"]);
    assert_eq!(code, 1);
    assert!(out.contains("its baseline deadline 2027-03-31 has passed"), "{out}");
    // A bad date is a usage error.
    let (code, _, err) = codelaw(&d, &["check", "--as-of", "2027-02-30"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("--as-of"), "{err}");

    // No git here and no as-of: deadlines are not evaluated, and the header says so.
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18", "--changed-since", "main"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("--changed-since main: not a git repository"), "{out}");
    let (code, out, _) = codelaw(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("as-of: none (not a git repository; pass --as-of YYYY-MM-DD or set CODELAW_AS_OF); deadlines are not evaluated\n"), "{out}");

    // A new use in a file: beyond its count, every occurrence fails.
    fs::write(d.join("a.py"), "print(1)\nprint(2)\nprint(9)\n").unwrap();
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18"]);
    assert_eq!(code, 1);
    assert!(out.contains("(3 in this file; the baseline allows 2)"), "{out}");
    // A use removed: the entry is stale, and `fix` lowers it.
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18"]);
    assert_eq!(code, 1);
    assert!(out.contains("codelaw/stale-baseline") && out.contains("allows 2 in this file, which now has 1"), "{out}");
    // Checking only b.py says nothing about a.py's entry.
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18", "b.py"]);
    assert_eq!(code, 0, "{out}");
    let (code, out, _) = codelaw(&d, &["fix", "--as-of", "2026-10-18"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains(&format!("pruned {BASELINE}")), "{out}");
    assert!(fs::read_to_string(d.join(BASELINE)).unwrap().contains("file = \"a.py\"\ncount = 1\n"));
    // Everything fixed: prune removes the entries, then the file.
    fs::write(d.join("a.py"), "x = 1\n").unwrap();
    fs::write(d.join("b.py"), "x = 1\n").unwrap();
    let (code, out, _) = codelaw(&d, &["baseline", "prune"]);
    assert_eq!(code, 0);
    assert_eq!(out, format!("pruned {BASELINE}\n"));
    assert!(!d.join(BASELINE).exists());
    let _ = fs::remove_dir_all(&d);
}

/// L5: the verdict comes from `HEAD`'s commit date, never the clock. The
/// same repo is checked on two dates at once — the wall clock in UTC+14 and
/// in UTC−12, which are always a calendar day or two apart — with a
/// deadline between them, and the output must be byte-identical.
#[test]
fn the_same_repo_on_two_dates_gives_the_same_output() {
    if !have_git() {
        eprintln!("skipped: no git on this host");
        return;
    }
    let d = temp("two-dates");
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let date = |secs: i64| codelaw_baseline::Date::from_days(secs.div_euclid(86_400));
    let (east, west) = (date(now + 14 * 3600), date(now - 12 * 3600));
    assert!(east > west);
    // The deadline is the western date: a clock read in the east would
    // find it passed, one read in the west would not.
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    fs::write(d.join(BASELINE), format!("[[allow]]\nfile = \"a.py\"\ncount = 1\nuntil = \"{west}\"\non_touch = \"error\"\n")).unwrap();
    git(&d, &["init", "-q", "-b", "main"]);
    git(&d, &["add", "-A"]);
    git(&d, &["commit", "-q", "-m", "one"]);
    let east_out = codelaw_env(&d, &["check", "--format", "json"], &[("TZ", "Pacific/Kiritimati")]);
    let west_out = codelaw_env(&d, &["check", "--format", "json"], &[("TZ", "Etc/GMT+12")]);
    assert_eq!(east_out, west_out);
    let v: serde_json::Value = serde_json::from_str(&east_out.1).unwrap();
    assert_eq!(v["as_of"], serde_json::json!({ "date": "2026-01-15", "source": "HEAD commit date" }));
    assert_eq!(severities(&v, RULE), vec!["info"], "judged on the commit date, before the deadline");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn the_change_set_and_the_ratchet() {
    if !have_git() {
        eprintln!("skipped: no git on this host");
        return;
    }
    let d = temp("git");
    fs::write(d.join("a.py"), "print(1)\nprint(2)\n").unwrap();
    fs::write(d.join("b.py"), "print(3)\n").unwrap();
    let (code, out, err) = codelaw(&d, &["baseline", "add", "--rule", RULE]);
    assert_eq!(code, 0, "{out}{err}");
    git(&d, &["init", "-q", "-b", "main"]);
    git(&d, &["add", "-A"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    git(&d, &["checkout", "-q", "-b", "topic"]);
    // The veto's case: one use removed, one added, the same count.
    fs::write(d.join("a.py"), "print(1)\nprint(9)\n").unwrap();
    let (code, out, _) = codelaw(&d, &["check"]);
    assert_eq!(code, 0, "nothing touched: the swap passes\n{out}");
    let (code, out, _) = codelaw(&d, &["check", "--changed-since", "main"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("you edited this file: migrate its 2 occurrences now (on_touch = error)"), "{out}");
    assert!(out.contains("b.py") && out.contains("grandfathered (1 left in this file)"), "b.py is untouched: {out}");
    // The index, for pre-commit.
    let (code, _, _) = codelaw(&d, &["check", "--staged"]);
    assert_eq!(code, 0, "nothing staged yet");
    git(&d, &["add", "a.py"]);
    let (code, out, _) = codelaw(&d, &["check", "--staged"]);
    assert_eq!(code, 1, "{out}");

    // The ratchet: lowering passes, raising fails.
    let (code, out, err) = codelaw(&d, &["baseline", "check", "--since", "main"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(err.contains("no CODEOWNERS file"), "{err}");
    let raised = fs::read_to_string(d.join(BASELINE)).unwrap().replace("count = 1", "count = 4");
    fs::write(d.join(BASELINE), raised).unwrap();
    fs::create_dir_all(d.join(".github")).unwrap();
    fs::write(d.join(".github/CODEOWNERS"), "/.codelaw/baseline/ @leads\n").unwrap();
    let (code, out, err) = codelaw(&d, &["baseline", "check", "--since", "main"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("fixture/py-no-print: b.py: count rose from 1 to 4"), "{out}");
    assert!(!err.contains("CODEOWNERS"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn deprecate_and_the_engine_rule() {
    let d = temp("deprecate");
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/utils.ts"), "export function formatDate(d: Date) {\n  return d.toISOString();\n}\nexport function formatISO(d: Date) {\n  return d.toISOString();\n}\n").unwrap();
    fs::write(d.join("src/list.ts"), "import { formatDate as fd } from \"./utils\";\n\nexport const a = fd(new Date());\nexport const b = fd(new Date());\n").unwrap();
    fs::write(d.join("src/own.ts"), "function formatDate() {\n  return 1;\n}\nexport const c = formatDate();\n").unwrap();
    let (code, out, err) = codelaw(
        &d,
        &["deprecate", "src/utils.ts#formatDate", "--replacement", "src/utils.ts#formatISO", "--until", "2027-03-31", "--reason", "Timezone bugs"],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("2 current uses grandfathered"), "{out}");
    let registry = fs::read_to_string(d.join(".codelaw/deprecations.toml")).unwrap();
    assert!(registry.contains("[[deprecated]]\nsymbol = \"src/utils.ts#formatDate\"\nreplacement = \"src/utils.ts#formatISO\"\nuntil = \"2027-03-31\"\nreason = \"Timezone bugs\"\n"), "{registry}");
    let baseline = fs::read_to_string(d.join(".codelaw/baseline/graph--no-deprecated-usage.toml")).unwrap();
    assert!(baseline.contains("file = \"src/list.ts\"\nsymbol = \"src/utils.ts#formatDate\"\ncount = 2\nuntil = \"2027-03-31\""), "{baseline}");

    let v = json(&d, &["check", "--as-of", "2026-10-18", "--format", "json"]);
    assert_eq!(severities(&v, "graph/no-deprecated-usage"), vec!["info", "info"], "{v}");
    // A new use elsewhere is an error at once, naming the replacement.
    fs::write(d.join("src/cart.ts"), "import { formatDate } from \"./utils\";\n\nexport const t = formatDate(new Date());\n").unwrap();
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("src/cart.ts") && out.contains("`formatDate` is deprecated; use `formatISO` (deadline 2027-03-31)"), "{out}");
    assert!(!out.contains("src/own.ts"), "a local function of the same name is not it: {out}");
    // After the deadline every use fails.
    let v = json(&d, &["check", "--as-of", "2027-04-01", "--format", "json"]);
    assert_eq!(severities(&v, "graph/no-deprecated-usage"), vec!["error"; 3], "{v}");
    // `--rule` reaches the engine's own rule; `off` turns it off.
    let v = json(&d, &["check", "--as-of", "2026-10-18", "--rule", "graph/no-deprecated-usage", "--format", "json"]);
    assert_eq!(severities(&v, "graph/no-deprecated-usage").len(), 3);
    fs::write(d.join("codelaw.toml"), "[rules]\n\"graph/no-deprecated-usage\" = \"off\"\n").unwrap();
    let v = json(&d, &["check", "--as-of", "2026-10-18", "--format", "json"]);
    assert!(severities(&v, "graph/no-deprecated-usage").is_empty(), "{v}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn an_in_code_marker_takes_the_deprecation_defaults() {
    let d = temp("marker");
    fs::write(d.join("codelaw.toml"), "[deprecation]\nuntil = \"2027-01-31\"\n").unwrap();
    fs::write(d.join("legacy.py"), "from typing_extensions import deprecated\n\n\n@deprecated(\"use new_tax\")\ndef old_tax(x):\n    return x\n").unwrap();
    fs::write(d.join("app.py"), "from legacy import old_tax as ot\n\not(1)\n").unwrap();
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("`old_tax` is deprecated (deadline 2027-01-31)") && out.contains("Why: use new_tax"), "{out}");
    // A bad key in `[deprecation]` is a config error.
    fs::write(d.join("codelaw.toml"), "[deprecation]\nuntill = \"2027-01-31\"\n").unwrap();
    let (code, _, err) = codelaw(&d, &["check"]);
    assert_eq!(code, 2);
    assert!(err.contains("untill"), "{err}");
    let _ = fs::remove_dir_all(&d);
}
