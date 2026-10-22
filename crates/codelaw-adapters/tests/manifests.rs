//! W-14 DoD 1 and 3: one manifest per adapter SR-LINTER decision 2 names,
//! each with a tested version range, each tool permissive (L2), and none
//! that L2 refuses.

use codelaw_adapters::{Output, manifests, selected};
use codelaw_core::Language;

/// L2's permissive, OSI-approved licences (SR-LAW-2 decisions 1 and 2), as
/// `crates/codelaw-cli/tests/licences.rs` holds them.
const PERMISSIVE: &[&str] = &["MIT", "MIT-0", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib", "0BSD", "Unlicense", "BSL-1.0"];

/// Tools L2 rules out, so no manifest may name them: yamllint is GPL-3.0.
const REFUSED: &[&str] = &["yamllint"];

#[test]
fn the_four_adapters_of_sr_linter_decision_2_have_manifests() {
    let names: Vec<&str> = manifests().iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["html-validate", "oxlint", "ruff", "stylelint"]);
    let defaults: Vec<&str> = selected(None).unwrap().iter().map(|m| m.name.as_str()).collect();
    assert_eq!(defaults, ["oxlint", "ruff"], "oxlint and Ruff by default when installed; stylelint and html-validate when named");
}

#[test]
fn every_manifest_is_complete_and_its_tool_permissive() {
    for m in manifests() {
        assert!(PERMISSIVE.contains(&m.licence.as_str()), "{}: {} does not pass L2", m.name, m.licence);
        assert!(!REFUSED.contains(&m.name.as_str()), "{} is refused by L2", m.name);
        assert!(!m.tested.comparators.is_empty(), "{}: no tested range", m.name);
        assert!(m.tested.comparators.iter().any(|c| c.op == semver::Op::Less), "{}: the tested range must be closed above", m.name);
        assert!(!m.args.is_empty() && !m.version_args.is_empty() && !m.languages.is_empty(), "{}", m.name);
        assert!(!m.config_files.is_empty(), "{}: its config files enter the cache key", m.name);
        assert!(m.args.iter().any(|a| a.contains("json")), "{}: the command line asks for JSON", m.name);
    }
}

#[test]
fn each_adapter_reads_its_own_output_and_languages() {
    for (name, output, language) in [
        ("oxlint", Output::Oxlint, Language::Typescript),
        ("ruff", Output::Ruff, Language::Python),
        ("stylelint", Output::Stylelint, Language::Css),
        ("html-validate", Output::HtmlValidate, Language::Html),
    ] {
        let m = codelaw_adapters::manifest(name).unwrap();
        assert_eq!(m.output, output);
        assert!(m.handles(language), "{name}");
    }
}

#[test]
fn the_config_schema_names_exactly_these_adapters() {
    let schema: serde_json::Value = serde_json::from_str(codelaw_config::SCHEMA).unwrap();
    let names: Vec<&str> =
        schema["properties"]["adapters"]["properties"]["enabled"]["items"]["enum"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(names, manifests().iter().map(|m| m.name.as_str()).collect::<Vec<_>>());
}

#[test]
fn an_unknown_adapter_in_config_is_refused() {
    let e = selected(Some(&["yamllint".to_owned()])).unwrap_err();
    assert!(e.contains("unknown adapter `yamllint`"), "{e}");
    assert!(selected(Some(&[])).unwrap().is_empty(), "`enabled = []` turns every adapter off");
}
