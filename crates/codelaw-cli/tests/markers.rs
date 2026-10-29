//! TODO markers and skipped tests through the binary (W-131): `[todo]` and
//! `[tests.skip]` reach the rules, dates are judged on `--as-of`, and a
//! wrong config value is a load error.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-markers-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_SITE_PACKAGES")
        .env_remove("CODELAW_AS_OF")
        .env("CODELAW_CACHE_DIR", cwd.join(".cache"))
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

fn project(name: &str, config: &str) -> PathBuf {
    let d = temp(name);
    copy_dir(&repo().join("packs/js"), &d.join("node_modules/@codelaw/rules-js"));
    fs::write(d.join("codelaw.toml"), format!("extends = [\"codelaw:js/markers\"]\npreview = true\n{config}")).unwrap();
    fs::write(
        d.join("a.js"),
        "// TODO: bare\n// TODO(ABC-1, 2026-10-01): overdue\n// TODO(ABC-2, 2026-11-05): soon\n// TODO(ABC-3, 2027-06-01): fine\nit.only(\"a\", f);\n// SKIP(ABC-4, 2026-10-01): old\nit.skip(\"b\", f);\nit.skip(\"c\", f);\n",
    )
    .unwrap();
    d
}

#[test]
fn the_preset_judges_dates_on_the_as_of_date() {
    let d = project("asof", "");
    let (code, out, err) = codelaw(&d, &["check", "--format", "agent", "--as-of", "2026-10-30", "a.js"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("a.js:1:4 js/todo-marker `TODO` needs a ticket matching"), "{out}");
    assert!(out.contains("a.js:2:4 js/todo-marker `TODO(ABC-1, 2026-10-01)` expired on 2026-10-01 (29 days ago)"), "{out}");
    assert!(out.contains("js/todo-marker `TODO(ABC-2, 2026-11-05)` expires in 6 days"), "{out}");
    assert!(!out.contains("ABC-3"), "{out}");
    assert!(out.contains("a.js:5:1 js/no-focused-tests"), "{out}");
    assert!(out.contains("js/skipped-test The skip of `it.skip` expired on 2026-10-01"), "{out}");
    assert!(out.contains("a.js:8:1 js/skipped-test `it.skip` needs a reason and an expiry date"), "{out}");
    // An earlier as-of date: the overdue one is not overdue yet.
    let (_, out, _) = codelaw(&d, &["check", "--format", "agent", "--as-of", "2026-09-01", "a.js"]);
    assert!(!out.contains("expired on"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn todo_and_tests_skip_tables_change_what_is_required() {
    let cfg = "[todo]\nrequire_ticket = false\nrequire_date = true\ntags = [\"TODO\"]\n[tests.skip]\nrequire_reason = false\nrequire_expiry = false\n";
    let d = project("config", cfg);
    let (_, out, err) = codelaw(&d, &["check", "--format", "agent", "--as-of", "2026-10-30", "a.js"]);
    assert!(out.contains("a.js:1:4 js/todo-marker `TODO` needs a date"), "{out}{err}");
    assert!(!out.contains("a ticket"), "{out}");
    assert!(!out.contains("needs a reason"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_bad_ticket_pattern_is_a_load_error() {
    let d = project("bad", "[todo]\nticket_pattern = \"[A-\"\n");
    let (code, out, err) = codelaw(&d, &["check", "a.js"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(err.contains("`[todo] ticket_pattern`"), "{err}");
    let d2 = project("badtags", "[tests.skip]\nwarn_days = -1\n");
    let (code, out, err) = codelaw(&d2, &["check", "a.js"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(err.contains("warn_days") || err.contains("invalid"), "{err}");
    let _ = fs::remove_dir_all(&d);
    let _ = fs::remove_dir_all(&d2);
}
