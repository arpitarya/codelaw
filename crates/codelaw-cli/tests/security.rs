//! The security packs' groundwork through the binary (W-53): presets that
//! extend another pack's, `security:` metadata in `rules --tag`, the
//! OWASP coverage matrix, the CWE in text output, SARIF's tags and
//! `security-severity`, and `from_source:` on a real check.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-sec-{name}-{}", std::process::id()));
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

fn install(d: &Path, short: &str) {
    copy_dir(&repo().join("packs").join(short), &d.join("node_modules/@codelaw").join(format!("rules-{short}")));
}

/// A team pack with one security rule, the way W-54–W-57's rules are written.
fn team_pack(dir: &Path, name: &str, id: &str, lang: (&str, &str, &str), block: &str, dont: &str, doo: &str) {
    let (ns, rule) = id.split_once('/').unwrap();
    fs::create_dir_all(dir.join("rules").join(ns)).unwrap();
    fs::write(dir.join("codelaw-pack.toml"), format!("name = \"{name}\"\nversion = \"1.0.0\"\nmin_codelaw = \"0.1.0\"\nlanguages = [\"{}\"]\n", lang.0)).unwrap();
    fs::write(
        dir.join("rules").join(ns).join(format!("{rule}.md")),
        format!(
            "---\nid: {id}\ntitle: request data reaches an HTML sink\nlanguages: [{}]\nfix: manual\nfix_reason: the safe API depends on what the value is\nstability: stable\nsecurity: {{ cwe: [79], owasp: [\"A05:2025\"], severity: high }}\n---\n\nRequest data is not written into HTML unescaped.\n\n**Why:** it is cross-site scripting. `from_source:` is not taint analysis.\n\n```codelaw\n{block}\n```\n\n## How to fix\n\n1. Escape it, or write it as text.\n\n## Examples\n\n### {}\n\n#### Don't\n\n```{}\n{dont}\n```\n\n#### Do\n\n```{}\n{doo}\n```\n",
            lang.0, lang.1, lang.2, lang.2
        ),
    )
    .unwrap();
}

/// The project of the tests that read what the first-party security packs
/// do; `teams_only` has just the two team packs, so a count of the rules a
/// CWE or an OWASP category holds is not moved by a first-party pack's rules.
fn project(name: &str) -> PathBuf {
    project_with(name, true)
}

fn teams_only(name: &str) -> PathBuf {
    project_with(name, false)
}

fn project_with(name: &str, first_party: bool) -> PathBuf {
    let d = temp(name);
    if first_party {
        for p in ["html", "security-html", "security-js", "security-ts"] {
            install(&d, p);
        }
    }
    team_pack(
        &d.join("team-js"),
        "@acme/security-js",
        "acme/no-url-to-inner-html",
        ("javascript", "JavaScript", "js"),
        "match:\n  pattern: $EL.innerHTML = $V\nwhere:\n  $V: { from_source: [location.hash, location.search] }\nmessage: \"`$V` comes from the URL and is written as HTML\"",
        "function show() {\n  const h = location.hash;\n  document.body.innerHTML = h;\n}",
        "function show() {\n  document.body.textContent = location.hash;\n}",
    );
    team_pack(
        &d.join("team-py"),
        "acme-security-py",
        "acme-py/no-request-mark-safe",
        ("python", "Python", "py"),
        "match:\n  pattern: mark_safe($V)\nwhere:\n  $V: { from_source: [request.GET, request.POST] }\nmessage: \"`$V` comes from the request and is marked safe\"",
        "def view(request):\n    name = request.GET[\"n\"]\n    return mark_safe(name)",
        "def view(request):\n    return escape(request.GET[\"n\"])",
    );
    fs::write(
        d.join("codelaw.toml"),
        if first_party {
            "extends = [\"codelaw:sec-html/recommended\", \"codelaw:sec-ts/strict\", \"path:team-js\", \"path:team-py\"]\npreview = true\n"
        } else {
            "extends = [\"path:team-js\", \"path:team-py\"]\npreview = true\n"
        },
    )
    .unwrap();
    fs::write(d.join("a.js"), "function show() {\n  const h = location.hash;\n  document.body.innerHTML = h;\n}\nfunction safe(h) {\n  document.body.innerHTML = h;\n}\n").unwrap();
    fs::write(d.join("v.py"), "def view(request):\n    name = request.GET[\"n\"]\n    return mark_safe(name)\n").unwrap();
    fs::write(d.join("page.html"), "<!doctype html>\n<html lang=\"en\">\n<head><title>x</title></head>\n<body>\n<button type=\"button\" onclick=\"go()\">Go</button>\n</body>\n</html>\n").unwrap();
    d
}

/// DoD 1: `rules --tag cwe:79` lists the rules of every pack that maps to
/// it; `owasp:` matches a category; a tag nothing has lists nothing.
#[test]
fn rules_by_tag_across_packs() {
    let d = teams_only("tag");
    let (code, out, err) = codelaw(&d, &["rules", "--tag", "cwe:79"]);
    assert_eq!(code, 0, "{out}{err}");
    let ids: Vec<&str> = out.lines().map(|l| l.split_whitespace().next().unwrap()).collect();
    assert_eq!(ids, ["acme-py/no-request-mark-safe", "acme/no-url-to-inner-html"], "{out}");
    assert!(out.contains("@acme/security-js@1.0.0") && out.contains("acme-security-py@1.0.0") && out.contains("(CWE-79)"), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--tag", "owasp:A05", "--tag", "security", "--format", "json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    let rules = v["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 2, "{out}");
    assert_eq!(rules[0]["security"]["cwe"], serde_json::json!([79]));
    assert_eq!(rules[0]["security"]["security_severity"], "8.0");
    let (code, out, _) = codelaw(&d, &["rules", "--tag", "cwe:78"]);
    assert_eq!((code, out.as_str()), (0, ""));
    let _ = fs::remove_dir_all(&d);
}

/// `rules --security-coverage`: the OWASP Top 10:2025 × rule matrix.
#[test]
fn security_coverage_matrix() {
    let d = teams_only("coverage");
    let (code, out, err) = codelaw(&d, &["rules", "--security-coverage"]);
    assert_eq!(code, 0, "{out}{err}");
    let injection = out.lines().find(|l| l.starts_with("A05:2025")).unwrap();
    assert!(injection.contains("Injection") && injection.contains(" 2  ") && injection.contains("acme/no-url-to-inner-html (CWE-79)"), "{out}");
    assert!(out.lines().find(|l| l.starts_with("A01:2025")).unwrap().ends_with('—'), "{out}");
    assert!(out.contains("1 of 10 categories covered by 2 security rule(s)"), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--security-coverage", "--format", "json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["owasp"].as_array().unwrap().len(), 10);
    assert_eq!(v["owasp"][4]["id"], "A05:2025");
    assert_eq!(v["owasp"][4]["rules"].as_array().unwrap().len(), 2);
    let _ = fs::remove_dir_all(&d);
}

/// A preset that extends another pack's: `sec-html/recommended` turns on
/// the html pack's security rules without copying them, `sec-ts/strict`
/// pulls in `sec-js`'s presets, and `rules --why` says which preset did it.
#[test]
fn presets_extend_other_packs() {
    let d = project("extends");
    let (code, out, err) = codelaw(&d, &["rules", "--why", "html/no-inline-script"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("codelaw:html/security (extended by codelaw:sec-html/recommended)"), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--for", "page.html"]);
    for id in ["html/autocomplete-password", "html/meta-refresh", "html/no-inline-script", "html/require-sri", "html/target-blank-rel"] {
        assert!(out.contains(id), "{id}: {out}");
    }
    // Only html/security's five: html/recommended is not extended.
    assert!(!out.contains("html/img-alt"), "{out}");
    let (code, out, err) = codelaw(&d, &["check", "page.html"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("html/no-inline-script"), "{out}");

    // A preset whose extended pack is not installed stops the run, naming
    // the pack and the preset that needs it.
    let e = temp("extends-missing");
    install(&e, "security-ts");
    fs::write(e.join("codelaw.toml"), "extends = [\"codelaw:sec-ts/recommended\"]\n").unwrap();
    fs::write(e.join("a.ts"), "let a = 1;\n").unwrap();
    let (code, out, err) = codelaw(&e, &["check"]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(err.contains("@codelaw/rules-security-js") && err.contains("extended by codelaw:sec-ts/recommended"), "{err}");
    let _ = fs::remove_dir_all(&d);
    let _ = fs::remove_dir_all(&e);
}

/// The finding names the CWE in text and agent output; `from_source:` finds
/// the value one assignment away in the same function and not the
/// parameter of another.
#[test]
fn findings_name_the_cwe() {
    let d = project("text");
    let (code, out, err) = codelaw(&d, &["check", "a.js", "v.py"]);
    assert_eq!(code, 1, "{out}{err}");
    assert!(out.contains("acme/no-url-to-inner-html (CWE-79) [manual]"), "{out}");
    assert!(out.contains("acme-py/no-request-mark-safe (CWE-79) [manual]"), "{out}");
    assert_eq!(out.matches("acme/no-url-to-inner-html").count(), 1, "the parameter in `safe` is not a source: {out}");
    let (_, out, _) = codelaw(&d, &["check", "a.js", "--format", "agent"]);
    assert!(out.starts_with("a.js:3:3 acme/no-url-to-inner-html (CWE-79) `h` comes from the URL"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

/// DoD 2: SARIF from a fixture carries `security-severity` and the CWE tags,
/// and is valid SARIF 2.1.0.
#[test]
fn sarif_carries_security_severity_and_cwe_tags() {
    let d = project("sarif");
    let (code, out, err) = codelaw(&d, &["check", "--format", "sarif"]);
    assert_eq!(code, 1, "{err}");
    let log: Value = serde_json::from_str(&out).unwrap();
    let schema: Value = serde_json::from_str(&fs::read_to_string(repo().join("crates/codelaw-report/tests/sarif-schema-2.1.0.json")).unwrap()).unwrap();
    let mut schemas = boon::Schemas::new();
    let mut c = boon::Compiler::new();
    c.add_resource("file:///sarif-schema-2.1.0.json", schema).unwrap();
    let idx = c.compile("file:///sarif-schema-2.1.0.json", &mut schemas).unwrap();
    if let Err(e) = schemas.validate(&log, idx) {
        panic!("not valid SARIF 2.1.0: {e:#}\n{out}");
    }
    let rules = log["runs"][0]["tool"]["driver"]["rules"].as_array().unwrap();
    let r = rules.iter().find(|r| r["id"] == "acme/no-url-to-inner-html").unwrap();
    assert_eq!(r["properties"]["security-severity"], "8.0");
    assert_eq!(r["properties"]["tags"], serde_json::json!(["security", "external/cwe/cwe-79", "external/owasp/A05:2025"]));
    // A rule without `security:` carries neither.
    let plain = rules.iter().find(|r| r["id"] == "html/no-inline-script").unwrap();
    assert!(plain["properties"].get("security-severity").is_none(), "{plain}");
    let _ = fs::remove_dir_all(&d);
}
