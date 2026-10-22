//! SR-CONFIG's veto check (decisions 3, 6 and 7): every key in
//! `codelaw.schema.json` is defined in exactly one record, every
//! record-defined key is in the schema, and the schema and the loader accept
//! the same keys.

use codelaw_config::{Config, RawConfig, SCHEMA};
use regex::Regex;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn schema() -> Value {
    serde_json::from_str(SCHEMA).expect("codelaw.schema.json is JSON")
}

/// name (`SR-CONFIG`) → (file name, text), for every record.
fn records() -> BTreeMap<String, (String, String)> {
    let mut out = BTreeMap::new();
    for e in fs::read_dir(repo().join("records")).unwrap().flatten() {
        let file = e.file_name().to_string_lossy().into_owned();
        if !file.ends_with(".md") || !file.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        let text = fs::read_to_string(e.path()).unwrap();
        let name = text.lines().find_map(|l| l.strip_prefix("name: ")).expect("a record has a name").trim().to_owned();
        out.insert(name, (file, text));
    }
    out
}

fn props(v: &Value) -> &serde_json::Map<String, Value> {
    v["properties"].as_object().expect("properties")
}

fn validated(section: &Value) -> bool {
    section.get("x-codelaw-validated") != Some(&json!(false))
}

fn is_section(v: &Value) -> bool {
    (v.get("type") == Some(&json!("object")) && v.get("properties").is_some()) || !validated(v)
}

/// The keys serde accepts at one level: the `expected one of` list of the
/// error an unknown key provokes. `None` when the level accepts anything.
fn accepted(probe: Value) -> Option<BTreeSet<String>> {
    let err = serde_json::from_value::<RawConfig>(probe).err()?.to_string();
    let tail = err.split("expected").nth(1).unwrap_or_else(|| panic!("not an unknown-key error: {err}"));
    Some(tail.split('`').skip(1).step_by(2).map(str::to_owned).collect())
}

fn compile(schema: Value) -> (boon::Schemas, boon::SchemaIndex) {
    let mut schemas = boon::Schemas::new();
    let mut c = boon::Compiler::new();
    c.add_resource("file:///codelaw.schema.json", schema).unwrap();
    let idx = c.compile("file:///codelaw.schema.json", &mut schemas).unwrap_or_else(|e| panic!("the schema compiles: {e:#}"));
    (schemas, idx)
}

#[test]
fn the_schema_and_the_loader_accept_the_same_keys() {
    let s = schema();
    let top: BTreeSet<String> = props(&s).keys().cloned().collect();
    assert_eq!(accepted(json!({ "__probe__": 0 })).unwrap(), top, "top-level keys: RawConfig vs codelaw.schema.json");
    for (name, section) in props(&s) {
        if !is_section(section) {
            continue;
        }
        let got = accepted(json!({ name: { "__probe__": 0 } }));
        if validated(section) {
            let want: BTreeSet<String> = props(section).keys().cloned().collect();
            assert_eq!(got.as_ref(), Some(&want), "[{name}]: RawConfig vs codelaw.schema.json");
        } else {
            assert_eq!(got, None, "[{name}] is validated by the loader now: give it its keys in the schema and drop x-codelaw-validated");
        }
    }
    let item: BTreeSet<String> = props(&s["properties"]["overrides"]["items"]).keys().cloned().collect();
    assert_eq!(accepted(json!({ "overrides": [{ "files": [], "__probe__": 0 }] })).unwrap(), item, "[[overrides]]");
    let langs: Vec<&str> = s["properties"]["languages"]["items"]["enum"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(langs, codelaw_core::Language::ALL.iter().map(|l| l.name()).collect::<Vec<_>>(), "`languages` names every L3 language");
}

#[test]
fn every_key_is_defined_in_exactly_one_record() {
    let s = schema();
    let records = records();
    let config = &records["SR-CONFIG"].1;
    let decision3 = config.split("3. **Sections and owners.**").nth(1).unwrap().split("\n4. ").next().unwrap();
    // SR-CONFIG §1 names each section's record: `[format]   # SR-FORMATTER`.
    let owners: BTreeMap<String, String> = Regex::new(r"\[(\w+)\]\s+# (SR-[A-Z0-9-]+)")
        .unwrap()
        .captures_iter(config)
        .map(|c| (c[1].to_owned(), c[2].to_owned()))
        .collect();
    let mut sections = BTreeSet::new();
    for (key, v) in props(&s) {
        let record = v["x-codelaw-record"].as_str().unwrap_or_else(|| panic!("`{key}` names no x-codelaw-record"));
        assert!(records.contains_key(record), "`{key}`: {record} is not a record");
        if is_section(v) {
            sections.insert(key.clone());
            assert_eq!(owners.get(key).map(String::as_str), Some(record), "[{key}]: SR-CONFIG §1 and the schema disagree on its record");
        } else {
            assert_eq!(record, "SR-CONFIG", "top-level `{key}` belongs to SR-CONFIG (decision 3)");
            assert!(
                [format!("`{key}`"), format!("`[{key}]`"), format!("`[[{key}]]`")].iter().any(|m| decision3.contains(m.as_str())),
                "SR-CONFIG decision 3 does not list `{key}`"
            );
        }
    }
    assert_eq!(owners.keys().cloned().collect::<BTreeSet<_>>(), sections, "SR-CONFIG §1's sections vs the schema's");

    let mention = |section: &str, key: &str| {
        Regex::new(&format!(r"`(\[{section}\]\s+{key}|{section}\.{key})\b")).unwrap()
    };
    for (section, v) in props(&s).iter().filter(|(_, v)| is_section(v) && validated(v)) {
        for (key, kv) in props(v) {
            let owner = kv.get("x-codelaw-record").or(v.get("x-codelaw-record")).and_then(Value::as_str).unwrap();
            let (owner_file, owner_text) = &records[owner];
            let re = mention(section, key);
            assert!(re.is_match(owner_text), "{owner} does not define `[{section}] {key}` (write it as `[{section}] {key}` or `{section}.{key}`)");
            for (name, (_, text)) in &records {
                if name == owner {
                    continue;
                }
                for line in text.lines().filter(|l| re.is_match(l)) {
                    assert!(line.contains(owner_file.as_str()), "{name} defines `[{section}] {key}` too, which is {owner}'s; cite it instead:\n  {line}");
                }
            }
        }
    }
}

#[test]
fn every_record_defined_key_is_in_the_schema() {
    let s = schema();
    let validated_sections: BTreeMap<&String, &Value> = props(&s).iter().filter(|(_, v)| is_section(v) && validated(v)).collect();
    let re = Regex::new(r"`(?:\[([a-z_]+)\]\s+|([a-z_]+)\.)([a-z_]+)\b").unwrap();
    for (name, (_, text)) in records() {
        for c in re.captures_iter(&text) {
            let section = c.get(1).or(c.get(2)).unwrap().as_str();
            let key = &c[3];
            if let Some(v) = validated_sections.get(&section.to_owned()) {
                assert!(props(v).contains_key(key), "{name} defines `[{section}] {key}`, which codelaw.schema.json does not have");
            }
        }
    }
}

#[test]
fn the_schema_and_the_loader_agree_on_examples() {
    let (schemas, idx) = compile(schema());
    let full = r#"
extends = ["codelaw:js/recommended", "path:./team"]
languages = ["typescript", "python"]
include = ["src/**"]
exclude = ["dist/**"]
rules_dir = ".codelaw/rules"
root = true
requires = ">=0.1"
preview = false
allow_pack_fixers = ["@acme/rules"]
[rules]
"py/no-print" = "off"
"js/max" = { severity = "warn", options = { max = 3 } }
[[overrides]]
files = ["src/**/*.tsx"]
exclude = ["src/legacy/**"]
rules = { "js/exports-at-end" = "off" }
[files]
follow_symlinks = false
generated_markers = ["DO NOT EDIT"]
lint_generated = false
max_file_size = "1 MiB"
[cache]
enabled = true
dir = ".codelaw/cache"
max_size = 512000000
[fix]
external_timeout = "500ms"
[lint]
tagged_templates = true
[markdown]
lint_code_blocks = true
[format]
line_width = 100
"#;
    let config = repo().join("records/0110_config.md");
    let example = fs::read_to_string(&config).unwrap().split("```toml\n").nth(1).unwrap().split("```").next().unwrap().to_owned();
    for (what, text) in [("a config with every key", full), ("SR-CONFIG §1's example", example.as_str())] {
        Config::from_toml(PathBuf::from("/x"), text).unwrap_or_else(|e| panic!("{what}: the loader refuses it: {e}"));
        let v: Value = serde_json::to_value(toml::from_str::<toml::Table>(text).unwrap()).unwrap();
        if let Err(e) = schemas.validate(&v, idx) {
            panic!("{what}: the schema refuses it: {e:#}");
        }
    }
    for (what, bad) in [
        ("an unknown key", json!({ "extend": [] })),
        ("an unknown [files] key", json!({ "files": { "max_file_sise": 1 } })),
        ("a bad severity", json!({ "rules": { "a/b": "eror" } })),
        ("an unknown language", json!({ "languages": ["pyhton"] })),
        ("a bad size", json!({ "cache": { "max_size": "1 parsec" } })),
    ] {
        assert!(schemas.validate(&bad, idx).is_err(), "the schema accepts {what}");
        assert!(Config::from_toml(PathBuf::from("/x"), &toml::to_string(&bad).unwrap()).is_err(), "the loader accepts {what}");
    }
}
