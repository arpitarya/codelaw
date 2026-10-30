//! The `data` shape and the hover card (W-137): each kind from its rule and
//! message, tolerant of rules not built yet.

use super::*;
use codelaw_core::{Diagnostic, FixClass, Position, Severity, TextEdit};
use serde_json::json;

fn diag(rule: &str, message: &str, severity: Severity, fix: FixClass, data: Option<Value>) -> Diagnostic {
    Diagnostic {
        rule: rule.into(),
        severity,
        file: "x".into(),
        start_byte: 0,
        end_byte: 1,
        start: Position { line: 1, column: 1 },
        end: Position { line: 1, column: 2 },
        message: message.into(),
        fix,
        edits: vec![],
        steps: vec![],
        source: "test".into(),
        note: None,
        data,
    }
}

fn data_of(d: &Diagnostic, rel: &str, as_of: Option<&str>) -> Value {
    let as_of = as_of.map(|s| codelaw_engine::Date::parse(s).unwrap().days());
    data(&Facts { diagnostic: d, rel, baseline: false, as_of, rule: None })
}

#[test]
fn kinds_come_from_the_rule_and_the_message() {
    assert_eq!(kind_of("deps/vulnerable", "x"), "vulnerable");
    assert_eq!(kind_of("data/outdated-dependency", "`a` ^1 — latest 2.0.0 (major)"), "outdated");
    assert_eq!(kind_of("deps/outdated", "`a` ^1 — 1.0.0 is deprecated: no; latest 2.0.0 (deprecated)"), "deprecated");
    assert_eq!(kind_of("deps/incomplete-scan", "x"), "scan");
    assert_eq!(kind_of("js/todo-marker", "`TODO(PAY-388, 2026-10-01)` expired on 2026-10-01 (29 days ago): do it"), "expired");
    assert_eq!(kind_of("py/todo-marker", "`TODO(A-1, 2026-11-15)` expires in 16 days (2026-11-15)"), "due");
    assert_eq!(kind_of("js/todo-marker", "`FIXME` needs a ticket matching `X`"), "ticket");
    assert_eq!(kind_of("js/flag-expiry", "flag `summer-sale` expired 3 days ago"), "flag-expired");
    assert_eq!(kind_of("js/flag-expiry", "Flag `new-checkout` expires on 2026-11-01, in 2 days"), "flag");
    // A flag marker or the registry is kept in line by an `auto` fix: plain code.
    assert_eq!(kind_of("js/flag-marker", "x"), "code");
    assert_eq!(kind_of("data/codeowners-coverage", "x"), "ownership");
    assert_eq!(kind_of("js/no-focused-tests", "x"), "focused");
    assert_eq!(kind_of("js/skipped-test", "x"), "skipped");
    assert_eq!(kind_of("js/complexity", "x"), "complexity");
    assert_eq!(kind_of("sec-js/no-dangerously-set-inner-html", "x"), "security");
    // A rule nobody has heard of is plain code.
    assert_eq!(kind_of("js/some-future-rule", "x"), "code");
    assert!(KINDS.contains(&"code"));
}

#[test]
fn the_view_follows_the_file_then_the_kind() {
    assert_eq!(view_of("web/package.json", "code"), "manifest");
    assert_eq!(view_of(".nvmrc", "runtime"), "manifest");
    assert_eq!(view_of("src/a.ts", "expired"), "marker");
    assert_eq!(view_of("src/a.test.ts", "skipped"), "marker");
    assert_eq!(view_of("src/a.test.ts", "focused"), "code");
    assert_eq!(view_of("src/a.ts", "flag"), "code");
}

#[test]
fn an_outdated_dependency_names_its_package_latest_and_grade() {
    let d = diag("data/outdated-dependency", "`axios` ^1.6.0 — installed 1.6.2, latest 1.13.2 (minor)", Severity::Info, FixClass::Auto, None);
    let v = data_of(&d, "package.json", None);
    assert_eq!(v["kind"], "outdated");
    assert_eq!(v["view"], "manifest");
    assert_eq!(v["package"], "axios");
    assert_eq!(v["version"], "1.6.2");
    assert_eq!(v["latest"], "1.13.2");
    assert_eq!(v["grade"], "minor");
    assert_eq!(v["badge"], "↑ 1.13.2 · minor");
    assert_eq!(v["title"], "axios 1.13.2 is out");
}

#[test]
fn a_vulnerable_dependency_prefers_the_rules_data() {
    let msg = "`lodash` 4.17.15 is vulnerable — GHSA-35jh-r3h4-6jhm (high, CVE-2021-23337) and 2 more; upgrade to 4.17.21";
    let facts = json!({ "package": "lodash", "version": "4.17.15", "advisory": "GHSA-35jh-r3h4-6jhm", "target": "4.17.21", "direct": true });
    let d = diag("deps/vulnerable", msg, Severity::Error, FixClass::Suggest, Some(facts.clone()));
    let v = data_of(&d, "package.json", None);
    assert_eq!(v["kind"], "vulnerable");
    assert_eq!(v["grade"], "high");
    assert_eq!(v["safe"], "4.17.21");
    assert_eq!(v["advisories"], json!(["GHSA-35jh-r3h4-6jhm"]));
    assert_eq!(v["more"], 2);
    assert_eq!(v["badge"], "HIGH · 3 advisories → 4.17.21");
    assert_eq!(v["title"], "lodash 4.17.15 is vulnerable");
    assert_eq!(v["facts"], facts);
}

#[test]
fn a_marker_counts_days_from_the_as_of_date() {
    let d = diag("js/todo-marker", "`TODO(PAY-388, 2026-10-01)` expired on 2026-10-01 (29 days ago): do the work", Severity::Error, FixClass::Manual, None);
    let v = data_of(&d, "src/coupon.ts", Some("2026-10-30"));
    assert_eq!(v["kind"], "expired");
    assert_eq!(v["view"], "marker");
    assert_eq!(v["due"], "2026-10-01");
    assert_eq!(v["days"], -29);
    assert_eq!(v["ticket"], "PAY-388");
    assert_eq!(v["tag"], "TODO");
    assert_eq!(v["badge"], "expired 29 d ago");
    assert_eq!(v["title"], "TODO expired 29 days ago");
    // With no as-of date, the message's own count.
    let v = data_of(&d, "src/coupon.ts", None);
    assert_eq!(v["days"], -29);
    let due = diag("js/todo-marker", "`TODO(PAY-412, 2026-11-15)` expires in 16 days (2026-11-15)", Severity::Warn, FixClass::Manual, None);
    assert_eq!(data_of(&due, "a.ts", Some("2026-10-30"))["badge"], "due in 16 d");
    let bare = diag("js/todo-marker", "`FIXME` needs a ticket matching `[A-Z]+-\\d+`: write `FIXME(ABC-123, 2027-01-31): …`", Severity::Error, FixClass::Manual, None);
    let v = data_of(&bare, "a.ts", None);
    assert_eq!(v["badge"], "needs a ticket");
    assert_eq!(v["title"], "FIXME needs a ticket");
}

#[test]
fn an_unknown_rule_is_code_with_its_rule_as_title() {
    let d = diag("js/whatever", "Something", Severity::Warn, FixClass::Manual, None);
    let v = data_of(&d, "a.js", None);
    assert_eq!(v["kind"], "code");
    assert_eq!(v["view"], "code");
    assert_eq!(v["title"], "js/whatever");
    assert!(v.get("fixable").is_none());
}

#[test]
fn the_card_has_chips_why_example_and_meta() {
    let mut d = diag("py/no-bare-except", "Bare `except:` catches everything", Severity::Warn, FixClass::Auto, None);
    d.edits = vec![TextEdit::new(0, 7, "except Exception:")];
    let v = data_of(&d, "a.py", None);
    assert_eq!(v["fixable"], true);
    let c = card(&d, &v, None, Some(Language::Python), Some(("    except:".into(), "    except Exception:".into())));
    assert!(c.starts_with("**py/no-bare-except** &nbsp; `warn` `one-click fix`"), "{c}");
    assert!(c.contains("except:\n```\n→\n```python\nexcept Exception:"), "{c}");
    assert!(c.ends_with("`py/no-bare-except` · new · fix: `auto`"), "{c}");
    let b = data(&Facts { diagnostic: &d, rel: "a.py", baseline: true, as_of: None, rule: None });
    let c = card(&d, &b, None, None, None);
    assert!(c.contains("`baseline`") && c.contains("in the baseline"), "{c}");
}

#[test]
fn a_dependency_card_shows_its_facts_as_rows() {
    let msg = "`lodash` 4.17.15 is vulnerable — GHSA-35jh-r3h4-6jhm (high) and 2 more; upgrade to 4.17.21";
    let d = diag("deps/vulnerable", msg, Severity::Error, FixClass::Manual, None);
    let v = data_of(&d, "package.json", None);
    let c = card(&d, &v, None, Some(Language::Json), None);
    assert!(c.starts_with("**lodash 4.17.15 is vulnerable** &nbsp; `high` `needs your call`"), "{c}");
    assert!(c.contains("| Safe version | **4.17.21** |"), "{c}");
    assert!(c.contains("| Also | **2 more advisories** |"), "{c}");
}

#[test]
fn a_sentence_ends_at_the_first_full_stop() {
    assert_eq!(sentence("It runs. Then it stops."), "It runs.");
    assert_eq!(sentence("Use e.g. this. And that."), "Use e.g. this.");
    assert_eq!(sentence("One"), "One");
}
