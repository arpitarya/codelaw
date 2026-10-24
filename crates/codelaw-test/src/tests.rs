use super::*;
use codelaw_rules::{RuleSource, parse_cases, parse_rule};
use std::path::Path;

/// A rule loaded under the `id:` its frontmatter names. Rules with
/// different matchers take different ids: the engine keeps compiled
/// matchers by id for the life of the process, and tests run in parallel.
fn loaded(text: &str, cases: Option<&str>) -> LoadedRule {
    let id = text.lines().find_map(|l| l.strip_prefix("id: ")).expect("an id").to_owned();
    let mut rule = parse_rule(Path::new(&format!("{id}.md")), text, Some(&id)).unwrap_or_else(|e| panic!("{e:?}"));
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

/// SR-FORMATTER decision 3: a `Don't` with the same tokens as its `Do` is
/// a layout rule, and `codelaw test` refuses it.
#[test]
fn a_layout_rule_is_refused() {
    let r = test_rule(&loaded(&rule("if (a==b) {}", "if (a == b) {\n}"), None), None);
    let f = failures(&r);
    assert!(
        f.contains(&"examples · JavaScript Don't (line 26): a layout rule: it has the same tokens as the Do (line 32), so the two differ only in layout, and layout belongs to the formatter (SR-FORMATTER decision 3)".to_string()),
        "{f:?}"
    );
    // A Do with another token is not a layout rule.
    let r = test_rule(&loaded(&rule("if (a==b) {}", "if (a===b) {}"), None), None);
    assert!(!failures(&r).iter().any(|f| f.contains("layout rule")), "{:?}", failures(&r));
}

/// A rule on a string literal: `regex` (a YAML single-quoted scalar) finds
/// it and the fix replaces it with `to`; the examples are `foo(<from>);`
/// and `foo(<to>);`.
fn string_rule(id: &str, regex: &str, from: &str, to: &str) -> String {
    let fix = to.replace('\'', "''");
    format!(
        "---\nid: acme/{id}\ntitle: Write the string right\nlanguages: [javascript]\nfix: auto\n---\n\nWrite the string right.\n\n**Why:** because.\n\n```codelaw\nmatch:\n  kind: string\n  regex: '{regex}'\nmessage: \"Write the string right\"\nfix: '{fix}'\n```\n\n## Examples\n\n### JavaScript\n\n#### Don't\n\n```js\nfoo({from});\n```\n\n#### Do\n\n```js\nfoo({to});\n```\n"
    )
}

/// SR-FORMATTER decision 3: a `Don't` and `Do` that differ only inside a
/// string literal differ in a token, so the rule is not a layout rule.
#[test]
fn a_token_change_is_not_a_layout_rule() {
    let r = test_rule(&loaded(&string_rule("no-double-space", "  ", "\"a  b\"", "\"a b\""), None), None);
    assert!(r.passed(), "{:?}", failures(&r));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

/// The formatter cross-check: tokens differ (`'a'` and `"a"`), but Biome
/// lays both out as `foo("a");` — a warning naming the formatter and B-045,
/// not a failure.
#[test]
fn a_formatter_that_joins_two_token_streams_warns() {
    let text = string_rule("double-quotes", "^''", "'a'", "\"a\"");
    let r = test_rule(&loaded(&text, None), None);
    assert!(r.passed(), "{:?}", failures(&r));
    assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
    let w = &r.warnings[0].message;
    assert!(w.contains("biome_js_formatter") && w.contains("B-045") && w.contains("tokens differ"), "{w}");
    assert!(render(&[r], false).contains("warning examples · JavaScript Don't"));
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
