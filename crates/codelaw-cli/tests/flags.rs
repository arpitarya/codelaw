//! W-130: feature-flag expiry (SR-PROJECT-GRAPH decision 11) through the
//! real binary, on `fixtures/flags`: one read per SDK preset in JS/TS and in
//! Python, a wrapper of a wrapper in each, an inline marker and the
//! registry, judged on a fixed as-of date; then `fix --unsafe` and
//! `flags accessors`.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RULES: &str = "js/flag-expiry,py/flag-expiry,js/flag-marker,py/flag-marker,data/flag-registry";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

/// The fixture, with the `js` and `data` packs where `codelaw:` finds them
/// and the `python` pack in a virtualenv (PyPI only, L6 decision 11).
fn repo(name: &str) -> PathBuf {
    let root = repo_root();
    let dir = std::env::temp_dir().join(format!("codelaw-flags-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    copy_dir(&root.join("fixtures/flags"), &dir);
    fs::create_dir_all(dir.join("node_modules/@codelaw")).unwrap();
    for pack in ["js", "data"] {
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("packs").join(pack), dir.join(format!("node_modules/@codelaw/rules-{pack}"))).unwrap();
    }
    copy_dir(&root.join("packs/python"), &dir.join(".venv/lib/python3.12/site-packages/codelaw_rules_python"));
    dir
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_AS_OF")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// `file:line rule severity` of every flag finding, sorted.
fn findings(cwd: &Path, as_of: &str) -> Vec<String> {
    let (_, out) = codelaw(cwd, &["check", "--format", "json", "--no-cache", "--as-of", as_of, "--rule", RULES]);
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    let mut got: Vec<String> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| format!("{}:{} {} {}", d["file"].as_str().unwrap(), d["range"]["start"]["line"], d["rule"].as_str().unwrap(), d["severity"].as_str().unwrap()))
        .collect();
    got.sort();
    got
}

#[test]
fn every_preset_and_a_two_level_wrapper_on_the_as_of_date() {
    let d = repo("check");
    assert_eq!(
        findings(&d, "2026-10-30"),
        vec![
            ".codelaw/flags.toml:62 data/flag-registry info",       // `retired`: no read left
            "app/main.py:5 py/flag-expiry error",                    // through two wrappers, expired, keep = "off"
            "app/presets.py:3 py/flag-expiry error",                 // launchdarkly, expired
            "app/presets.py:5 py/flag-expiry warn",                  // split, in 2 days
            "app/presets.py:7 py/flag-expiry error",                 // growthbook, unregistered
            "src/app.ts:4 js/flag-expiry error",                     // through two wrappers, expired, keep = "on"
            "src/flagsmith.js:4 js/flag-expiry warn",                // flagsmith, its inline marker: in 11 days
            "src/growthbook.jsx:1 js/flag-expiry error",             // growthbook, unregistered
            "src/launchdarkly.ts:3 js/flag-expiry error",            // launchdarkly, expired
            "src/unleash.tsx:1 js/flag-expiry warn",                 // unleash, in 6 days
        ]
    );
    // A year earlier only the unregistered reads, the decided ones and the
    // unread entry are left; nothing depends on the clock.
    assert_eq!(
        findings(&d, "2025-10-30"),
        vec![
            ".codelaw/flags.toml:62 data/flag-registry info",
            "app/main.py:5 py/flag-expiry warn",
            "app/presets.py:7 py/flag-expiry error",
            "src/app.ts:4 js/flag-expiry warn",
            "src/growthbook.jsx:1 js/flag-expiry error",
        ]
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn fix_keeps_the_decided_branch_and_drops_the_unread_entry() {
    let d = repo("fix");
    let (_, out) = codelaw(&d, &["fix", "--unsafe", "--no-cache", "--as-of", "2026-10-30", "--rule", RULES]);
    assert_eq!(
        fs::read_to_string(d.join("src/app.ts")).unwrap(),
        "import { enabled } from \"./lib/feature\";\n\nexport function render(): string {\n  return \"new\";\n}\n",
        "{out}"
    );
    assert_eq!(fs::read_to_string(d.join("app/main.py")).unwrap(), "from app.feature import enabled\n\n\ndef render():\n    return \"old\"\n", "{out}");
    // The collapse left `wrapped` and `py-wrapped` with no read, so their
    // entries go too, with the one no read named: the cleanup is one run.
    let registry = fs::read_to_string(d.join(".codelaw/flags.toml")).unwrap();
    for gone in ["retired", "\"wrapped\"", "py-wrapped"] {
        assert!(!registry.contains(gone), "{gone}: {registry}");
    }
    assert!(registry.ends_with("name = \"fs-py\"\nowner = \"@api\"\nexpires = \"2027-01-01\"\n"), "{registry}");
    // Without --unsafe, only the auto fix: the entry.
    let d2 = repo("fix-safe");
    codelaw(&d2, &["fix", "--no-cache", "--as-of", "2026-10-30", "--rule", RULES]);
    assert!(fs::read_to_string(d2.join("src/app.ts")).unwrap().contains("enabled(\"wrapped\")"));
    assert!(!fs::read_to_string(d2.join(".codelaw/flags.toml")).unwrap().contains("retired"));
    let _ = fs::remove_dir_all(&d);
    let _ = fs::remove_dir_all(&d2);
}

#[test]
fn accessors_lists_presets_config_and_wrappers() {
    let d = repo("accessors");
    let (code, out) = codelaw(&d, &["flags", "accessors", "--format", "json"]);
    assert_eq!(code, 0, "{out}");
    let rows: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    let rows = rows.as_array().unwrap();
    let wrappers: Vec<String> = rows
        .iter()
        .filter(|r| r["file"].is_string())
        .map(|r| format!("{} {} {}", r["pattern"].as_str().unwrap(), r["file"].as_str().unwrap(), r["origin"].as_str().unwrap()))
        .collect();
    assert_eq!(
        wrappers,
        vec![
            "enabled($NAME) app/feature.py wrapper of `is_on($NAME)`",
            "is_on($NAME) app/flags.py wrapper of `$_.variation($NAME, $$$)`",
            "enabled($NAME) src/lib/feature.ts wrapper of `isOn($NAME)`",
            "isOn($NAME) src/lib/flags.ts wrapper of `$_.variation($NAME, $$$)`",
        ]
    );
    for p in ["launchdarkly", "unleash", "split", "openfeature", "growthbook", "flagsmith"] {
        assert!(rows.iter().any(|r| r["origin"] == format!("preset {p}")), "no preset {p}: {out}");
    }
    // A config that cannot be used is a config error, not a silent nothing.
    fs::write(d.join("codelaw.toml"), "[flags]\naccessors = [\"isOn($_)\"]\n").unwrap();
    let (code, out) = codelaw(&d, &["flags", "accessors"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("needs exactly one `$NAME`"), "{out}");
    let _ = fs::remove_dir_all(&d);
}
