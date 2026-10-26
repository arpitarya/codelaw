//! The dev-server plugins (W-49; SR-PACKAGES decision 3a): `packaging/npm`'s
//! Vite, webpack/Rspack and Next.js plugins against this binary. They only talk
//! to `codelaw check`, so the test runs their `node --test` suite with
//! `CODELAW_BINARY` set; a host without `node` skips it and says so. The three
//! plugin folders' shared sources must be the copies `package.py sync-dev-core`
//! writes.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

#[test]
fn the_plugins_report_and_fail_builds_through_the_binary() {
    if !Command::new("node").arg("--version").output().is_ok_and(|o| o.status.success()) {
        eprintln!("skipped: no node on this host");
        return;
    }
    let out = Command::new("node")
        .args(["--test", "--test-timeout=60000", "packaging/npm/dev-core/test/plugins.test.js"])
        .current_dir(repo())
        .env("CODELAW_BINARY", env!("CARGO_BIN_EXE_codelaw"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
}

#[test]
fn each_plugin_carries_the_shared_sources_unchanged() {
    let npm = repo().join("packaging/npm");
    for (dst, src) in [
        ("vite-plugin/core.js", "dev-core/core.js"),
        ("webpack-plugin/core.js", "dev-core/core.js"),
        ("next/core.js", "dev-core/core.js"),
        ("next/plugin.js", "webpack-plugin/index.js"),
    ] {
        assert_eq!(std::fs::read(npm.join(dst)).unwrap(), std::fs::read(npm.join(src)).unwrap(), "{dst} differs from {src}: run `python3 scripts/package.py sync-dev-core`");
    }
}
