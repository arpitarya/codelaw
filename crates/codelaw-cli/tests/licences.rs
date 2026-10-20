//! L2's licence check over the Cargo manifest (SR-LAW-2): every resolved
//! dependency, transitive ones included, is OSI-approved and permissive. A
//! multi-licensed crate qualifies through any one licence that does
//! (decision 6). The npm and PyPI manifests join this check when they land.

use serde_json::Value;
use std::process::Command;

/// Permissive and on the OSI-approved list (decisions 1 and 2).
const ALLOWED: &[&str] = &[
    "MIT",
    "MIT-0",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "0BSD",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Unlicense",
    "BSL-1.0",
];

fn tokens(expr: &str) -> Vec<String> {
    let spaced = expr.replace('/', " OR ").replace('(', " ( ").replace(')', " ) ");
    let mut out: Vec<String> = Vec::new();
    for t in spaced.split_whitespace() {
        // `WITH` binds to the licence before it.
        if let Some(last) = out.last_mut()
            && (t == "WITH" || last.ends_with(" WITH"))
        {
            last.push(' ');
            last.push_str(t);
            continue;
        }
        out.push(t.to_owned());
    }
    out
}

/// Evaluate an SPDX expression: `OR` takes any side, `AND` needs both.
fn allowed(tokens: &[String], pos: &mut usize) -> bool {
    let mut result = term(tokens, pos);
    while *pos < tokens.len() && tokens[*pos] == "OR" {
        *pos += 1;
        let rhs = term(tokens, pos);
        result = result || rhs;
    }
    result
}

fn term(tokens: &[String], pos: &mut usize) -> bool {
    let mut result = atom(tokens, pos);
    while *pos < tokens.len() && tokens[*pos] == "AND" {
        *pos += 1;
        let rhs = atom(tokens, pos);
        result = result && rhs;
    }
    result
}

fn atom(tokens: &[String], pos: &mut usize) -> bool {
    let Some(t) = tokens.get(*pos) else { return false };
    *pos += 1;
    if t == "(" {
        let v = allowed(tokens, pos);
        if tokens.get(*pos).is_some_and(|c| c == ")") {
            *pos += 1;
        }
        return v;
    }
    ALLOWED.contains(&t.as_str())
}

pub fn expression_allowed(expr: &str) -> bool {
    let t = tokens(expr);
    let mut pos = 0;
    allowed(&t, &mut pos) && pos == t.len()
}

#[test]
fn spdx_expressions() {
    assert!(expression_allowed("MIT"));
    assert!(expression_allowed("MIT OR Apache-2.0"));
    assert!(expression_allowed("MIT/Apache-2.0"));
    assert!(expression_allowed("(MIT OR Apache-2.0) AND Unicode-3.0"));
    assert!(expression_allowed("CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception"));
    assert!(!expression_allowed("GPL-3.0"));
    assert!(!expression_allowed("MIT AND MPL-2.0"));
    assert!(!expression_allowed("CC0-1.0"), "CC0 is not OSI-approved");
    assert!(!expression_allowed("LGPL-2.1 OR GPL-2.0"));
    assert!(expression_allowed("MPL-2.0 OR MIT"), "one qualifying licence is enough");
}

#[test]
fn every_cargo_dependency_is_free_and_permissive() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        // Every platform's dependencies, not only this machine's. Under the
        // L5 network fence, run `cargo fetch` first so this reads the cache.
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let meta: Value = serde_json::from_slice(&out.stdout).unwrap();
    let members: Vec<&str> = meta["workspace_members"].as_array().unwrap().iter().map(|m| m.as_str().unwrap()).collect();
    let mut refused = Vec::new();
    let mut checked = 0;
    for p in meta["packages"].as_array().unwrap() {
        if members.contains(&p["id"].as_str().unwrap()) {
            continue;
        }
        checked += 1;
        let name = format!("{} {}", p["name"].as_str().unwrap(), p["version"].as_str().unwrap());
        match p["license"].as_str() {
            Some(l) if expression_allowed(l) => {}
            Some(l) => refused.push(format!("{name}: `{l}`")),
            None => refused.push(format!("{name}: no licence expression")),
        }
    }
    assert!(checked > 50, "only {checked} dependencies seen");
    assert!(refused.is_empty(), "L2 refuses:\n{}", refused.join("\n"));
}

/// The editor extension's npm lockfile (SR-EDITOR decision 4): runtime and
/// dev dependencies alike (decision 4).
#[test]
fn every_npm_dependency_of_the_extension_is_free_and_permissive() {
    let lock = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/vscode/package-lock.json");
    let lock: Value = serde_json::from_str(&std::fs::read_to_string(&lock).expect("packaging/vscode/package-lock.json")).unwrap();
    let mut refused = Vec::new();
    let mut checked = 0;
    for (path, p) in lock["packages"].as_object().unwrap() {
        if path.is_empty() {
            continue;
        }
        checked += 1;
        let name = format!("{path} {}", p["version"].as_str().unwrap_or("?"));
        match p["license"].as_str() {
            Some(l) if expression_allowed(l) => {}
            Some(l) => refused.push(format!("{name}: `{l}`")),
            None => refused.push(format!("{name}: no licence expression")),
        }
    }
    assert!(checked >= 5, "only {checked} npm packages seen");
    assert!(refused.is_empty(), "L2 refuses:\n{}", refused.join("\n"));
}
