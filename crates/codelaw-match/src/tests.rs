use super::*;
use codelaw_core::Language;
use std::path::Path;

fn rule(language: Language, block: &str) -> RuleFile {
    let ext = language.extension();
    let display = language.display();
    let text = format!(
        "---\nid: t/r\ntitle: t\nlanguages: [{}]\nfix: manual\nfix_reason: test\n---\n\nSummary.\n\n**Why:** test.\n\n```codelaw\n{block}\n```\n\n## How to fix\n\n1. Fix it.\n\n## Examples\n\n### {display}\n\n#### Don't\n\n```{ext}\nx\n```\n\n#### Do\n\n```{ext}\ny\n```\n",
        language.name()
    );
    parse_rule_ok(&text)
}

fn parse_rule_ok(text: &str) -> RuleFile {
    codelaw_rules::parse_rule(Path::new("t/r.md"), text, Some("t/r")).unwrap_or_else(|e| panic!("{e:?}"))
}

fn matches(language: Language, block: &str, src: &str) -> Vec<String> {
    let r = rule(language, block);
    let grammar = Grammar::for_language(language);
    let compiled = compile(&r, grammar, &BTreeMap::new()).unwrap();
    let root = codelaw_lang::parse(language, src);
    let model = codelaw_semantic::build(language, &root, src);
    let kinds = compiled.kinds();
    root.root()
        .dfs()
        .filter(|n| kinds.as_ref().is_none_or(|k| k.contains(&n.kind_id())))
        .filter_map(|n| compiled.try_match(n, model.as_ref()))
        .map(|m| interpolate(r.block.message.as_deref().unwrap(), &m, src))
        .collect()
}

#[test]
fn kind_filter_and_relational_clauses() {
    let block = "match:\n  kind: export_statement\n  has: { field: declaration, pattern: $DECL }\nmessage: \"inline export: $DECL\"";
    let src = "export function a() {}\nconst b = 1;\nexport { b };\nexport const c = 2;\n";
    assert_eq!(matches(Language::Typescript, block, src), ["inline export: function a() {}", "inline export: const c = 2;"]);
    let r = rule(Language::Typescript, block);
    let k = compile(&r, Grammar::TypeScript, &BTreeMap::new()).unwrap().kinds().unwrap();
    assert_eq!(k.len(), 1);
}

#[test]
fn unused_predicate_on_a_capture() {
    let block = "match:\n  kind: variable_declarator\n  has: { field: name, pattern: $NAME }\nwhere:\n  $NAME: unused\nmessage: \"`$NAME` is never used\"";
    let src = "const a = 1;\nconst b = 2;\nconsole.log(b);\n";
    assert_eq!(matches(Language::Javascript, block, src), ["`a` is never used"]);
}

#[test]
fn undeclared_with_globals_and_match_key() {
    let block = "match:\n  kind: identifier\nwhere:\n  $$MATCH: { undeclared: { globals: [window] } }\nmessage: \"`$$MATCH` undeclared\"";
    let r = rule(Language::Javascript, block);
    let compiled = compile(&r, Grammar::JavaScript, &BTreeMap::new()).unwrap();
    let src = "window.x; document.y; let z; z;\n";
    let root = codelaw_lang::parse(Language::Javascript, src);
    let model = codelaw_semantic::build(Language::Javascript, &root, src).unwrap();
    let hits: Vec<String> = root.root().dfs().filter_map(|n| compiled.try_match(n, Some(&model))).map(|m| src[m.start..m.end].to_owned()).collect();
    assert_eq!(hits, ["document"]);
}

#[test]
fn imported_from_and_combinators() {
    let block = "match:\n  kind: call_expression\n  has: { field: function, pattern: $F }\nwhere:\n  $F: [{ imported-from: { module: react, name: useState } }, { not: reassigned }]\nmessage: \"hook $F\"";
    let src = "import { useState as us, useEffect } from 'react';\nus(1);\nuseEffect(f);\nlocal(2);\n";
    assert_eq!(matches(Language::Jsx, block, src), ["hook us"]);
}

#[test]
fn an_empty_multi_capture_interpolates_as_nothing() {
    let block = "match:\n  pattern: new Symbol($$$ARGS)\nmessage: \"Symbol($$$ARGS)\"";
    assert_eq!(matches(Language::Javascript, block, "new Symbol();\nnew Symbol('a', b);\n"), ["Symbol()", "Symbol('a', b)"]);
}

#[test]
fn python_patterns_use_the_expando() {
    let block = "match:\n  pattern: print($$$ARGS)\nmessage: \"print of $$$ARGS\"";
    assert_eq!(matches(Language::Python, block, "print(a, b)\nlog(c)\n"), ["print of a, b"]);
}

#[test]
fn toml_and_yaml_and_css_match() {
    assert_eq!(matches(Language::Toml, "match:\n  kind: bare_key\n  regex: ^name$\nmessage: key $$MATCH", "[project]\nname = \"x\"\n"), ["key name"]);
    assert_eq!(matches(Language::Css, "match:\n  kind: important\nmessage: important", "a { color: red !important; }\n"), ["important"]);
    assert_eq!(
        matches(Language::Yaml, "match:\n  kind: block_mapping_pair\n  has: { field: key, regex: ^on$ }\nmessage: trigger", "on: push\njobs: {}\n"),
        ["trigger"]
    );
}

#[test]
fn where_on_a_list_capture_or_a_language_without_a_model_is_refused() {
    let r = rule(Language::Javascript, "match:\n  pattern: f($$$ARGS)\nwhere:\n  $ARGS: unused\nmessage: x");
    assert!(compile(&r, Grammar::JavaScript, &Default::default()).err().unwrap().contains("captures a list"));
    let r = rule(Language::Javascript, "match:\n  pattern: f($$$ARGS)\nwhere:\n  $$$ARGS: unused\nmessage: x");
    assert!(compile(&r, Grammar::JavaScript, &Default::default()).is_err());
    let r = rule(Language::Css, "match:\n  kind: identifier\nwhere:\n  $$MATCH: unused\nmessage: x");
    assert!(compile(&r, Grammar::Css, &Default::default()).err().unwrap().contains("semantic model"));
}

#[test]
fn bad_where_is_a_compile_error() {
    let r = rule(Language::Javascript, "match:\n  kind: identifier\nwhere:\n  $NAME: unusd\nmessage: x");
    let e = compile(&r, Grammar::JavaScript, &BTreeMap::new()).err().unwrap();
    assert!(e.contains("did you mean `unused`"), "{e}");
    let r = rule(Language::Javascript, "match:\n  kind: identifier\nwhere:\n  $NAME: unused\nmessage: x");
    let e = compile(&r, Grammar::JavaScript, &BTreeMap::new()).err().unwrap();
    assert!(e.contains("does not capture"), "{e}");
    // A kind no grammar has compiles to nothing here; the engine refuses it
    // when no grammar of the rule has it.
    let r = rule(Language::Javascript, "match:\n  kind: no_such_kind\nmessage: x");
    assert!(compile(&r, Grammar::JavaScript, &BTreeMap::new()).is_ok());
    assert_eq!(kinds_named(&r), ["no_such_kind"]);
}

#[test]
fn a_kind_missing_from_a_grammar_matches_nothing_there() {
    let block = "match:\n  kind: variable_declarator\n  not:\n    inside: { kind: ambient_declaration, stopBy: end }\nmessage: x";
    assert_eq!(matches(Language::Javascript, block, "let a = 1;\n").len(), 1);
    assert_eq!(matches(Language::Typescript, block, "let a = 1;\ndeclare const b: number;\n").len(), 1);
}

#[test]
fn data_tables_feed_undeclared() {
    let mut data = BTreeMap::new();
    data.insert("globals/browser".to_owned(), Arc::new(serde_json::json!(["document", "window"])));
    let r = rule(Language::Javascript, "match:\n  kind: identifier\nwhere:\n  $$MATCH: { undeclared: { tables: [globals/browser] } }\nmessage: x");
    assert!(compile(&r, Grammar::JavaScript, &data).is_ok());
    assert!(compile(&r, Grammar::JavaScript, &BTreeMap::new()).err().unwrap().contains("no data table"));
}
