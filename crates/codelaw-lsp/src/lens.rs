//! Inlay hints and CodeLens (W-137, SR-EDITOR decision 8): the badge at
//! the end of a manifest's or a marker's line, "latest" beside a clean
//! dependency, and a manifest's one-line summary with its bulk fixes.
//!
//! Ordinary code gets neither: there the squiggle is the whole signal
//! (docs/design/editor, board H). Everything here is built from the
//! diagnostics the engine already gave and from files it reads — the
//! manifest's text and the committed snapshot; nothing is judged anew.

use crate::card::is_manifest;
use lsp_types::{CodeLens, Command, InlayHint, InlayHintLabel, InlayHintTooltip, Position, Range};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The command a CodeLens runs to apply every fix of some kinds in one
/// file: `[uri, kinds, grades]` (`grades` empty: any).
pub const APPLY_FIXES: &str = "codelaw.applyFixes";

/// One diagnostic as shown: its range in the editor and its `data`.
pub struct Shown<'a> {
    pub range: Range,
    pub data: &'a Value,
    pub message: &'a str,
}

fn field<'v>(data: &'v Value, key: &str) -> Option<&'v str> {
    data.get(key).and_then(Value::as_str)
}

/// The end of line `line` of `text`, in UTF-16 units.
fn line_end(text: &str, line: u32) -> Position {
    let l = text.split('\n').nth(line as usize).unwrap_or("");
    let l = l.strip_suffix('\r').unwrap_or(l);
    Position { line, character: l.encode_utf16().count() as u32 }
}

/// A dependency with no finding, and the latest version the snapshot has.
#[derive(Debug, PartialEq, Eq)]
pub struct Clean {
    pub line: u32,
    pub name: String,
    pub latest: String,
    /// Whether the written requirement names the latest version.
    pub at_latest: bool,
}

/// The inlay hints of one document within `range`: one badge per line
/// that holds a manifest or marker finding (the most severe first), and
/// "latest" on each clean dependency when `clean` asks for it.
pub fn inlay_hints(text: &str, shown: &[Shown<'_>], clean: &[Clean], range: Range) -> Vec<InlayHint> {
    let mut by_line: BTreeMap<u32, &Shown<'_>> = BTreeMap::new();
    for s in shown {
        if !matches!(field(s.data, "view"), Some("manifest" | "marker")) {
            continue;
        }
        let line = s.range.start.line;
        by_line.entry(line).or_insert(s);
    }
    let within = |line: u32| line >= range.start.line && line <= range.end.line;
    let mut out: Vec<InlayHint> = by_line
        .into_iter()
        .filter(|(line, _)| within(*line))
        .map(|(line, s)| InlayHint {
            position: line_end(text, line),
            label: InlayHintLabel::String(field(s.data, "badge").unwrap_or_default().to_owned()),
            kind: None,
            text_edits: None,
            tooltip: Some(InlayHintTooltip::String(s.message.to_owned())),
            padding_left: Some(true),
            padding_right: None,
            data: Some(s.data.clone()),
        })
        .collect();
    for c in clean.iter().filter(|c| c.at_latest && within(c.line)) {
        out.push(InlayHint {
            position: line_end(text, c.line),
            label: InlayHintLabel::String("latest".into()),
            kind: None,
            text_edits: None,
            tooltip: Some(InlayHintTooltip::String(format!("{} is at the latest version the snapshot has, {}", c.name, c.latest))),
            padding_left: Some(true),
            padding_right: None,
            data: Some(json!({ "kind": "clean", "view": "manifest", "package": c.name, "latest": c.latest, "badge": "latest" })),
        });
    }
    out.sort_by_key(|h| (h.position.line, h.position.character));
    out
}

/// A manifest's CodeLens: a summary over its first line with a finding
/// (`codelaw · 2 vulnerable · 1 deprecated · 2 outdated · 2 up to date`)
/// and one lens per bulk fix. Nothing for any other file.
pub fn code_lenses(rel: &str, uri: &str, shown: &[Shown<'_>], clean: usize) -> Vec<CodeLens> {
    if !is_manifest(rel) || (shown.is_empty() && clean == 0) {
        return vec![];
    }
    let count = |kind: &str| shown.iter().filter(|s| field(s.data, "kind") == Some(kind)).count();
    let mut words = vec!["codelaw".to_owned()];
    for (kind, label) in [("vulnerable", "vulnerable"), ("deprecated", "deprecated"), ("licence", "licence denied"), ("outdated", "outdated"), ("runtime", "runtime mismatch")] {
        let n = count(kind);
        if n > 0 {
            words.push(format!("{n} {label}"));
        }
    }
    if clean > 0 {
        words.push(format!("{clean} up to date"));
    }
    let line = shown.iter().map(|s| s.range.start.line).min().unwrap_or(0);
    let at = Range { start: Position { line, character: 0 }, end: Position { line, character: 0 } };
    let lens = |title: String, command: &str, arguments: Option<Vec<Value>>| CodeLens {
        range: at,
        command: Some(Command { title, command: command.to_owned(), arguments }),
        data: None,
    };
    // The summary is words, not an action: an empty command.
    let mut out = vec![lens(words.join(" · "), "", None)];
    let fixable = |kind: &str, grades: &[&str]| {
        shown
            .iter()
            .filter(|s| field(s.data, "kind") == Some(kind) && s.data.get("fixable").is_some())
            .filter(|s| grades.is_empty() || field(s.data, "grade").is_some_and(|g| grades.contains(&g)))
            .count()
    };
    let n = fixable("vulnerable", &[]);
    if n > 0 {
        out.push(lens(format!("Fix vulnerable ({n})"), APPLY_FIXES, Some(vec![json!(uri), json!(["vulnerable"]), json!([])])));
    }
    let n = fixable("outdated", &["minor", "patch"]);
    if n > 0 {
        out.push(lens(format!("Update minor and patch ({n})"), APPLY_FIXES, Some(vec![json!(uri), json!(["outdated"]), json!(["minor", "patch"])])));
    }
    out
}

/// Whether a diagnostic's `data` is one `codelaw.applyFixes` takes.
pub fn wanted(data: &Value, kinds: &[String], grades: &[String]) -> bool {
    data.get("fixable").is_some()
        && field(data, "kind").is_some_and(|k| kinds.iter().any(|w| w == k))
        && (grades.is_empty() || field(data, "grade").is_some_and(|g| grades.iter().any(|w| w == g)))
}

/// The dependencies of a manifest that no finding is on, each with the
/// latest version in the committed snapshot (`.codelaw/deps-snapshot.json`).
/// `flagged` holds the 0-based lines that have a finding. Only
/// `package.json` and `pyproject.toml` are read; a dependency the snapshot
/// does not hold is left out.
pub fn clean_dependencies(rel: &str, text: &str, snapshot: &str, flagged: &[u32]) -> Vec<Clean> {
    let Ok(snap) = serde_json::from_str::<Value>(snapshot) else { return vec![] };
    let Some(packages) = snap.get("packages").and_then(Value::as_object) else { return vec![] };
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let deps = match name {
        "package.json" => npm_deps(text),
        "pyproject.toml" => pypi_deps(text),
        _ => vec![],
    };
    deps.into_iter()
        .filter(|(line, ..)| !flagged.contains(line))
        .filter_map(|(line, key, name, spec)| {
            let latest = packages.get(&key)?.get("latest")?.as_str()?.to_owned();
            let at_latest = spec_names(&spec, &latest);
            Some(Clean { line, name, latest, at_latest })
        })
        .collect()
}

/// Whether a requirement names exactly `version` (`^19.2.0`, `>=19.2.0`,
/// `==19.2.0` name 19.2.0; `^19.2.0-rc` and `19.2.01` do not).
fn spec_names(spec: &str, version: &str) -> bool {
    spec.match_indices(version).any(|(i, _)| {
        let before = spec[..i].chars().next_back();
        let after = spec[i + version.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '.') && !after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    })
}

const NPM_GROUPS: [&str; 4] = ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];

/// `(line, snapshot key, name, spec)` of each `package.json` dependency.
fn npm_deps(text: &str) -> Vec<(u32, String, String, String)> {
    let Ok(Value::Object(top)) = serde_json::from_str::<Value>(text) else { return vec![] };
    let mut out = Vec::new();
    for group in NPM_GROUPS {
        let Some(Value::Object(deps)) = top.get(group) else { continue };
        let Some(g) = text.find(&format!("\"{group}\"")) else { continue };
        for (name, spec) in deps {
            let Some(spec) = spec.as_str() else { continue };
            let Some(at) = text[g..].find(&format!("\"{name}\"")).map(|i| g + i) else { continue };
            out.push((text[..at].matches('\n').count() as u32, format!("npm:{name}"), name.clone(), spec.to_owned()));
        }
    }
    out
}

/// PEP 503's normalised name.
fn normalise(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if matches!(c, '-' | '_' | '.') {
            if !out.ends_with('-') {
                out.push('-');
            }
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

/// `(line, snapshot key, name, spec)` of each requirement in a
/// `pyproject.toml`'s `[project] dependencies`, `[project.optional-dependencies]`
/// and `[dependency-groups]` arrays.
fn pypi_deps(text: &str) -> Vec<(u32, String, String, String)> {
    let mut out = Vec::new();
    let mut table = String::new();
    let mut in_array = false;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.split(" #").next().unwrap_or(raw).trim();
        if line.starts_with('[') && !in_array {
            table = line.trim_matches(['[', ']', ' ']).to_owned();
            continue;
        }
        let mut rest = line;
        if !in_array {
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let takes = (table == "project" && key == "dependencies") || table == "project.optional-dependencies" || table == "dependency-groups";
            let value = value.trim();
            if !takes || !value.starts_with('[') {
                continue;
            }
            in_array = true;
            rest = &value[1..];
        }
        let mut chunk = rest;
        // The text outside the quoted strings: where the closing `]` is.
        let mut outside = String::new();
        while let Some(q) = chunk.find(['"', '\'']) {
            outside.push_str(&chunk[..q]);
            let quote = chunk[q..].chars().next().unwrap_or('"');
            let Some(end) = chunk[q + 1..].find(quote) else { break };
            let spec = &chunk[q + 1..q + 1 + end];
            let name: String = spec.chars().take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')).collect();
            if !name.is_empty() {
                out.push((i as u32, format!("pypi:{}", normalise(&name)), name, spec.to_owned()));
            }
            chunk = &chunk[q + 1 + end + 1..];
        }
        outside.push_str(chunk);
        if outside.contains(']') {
            in_array = false;
        }
    }
    out
}

#[cfg(test)]
#[path = "../tests/unit/lens.rs"]
mod tests;
