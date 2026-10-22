//! Finding, versioning and running an adapter (SR-ARCHITECTURE decision
//! 19). A stand-in program that prints a fixture's captured report runs
//! everywhere; the real tools run only where they are installed, and the
//! test says it skipped them otherwise.

use codelaw_adapters::{Ready, check_files, locate, manifest, manifests, resolve, version};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}

/// A fresh copy of a fixture's sources and tool config.
fn project(test: &str, name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adapters-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    copy_dir(&fixture(name), &d);
    d
}

/// The fixture's source file, project-relative.
fn source_of(name: &str) -> &'static str {
    match name {
        "oxlint" => "src/app.js",
        "ruff" => "src/mod.py",
        "stylelint" => "src/style.css",
        _ => "src/page.html",
    }
}

/// `node_modules/.bin/<binary>`: a shell script standing in for the tool.
#[cfg(unix)]
fn stand_in(root: &Path, binary: &str, script: &str) {
    use std::os::unix::fs::PermissionsExt;
    let bin = root.join("node_modules/.bin");
    std::fs::create_dir_all(&bin).unwrap();
    let file = bin.join(binary);
    std::fs::write(&file, format!("#!/bin/sh\n{script}\n")).unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A stand-in that reports `version` and prints the fixture's report with
/// `@ROOT@` replaced by the folder it runs in.
#[cfg(unix)]
fn replaying(root: &Path, name: &str, version: &str) {
    let m = manifest(name).unwrap();
    let dir = fixture(name);
    let (stream, file) = if name == "stylelint" { ("1>&2", dir.join("stderr.json")) } else { ("", dir.join("stdout.json")) };
    let script = format!(
        "case \"$1\" in --version) echo \"{name} {version}\"; exit 0;; esac\nsed \"s|@ROOT@|$(pwd -P)|g\" '{}' {stream}\nexit 1",
        file.display()
    );
    stand_in(root, &m.binary, &script);
}

#[cfg(unix)]
#[test]
fn a_stand_in_runs_and_its_report_lands_on_project_paths() {
    for m in manifests() {
        let root = project(&format!("replay-{}", m.name), &m.name);
        let start = m.tested.comparators[0].clone();
        let v = format!("{}.{}.{}", start.major, start.minor.unwrap_or(0), start.patch.unwrap_or(0));
        replaying(&root, &m.name, &v);
        let names = vec![m.name.clone()];
        let resolved = resolve(Some(&names), &root, None).unwrap();
        assert!(resolved.warnings.is_empty(), "{}: {:?}", m.name, resolved.warnings);
        let ready = &resolved.ready[0];
        assert_eq!(ready.source(), format!("{}@{v}", m.name));
        let rel = source_of(&m.name).to_owned();
        // A non-zero exit is how these tools say they found something.
        let report = check_files(ready, &root, std::slice::from_ref(&rel), Duration::from_secs(20)).unwrap();
        assert!(!report.findings[&rel].is_empty(), "{}: no findings mapped to {rel}", m.name);
    }
}

#[cfg(unix)]
#[test]
fn versions_outside_the_range_warn_and_missing_named_adapters_warn() {
    let root = project("range", "ruff");
    replaying(&root, "ruff", "0.1.0");
    let resolved = resolve(Some(&["ruff".to_owned()]), &root, None).unwrap();
    assert_eq!(resolved.ready.len(), 1, "it still runs");
    assert!(resolved.warnings[0].contains("outside its tested range"), "{:?}", resolved.warnings);
    // Named and not installed: a warning, not an error.
    let resolved = resolve(Some(&["html-validate".to_owned()]), &root, None).unwrap();
    assert!(resolved.ready.is_empty());
    assert!(resolved.warnings[0].contains("not installed"), "{:?}", resolved.warnings);
    // A default adapter that is not installed is skipped quietly.
    let empty = project("defaults", "html-validate");
    let resolved = resolve(None, &empty, None).unwrap();
    assert!(resolved.ready.is_empty() && resolved.warnings.is_empty(), "{:?}", resolved.warnings);
}

#[cfg(unix)]
#[test]
fn a_hung_adapter_is_stopped_at_its_timeout() {
    let root = project("hang", "ruff");
    stand_in(&root, "ruff", "case \"$1\" in --version) echo 'ruff 0.16.9'; exit 0;; esac\nsleep 30");
    let m = manifest("ruff").unwrap();
    let program = locate(m, &root, None).unwrap();
    let ready = Ready { manifest: m, version: version(m, &program, &root).unwrap(), program };
    let started = std::time::Instant::now();
    let e = check_files(&ready, &root, &["src/mod.py".to_owned()], Duration::from_millis(300)).unwrap_err();
    assert!(e.contains("timed out"), "{e}");
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[cfg(unix)]
#[test]
fn a_report_it_cannot_read_names_the_tool_and_its_last_words() {
    let root = project("garbage", "ruff");
    stand_in(&root, "ruff", "case \"$1\" in --version) echo 'ruff 0.16.9'; exit 0;; esac\necho 'ruff failed: bad config' 1>&2\nexit 2");
    let resolved = resolve(Some(&["ruff".to_owned()]), &root, None).unwrap();
    let e = check_files(&resolved.ready[0], &root, &["src/mod.py".to_owned()], Duration::from_secs(20)).unwrap_err();
    assert!(e.contains("ruff failed: bad config"), "{e}");
}

/// Each real tool, where it is installed (`PATH`): its report on the
/// fixture maps onto in-range diagnostics. Absent tools are skipped.
#[test]
fn the_real_tools_where_installed() {
    let path = std::env::var_os("PATH");
    for m in manifests() {
        let Some(program) = locate(m, Path::new("/nonexistent"), path.as_deref()) else {
            eprintln!("skipped: `{}` is not installed", m.binary);
            continue;
        };
        let root = project(&format!("real-{}", m.name), &m.name);
        let v = match version(m, &program, &root) {
            Ok(v) => v,
            Err(e) => panic!("{}: {e}", m.name),
        };
        let ready = Ready { manifest: m, program, version: v };
        let rel = source_of(&m.name).to_owned();
        let report = check_files(&ready, &root, std::slice::from_ref(&rel), Duration::from_secs(60)).unwrap_or_else(|e| panic!("{}: {e}", m.name));
        let text = std::fs::read_to_string(root.join(&rel)).unwrap();
        let findings = &report.findings[&rel];
        assert!(!findings.is_empty(), "{}: the fixture has findings", m.name);
        for f in findings {
            let d = f.to_diagnostic(m, &rel, &text, &ready.source());
            assert!(d.start_byte <= d.end_byte && d.end_byte <= text.len(), "{}: {d:?}", m.name);
            assert!(d.rule.starts_with(&format!("adapter/{}/", m.name)));
        }
        eprintln!("ran: {} {}: {} findings", m.name, ready.version, findings.len());
    }
}
