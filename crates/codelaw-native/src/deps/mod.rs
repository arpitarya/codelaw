//! The `deps` pack's rules (SR-RULE-PACKS decision 15): dependencies,
//! their latest versions and their advisories.
//!
//! The manifests (`package.json`, `pyproject.toml`), their lockfiles, the
//! committed snapshot of registry and advisory facts, and the version
//! arithmetic. `deps/vulnerable`, `deps/incomplete-scan`, `deps/outdated`
//! and `deps/stale-snapshot` read only files; `codelaw deps refresh` —
//! outside the engine — is the one command that reads the network and
//! writes the snapshot (L5 decision 3).

pub mod advisory;
pub mod lock;
pub mod npm;
pub mod pep440;
mod rules;
mod vulnerable;
pub mod snapshot;

pub use rules::{OutdatedDependency, StaleDepsSnapshot};
pub use vulnerable::{Assessment, IncompleteScan, Vulnerable, assess, check_dependency};

use crate::data_json::{pairs, string_value, top_object};
use crate::data_toml::{key_node, key_path, pair_value, string_text};
use crate::util::{named_children, text};
use codelaw_core::Language;
use tree_sitter::Node;

/// The registry a dependency comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ecosystem {
    Npm,
    Pypi,
}

impl Ecosystem {
    pub fn prefix(self) -> &'static str {
        match self {
            Ecosystem::Npm => "npm",
            Ecosystem::Pypi => "pypi",
        }
    }
}

/// One dependency line of a manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dep {
    pub ecosystem: Ecosystem,
    /// The name as the manifest writes it.
    pub name: String,
    /// The snapshot key: `npm:react`, `pypi:requests` (normalised).
    pub key: String,
    /// npm: the range; PyPI: the whole requirement string.
    pub spec: String,
    /// Byte offsets of `spec` in the manifest.
    pub start: usize,
    pub end: usize,
}

const NPM_GROUPS: [&str; 4] = ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];

/// The dependencies a manifest declares, in file order. `path` is the
/// manifest's project-relative path; only `package.json` and
/// `pyproject.toml` are manifests.
pub fn manifest_deps(language: Language, path: &str, doc: Node<'_>, src: &str) -> Vec<Dep> {
    let file = path.rsplit('/').next().unwrap_or(path);
    match (language, file) {
        (Language::Json, "package.json") => package_json(doc, src),
        (Language::Toml, "pyproject.toml") => pyproject(doc, src),
        _ => Vec::new(),
    }
}

/// The folder of a project-relative path (`""` for the root).
pub fn folder_of(path: &str) -> &str {
    path.rsplit_once('/').map(|(d, _)| d).unwrap_or("")
}

fn package_json(doc: Node<'_>, src: &str) -> Vec<Dep> {
    let Some(obj) = top_object(doc) else { return Vec::new() };
    let mut out = Vec::new();
    for (key, _, value) in pairs(obj, src) {
        if !NPM_GROUPS.contains(&key.as_str()) {
            continue;
        }
        let Some(group) = value.filter(|v| v.kind() == "object") else { continue };
        for (name, _, v) in pairs(group, src) {
            let Some(v) = v.filter(|v| v.kind() == "string") else { continue };
            let raw = text(v, src);
            if raw.len() < 2 {
                continue;
            }
            let inner = &raw[1..raw.len() - 1];
            // An escaped range is not one the fix could write back.
            if string_value(v, src) != inner {
                continue;
            }
            let start = v.start_byte() + 1;
            out.push(Dep { ecosystem: Ecosystem::Npm, key: format!("npm:{name}"), name, spec: inner.to_owned(), start, end: start + inner.len() });
        }
    }
    out
}

/// The requirement strings of an array, with their offsets.
fn requirement_strings(array: Node<'_>, src: &str, out: &mut Vec<Dep>) {
    for item in named_children(array) {
        let Some(inner) = string_text(item, src) else { continue };
        let raw = text(item, src);
        // Single-line strings only: their offsets are the text's.
        if raw.starts_with("\"\"\"") || raw.starts_with("'''") || (raw.starts_with('"') && inner.contains('\\')) {
            continue;
        }
        let Some(req) = pep440::requirement(&inner) else { continue };
        let start = item.start_byte() + 1;
        out.push(Dep { ecosystem: Ecosystem::Pypi, key: format!("pypi:{}", req.normalized), name: req.name, spec: inner.clone(), start, end: start + inner.len() });
    }
}

/// The groups of an inline table or a table: each array value.
fn group_arrays<'t>(pairs: &[(Vec<String>, Node<'t>, Option<Node<'t>>)], prefix: &[&str], out: &mut Vec<Node<'t>>) {
    for (path, _, value) in pairs {
        let value = *value;
        let Some(v) = value else { continue };
        let rest: Vec<&str> = path.iter().map(String::as_str).collect();
        if rest.len() < prefix.len() || rest[..prefix.len()] != *prefix {
            continue;
        }
        match (&rest[prefix.len()..], v.kind()) {
            ([_group], "array") => out.push(v),
            ([], "inline_table") => {
                for p in named_children(v).into_iter().filter(|p| p.kind() == "pair") {
                    if let Some(a) = pair_value(p).filter(|a| a.kind() == "array") {
                        out.push(a);
                    }
                }
            }
            _ => {}
        }
    }
}

fn pyproject(doc: Node<'_>, src: &str) -> Vec<Dep> {
    // Every pair with its full key path: root pairs, then each table's.
    let mut all: Vec<(Vec<String>, Node<'_>, Option<Node<'_>>)> = Vec::new();
    for n in named_children(doc) {
        match n.kind() {
            "pair" => {
                if let Some(k) = key_node(n) {
                    all.push((key_path(k, src), k, pair_value(n)));
                }
            }
            "table" => {
                let Some(h) = key_node(n) else { continue };
                let head = key_path(h, src);
                for p in named_children(n).into_iter().filter(|p| p.kind() == "pair") {
                    if let Some(k) = key_node(p) {
                        let mut path = head.clone();
                        path.extend(key_path(k, src));
                        all.push((path, k, pair_value(p)));
                    }
                }
            }
            _ => {}
        }
    }
    let mut arrays = Vec::new();
    for (path, _, value) in &all {
        if path == &["project", "dependencies"]
            && let Some(v) = value.filter(|v| v.kind() == "array")
        {
            arrays.push(v);
        }
    }
    group_arrays(&all, &["project", "optional-dependencies"], &mut arrays);
    group_arrays(&all, &["dependency-groups"], &mut arrays);
    arrays.sort_by_key(|a| a.start_byte());
    let mut out = Vec::new();
    for a in arrays {
        requirement_strings(a, src, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deps(lang: Language, path: &str, src: &str) -> Vec<(String, String, String)> {
        let root = codelaw_lang::parse(lang, src);
        manifest_deps(lang, path, root.root().get_inner_node(), src).into_iter().map(|d| (d.key, d.spec.clone(), src[d.start..d.end].to_owned())).collect()
    }

    #[test]
    fn package_json_groups() {
        let src = r#"{ "name": "a", "dependencies": { "react": "^18.3.1" }, "devDependencies": { "@types/node": "~22.1" }, "scripts": { "x": "y" } }"#;
        let got = deps(Language::Json, "web/package.json", src);
        assert_eq!(got, [("npm:react".into(), "^18.3.1".into(), "^18.3.1".into()), ("npm:@types/node".into(), "~22.1".into(), "~22.1".into())]);
        assert!(deps(Language::Json, "tsconfig.json", src).is_empty());
    }

    #[test]
    fn pyproject_groups() {
        let src = "[project]\nname = \"a\"\ndependencies = [\"Requests>=2.31\", 'django~=4.2']\n\n[project.optional-dependencies]\ndev = [\"pytest==8.0\"]\n\n[dependency-groups]\nlint = [\"ruff>=0.5\", { include-group = \"dev\" }]\n";
        let got: Vec<String> = deps(Language::Toml, "pyproject.toml", src).into_iter().map(|(k, _, t)| format!("{k} {t}")).collect();
        assert_eq!(got, ["pypi:requests Requests>=2.31", "pypi:django django~=4.2", "pypi:pytest pytest==8.0", "pypi:ruff ruff>=0.5"]);
        let inline = "[project]\noptional-dependencies = { dev = [\"pytest==8.0\"] }\n";
        assert_eq!(deps(Language::Toml, "pyproject.toml", inline).len(), 1);
    }
}
