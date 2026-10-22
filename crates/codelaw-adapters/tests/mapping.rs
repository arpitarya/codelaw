//! W-14 DoD 2: each adapter's real output, mapped onto diagnostics with a
//! fix class (L4 decision 9). No tool is needed: `tests/fixtures/<name>/`
//! holds a source file and what the tool printed for it, captured from the
//! version its manifest's `tested` range starts at, in that folder:
//!
//! ```text
//! npx --yes oxlint@1.89.0 --format=json src/app.js                                   > stdout.json
//! uvx ruff==0.16.9 check --output-format=json --no-cache --no-fix --exit-zero \
//!     --force-exclude src/mod.py                                                     > stdout.json
//! npx --yes stylelint@17.16.0 --formatter json --compute-edit-info \
//!     --allow-empty-input src/style.css                                             2> stderr.json
//! npx --yes html-validate@11.17.0 --formatter json src/page.html                     > stdout.json
//! ```
//!
//! The absolute folder they ran in is replaced by `@ROOT@`. The sources put
//! non-ASCII text before findings, so each tool's unit (bytes, code points,
//! UTF-16) is exercised.

use codelaw_adapters::{Output, manifest, parse};
use codelaw_core::{Diagnostic, FixClass, Position, Severity, TextEdit};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every diagnostic the fixture's report maps to, in report order.
fn mapped(name: &str, source: &str) -> (String, Vec<Diagnostic>) {
    let m = manifest(name).unwrap();
    let dir = fixture(name);
    let out = |f: &str| std::fs::read_to_string(dir.join(f)).unwrap_or_default().replace("@ROOT@", "/root");
    let parsed = parse(m.output, &out("stdout.json"), &out("stderr.json")).unwrap();
    let text = read(&dir.join(source));
    let d = parsed
        .findings
        .iter()
        .map(|f| {
            assert!(f.file == source || f.file == format!("/root/{source}"), "{name}: unexpected file {}", f.file);
            f.to_diagnostic(m, source, &text, &format!("{name}@test"))
        })
        .collect();
    (text, d)
}

fn find<'a>(d: &'a [Diagnostic], rule: &str) -> &'a Diagnostic {
    d.iter().find(|x| x.rule == rule).unwrap_or_else(|| panic!("no {rule} in {:#?}", d.iter().map(|x| &x.rule).collect::<Vec<_>>()))
}

fn at(line: u32, column: u32) -> Position {
    Position { line, column }
}

fn apply(text: &str, edits: &[TextEdit]) -> String {
    let mut sorted = edits.to_vec();
    sorted.sort();
    let mut out = text.to_owned();
    for e in sorted.iter().rev() {
        out.replace_range(e.start..e.end, &e.text);
    }
    out
}

#[test]
fn oxlint_maps_byte_spans_and_every_finding_is_manual() {
    let (text, d) = mapped("oxlint", "src/app.js");
    assert_eq!(d.len(), 2);
    let unused = find(&d, "adapter/oxlint/eslint/no-unused-vars");
    assert_eq!((unused.start, unused.end), (at(2, 7), at(2, 13)));
    assert_eq!(&text[unused.start_byte..unused.end_byte], "unused");
    assert_eq!(unused.severity, Severity::Warn);
    let debugger = find(&d, "adapter/oxlint/eslint/no-debugger");
    assert_eq!(&text[debugger.start_byte..debugger.end_byte], "debugger;");
    for x in &d {
        assert_eq!(x.fix, FixClass::Manual);
        assert!(x.edits.is_empty());
        assert!(x.steps.last().unwrap().contains("https://oxc.rs/docs/guide/usage/linter/rules/eslint/"), "{:?}", x.steps);
        assert_eq!(x.source, "oxlint@test");
    }
    assert_eq!(debugger.steps[0], "Remove the debugger statement");
}

#[test]
fn ruff_maps_code_point_columns_and_its_applicability() {
    let (text, d) = mapped("ruff", "src/mod.py");
    assert_eq!(d.len(), 5);
    // safe → auto: the edit removes the whole import line.
    let f401 = find(&d, "adapter/ruff/F401");
    assert_eq!((f401.fix, f401.start, f401.end), (FixClass::Auto, at(2, 8), at(2, 10)));
    assert!(!apply(&text, &f401.edits).contains("import os"));
    assert!(f401.steps.is_empty());
    // unsafe → suggest, with its edits.
    let e711 = find(&d, "adapter/ruff/E711");
    assert_eq!(e711.fix, FixClass::Suggest);
    assert!(apply(&text, &e711.edits).contains("    if x is None:\n"));
    assert_eq!(find(&d, "adapter/ruff/E731").fix, FixClass::Suggest);
    // No fix → manual, with the rule's docs.
    let e741 = find(&d, "adapter/ruff/E741");
    assert_eq!(e741.fix, FixClass::Manual);
    assert!(e741.steps[0].contains("https://docs.astral.sh/ruff/rules/ambiguous-variable-name"), "{:?}", e741.steps);
    // `print("☕", undefined_name)`: Ruff counts code points, as codelaw does.
    let f821 = find(&d, "adapter/ruff/F821");
    assert_eq!((f821.start, f821.end), (at(13, 12), at(13, 26)));
    assert_eq!(&text[f821.start_byte..f821.end_byte], "undefined_name");
    assert!(d.iter().all(|x| x.severity == Severity::Error));
}

#[test]
fn stylelint_maps_utf16_edits_to_auto() {
    let (text, d) = mapped("stylelint", "src/style.css");
    assert_eq!(d.len(), 4);
    let hex = find(&d, "adapter/stylelint/color-hex-length");
    let zero = find(&d, "adapter/stylelint/length-zero-no-unit");
    assert_eq!((hex.fix, zero.fix), (FixClass::Auto, FixClass::Auto));
    assert_eq!(&text[hex.start_byte..hex.end_byte], "#ffffff");
    // The edits' offsets are UTF-16 units past a line holding `é` and `☕`.
    let both: Vec<TextEdit> = hex.edits.iter().chain(&zero.edits).cloned().collect();
    assert_eq!(apply(&text, &both).lines().nth(1), Some("a { color: #fff; margin: 0; }"));
    let empty = find(&d, "adapter/stylelint/block-no-empty");
    assert_eq!((empty.fix, empty.start, empty.end), (FixClass::Manual, at(3, 3), at(3, 5)));
    assert_eq!(empty.message, "Empty block");
    assert!(empty.steps[0].contains("https://stylelint.io/user-guide/rules/block-no-empty"));
    assert_eq!(find(&d, "adapter/stylelint/color-no-invalid-hex").fix, FixClass::Manual);
}

#[test]
fn html_validate_maps_utf16_offsets() {
    let (text, d) = mapped("html-validate", "src/page.html");
    assert_eq!(d.len(), 2);
    // `<title>café ☕</title>` comes first, so bytes and UTF-16 units differ.
    let alt = find(&d, "adapter/html-validate/wcag/h37");
    assert_eq!(&text[alt.start_byte..alt.end_byte], "img");
    assert_eq!(alt.start, at(5, 2));
    let dup = find(&d, "adapter/html-validate/no-dup-id");
    assert_eq!(&text[dup.start_byte..dup.end_byte], "x");
    for x in &d {
        assert_eq!((x.fix, x.severity), (FixClass::Manual, Severity::Error));
        assert!(x.steps[0].contains("https://html-validate.org/rules/"));
    }
}

#[test]
fn a_report_the_adapter_cannot_read_is_an_error() {
    assert!(parse(Output::Oxlint, "Failed to parse configuration file", "").is_err());
    assert!(parse(Output::Stylelint, "", "Error: No configuration provided for /x.css").is_err());
}
