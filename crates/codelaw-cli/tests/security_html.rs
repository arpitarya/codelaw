//! The `security-html` pack through the binary (W-56): its preset turns on
//! the html pack's five security rules beside its own, the findings name
//! their CWE, `--tag` and the OWASP matrix find the rules, and `fix` applies
//! what is safe.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-sechtml-{name}-{}", std::process::id()));
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

const PAGE: &str = "<!doctype html>\n<html lang=\"en\">\n<head>\n<title>Sign in</title>\n<base href=\"https://cdn.example.com/\">\n<meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'self' 'unsafe-inline'\">\n</head>\n<body>\n<button type=\"button\" onclick=\"go()\">Go</button>\n<a href=\"javascript:void(0)\">Menu</a>\n<img src=\"http://a.example.com/p.png\" alt=\"\">\n<iframe src=\"/widget\"></iframe>\n<form action=\"/login\" method=\"get\"><input type=\"password\" name=\"pw\"></form>\n<!-- password: hunter2 -->\n</body>\n</html>\n";

fn project(name: &str, preset: &str) -> PathBuf {
    let d = temp(name);
    for p in ["html", "security-html"] {
        copy_dir(&repo().join("packs").join(p), &d.join("node_modules/@codelaw").join(format!("rules-{p}")));
    }
    fs::write(d.join("codelaw.toml"), format!("extends = [\"codelaw:{preset}\"]\npreview = true\n")).unwrap();
    fs::write(d.join("page.html"), PAGE).unwrap();
    d
}

/// DoD 2: the preset enables the five existing `html/*` rules, and the
/// findings carry the CWE.
#[test]
fn recommended_runs_its_rules_and_the_html_five() {
    let d = project("rec", "sec-html/recommended");
    let (_, out, err) = codelaw(&d, &["rules", "--for", "page.html"]);
    let out = format!("{out}{err}");
    for id in ["html/autocomplete-password", "html/meta-refresh", "html/no-inline-script", "html/require-sri", "html/target-blank-rel"] {
        assert!(out.contains(id), "{id}: {out}");
    }
    let (code, out, err) = codelaw(&d, &["check", "page.html"]);
    assert_eq!(code, 1, "{out}{err}");
    for want in [
        "sec-html/no-inline-event-handler (CWE-79)",
        "sec-html/no-javascript-url (CWE-79)",
        "sec-html/no-mixed-content (CWE-319)",
        "sec-html/iframe-sandbox (CWE-1021)",
        "sec-html/password-form-post (CWE-598)",
        "sec-html/csp-meta-unsafe (CWE-693)",
    ] {
        assert!(out.contains(want), "{want}: {out}");
    }
    // The heuristic rules wait for `strict`.
    assert!(!out.contains("sec-html/no-external-base") && !out.contains("sec-html/no-sensitive-comments"), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--tag", "cwe:79"]);
    for id in ["sec-html/no-inline-event-handler", "sec-html/no-javascript-url", "sec-html/no-template-html-sink"] {
        assert!(out.contains(id), "{id}: {out}");
    }
    let (code, out, err) = codelaw(&d, &["rules", "--security-coverage"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("sec-html/no-javascript-url (CWE-79)"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

/// `strict` adds the heuristic rules at `warn`; the comment rule is `auto`.
#[test]
fn strict_adds_the_heuristic_rules_and_fixes_the_comment() {
    let d = project("strict", "sec-html/strict");
    let (_, out, _) = codelaw(&d, &["check", "page.html"]);
    assert!(out.contains("sec-html/no-external-base") && out.contains("sec-html/no-sensitive-comments"), "{out}");
    let (_, out, err) = codelaw(&d, &["fix", "page.html"]);
    let after = fs::read_to_string(d.join("page.html")).unwrap();
    assert!(!after.contains("hunter2"), "{out}{err}\n{after}");
    // `suggest` fixes wait for --unsafe.
    assert!(after.contains("method=\"get\"") && after.contains("http://a.example.com"), "{after}");
    let _ = fs::remove_dir_all(&d);
}

/// The `suggest` fixes apply with `--unsafe`. (Under `strict` the password
/// form's fix is refused: it would make a POST form with no CSRF token, a
/// finding the file did not have, and a fix may not add one.)
#[test]
fn suggest_fixes_apply_with_unsafe() {
    let d = project("unsafe", "sec-html/recommended");
    let _ = codelaw(&d, &["fix", "--unsafe", "page.html"]);
    let after = fs::read_to_string(d.join("page.html")).unwrap();
    assert!(after.contains("method=\"post\"") && after.contains("https://a.example.com/p.png") && after.contains("<iframe sandbox src"), "{after}");
    let _ = fs::remove_dir_all(&d);
}
