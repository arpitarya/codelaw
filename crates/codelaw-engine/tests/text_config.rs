//! A finding in a plain-text config file (SR-LAW-3 decision 10,
//! SR-ARCHITECTURE decision 15d): a native rule that read `.nvmrc` reports on
//! `package.json` and on the `.nvmrc` line, through the test-only rules of
//! `codelaw-native`'s `fault` module.

use codelaw_core::{Diagnostic, FixClass};
use codelaw_engine::{EngineOptions, FixOptions, Project};
use std::fs;
use std::path::PathBuf;

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Repo {
        let d = std::env::temp_dir().join(format!("codelaw-text-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        Repo(fs::canonicalize(d).unwrap())
    }

    fn file(&self, rel: &str, text: &str) -> &Repo {
        let p = self.0.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
        self
    }

    fn project(&self) -> Project {
        Project::load(&self.0, EngineOptions { no_cache: true, ..Default::default() }).unwrap()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The `data` pack with one rule, `data/test-<native>`, running the
/// test-only native rule `test/<native>` at `warn`.
fn repo(name: &str, native: &str) -> Repo {
    let r = Repo::new(name);
    let id = format!("data/test-{native}");
    let rule = format!(
        "---\nid: {id}\ntitle: test rule\nlanguages: [json]\nfix: manual\nfix_reason: test\nstability: stable\n---\n\nSummary.\n\n**Why:** test.\n\n```codelaw\nnative: test/{native}\n```\n\n## How to fix\n\n1. Make them agree.\n\n## Examples\n\n### JSON\n\n#### Don't\n\n```json\n{{}}\n```\n\n#### Do\n\n```json\n{{}}\n```\n"
    );
    let dir = "node_modules/@codelaw/rules-data";
    r.file(&format!("{dir}/codelaw-pack.toml"), "name = \"@codelaw/rules-data\"\nversion = \"0.1.0\"\nmin_codelaw = \"0.1.0\"\nlanguages = [\"json\"]\npresets = [\"data/recommended\"]\n");
    r.file(&format!("{dir}/rules/{id}.md"), &rule);
    r.file(&format!("{dir}/presets/data/recommended.toml"), &format!("[rules]\n\"{id}\" = \"warn\"\n"));
    r.file("codelaw.toml", "extends = [\"codelaw:data/recommended\"]\n");
    r
}

fn summary(d: &[Diagnostic]) -> Vec<String> {
    d.iter().map(|d| format!("{}:{}:{} {} {}", d.file, d.start.line, d.start.column, d.rule, d.fix)).collect()
}

const PACKAGE: &str = "{\n  \"name\": \"a\",\n  \"engines\": { \"node\": \">=18\" }\n}\n";

#[test]
fn a_finding_shows_on_the_manifest_and_on_the_text_file_line() {
    let r = repo("both", "node-version");
    r.file("package.json", PACKAGE).file(".nvmrc", "20.11.0\n");
    let d = r.project().check(&[]).unwrap().diagnostics;
    assert_eq!(summary(&d), [".nvmrc:1:1 data/test-node-version manual", "package.json:3:24 data/test-node-version manual"]);
    let (copy, primary) = (&d[0], &d[1]);
    assert_eq!(copy.message, primary.message);
    assert_eq!(copy.severity, primary.severity);
    assert_eq!((copy.start_byte, copy.end_byte), (0, 7), "the whole line, no line ending");
    assert_eq!(copy.end.column, 8);
    assert_eq!(copy.data.as_ref().unwrap()["primary"], serde_json::json!({ "file": "package.json", "line": 3, "column": 24 }));
    assert!(copy.steps.last().unwrap().contains("`package.json:3:24`"), "{:?}", copy.steps);
    assert_eq!(copy.fix, FixClass::Manual);
}

#[test]
fn agreeing_files_give_nothing_and_a_cached_run_follows_the_text_file() {
    let r = repo("cache", "node-version");
    r.file("package.json", PACKAGE).file(".nvmrc", "18\n");
    let opts = || EngineOptions { cache_dir_env: Some(r.0.join("cache").to_string_lossy().into()), ..Default::default() };
    assert!(Project::load(&r.0, opts()).unwrap().check(&[]).unwrap().diagnostics.is_empty());
    // Only `.nvmrc` changes: the cached result of `package.json` asked about
    // it, so it is not reused.
    r.file(".nvmrc", "22\n");
    let d = Project::load(&r.0, opts()).unwrap().check(&[]).unwrap().diagnostics;
    assert_eq!(summary(&d).len(), 2, "{:?}", summary(&d));
}

#[test]
fn fix_never_writes_a_text_config_file() {
    let r = repo("fix", "node-version");
    r.file("package.json", PACKAGE).file(".nvmrc", "20\n");
    // `fix` writes no text config file and reports the copy as `check` does.
    let report = r.project().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(summary(&report.check.diagnostics).len(), 2);
    assert_eq!(fs::read_to_string(r.0.join(".nvmrc")).unwrap(), "20\n");
    assert!(report.files.iter().all(|f| f.file != ".nvmrc"));
}

#[test]
fn the_editor_and_the_hook_get_a_text_files_findings_from_its_manifests() {
    let r = repo("editor", "node-version");
    r.file("web/package.json", PACKAGE).file("web/.nvmrc", "20\n").file("package.json", "{}\n");
    let p = r.project();
    let d = p.check_text_config(&r.0.join("web/.nvmrc"));
    assert_eq!(summary(&d), ["web/.nvmrc:1:1 data/test-node-version manual"]);
    assert!(p.check_text_config(&r.0.join("web/package.json")).is_empty(), "only a listed file");
    assert!(p.check_text_config(&r.0.join(".nvmrc")).is_empty(), "nothing reports into a file that is not there");
}

#[test]
fn a_place_outside_the_list_fails_the_rule() {
    let r = repo("outside", "elsewhere-outside-list");
    r.file("a.json", "{}\n").file("notes.txt", "x\n");
    let report = r.project().check(&[]).unwrap();
    let s = summary(&report.diagnostics);
    assert!(s.contains(&"a.json:1:1 codelaw/internal-error manual".to_owned()), "{s:?}");
    assert!(s.contains(&"a.json:1:1 data/test-elsewhere-outside-list manual".to_owned()), "the finding itself stays: {s:?}");
    assert!(!s.iter().any(|l| l.starts_with("notes.txt")), "{s:?}");
    let e = report.diagnostics.iter().find(|d| d.rule == "codelaw/internal-error").unwrap();
    assert!(e.message.contains("`notes.txt`") && e.message.contains("not a text config file"), "{}", e.message);
}
