//! W-34, L6 decision 12: a rule pack's optional Fux index package is never
//! installed by `codelaw init`, with or without Fux in the repo; `init` only
//! prints how to add it, and only when the repo has a `.fux/` folder.
//!
//! The install is run against a stand-in `npm` that records its arguments, so
//! what `init` would install is what the test reads.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A bare JavaScript repo, with `.fux/` when `fux` is set.
fn repo(name: &str, fux: bool) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-fux-index-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("src")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    fs::write(d.join("package.json"), "{\"name\": \"app\", \"private\": true}\n").unwrap();
    fs::write(d.join("src/app.js"), "export const answer = 42;\n").unwrap();
    if fux {
        fs::create_dir_all(d.join(".fux")).unwrap();
    }
    d
}

/// A directory holding an `npm` that appends its arguments to `npm.log`.
fn fake_npm(d: &Path) -> PathBuf {
    let bin = d.join("fake-bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("npm");
    fs::write(&script, format!("#!/bin/sh\necho \"$@\" >> '{}'\n", d.join("npm.log").display())).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

/// `codelaw init` (the given extra args) with the stand-in `npm` first on PATH:
/// the output, and every line the stand-in recorded.
fn init(d: &Path, extra: &[&str]) -> (String, Vec<String>) {
    let bin = fake_npm(d);
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap_or_default());
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .arg("init")
        .args(extra)
        .current_dir(d)
        .env("PATH", path)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .output()
        .unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "init failed: {text}");
    let log = fs::read_to_string(d.join("npm.log")).unwrap_or_default();
    (text, log.lines().map(str::to_owned).collect())
}

#[test]
fn init_installs_no_fux_index_package_even_where_fux_is_in_use() {
    for fux in [false, true] {
        let d = repo(&format!("install-{fux}"), fux);
        let (out, ran) = init(&d, &[]);
        assert!(ran.iter().any(|l| l.contains("@codelaw/rules-js")), "init installed no pack: {ran:?}\n{out}");
        assert!(ran.iter().all(|l| !l.contains("fux-index")), "init installed an index package (fux={fux}): {ran:?}");
        let config = fs::read_to_string(d.join("codelaw.toml")).unwrap();
        assert!(!config.contains("fux"), "codelaw.toml names Fux: {config}");
        let _ = fs::remove_dir_all(&d);
    }
}

#[test]
fn init_prints_the_index_package_lines_only_where_fux_is_in_use() {
    let without = repo("print-without", false);
    let (out, _) = init(&without, &["--no-install"]);
    assert!(!out.to_lowercase().contains("fux"), "init mentioned Fux in a repo without .fux/: {out}");
    let _ = fs::remove_dir_all(&without);

    let with = repo("print-with", true);
    let (out, ran) = init(&with, &["--no-install"]);
    assert!(ran.is_empty(), "--no-install ran npm: {ran:?}");
    // the packs it chose are listed first, and each has its index package after
    let packs = out.find("npm install -D @codelaw/rules-data @codelaw/rules-js\n").expect(&out);
    let index = out.find("npm install -D @codelaw/rules-data-fux-index @codelaw/rules-js-fux-index\n").expect(&out);
    assert!(packs < index, "{out}");
    assert!(out.contains("fux setup --index"), "{out}");
    let _ = fs::remove_dir_all(&with);
}
