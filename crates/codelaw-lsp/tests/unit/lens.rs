//! Inlay hints, CodeLens and clean dependencies (W-137).

use super::*;
use serde_json::json;

const SNAPSHOT: &str = r#"{ "fetched": "2026-10-29", "packages": {
  "npm:react": { "latest": "19.2.0" }, "npm:axios": { "latest": "1.13.2" }, "npm:zod": { "latest": "3.24.1" },
  "pypi:httpx": { "latest": "0.28.1" }, "pypi:pydantic-core": { "latest": "2.27.0" } } }"#;

fn at(line: u32) -> Range {
    Range { start: Position { line, character: 2 }, end: Position { line, character: 9 } }
}

const ALL: Range = Range { start: Position { line: 0, character: 0 }, end: Position { line: 99, character: 0 } };

#[test]
fn clean_npm_dependencies_carry_latest_when_they_name_it() {
    let text = "{\n  \"dependencies\": {\n    \"react\": \"^19.2.0\",\n    \"axios\": \"^1.6.0\",\n    \"zod\": \"^3.24.0\",\n    \"left\": \"1\"\n  }\n}\n";
    let got = clean_dependencies("package.json", text, SNAPSHOT, &[3]);
    assert_eq!(got.len(), 2, "{got:?}");
    assert_eq!(got[0], Clean { line: 2, name: "react".into(), latest: "19.2.0".into(), at_latest: true });
    assert_eq!(got[1].name, "zod");
    assert!(!got[1].at_latest);
    // A broken snapshot or manifest is nothing, not an error.
    assert!(clean_dependencies("package.json", text, "{", &[]).is_empty());
    assert!(clean_dependencies("package.json", "{", SNAPSHOT, &[]).is_empty());
    assert!(clean_dependencies("src/a.js", text, SNAPSHOT, &[]).is_empty());
}

#[test]
fn clean_pypi_dependencies_are_read_from_the_arrays() {
    let text = "[project]\nname = \"x\"\ndependencies = [\n  \"httpx[http2]>=0.28.1\",\n  \"Pydantic_Core==2.27.0\", # pinned\n]\n[tool.x]\ndependencies = [\"httpx==0.1\"]\n";
    let got = clean_dependencies("pyproject.toml", text, SNAPSHOT, &[]);
    assert_eq!(got.iter().map(|c| (c.line, c.name.as_str(), c.at_latest)).collect::<Vec<_>>(), vec![(3, "httpx", true), (4, "Pydantic_Core", true)]);
}

#[test]
fn a_spec_names_a_version_only_whole() {
    assert!(spec_names("^19.2.0", "19.2.0"));
    assert!(spec_names(">=19.2.0,<20", "19.2.0"));
    assert!(!spec_names("^19.2.01", "19.2.0"));
    assert!(!spec_names("^119.2.0", "19.2.0"));
    assert!(!spec_names("19.2.0-rc.1", "19.2.0"));
}

#[test]
fn hints_are_badges_on_manifest_and_marker_lines_only() {
    let text = "a\nb\nc\n";
    let vuln = json!({ "kind": "vulnerable", "view": "manifest", "badge": "HIGH · 1 advisory", "fixable": true });
    let code = json!({ "kind": "code", "view": "code", "badge": "js/x" });
    let shown = [Shown { range: at(1), data: &vuln, message: "m" }, Shown { range: at(2), data: &code, message: "n" }];
    let clean = [Clean { line: 0, name: "react".into(), latest: "19.2.0".into(), at_latest: true }];
    let hints = inlay_hints(text, &shown, &clean, ALL);
    let labels: Vec<_> = hints.iter().map(|h| (h.position.line, h.position.character, match &h.label {
        InlayHintLabel::String(s) => s.clone(),
        _ => String::new(),
    })).collect();
    assert_eq!(labels, vec![(0, 1, "latest".to_owned()), (1, 1, "HIGH · 1 advisory".to_owned())]);
    // Outside the asked range: nothing.
    let none = Range { start: Position { line: 2, character: 0 }, end: Position { line: 2, character: 0 } };
    assert!(inlay_hints(text, &shown, &clean, none).is_empty());
}

#[test]
fn a_manifest_gets_a_summary_and_its_bulk_fixes() {
    let vuln = json!({ "kind": "vulnerable", "view": "manifest", "fixable": true });
    let minor = json!({ "kind": "outdated", "view": "manifest", "grade": "minor", "fixable": true });
    let major = json!({ "kind": "outdated", "view": "manifest", "grade": "major", "fixable": true });
    let shown = [Shown { range: at(5), data: &vuln, message: "" }, Shown { range: at(3), data: &minor, message: "" }, Shown { range: at(4), data: &major, message: "" }];
    let lenses = code_lenses("package.json", "file:///p/package.json", &shown, 2);
    let titles: Vec<_> = lenses.iter().map(|l| l.command.as_ref().unwrap().title.clone()).collect();
    assert_eq!(titles, vec!["codelaw · 1 vulnerable · 2 outdated · 2 up to date", "Fix vulnerable (1)", "Update minor and patch (1)"]);
    assert!(lenses.iter().all(|l| l.range.start.line == 3));
    assert_eq!(lenses[0].command.as_ref().unwrap().command, "");
    assert_eq!(lenses[2].command.as_ref().unwrap().arguments, Some(vec![json!("file:///p/package.json"), json!(["outdated"]), json!(["minor", "patch"])]));
    // Ordinary code: no lens.
    assert!(code_lenses("src/a.ts", "file:///p/src/a.ts", &shown, 0).is_empty());
    assert!(wanted(&minor, &["outdated".into()], &["minor".into()]));
    assert!(!wanted(&major, &["outdated".into()], &["minor".into()]));
}
