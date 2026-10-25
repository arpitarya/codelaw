use super::*;
use codelaw_core::{Diagnostic, FixClass, Position, Severity};
use std::collections::BTreeSet;

fn d(rule: &str, file: &str, line: usize) -> Diagnostic {
    let p = Position { line: line as u32, column: 1 };
    Diagnostic {
        rule: rule.into(),
        severity: Severity::Error,
        file: file.into(),
        start_byte: line * 10,
        end_byte: line * 10 + 3,
        start: p,
        end: p,
        message: "Use of `x`".into(),
        fix: FixClass::Manual,
        edits: Vec::new(),
        steps: Vec::new(),
        source: "engine".into(),
        note: None,
    }
}

fn occ(rule: &str, file: &str, lines: &[usize]) -> Vec<Occurrence> {
    lines.iter().map(|l| Occurrence::plain(d(rule, file, *l))).collect()
}

fn baselines(rule: &str, text: &str) -> Baselines {
    let mut b = Baselines::default();
    b.insert(BaselineFile::parse(rule, text).unwrap());
    b
}

fn date(s: &str) -> Date {
    Date::parse(s).unwrap()
}

fn all(_: &str, _: &str) -> bool {
    true
}

fn run(occurrences: Vec<Occurrence>, b: &Baselines, as_of: Option<&str>, touched: &[&str]) -> Outcome {
    let touched: BTreeSet<String> = touched.iter().map(|s| s.to_string()).collect();
    apply(occurrences, b, &Context { as_of: as_of.map(date), touched: &touched, evaluated: &all, exceptions: Exceptions::default() })
}

const ONE: &str = r#"
[[allow]]
file = "a.ts"
count = 2
until = "2027-03-31"
warn_from = "2027-03-01"
on_touch = "error"
reason = "ORD-1432"
"#;

#[test]
fn dates_parse_print_and_count_days() {
    assert_eq!(date("2027-03-31").to_string(), "2027-03-31");
    assert!(Date::parse("2027-02-29").is_err());
    assert!(Date::parse("2028-02-29").is_ok());
    assert!(Date::parse("2027-3-31").is_err());
    assert!(Date::parse("2027-03-31T00:00:00Z").is_err());
    assert_eq!(date("1970-01-01").days(), 0);
    for s in ["1969-12-31", "2000-02-29", "2026-10-22", "2100-03-01"] {
        assert_eq!(Date::from_days(date(s).days()), date(s), "{s}");
    }
    assert_eq!(date("2027-03-19").days_until(date("2027-03-31")), 12);
}

#[test]
fn file_names_round_trip() {
    assert_eq!(file::file_name("graph/no-deprecated-usage"), "graph--no-deprecated-usage.toml");
    assert_eq!(file::rule_of("graph--no-deprecated-usage.toml").as_deref(), Some("graph/no-deprecated-usage"));
}

#[test]
fn a_baseline_writes_what_it_parses_and_refuses_unknown_keys() {
    let b = BaselineFile::parse("js/x", ONE).unwrap();
    assert_eq!(b.entries[0].line, 2, "the line of its `[[allow]]`");
    let mut again = BaselineFile::parse("js/x", &b.write()).unwrap();
    again.entries[0].line = b.entries[0].line;
    assert_eq!(again, b);
    assert!(BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\ncount = 1\nline = 3\n").unwrap_err().contains("unknown key `line`"));
    assert!(BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\n").unwrap_err().contains("`count` is required"));
    assert!(BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\ncount = 1\n[[allow]]\nfile = \"a\"\ncount = 2\n").unwrap_err().contains("two"));
    // Bare TOML dates are dates too.
    assert_eq!(BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\ncount = 1\nexpires = 2027-03-31\n").unwrap().entries[0].expires, Some(date("2027-03-31")));
    // `until` is the last allowed day: it reads as `expires` the day after;
    // both on one entry is refused.
    assert_eq!(BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\ncount = 1\nuntil = 2027-03-31\n").unwrap().entries[0].expires, Some(date("2027-04-01")));
    assert!(BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\ncount = 1\nuntil = 2027-03-31\nexpires = 2027-04-01\n").unwrap_err().contains("`until` and `expires`"));
    assert!(BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\ncount = 1\nwarn_days = -1\n").unwrap_err().contains("warn_days"));
    let full = BaselineFile::parse("js/x", "[[allow]]\nfile = \"a\"\ncount = 1\nexpires = \"2026-11-15\"\nwarn_days = 5\nreason = \"r\"\nadded_by = \"a@example.com\"\n").unwrap();
    assert_eq!((full.entries[0].warn_days, full.entries[0].added_by.as_deref()), (Some(5), Some("a@example.com")));
    assert!(full.write().ends_with("count = 1\nexpires = \"2026-11-15\"\nwarn_days = 5\nreason = \"r\"\nadded_by = \"a@example.com\"\n"), "{}", full.write());
}

#[test]
fn each_situation_of_decision_2() {
    let b = baselines("js/x", ONE);
    // Within allowance, before warn_from: info. `until = 2027-03-31` is
    // `expires = 2027-04-01`.
    let o = run(occ("js/x", "a.ts", &[1, 2]), &b, Some("2026-10-18"), &[]);
    assert!(o.diagnostics.iter().all(|d| d.severity == Severity::Info), "{:?}", o.diagnostics);
    assert!(o.diagnostics[0].message.contains("grandfathered, expires 2027-04-01 (2 left in this file)"));
    assert!(o.notes.is_empty());
    // After warn_from: the findings stay info, and the entry gets one
    // warning at its own line, with the days left.
    let o = run(occ("js/x", "a.ts", &[1, 2]), &b, Some("2027-03-20"), &[]);
    assert!(o.diagnostics.iter().all(|d| d.severity == Severity::Info));
    assert_eq!(o.notes.len(), 1);
    assert_eq!((o.notes[0].rule, o.notes[0].line), (EXPIRING_RULE, 2));
    assert!(o.notes[0].message.contains("expires in 12 days (2027-04-01)"), "{}", o.notes[0].message);
    // On `expires`: the entry covers nothing; the findings are back at the
    // rule's own severity, and the entry is an error.
    let o = run(occ("js/x", "a.ts", &[1, 2]), &b, Some("2027-04-01"), &[]);
    assert!(o.diagnostics.iter().all(|d| d.severity == Severity::Error));
    assert!(o.diagnostics[0].message.contains("its baseline entry expired on 2027-04-01"));
    assert_eq!(o.notes[0].rule, EXPIRED_RULE);
    // Touched: every occurrence in the file is an error.
    let o = run(occ("js/x", "a.ts", &[1, 2]), &b, Some("2026-10-18"), &["a.ts"]);
    assert!(o.diagnostics.iter().all(|d| d.severity == Severity::Error && d.message.contains("migrate its 2 occurrences now")));
    // Beyond the count: each at the rule's severity.
    let o = run(occ("js/x", "a.ts", &[1, 2, 3]), &b, Some("2026-10-18"), &[]);
    assert!(o.diagnostics.iter().all(|d| d.severity == Severity::Error && d.message.contains("3 in this file; the baseline allows 2")));
    // No entry: unchanged.
    let o = run(occ("js/x", "b.ts", &[1]), &b, Some("2026-10-18"), &[]);
    assert_eq!(o.diagnostics[0].message, "Use of `x`");
    // Fewer than allowed: stale.
    let o = run(occ("js/x", "a.ts", &[1]), &b, Some("2026-10-18"), &[]);
    assert_eq!(o.stale, vec![Stale { rule: "js/x".into(), file: "a.ts".into(), symbol: None, allowed: 2, found: 1 }]);
    // No as-of date: deadlines are not evaluated.
    let o = run(occ("js/x", "a.ts", &[1, 2]), &b, None, &[]);
    assert!(o.diagnostics.iter().all(|d| d.severity == Severity::Info));
}

#[test]
fn on_touch_warn_and_keep() {
    let b = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 1\non_touch = \"warn\"\n[[allow]]\nfile = \"b.ts\"\ncount = 1\non_touch = \"keep\"\n");
    let mut o = occ("js/x", "a.ts", &[1]);
    o.extend(occ("js/x", "b.ts", &[1]));
    let out = run(o, &b, Some("2026-10-18"), &["a.ts", "b.ts"]);
    assert_eq!(out.diagnostics[0].severity, Severity::Warn);
    assert_eq!(out.diagnostics[1].severity, Severity::Info);
}

#[test]
fn an_entry_inherits_the_deprecation_policy() {
    let b = baselines(DEPRECATED_RULE, "[[allow]]\nfile = \"a.ts\"\nsymbol = \"p#f\"\ncount = 1\n");
    let mut o = Occurrence::plain(d(DEPRECATED_RULE, "a.ts", 1));
    o.symbol = Some("p#f".into());
    o.policy = Policy { expires: Some(date("2027-01-02")), warn_from: None, warn_days: None, on_touch: Some(OnTouch::Keep) };
    let out = run(vec![o.clone()], &b, Some("2027-01-02"), &[]);
    assert_eq!(out.diagnostics[0].severity, Severity::Error);
    let out = run(vec![o], &b, Some("2026-01-02"), &["a.ts"]);
    assert_eq!(out.diagnostics[0].severity, Severity::Info, "on_touch = keep inherited");
}

#[test]
fn prune_only_lowers_and_add_records_current_counts() {
    let b = baselines("js/x", ONE);
    let c = counts(&occ("js/x", "a.ts", &[1]));
    let changed = prune(&b, &c, &all);
    assert_eq!(changed[0].entries[0].count, 1);
    // Nothing found at all: the entry goes.
    let changed = prune(&b, &counts(&[]), &all);
    assert!(changed[0].entries.is_empty());
    // More found than allowed: prune never raises.
    assert!(prune(&b, &counts(&occ("js/x", "a.ts", &[1, 2, 3])), &all).is_empty());
    // Not evaluated: kept.
    assert!(prune(&b, &counts(&[]), &|_, _| false).is_empty());
    let none = BTreeSet::new();
    let added = add(None, &counts(&occ("js/x", "z.ts", &[1, 2])), &AddOptions { rule: "js/x", files: &none, symbol: None, expires: Some(date("2027-07-01")), warn_from: None, warn_days: Some(5), on_touch: None, reason: Some("adopt"), added_by: Some("a@example.com") });
    assert_eq!(added.entries[0].count, 2);
    assert_eq!(added.entries[0].on_touch, Some(OnTouch::Error));
    assert_eq!(added.write(), "# The baseline of `js/x` (SR-BASELINE). Written by `codelaw baseline`;\n# review every change: a count may only go down.\n\n[[allow]]\nfile = \"z.ts\"\ncount = 2\nexpires = \"2027-07-01\"\nwarn_days = 5\non_touch = \"error\"\nreason = \"adopt\"\nadded_by = \"a@example.com\"\n");
}

#[test]
fn the_ratchet_refuses_every_loosening() {
    let before = baselines("js/x", ONE);
    assert!(loosened(&before, &before).is_empty());
    let after = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 3\nuntil = \"2027-04-30\"\non_touch = \"keep\"\n[[allow]]\nfile = \"b.ts\"\ncount = 1\n");
    let l = loosened(&before, &after);
    assert_eq!(l.len(), 4, "{l:?}");
    assert!(l.iter().any(|m| m.contains("count rose from 2 to 3")));
    assert!(l.iter().any(|m| m.contains("deadline extended, expires moved from 2027-04-01 to 2027-05-01")), "{l:?}");
    assert!(l.iter().any(|m| m.contains("on_touch loosened from error to keep")));
    assert!(l.iter().any(|m| m.contains("b.ts: entry added")));
    // Lowering and removing are always allowed.
    let lower = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 1\nuntil = \"2027-01-31\"\non_touch = \"error\"\n");
    assert!(loosened(&before, &lower).is_empty());
    // `until = D-1` rewritten as `expires = D` is no change; a reason or an
    // author added never loosens; `expires` one day later does.
    let same = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 2\nexpires = \"2027-04-01\"\nwarn_from = \"2027-03-01\"\non_touch = \"error\"\nreason = \"other\"\nadded_by = \"a@example.com\"\n");
    assert!(loosened(&before, &same).is_empty(), "{:?}", loosened(&before, &same));
    let later = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 2\nexpires = \"2027-04-02\"\non_touch = \"error\"\n");
    assert_eq!(loosened(&before, &later).len(), 1);
    assert!(loosened(&before, &Baselines::default()).is_empty());
}

#[test]
fn codeowners_coverage() {
    let p = ".codelaw/baseline/js--x.toml";
    assert!(codeowners_covers("/.codelaw/baseline/ @team\n", p));
    assert!(codeowners_covers(".codelaw/ @team\n", p));
    assert!(codeowners_covers("* @everyone\n", p));
    assert!(codeowners_covers("/.codelaw/baseline/*.toml @team\n", p));
    assert!(!codeowners_covers("/src/ @team\n", p));
    assert!(!codeowners_covers("* @everyone\n/.codelaw/baseline/\n", p), "a later line with no owners takes them away");
}

/// The veto check of SR-BASELINE (was B-014): can a new use offset a
/// removed one? The counts alone cannot tell, so with nothing touched the
/// swap passes; the touch policy is what closes it — the swap edits the
/// file, so a change set that names it makes every occurrence an error.
#[test]
fn veto_a_new_use_offsetting_a_removed_one() {
    let b = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 2\non_touch = \"error\"\n");
    // Before: uses on lines 1 and 2. After: line 2 removed, line 9 added.
    let swapped = || occ("js/x", "a.ts", &[1, 9]);
    let plain = run(swapped(), &b, Some("2026-10-18"), &[]);
    assert!(plain.diagnostics.iter().all(|d| d.severity == Severity::Info), "without a change set the offset passes");
    let ci = run(swapped(), &b, Some("2026-10-18"), &["a.ts"]);
    assert!(ci.diagnostics.iter().all(|d| d.severity == Severity::Error), "with the file in the change set it fails");
    let keep = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 2\non_touch = \"keep\"\n");
    let ci_keep = run(swapped(), &keep, Some("2026-10-18"), &["a.ts"]);
    assert!(ci_keep.diagnostics.iter().all(|d| d.severity == Severity::Info), "on_touch = keep reopens the hole");
}

#[test]
fn the_registry_and_its_defaults() {
    let r = Registry::parse(
        r#"
[[deprecated]]
symbol = "@acme/utils#formatDate"
replacement = "@acme/utils#formatISO"
until = "2027-03-31"
warn_from = "2027-03-01"
on_touch = "error"
reason = "Timezone bugs"
rewrite = { from = "formatDate($D)", to = "formatISO($D)" }
"#,
    )
    .unwrap();
    assert_eq!(r.deprecated[0].policy.expires, Some(date("2027-04-01")));
    assert_eq!(r.deprecated[0].line, 2);
    assert!(r.deprecated[0].rewrite.is_some());
    assert!(Registry::parse("[[deprecated]]\nsymbol = \"nohash\"\n").is_err());
    assert!(Registry::parse("[[deprecated]]\nsymbol = \"a#b\"\nuntil = \"2027-01-01\"\nwarn_from = \"2027-02-01\"\n").unwrap_err().contains("not before"));
    let t = |until, expires, on_touch| registry::DefaultsTable { until, expires, on_touch, ..Default::default() };
    assert!(registry::defaults(&t(Some("2027-13-01"), None, None)).unwrap_err().contains("[deprecation]: `until`"));
    assert!(registry::defaults(&t(None, None, Some("never"))).unwrap_err().contains("on_touch"));
    assert_eq!(registry::defaults(&t(Some("2027-01-31"), None, None)).unwrap().expires, Some(date("2027-02-01")));
    assert_eq!(registry::defaults(&t(None, Some("2027-02-01"), None)).unwrap().expires, Some(date("2027-02-01")));
    assert!(registry::defaults(&t(Some("2027-01-31"), Some("2027-02-01"), None)).is_err());
    assert_eq!(Registry::parse(&registry::entry_text(&r.deprecated[0])).unwrap().deprecated[0].policy, r.deprecated[0].policy);
}

/// Decision 9's day boundary, written first (the item's hazard): `expires`
/// is the first failing day; the warning starts `warn_days` before it.
#[test]
fn the_expiry_timeline_and_its_day_boundary() {
    let e = Some(date("2026-11-15"));
    let at = |d: &str| exception::expiry(Some(date(d)), e, None, 3);
    assert_eq!(at("2026-11-11"), Expiry::Holds, "D-4");
    assert_eq!(at("2026-11-12"), Expiry::Expiring { days: 3, expires: date("2026-11-15") }, "D-3");
    assert_eq!(at("2026-11-14"), Expiry::Expiring { days: 1, expires: date("2026-11-15") }, "D-1");
    assert_eq!(at("2026-11-15"), Expiry::Expired { expires: date("2026-11-15") }, "D");
    assert_eq!(at("2027-01-01"), Expiry::Expired { expires: date("2026-11-15") });
    // `warn_from` wins over `warn_days`; no as-of date, no verdict.
    assert_eq!(exception::expiry(Some(date("2026-11-01")), e, Some(date("2026-11-01")), 3), Expiry::Expiring { days: 14, expires: date("2026-11-15") });
    assert_eq!(exception::expiry(None, e, None, 3), Expiry::Holds);
    assert_eq!(exception::expiry(Some(date("2026-11-15")), None, None, 3), Expiry::Holds);
    // `warn_days = 0`: no warning, straight to the error.
    assert_eq!(exception::expiry(Some(date("2026-11-14")), e, None, 0), Expiry::Holds);
    assert_eq!(exception::when(1, date("2026-11-15")), "in 1 day (2026-11-15)");
    assert_eq!(exception::when(2, date("2026-11-15")), "in 2 days (2026-11-15)");
}

#[test]
fn until_d_minus_1_and_expires_d_are_the_same_entry() {
    let a = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 1\nuntil = \"2026-11-14\"\n");
    let b = baselines("js/x", "[[allow]]\nfile = \"a.ts\"\ncount = 1\nexpires = \"2026-11-15\"\n");
    assert_eq!(a, b);
    for day in ["2026-11-11", "2026-11-12", "2026-11-15"] {
        let x = run(occ("js/x", "a.ts", &[1]), &a, Some(day), &[]);
        let y = run(occ("js/x", "a.ts", &[1]), &b, Some(day), &[]);
        assert_eq!(format!("{:?}{:?}", x.diagnostics, x.notes), format!("{:?}{:?}", y.diagnostics, y.notes), "{day}");
    }
}

#[test]
fn emails_and_header_lines() {
    for ok in ["a@example.com", "first.last+tag@sub.example.org"] {
        assert!(exception::valid_email(ok), "{ok}");
    }
    for bad in ["", "a", "a@b", "@example.com", "a@.com", "a@example.", "a b@example.com", "a@@example.com", "<a@example.com>"] {
        assert!(!exception::valid_email(bad), "{bad}");
    }
    assert_eq!(exception::header_lines("# x\n[[allow]]\nfile = 1\n\n [[ allow ]] # two\n", "allow"), vec![2, 5]);
    assert_eq!(Exceptions::from_table(None, None).unwrap(), Exceptions { require_expiry: false, warn_days: 3 });
    assert!(Exceptions::from_table(Some(true), Some(-1)).is_err());
}
