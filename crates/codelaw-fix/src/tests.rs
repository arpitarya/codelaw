use super::*;
use codelaw_core::{Language, TextEdit};
use codelaw_lang::Grammar;
use codelaw_match::{Match, compile};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Compile a `codelaw` block and return a closure listing its matches.
fn matcher(language: Language, block: &str) -> (codelaw_rules::RuleFile, codelaw_match::CompiledMatch) {
    let ext = language.extension();
    let text = format!(
        "---\nid: t/r\ntitle: t\nlanguages: [{}]\nfix: auto\n---\n\nSummary.\n\n**Why:** test.\n\n```codelaw\n{block}\n```\n\n## Examples\n\n### {}\n\n#### Don't\n\n```{ext}\nx\n```\n\n#### Do\n\n```{ext}\ny\n```\n",
        language.name(),
        language.display()
    );
    let rule = codelaw_rules::parse_rule(Path::new("t/r.md"), &text, Some("t/r")).unwrap_or_else(|e| panic!("{e:?}"));
    let compiled = compile(&rule, Grammar::for_language(language), &BTreeMap::new()).unwrap();
    (rule, compiled)
}

fn find(language: Language, compiled: &codelaw_match::CompiledMatch, root: &codelaw_lang::Root, src: &str) -> Vec<Match> {
    let model = codelaw_semantic::build(language, root, src);
    root.root().dfs().filter_map(|n| compiled.try_match(n, model.as_ref())).collect()
}

/// Run one rule's fix to a fixpoint.
fn fix(language: Language, block: &str, src: &str) -> LoopOutcome {
    let (rule, compiled) = matcher(language, block);
    let spec = parse_fix(rule.block.fix.as_ref().unwrap()).unwrap();
    fix_loop(language, src, |text, _| {
        let root = codelaw_lang::parse(language, text);
        let model = codelaw_semantic::build(language, &root, text);
        let ctx = FixContext { source: text, root: &root, language, semantic: model.as_ref() };
        let matches = find(language, &compiled, &root, text);
        if spec.is_group() {
            if matches.is_empty() {
                return vec![];
            }
            vec![Candidate { rule: "t/r".into(), edits: group_edits(&spec, &matches, &ctx).unwrap() }]
        } else {
            matches.iter().map(|m| Candidate { rule: "t/r".into(), edits: edits_for(&spec, m, &ctx).unwrap() }).collect()
        }
    })
}

#[test]
fn apply_and_overlap() {
    assert_eq!(apply_edits("abcdef", &[TextEdit::new(1, 2, "X"), TextEdit::insert(4, "Y")]).unwrap(), "aXcdYef");
    assert!(apply_edits("abcdef", &[TextEdit::new(1, 3, "X"), TextEdit::new(2, 4, "Y")]).is_err());
    assert!(overlaps(&[TextEdit::insert(3, "a")], &[TextEdit::new(2, 4, "b")]));
    assert!(!overlaps(&[TextEdit::new(0, 1, "a")], &[TextEdit::new(1, 2, "b")]));
}

#[test]
fn exports_at_end() {
    let block = "match:\n  kind: export_statement\n  has: { field: declaration, pattern: $DECL }\n  not: { has: { regex: ^default$ } }\nmessage: \"export at the end\"\nfix:\n  - strip-export-keyword: $DECL\n  - collect-into-block:\n      target: $DECL\n      block: \"export { $$$NAMES };\"\n      type_block: \"export type { $$$TYPE_NAMES };\"\n      position: end-of-file";
    let src = "/** Adds. */\nexport function add(a: number, b: number) { return a + b; }\nexport interface Point { x: number }\nexport const a = 1, b = 2;\n";
    let out = fix(Language::Typescript, block, src);
    assert_eq!(
        out.text,
        "/** Adds. */\nfunction add(a: number, b: number) { return a + b; }\ninterface Point { x: number }\nconst a = 1, b = 2;\nexport { add, a, b };\nexport type { Point };\n"
    );
    assert_eq!(out.passes, 1);
    assert!(out.failed.is_empty());
}

#[test]
fn remove_takes_the_comma_and_the_line() {
    let block = "match:\n  kind: number\n  regex: ^2$\nmessage: two\nfix: [remove]";
    assert_eq!(fix(Language::Javascript, block, "f(1, 2, 3);\n").text, "f(1, 3);\n");
    assert_eq!(fix(Language::Javascript, block, "f(1, 2);\n").text, "f(1);\n");
    let stmt = "match:\n  kind: expression_statement\n  has: { kind: call_expression, regex: ^debug }\nmessage: debug\nfix: [remove]";
    assert_eq!(fix(Language::Javascript, stmt, "a();\n  debug(x);\nb();\n").text, "a();\nb();\n");
}

#[test]
fn a_fix_that_breaks_the_parse_is_refused() {
    let block = "match:\n  kind: number\nmessage: n\nfix: \"(\"";
    let out = fix(Language::Javascript, block, "const a = 1;\n");
    assert_eq!(out.text, "const a = 1;\n");
    assert_eq!(out.failed, BTreeSet::from([("t/r".to_string(), Failure::VerifyFailed)]));
}

#[test]
fn alternating_fixes_are_a_conflict() {
    // a -> b and b -> a in one rule: the file comes back to where it was.
    let block = "match:\n  kind: identifier\n  regex: ^[ab]$\nmessage: flip\nfix: \"x\"";
    let (rule, compiled) = matcher(Language::Javascript, block);
    let _ = rule;
    let out = fix_loop(Language::Javascript, "a;\n", |text, _| {
        let root = codelaw_lang::parse(Language::Javascript, text);
        find(Language::Javascript, &compiled, &root, text)
            .into_iter()
            .map(|m| Candidate { rule: "t/r".into(), edits: vec![TextEdit::new(m.start, m.end, if &text[m.start..m.end] == "a" { "b" } else { "a" })] })
            .collect()
    });
    assert_eq!(out.text, "b;\n");
    assert!(out.failed.contains(&("t/r".to_string(), Failure::Conflict)));
}

#[test]
fn rename_symbol_local() {
    let block = "match:\n  kind: variable_declarator\n  has: { field: name, regex: ^tmp$, pattern: $N }\nmessage: rename\nfix:\n  - rename-symbol-local: { target: $N, to: total }";
    assert_eq!(fix(Language::Javascript, block, "let tmp = 1;\ntmp += 2;\nlog(tmp, o.tmp);\n").text, "let total = 1;\ntotal += 2;\nlog(total, o.tmp);\n");
    let (rule, compiled) = matcher(Language::Javascript, block);
    let spec = parse_fix(rule.block.fix.as_ref().unwrap()).unwrap();
    let src = "export let tmp = 1;\n";
    let root = codelaw_lang::parse(Language::Javascript, src);
    let model = codelaw_semantic::build(Language::Javascript, &root, src);
    let ctx = FixContext { source: src, root: &root, language: Language::Javascript, semantic: model.as_ref() };
    let m = &find(Language::Javascript, &compiled, &root, src)[0];
    assert!(edits_for(&spec, m, &ctx).unwrap_err().contains("exported"));
}

#[test]
fn bad_fix_specs() {
    assert!(parse_fix(&serde_json::json!([{"strip-export": "$X"}])).unwrap_err().contains("did you mean `strip-export-keyword`"));
    assert!(parse_fix(&serde_json::json!([{"replace": {"target": "$X"}}])).unwrap_err().contains("needs `with`"));
    assert!(parse_fix(&serde_json::json!({"external": "x", "y": 1})).is_err());
    // A multi-byte name is an error, not a panic (W-17).
    for name in ["fixé", "éééé-x", "a😀", ""] {
        assert!(parse_fix(&serde_json::json!([{ name: "$X" }])).unwrap_err().contains("unknown fix primitive"), "{name}");
    }
}

#[cfg(unix)]
#[test]
fn external_fixer_contract() {
    let (_, compiled) = matcher(Language::Javascript, "match:\n  kind: number\nmessage: n\nfix: { external: x }");
    let src = "f(1);\n";
    let root = codelaw_lang::parse(Language::Javascript, src);
    let matches = find(Language::Javascript, &compiled, &root, src);
    let call = |command: &'static str| ExternalCall {
        command,
        cwd: Path::new("."),
        timeout: std::time::Duration::from_secs(5),
        rule: "t/r",
        file: "a.js",
        language: "javascript",
        source: src,
        matches: &matches,
    };
    let json = request_json(&call("x"));
    assert!(json.starts_with("{\"schema\":1,\"rule\":\"t/r\""), "{json}");
    let edits = run_external(&call("cat >/dev/null; echo '{\"edits\":[{\"range\":[2,3],\"text\":\"42\"}]}'")).unwrap();
    assert_eq!(edits, vec![TextEdit::new(2, 3, "42")]);
    assert!(run_external(&call("cat >/dev/null; exit 3")).unwrap_err().contains("exited"));
    let slow = ExternalCall { timeout: std::time::Duration::from_millis(100), ..call("sleep 5") };
    assert!(run_external(&slow).unwrap_err().contains("timed out"));
    // W-17: a fixer that writes far more to stderr than a pipe holds is
    // not blocked; its last stderr line names a failure.
    let chatty = "cat >/dev/null; i=0; while [ $i -lt 4000 ]; do echo 'noise noise noise noise noise noise noise noise noise noise' >&2; i=$((i+1)); done; echo '{\"edits\":[]}'";
    assert_eq!(run_external(&call(chatty)).unwrap(), vec![]);
    assert!(run_external(&call("cat >/dev/null; echo 'bad input' >&2; exit 1")).unwrap_err().contains("bad input"));
    // A grandchild that keeps stdout open neither hangs the run nor
    // outlives the timeout: the whole process group is stopped.
    let started = std::time::Instant::now();
    let held = run_external(&call("cat >/dev/null; (sleep 30) & echo '{\"edits\":[]}'"));
    assert_eq!(held.unwrap(), vec![]);
    let slow_tree = ExternalCall { timeout: std::time::Duration::from_millis(200), ..call("cat >/dev/null; sleep 30 & sleep 30") };
    assert!(run_external(&slow_tree).unwrap_err().contains("timed out"));
    assert!(started.elapsed() < std::time::Duration::from_secs(10), "{:?}", started.elapsed());
}

#[test]
fn the_trace_names_every_step() {
    let block = "match:\n  kind: identifier\n  regex: ^[ab]$\nmessage: flip\nfix: \"x\"";
    let (_, compiled) = matcher(Language::Javascript, block);
    let mut trace = Some(crate::LoopTrace::default());
    let out = crate::fix_loop_traced(
        Language::Javascript,
        "a;\n",
        |text, _| {
            let root = codelaw_lang::parse(Language::Javascript, text);
            find(Language::Javascript, &compiled, &root, text)
                .into_iter()
                .map(|m| Candidate { rule: "t/r".into(), edits: vec![TextEdit::new(m.start, m.end, if &text[m.start..m.end] == "a" { "b" } else { "a" })] })
                .collect()
        },
        &mut trace,
    );
    use crate::TraceEvent::*;
    let trace = trace.unwrap();
    assert_eq!(out.text, "b;\n");
    assert_eq!(trace.end, crate::LoopEnd::Conflict);
    let r = || "t/r".to_string();
    assert_eq!(
        trace.passes,
        vec![
            vec![Offered { rule: r(), edits: 1 }, Applied { rule: r(), edits: 1 }],
            vec![Offered { rule: r(), edits: 1 }, Applied { rule: r(), edits: 1 }, Refused { rule: r(), why: Failure::Conflict }],
        ]
    );
}
