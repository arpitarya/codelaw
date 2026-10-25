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
    // No machine's own git identity leaks into a written `added_by`.
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
    // `--until` is the last allowed day: it is written as `expires`, the day after.
    assert!(written.contains("[[allow]]\nfile = \"a.py\"\ncount = 2\nexpires = \"2027-04-01\"\nwarn_from = \"2027-03-01\"\non_touch = \"error\"\nreason = \"adopt\"\n"), "{written}");

    // Within allowance: info, exit 0, the as-of date in the header.
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("as-of 2026-10-18 (--as-of)\n"), "{out}");
    assert!(out.contains("grandfathered, expires 2027-04-01 (2 left in this file)"), "{out}");
    // CODELAW_AS_OF works the same.
    let (_, out, _) = codelaw_env(&d, &["check"], &[("CODELAW_AS_OF", "2026-10-18")]);
    assert!(out.starts_with("as-of 2026-10-18 (CODELAW_AS_OF)\n"), "{out}");
    // In the warning window the findings stay info and each entry warns at
    // its own line; on `expires` the findings are back and each entry fails.
    let v = json(&d, &["check", "--as-of", "2027-03-19", "--format", "json"]);
    assert_eq!(severities(&v, RULE), vec!["info"; 3]);
    assert_eq!(severities(&v, "codelaw/expiring-exception"), vec!["warn"; 2], "{v}");
    assert_eq!(v["as_of"], serde_json::json!({ "date": "2027-03-19", "source": "--as-of" }));
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2027-04-01"]);
    assert_eq!(code, 1);
    assert!(out.contains("its baseline entry expired on 2027-04-01") && out.contains("codelaw/expired-exception"), "{out}");
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
    assert!(registry.contains("[[deprecated]]\nsymbol = \"src/utils.ts#formatDate\"\nreplacement = \"src/utils.ts#formatISO\"\nexpires = \"2027-04-01\"\nreason = \"Timezone bugs\"\n"), "{registry}");
    let baseline = fs::read_to_string(d.join(".codelaw/baseline/graph--no-deprecated-usage.toml")).unwrap();
    assert!(baseline.contains("file = \"src/list.ts\"\nsymbol = \"src/utils.ts#formatDate\"\ncount = 2\nexpires = \"2027-04-01\""), "{baseline}");

    let v = json(&d, &["check", "--as-of", "2026-10-18", "--format", "json"]);
    assert_eq!(severities(&v, "graph/no-deprecated-usage"), vec!["info", "info"], "{v}");
    // A new use elsewhere is an error at once, naming the replacement.
    fs::write(d.join("src/cart.ts"), "import { formatDate } from \"./utils\";\n\nexport const t = formatDate(new Date());\n").unwrap();
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-10-18"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("src/cart.ts") && out.contains("`formatDate` is deprecated; use `formatISO` (expires 2027-04-01)"), "{out}");
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
    assert!(out.contains("`old_tax` is deprecated (expires 2027-02-01)") && out.contains("Why: use new_tax"), "{out}");
    // A bad key in `[deprecation]` is a config error.
    fs::write(d.join("codelaw.toml"), "[deprecation]\nuntill = \"2027-01-31\"\n").unwrap();
    let (code, _, err) = codelaw(&d, &["check"]);
    assert_eq!(code, 2);
    assert!(err.contains("untill"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

// SR-BASELINE decision 9 (W-40): dated exceptions. `expires = D` is the
// first failing day; the warning starts `warn_days` (default 3) before it.

const EXPIRING: &str = "codelaw/expiring-exception";
const EXPIRED: &str = "codelaw/expired-exception";

/// `check --as-of <day>`: the exit code, and every rule id reported at
/// `warn` or `error`.
fn verdict(d: &Path, day: &str) -> (i32, Vec<String>) {
    let (code, out, err) = codelaw(d, &["check", "--as-of", day, "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    let loud = v["diagnostics"].as_array().unwrap().iter().filter(|x| x["severity"] != "info").map(|x| format!("{}:{}", x["severity"].as_str().unwrap(), x["rule"].as_str().unwrap())).collect();
    (code, loud)
}

/// DoD 1, for one form: D−4 passes silently, D−3 warns, D fails.
fn the_timeline(d: &Path, form: &str) {
    let (code, loud) = verdict(d, "2026-11-11");
    assert_eq!((code, loud.clone()), (0, Vec::<String>::new()), "{form}, D-4");
    let (code, loud) = verdict(d, "2026-11-12");
    assert_eq!(code, 0, "{form}, D-3: {loud:?}");
    assert_eq!(loud, vec![format!("warn:{EXPIRING}")], "{form}, D-3");
    let (code, out, _) = codelaw(d, &["check", "--as-of", "2026-11-12"]);
    assert!(out.contains("expires in 3 days (2026-11-15)"), "{form}: {out}");
    let (code_d, loud) = verdict(d, "2026-11-15");
    assert_eq!(code_d, 1, "{form}, D: {loud:?}");
    assert!(loud.contains(&format!("error:{EXPIRED}")), "{form}, D: {loud:?}");
    let _ = code;
}

#[test]
fn dated_exceptions_warn_before_and_fail_on_the_day() {
    // The baseline form.
    let d = temp("dated-baseline");
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    fs::write(d.join(BASELINE), "[[allow]]\nfile = \"a.py\"\ncount = 1\nexpires = \"2026-11-15\"\nreason = \"legacy\"\nadded_by = \"a@example.com\"\n").unwrap();
    the_timeline(&d, "baseline");
    let (_, loud) = verdict(&d, "2026-11-15");
    assert!(loud.contains(&format!("error:{RULE}")), "on the day the finding is back at the rule's own severity: {loud:?}");
    let (_, out, _) = codelaw(&d, &["check", "--as-of", "2026-11-15"]);
    assert!(out.contains(".codelaw/baseline/fixture--py-no-print.toml\n  1:1  error"), "the error is at the entry's own line: {out}");
    let _ = fs::remove_dir_all(&d);

    // The deprecation form: the entry inherits the registry's `expires`.
    let d = temp("dated-deprecation");
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/utils.ts"), "export function formatDate(d: Date) {\n  return d.toISOString();\n}\n").unwrap();
    fs::write(d.join("src/list.ts"), "import { formatDate } from \"./utils\";\n\nexport const a = formatDate(new Date());\n").unwrap();
    let (code, out, err) = codelaw(&d, &["deprecate", "src/utils.ts#formatDate", "--expires", "2026-11-15", "--added-by", "a@example.com"]);
    assert_eq!(code, 0, "{out}{err}");
    let registry = fs::read_to_string(d.join(".codelaw/deprecations.toml")).unwrap();
    assert!(registry.contains("expires = \"2026-11-15\"\nadded_by = \"a@example.com\"\n"), "{registry}");
    the_timeline(&d, "deprecation");
    let _ = fs::remove_dir_all(&d);

    // The inline form.
    let d = temp("dated-inline");
    fs::write(d.join("a.py"), "x = 1\n# codelaw-ignore fixture/py-no-print expires=2026-11-15 by=a@example.com: until the logger lands\nprint(1)\n").unwrap();
    the_timeline(&d, "inline");
    let (_, loud) = verdict(&d, "2026-11-15");
    assert!(loud.contains(&format!("error:{RULE}")), "an expired ignore suppresses nothing: {loud:?}");
    let (_, out, _) = codelaw(&d, &["check", "--as-of", "2026-11-15"]);
    assert!(out.contains("2:1  error  This `codelaw-ignore fixture/py-no-print` expired on 2026-11-15"), "{out}");
    // `warn_days=` on the comment widens its own window.
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print expires=2026-11-15 warn_days=10: legacy\n").unwrap();
    assert_eq!(verdict(&d, "2026-11-05").1, vec![format!("warn:{EXPIRING}")]);
    // A malformed author is an error wherever it runs; a bad date makes
    // the comment unreadable, so it covers nothing.
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print by=not-an-email: legacy\n").unwrap();
    assert_eq!(verdict(&d, "2026-11-01"), (1, vec!["error:codelaw/exception-author".to_owned()]));
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print expires=2026-02-30: legacy\n").unwrap();
    let (code, loud) = verdict(&d, "2026-11-01");
    assert_eq!(code, 1);
    assert!(loud.contains(&"error:codelaw/unknown-suppression".to_owned()) && loud.contains(&format!("error:{RULE}")), "{loud:?}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn require_expiry_makes_the_date_mandatory() {
    let d = temp("require-expiry");
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print: legacy\n").unwrap();
    fs::write(d.join("codelaw.toml"), "[exceptions]\nrequire_expiry = false\n").unwrap();
    assert_eq!(verdict(&d, "2026-11-01"), (0, Vec::new()));
    fs::write(d.join("codelaw.toml"), "[exceptions]\nrequire_expiry = true\n").unwrap();
    assert_eq!(verdict(&d, "2026-11-01"), (1, vec!["error:codelaw/exception-without-expiry".to_owned()]));
    // A dated one passes; so does a baseline entry that has its date.
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print expires=2027-01-01: legacy\nprint(2)\n").unwrap();
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    fs::write(d.join(BASELINE), "[[allow]]\nfile = \"a.py\"\ncount = 1\n").unwrap();
    assert_eq!(verdict(&d, "2026-11-01"), (1, vec!["error:codelaw/exception-without-expiry".to_owned()]), "the undated baseline entry");
    let (code, out, err) = codelaw(&d, &["baseline", "add", "--rule", RULE, "--expires", "2027-01-01"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(verdict(&d, "2026-11-01"), (0, Vec::new()));
    // `[exceptions]` is checked like every validated table.
    fs::write(d.join("codelaw.toml"), "[exceptions]\nrequire_expiry = true\nwarn_dayz = 3\n").unwrap();
    let (code, _, err) = codelaw(&d, &["check"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("warn_dayz"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

/// DoD 4: `until = D−1` is `expires = D`, byte for byte.
#[test]
fn until_is_read_as_expires_the_day_after() {
    let d = temp("until-alias");
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    let run = |entry: &str, comment: &str| {
        fs::write(d.join(BASELINE), format!("[[allow]]\nfile = \"a.py\"\ncount = 1\n{entry}\n")).unwrap();
        fs::write(d.join("b.py"), format!("print(2)  # codelaw-ignore fixture/py-no-print {comment}: legacy\n")).unwrap();
        ["2026-11-11", "2026-11-12", "2026-11-14", "2026-11-15"].map(|day| codelaw(&d, &["check", "--as-of", day]))
    };
    let old = run("until = \"2026-11-14\"", "until=2026-11-14");
    let new = run("expires = \"2026-11-15\"", "expires=2026-11-15");
    assert_eq!(old, new);
    // Both on one entry is a load error.
    fs::write(d.join(BASELINE), "[[allow]]\nfile = \"a.py\"\ncount = 1\nuntil = \"2026-11-14\"\nexpires = \"2026-11-15\"\n").unwrap();
    let (code, _, err) = codelaw(&d, &["check", "--as-of", "2026-11-01"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("`until` and `expires`"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

/// DoD 3 and 5: `added_by` against git, and the ratchet on `expires`.
#[test]
fn added_by_is_checked_against_git_and_expires_only_moves_earlier() {
    if !have_git() {
        eprintln!("skipped: no git on this host");
        return;
    }
    let d = temp("author");
    fs::write(d.join("a.py"), "x = 1\n").unwrap();
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    fs::write(d.join(BASELINE), "[[allow]]\nfile = \"b.py\"\ncount = 1\nexpires = \"2026-11-15\"\n").unwrap();
    fs::write(d.join("b.py"), "print(1)\n").unwrap();
    git(&d, &["init", "-q", "-b", "main"]);
    git(&d, &["config", "user.email", "me@example.com"]);
    git(&d, &["add", "-A"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    git(&d, &["checkout", "-q", "-b", "topic"]);

    // `--staged`: the exception must name `git config user.email`.
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print by=other@example.com: legacy\n").unwrap();
    git(&d, &["add", "a.py"]);
    let (code, out, _) = codelaw(&d, &["check", "--staged", "--as-of", "2026-11-01"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("names `other@example.com` as its author, but git names `me@example.com`"), "{out}");
    let (code, out, _) = codelaw(&d, &["check", "--as-of", "2026-11-01"]);
    assert_eq!(code, 0, "a plain check only checks the format: {out}");
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print by=ME@example.com: legacy\n").unwrap();
    git(&d, &["add", "a.py"]);
    let (code, out, _) = codelaw(&d, &["check", "--staged", "--as-of", "2026-11-01"]);
    assert_eq!(code, 0, "{out}");

    // `--changed-since`: the blame author of the line (here `t@`, the
    // committer the helper sets), not the one running the check.
    git(&d, &["commit", "-q", "-m", "topic"]);
    let (code, out, _) = codelaw(&d, &["check", "--changed-since", "main", "--as-of", "2026-11-01"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("names `ME@example.com` as its author, but git names `t@example.com`"), "{out}");
    fs::write(d.join("a.py"), "print(1)  # codelaw-ignore fixture/py-no-print by=t@example.com: legacy\n").unwrap();
    git(&d, &["commit", "-q", "-am", "author"]);
    let (code, out, _) = codelaw(&d, &["check", "--changed-since", "main", "--as-of", "2026-11-01"]);
    assert_eq!(code, 0, "{out}");
    // A baseline entry's `added_by`, changed in the index.
    let entry = "[[allow]]\nfile = \"b.py\"\ncount = 1\nexpires = \"2026-11-15\"\nadded_by = \"other@example.com\"\n";
    fs::write(d.join(BASELINE), entry).unwrap();
    git(&d, &["add", BASELINE]);
    let (code, out, _) = codelaw(&d, &["check", "--staged", "--as-of", "2026-11-01"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("The baseline entry of `fixture/py-no-print` for `b.py` names `other@example.com`"), "{out}");
    git(&d, &["checkout", "-q", "--", "."]);
    git(&d, &["reset", "-q", "--hard"]);

    // The ratchet: `expires` moved later is a deadline extended; adding
    // `added_by` or a `reason` is not.
    fs::write(d.join(BASELINE), "[[allow]]\nfile = \"b.py\"\ncount = 1\nexpires = \"2026-11-15\"\nreason = \"r\"\nadded_by = \"t@example.com\"\n").unwrap();
    let (code, out, err) = codelaw(&d, &["baseline", "check", "--since", "main"]);
    assert_eq!(code, 0, "{out}{err}");
    fs::write(d.join(BASELINE), "[[allow]]\nfile = \"b.py\"\ncount = 1\nexpires = \"2026-11-20\"\n").unwrap();
    let (code, out, err) = codelaw(&d, &["baseline", "check", "--since", "main"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("b.py: deadline extended, expires moved from 2026-11-15 to 2026-11-20"), "{out}");
    let _ = fs::remove_dir_all(&d);
}
