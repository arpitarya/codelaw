use super::*;
use codelaw_rules::{RuleSource, parse_cases, parse_rule};
use std::path::Path;

fn loaded(text: &str, cases: Option<&str>) -> LoadedRule {
    let mut rule = parse_rule(Path::new("acme/eqeqeq.md"), text, Some("acme/eqeqeq")).unwrap_or_else(|e| panic!("{e:?}"));
    if let Some(c) = cases {
        rule.cases = parse_cases(Path::new("acme/eqeqeq.cases.md"), c).unwrap();
    }
    LoadedRule {
        rule: Arc::new(rule),
        source: RuleSource::Local,
        default: None,
        enabled_by: None,
        shadows: vec![],
        data: Default::default(),
        preview_blocked: false,
        external_allowed: true,
    }
}

fn rule(dont: &str, do_: &str) -> String {
    format!(
        "---\nid: acme/eqeqeq\ntitle: Use ===\nlanguages: [javascript]\nfix: auto\nfiles: [\"src/**\"]\n---\n\nUse `===`.\n\n**Why:** `==` coerces.\n\n```codelaw\nmatch:\n  pattern: $A == $B\nmessage: \"Use ===\"\nfix: \"$A === $B\"\n```\n\n## Examples\n\n### JavaScript\n\n#### Don't\n\n```js\n{dont}\n```\n\n#### Do\n\n```js\n{do_}\n```\n"
    )
}

fn failures(r: &RuleReport) -> Vec<String> {
    r.failures.iter().map(|f| format!("{} · {}: {}", f.level.name(), f.at, f.message.lines().next().unwrap_or_default())).collect()
}

#[test]
fn a_good_rule_passes() {
    let r = test_rule(&loaded(&rule("if (a == b) {}", "if (a === b) {}"), None), None);
    assert!(r.passed(), "{:?}", failures(&r));
    assert!(r.checks >= 5);
}

#[test]
fn a_dont_that_does_not_fire() {
    let r = test_rule(&loaded(&rule("a === b;", "a === b;"), None), None);
    assert_eq!(failures(&r), ["examples · JavaScript Don't (line 26): does not fire"]);
}

#[test]
fn a_fix_that_differs_from_do() {
    let r = test_rule(&loaded(&rule("a == b;", "a !== b;"), None), None);
    assert_eq!(
        failures(&r),
        ["examples · JavaScript Don't (line 26): the fix does not give the expected output (line 32): line 1: expected `a !== b;`, got `a === b;`"]
    );
    let r = test_rule(&loaded(&rule("a == b;", "a == b;"), None), None);
    assert!(failures(&r).contains(&"examples · JavaScript Do (line 32): fires at 1:1 acme/eqeqeq".to_string()), "{:?}", failures(&r));
}

#[test]
fn cases_compare_exactly() {
    let cases = "### case: two\n\n```js\na == b; c == d;\n```\n\n```expect\n1:1 acme/eqeqeq\n1:9 acme/eqeqeq\n```\n\n```fixed\na === b; c === d;\n```\n\n### case: wrong\n\n```js\nx == y;\n```\n\n```expect\n(none)\n```\n";
    let r = test_rule(&loaded(&rule("a == b;", "a === b;"), Some(cases)), None);
    assert_eq!(failures(&r), ["cases · case `wrong` (line 18): expected (none), got 1:1 acme/eqeqeq"]);
    let only = test_rule(&loaded(&rule("a == b;", "a === b;"), Some(cases)), Some("two"));
    assert!(only.passed(), "{:?}", failures(&only));
}

#[test]
fn unknown_invariant() {
    let text = rule("a == b;", "a === b;").replace("fix: auto\n", "fix: auto\ninvariant: types-unchanged\n");
    let r = test_rule(&loaded(&text, None), None);
    assert!(failures(&r)[0].contains("unknown invariant `types-unchanged`"), "{:?}", failures(&r));
}
