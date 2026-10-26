//! SR-BASELINE decision 11 through the binary: `codelaw exceptions` lists
//! every exception (baseline entry, deprecation, inline ignore), judged on
//! the as-of date and never the clock.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-exceptions-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    fs::copy(repo().join("fixtures/l3/rules/fixture/py-no-print.md"), d.join(".codelaw/rules/fixture/py-no-print.md")).unwrap();
    d
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_AS_OF").env_remove("CODELAW_CACHE_DIR").env("NO_COLOR", "1");
    c.env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1");
    let out = c.output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

/// One of each form, as-of 2026-10-25:
/// - baseline `a.py`, 2 uses, expires 2026-11-02 (8 days), by arpit;
/// - baseline `b.py`, 1 use, no date, no author;
/// - deprecation `src/utils.ts#formatDate`, expires 2026-10-20 (expired), by lead;
/// - inline in `c.py`, expires 2026-11-20 (26 days), by arpit, with a reason.
fn project(name: &str) -> PathBuf {
    let d = temp(name);
    fs::create_dir_all(d.join(".codelaw/baseline")).unwrap();
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("a.py"), "print(1)\nprint(2)\n").unwrap();
    fs::write(d.join("b.py"), "print(3)\n").unwrap();
    fs::write(d.join("c.py"), "x = 1\n# codelaw-ignore fixture/py-no-print expires=2026-11-20 by=Arpit@example.com: needed, ORD-1600 | pipe\nprint(4)\n").unwrap();
    fs::write(d.join("src/utils.ts"), "export function formatDate(d: Date) {\n  return d.toISOString();\n}\n").unwrap();
    fs::write(d.join("src/list.ts"), "import { formatDate } from \"./utils\";\n\nexport const a = formatDate(new Date());\n").unwrap();
    fs::write(
        d.join(".codelaw/baseline/fixture--py-no-print.toml"),
        "[[allow]]\nfile = \"a.py\"\ncount = 2\nexpires = \"2026-11-02\"\nadded_by = \"arpit@example.com\"\nreason = \"debug panel\"\n\n[[allow]]\nfile = \"b.py\"\ncount = 1\n",
    )
    .unwrap();
    fs::write(
        d.join(".codelaw/deprecations.toml"),
        "[[deprecated]]\nsymbol = \"src/utils.ts#formatDate\"\nreplacement = \"src/utils.ts#formatISO\"\nexpires = \"2026-10-20\"\nadded_by = \"lead@example.com\"\n",
    )
    .unwrap();
    // The deprecation's one grandfathered use.
    fs::write(d.join(".codelaw/baseline/graph--no-deprecated-usage.toml"), "[[allow]]\nfile = \"src/list.ts\"\nsymbol = \"src/utils.ts#formatDate\"\ncount = 1\n").unwrap();
    d
}

fn json(d: &Path, args: &[&str]) -> serde_json::Value {
    let (code, out, err) = codelaw(d, args);
    assert_eq!(code, 0, "{out}{err}");
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"))
}

const DAY: &str = "2026-10-25";

fn rules_and_files(v: &serde_json::Value) -> Vec<String> {
    v["exceptions"].as_array().unwrap().iter().map(|e| format!("{}:{}:{}", e["kind"].as_str().unwrap(), e["rule"].as_str().unwrap(), e["files"][0].as_str().unwrap_or("-"))).collect()
}

#[test]
fn every_exception_form_appears_once_with_its_fields() {
    let d = project("forms");
    let v = json(&d, &["exceptions", "--as-of", DAY, "--format", "json"]);
    // Sorted by `expires`: the expired deprecation first, the undated last.
    assert_eq!(
        rules_and_files(&v),
        [
            "deprecation:graph/no-deprecated-usage:src/list.ts",
            "baseline:fixture/py-no-print:a.py",
            "inline:fixture/py-no-print:c.py",
            "baseline:fixture/py-no-print:b.py",
        ]
    );
    let e = &v["exceptions"];
    assert_eq!((e[0]["days_left"].as_i64(), e[0]["uses"].as_u64(), e[0]["added_by"].as_str()), (Some(-5), Some(1), Some("lead@example.com")));
    assert_eq!((e[1]["days_left"].as_i64(), e[1]["uses"].as_u64(), e[1]["reason"].as_str()), (Some(8), Some(2), Some("debug panel")));
    assert_eq!((e[2]["days_left"].as_i64(), e[2]["line"].as_u64(), e[2]["reason"].as_str()), (Some(26), Some(2), Some("needed, ORD-1600 | pipe")));
    assert!(e[3]["expires"].is_null() && e[3]["days_left"].is_null() && e[3]["added_by"].is_null());
    assert_eq!(v["total"]["exceptions"], 4);
    assert_eq!(v["as_of"]["date"], DAY);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn expiring_and_group_by_combine() {
    let d = project("combine");
    let (code, out, err) = codelaw(&d, &["exceptions", "--as-of", DAY, "--expiring", "30d", "--group-by", "added_by"]);
    assert_eq!(code, 0, "{err}");
    // The expired deprecation is not "expiring"; the undated one has no date.
    assert_eq!(
        out,
        "as-of 2026-10-25 (--as-of)\n\
         Arpit@example.com  1 exception, 1 use\n  2026-11-20  in 26 days  fixture/py-no-print  c.py:2    needed, ORD-1600 | pipe\n\
         arpit@example.com  1 exception, 2 uses\n  2026-11-02  in 8 days   fixture/py-no-print  a.py (2)  debug panel\n"
    );
    // A tighter window drops the 26-day one.
    let v = json(&d, &["exceptions", "--as-of", DAY, "--expiring", "10d", "--format", "json"]);
    assert_eq!(rules_and_files(&v), ["baseline:fixture/py-no-print:a.py"]);
    // `--expired` is the other side; both together are the union.
    let v = json(&d, &["exceptions", "--as-of", DAY, "--expired", "--format", "json"]);
    assert_eq!(rules_and_files(&v), ["deprecation:graph/no-deprecated-usage:src/list.ts"]);
    let v = json(&d, &["exceptions", "--as-of", DAY, "--expired", "--expiring", "10d", "--format", "json"]);
    assert_eq!(v["total"]["exceptions"], 2);
    // The three other groupings, with the missing-key bucket last.
    let v = json(&d, &["exceptions", "--as-of", DAY, "--group-by", "added_by", "--format", "json"]);
    let keys: Vec<&str> = v["groups"].as_array().unwrap().iter().map(|g| g["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["Arpit@example.com", "arpit@example.com", "lead@example.com", "(no added_by)"]);
    let v = json(&d, &["exceptions", "--as-of", DAY, "--group-by", "expires-month", "--format", "json"]);
    let keys: Vec<&str> = v["groups"].as_array().unwrap().iter().map(|g| g["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["2026-10", "2026-11", "(no expiry)"]);
    assert_eq!(v["groups"][1]["exceptions"], 2);
    let v = json(&d, &["exceptions", "--as-of", DAY, "--group-by", "rule", "--format", "json"]);
    assert_eq!(v["groups"][0]["key"], "fixture/py-no-print");
    assert_eq!(v["groups"][0]["uses"], 4);
    let v = json(&d, &["exceptions", "--as-of", DAY, "--group-by", "file", "--format", "json"]);
    let keys: Vec<&str> = v["groups"].as_array().unwrap().iter().map(|g| g["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["a.py", "b.py", "c.py", "src/list.ts"]);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn rule_and_path_filter() {
    let d = project("filters");
    let v = json(&d, &["exceptions", "--as-of", DAY, "--rule", "graph/no-deprecated-usage", "--format", "json"]);
    assert_eq!(rules_and_files(&v), ["deprecation:graph/no-deprecated-usage:src/list.ts"]);
    let v = json(&d, &["exceptions", "--as-of", DAY, "--path", "src", "--format", "json"]);
    assert_eq!(v["total"]["exceptions"], 1);
    let v = json(&d, &["exceptions", "--as-of", DAY, "--path", "a.py", "--format", "json"]);
    assert_eq!(rules_and_files(&v), ["baseline:fixture/py-no-print:a.py"]);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn as_of_changes_the_days_left_and_nothing_else() {
    let d = project("as-of");
    let strip = |mut v: serde_json::Value| {
        for e in v["exceptions"].as_array_mut().unwrap() {
            e["days_left"] = serde_json::Value::Null;
        }
        v["as_of"] = serde_json::Value::Null;
        v
    };
    let a = json(&d, &["exceptions", "--as-of", "2026-10-25", "--format", "json"]);
    let b = json(&d, &["exceptions", "--as-of", "2026-10-30", "--format", "json"]);
    assert_ne!(a["exceptions"][1]["days_left"], b["exceptions"][1]["days_left"]);
    assert_eq!(b["exceptions"][1]["days_left"], 3);
    assert_eq!(strip(a), strip(b));
    // Without a date (not a git repo, no flag) days are blank and a window is an error.
    let (code, out, err) = codelaw(&d, &["exceptions", "--format", "json"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\"days_left\": null"), "{out}");
    let (code, _, err) = codelaw(&d, &["exceptions", "--expiring", "30d"]);
    assert_eq!(code, 2);
    assert!(err.contains("need an as-of date"), "{err}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn csv_and_markdown_render() {
    let d = project("render");
    let (code, out, _) = codelaw(&d, &["exceptions", "--as-of", DAY, "--expiring", "30d", "--format", "csv"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "kind,rule,file,symbol,expires,days_left,added_by,reason,uses,at,line");
    assert_eq!(lines[1], "baseline,fixture/py-no-print,a.py,,2026-11-02,8,arpit@example.com,debug panel,2,.codelaw/baseline/fixture--py-no-print.toml,1");
    // A comma in a reason is quoted.
    assert_eq!(lines[2], "inline,fixture/py-no-print,c.py,,2026-11-20,26,Arpit@example.com,\"needed, ORD-1600 | pipe\",1,c.py,2");
    let (_, out, _) = codelaw(&d, &["exceptions", "--as-of", DAY, "--expiring", "30d", "--group-by", "rule", "--format", "csv"]);
    assert!(out.starts_with("group,kind,rule,"), "{out}");
    let (_, out, _) = codelaw(&d, &["exceptions", "--as-of", DAY, "--expiring", "30d", "--format", "markdown"]);
    assert!(out.contains("| expires | when | rule | where | uses | added by | reason |\n|---|---|---|---|--:|---|---|\n"), "{out}");
    assert!(out.contains("| 2026-11-02 | in 8 days | `fixture/py-no-print` | `a.py` | 2 | arpit@example.com | debug panel |"), "{out}");
    // A `|` in a reason must not break the table.
    assert!(out.contains("needed, ORD-1600 \\| pipe |"), "{out}");
    let (_, out, _) = codelaw(&d, &["exceptions", "--as-of", DAY, "--expiring", "30d", "--group-by", "added_by", "--format", "markdown"]);
    assert!(out.contains("### arpit@example.com: 1 exception, 2 uses\n"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn fail_if_expiring_is_the_only_nonzero_exit() {
    let d = project("fail");
    // Exit 0 always, even with an expired one in the list.
    assert_eq!(codelaw(&d, &["exceptions", "--as-of", DAY]).0, 0);
    // The deprecation expired 5 days ago: within any window.
    assert_eq!(codelaw(&d, &["exceptions", "--as-of", DAY, "--fail-if-expiring", "7d"]).0, 1);
    // Filtered to the 8-day and 26-day ones: 7d passes, 10d fails.
    let base = ["exceptions", "--as-of", DAY, "--expiring", "30d"];
    let with = |n: &'static str| [&base[..], &["--fail-if-expiring", n]].concat();
    assert_eq!(codelaw(&d, &with("7d")).0, 0);
    assert_eq!(codelaw(&d, &with("1w")).0, 0);
    assert_eq!(codelaw(&d, &with("10d")).0, 1);
    assert_eq!(codelaw(&d, &["exceptions", "--fail-if-expiring", "soon"]).0, 2);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn mine_is_the_git_user() {
    if !Command::new("git").arg("--version").output().is_ok_and(|o| o.status.success()) {
        return;
    }
    let d = project("mine");
    let (code, _, err) = codelaw(&d, &["exceptions", "--mine"]);
    assert_eq!(code, 2);
    assert!(err.contains("user.email"), "{err}");
    let git = |args: &[&str]| {
        let out = Command::new("git").args(args).current_dir(&d).env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1").output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "ARPIT@example.com"]);
    let v = json(&d, &["exceptions", "--as-of", DAY, "--mine", "--format", "json"]);
    // Case-insensitive: both the `arpit@` and the `Arpit@` ones.
    assert_eq!(rules_and_files(&v), ["baseline:fixture/py-no-print:a.py", "inline:fixture/py-no-print:c.py"]);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn text_header_and_empty() {
    let d = temp("empty");
    let (code, out, _) = codelaw(&d, &["exceptions", "--as-of", DAY]);
    assert_eq!((code, out.as_str()), (0, "as-of 2026-10-25 (--as-of)\nno exceptions\n"));
    let _ = fs::remove_dir_all(&d);
}
