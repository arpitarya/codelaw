//! The `target:` fixes, each checked to leave every byte outside its edit
//! as it was.

use super::*;
use serde_json::json;

/// Run `fix` at `target` where `assert` fails; the fixed text.
fn fixed(language: Language, src: &str, target: &str, assert: J, fix: J) -> Result<String, String> {
    let tables = Tables::new();
    let pred = Pred::parse(&assert, &tables)?;
    let steps = parse_data_fix(&fix, &tables)?;
    let root = codelaw_lang::parse(language, src);
    let tree = codelaw_match::data::build(language, root.root().get_inner_node(), src).ok_or("no tree")?;
    let doc = tree.documents[0];
    let slot = resolve(&tree, doc, &[], &Path::parse(target)?).into_iter().next().ok_or("no slot")?;
    assert!(!pred.holds(&tree, doc, &slot, &mut Vec::new()), "the assertion already holds");
    let ctx = DataFixContext { source: src, language, tree: &tree, document: doc, assert: &pred };
    let edits = data_edits(&steps, &ctx, &slot)?;
    // Byte for byte outside the edited range.
    let (lo, hi) = (edits.first().unwrap().start, edits.last().unwrap().end);
    let out = apply_edits(src, &edits)?;
    assert_eq!(&out[..lo], &src[..lo]);
    assert_eq!(&out[out.len() - (src.len() - hi)..], &src[hi..]);
    Ok(out)
}

#[test]
fn yaml_sort_keeps_comments_with_their_keys() {
    let src = "# head\nname: x\ndeps:\n  # the web framework\n  zeta: 1 # pinned\n  alpha:\n    nested: true\n\n  # beta comes last\n  beta: 2\ntail: y # end\n";
    let out = fixed(Language::Yaml, src, "deps", json!({"sorted": true}), json!(["sort-keys"])).unwrap();
    // The blank line between the items stays where it was.
    assert_eq!(out, "# head\nname: x\ndeps:\n  alpha:\n    nested: true\n\n  # beta comes last\n  beta: 2\n  # the web framework\n  zeta: 1 # pinned\ntail: y # end\n");
}

#[test]
fn yaml_sort_at_end_of_file_without_newline() {
    let out = fixed(Language::Yaml, "b: 1\na: 2", "$", json!({"sorted": true}), json!(["sort-keys"])).unwrap();
    assert_eq!(out, "a: 2\nb: 1");
}

#[test]
fn toml_sort_keeps_comments_and_other_tables() {
    let src = "# top\n[deps]\n# web\nzeta = \"1\" # pinned\nalpha = [\n  \"x\",\n]\n\n[other]\nb = 1\na = 2\n";
    let out = fixed(Language::Toml, src, "deps", json!({"sorted": true}), json!(["sort-keys"])).unwrap();
    assert_eq!(out, "# top\n[deps]\nalpha = [\n  \"x\",\n]\n# web\nzeta = \"1\" # pinned\n\n[other]\nb = 1\na = 2\n");
}

#[test]
fn arrays_of_strings_sort() {
    let out = fixed(Language::Toml, "[project]\ndependencies = [\"b\", \"a\"]\n", "project.dependencies", json!({"sorted": true}), json!(["sort-keys"])).unwrap();
    assert_eq!(out, "[project]\ndependencies = [\"a\", \"b\"]\n");
    let out = fixed(Language::Yaml, "list:\n  - b\n  # a\n  - a\n", "list", json!({"sorted": true}), json!(["sort-keys"])).unwrap();
    assert_eq!(out, "list:\n  # a\n  - a\n  - b\n");
}

#[test]
fn sort_with_listed_keys_first() {
    let src = "{\n  \"version\": \"1\",\n  \"b\": 1,\n  \"name\": \"x\"\n}\n";
    let spec = json!({"first": ["name", "version"]});
    let out = fixed(Language::Json, src, "$", json!({"sorted": spec}), json!([{"sort-keys": spec}])).unwrap();
    assert_eq!(out, "{\n  \"name\": \"x\",\n  \"version\": \"1\",\n  \"b\": 1\n}\n");
}

#[test]
fn comments_between_bracketed_items_refuse_the_sort() {
    let e = fixed(Language::Json, "{\"b\": 1, /* c */ \"a\": 2}", "$", json!({"sorted": true}), json!(["sort-keys"])).unwrap_err();
    assert!(e.contains("comments"), "{e}");
    let e = fixed(Language::Toml, "x = [\n  \"b\", # c\n  \"a\",\n]\n", "x", json!({"sorted": true}), json!(["sort-keys"])).unwrap_err();
    assert!(e.contains("comments"), "{e}");
}

#[test]
fn ensure_key_in_each_language() {
    let present = json!({"path": {"strict": {"present": true, "equals": true}}});
    let fix = json!([{"ensure-key": {"key": "strict", "value": true}}]);
    let out = fixed(Language::Json, "{\n  \"compilerOptions\": {\n    \"target\": \"es2022\"\n  }\n}\n", "compilerOptions", present.clone(), fix.clone()).unwrap();
    assert_eq!(out, "{\n  \"compilerOptions\": {\n    \"target\": \"es2022\",\n    \"strict\": true\n  }\n}\n");
    let out = fixed(Language::Yaml, "opts:\n  target: es2022 # t\nother: 1\n", "opts", present.clone(), fix.clone()).unwrap();
    assert_eq!(out, "opts:\n  target: es2022 # t\n  strict: true\nother: 1\n");
    let out = fixed(Language::Toml, "[opts]\ntarget = \"es2022\" # t\n\n[other]\nx = 1\n", "opts", present.clone(), fix.clone()).unwrap();
    assert_eq!(out, "[opts]\ntarget = \"es2022\" # t\nstrict = true\n\n[other]\nx = 1\n");
    // A value that is there but wrong is replaced.
    let out = fixed(Language::Json, "{\"opts\": {\"strict\": false}}", "opts", present, fix).unwrap();
    assert_eq!(out, "{\"opts\": {\"strict\": true}}");
}

#[test]
fn ensure_key_writes_missing_parents() {
    let a = json!({"present": true, "equals": ">=3.11"});
    let fix = json!([{"ensure-key": {"value": ">=3.11"}}]);
    let out = fixed(Language::Toml, "[tool.ruff]\nline-length = 100\n", "project.requires-python", a.clone(), fix.clone()).unwrap();
    assert_eq!(out, "[tool.ruff]\nline-length = 100\n\n[project]\nrequires-python = \">=3.11\"\n");
    let out = fixed(Language::Json, "{\n  \"name\": \"x\"\n}\n", "engines.node", a.clone(), fix.clone()).unwrap();
    assert_eq!(out, "{\n  \"name\": \"x\",\n  \"engines\": {\n    \"node\": \">=3.11\"\n  }\n}\n");
    let out = fixed(Language::Yaml, "name: x\n", "engines.node", a, fix).unwrap();
    assert_eq!(out, "name: x\nengines:\n  node: \">=3.11\"\n");
}

#[test]
fn when_picks_a_branch() {
    let a = json!({"any": [{"path": {"$.extends": {"present": true}}}, {"path": {"strict": {"present": true, "equals": true}}}]});
    let fix = json!([{"when": {"path": {"strict": {"present": true}}}, "then": [{"ensure-key": {"key": "strict", "value": true}}], "else": [{"ensure-key": {"key": "strict", "value": true}}, "sort-keys"]}]);
    let out = fixed(Language::Json, "{\"opts\": {\"strict\": false}}", "opts", a.clone(), fix.clone()).unwrap();
    assert_eq!(out, "{\"opts\": {\"strict\": true}}");
    // `else` runs both steps in order: the sort places the key just written.
    let out = fixed(Language::Json, "{\"opts\": {\"x\": 1, \"a\": 2}}", "opts", a.clone(), fix.clone()).unwrap();
    assert_eq!(out, "{\"opts\": {\"a\": 2, \"strict\": true, \"x\": 1}}");
    // A container already in order needs only the key.
    let out = fixed(Language::Yaml, "opts:\n  a: 1 # one\n  x: 2\n", "opts", a.clone(), fix).unwrap();
    assert_eq!(out, "opts:\n  a: 1 # one\n  strict: true\n  x: 2\n");
    let fix = json!([{"when": {"path": {"strict": {"present": true}}}, "then": [{"ensure-key": {"key": "strict", "value": true}}]}]);
    let e = fixed(Language::Json, "{\"opts\": {\"x\": 1}}", "opts", a.clone(), fix).unwrap_err();
    assert!(e.contains("nothing to change"), "{e}");
    let fix = json!([{"when": {"path": {"strict": {"present": true}}}, "then": [], "else": [{"ensure-key": {"key": "strict", "value": true}}]}]);
    let out = fixed(Language::Json, "{\"opts\": {\"x\": 1}}", "opts", a, fix).unwrap();
    assert_eq!(out, "{\"opts\": {\"x\": 1, \"strict\": true}}");
}

#[test]
fn remove_and_replace() {
    let a = json!({"present": false});
    let out = fixed(Language::Json, "{\n  \"a\": 1,\n  \"b\": 2\n}\n", "b", a.clone(), json!(["remove"])).unwrap();
    assert_eq!(out, "{\n  \"a\": 1\n}\n");
    let out = fixed(Language::Yaml, "a: 1 # one\nb: 2\n", "a", a.clone(), json!(["remove"])).unwrap();
    assert_eq!(out, "b: 2\n");
    let out = fixed(Language::Toml, "a = 1\nb = { x = 1, y = 2 }\n", "b.x", a, json!(["remove"])).unwrap();
    assert_eq!(out, "a = 1\nb = { y = 2 }\n");
    let out = fixed(Language::Yaml, "on: push\nlevel: 'x'\n", "level", json!({"type": "integer"}), json!([{"replace": "3"}])).unwrap();
    assert_eq!(out, "on: push\nlevel: 3\n");
}

#[test]
fn a_fix_that_changes_other_values_is_refused() {
    let e = fixed(Language::Json, "{\"a\": {\"b\": 1}, \"c\": 2}", "a", json!({"type": "integer"}), json!([{"replace": "1, \"d\": 3"}])).unwrap_err();
    assert!(e.contains("change values") || e.contains("parse"), "{e}");
    let e = fixed(Language::Json, "{\"a\": \"x\"}", "a", json!({"type": "integer"}), json!([{"replace": "\"y\""}])).unwrap_err();
    assert!(e.contains("does not make the assertion hold"), "{e}");
}

#[test]
fn parse_errors() {
    let t = Tables::new();
    assert!(parse_data_fix(&json!(["sort-key"]), &t).unwrap_err().contains("not a fix primitive"));
    assert!(parse_data_fix(&json!([{"ensure-key": {"value": 1, "at": "x"}}]), &t).unwrap_err().contains("not `at`"));
    assert!(parse_data_fix(&json!([{"when": {"present": true}}]), &t).unwrap_err().contains("needs `then:`"));
    assert!(parse_data_fix(&json!({"external": "x"}), &t).is_err());
}
