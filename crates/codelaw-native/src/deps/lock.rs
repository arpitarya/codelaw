//! The installed version of each dependency, read from the lockfile beside
//! a manifest or above it (a workspace's lockfile is at its root):
//! `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `uv.lock`,
//! `poetry.lock`. Files are read through `read` (project-relative paths),
//! so the engine records each one a rule asked for (SR-ARCHITECTURE
//! decision 15b).

use super::Ecosystem;
use super::pep440::normalize;
use serde_json::Value;
use std::collections::BTreeMap;

/// The folders from `dir` up to the project root (`""`), nearest first.
fn folders(dir: &str) -> Vec<String> {
    let mut out = vec![dir.to_owned()];
    let mut d = dir.to_owned();
    while !d.is_empty() {
        d = d.rsplit_once('/').map(|(a, _)| a.to_owned()).unwrap_or_default();
        out.push(d.clone());
    }
    out
}

fn join(dir: &str, name: &str) -> String {
    if dir.is_empty() { name.to_owned() } else { format!("{dir}/{name}") }
}

/// `dir` relative to `base` (both project-relative, `base` an ancestor).
fn relative(dir: &str, base: &str) -> String {
    if base.is_empty() {
        return dir.to_owned();
    }
    dir.strip_prefix(base).map(|r| r.trim_start_matches('/').to_owned()).unwrap_or_default()
}

/// Installed versions for the manifest in `dir`, keyed by the name the
/// manifest uses (npm) or its normalised name (PyPI). The nearest lockfile
/// wins; none gives an empty map.
pub fn installed(eco: Ecosystem, dir: &str, specs: &BTreeMap<String, String>, read: &dyn Fn(&str) -> Option<String>) -> BTreeMap<String, String> {
    for folder in folders(dir) {
        let found = match eco {
            Ecosystem::Npm => {
                if let Some(t) = read(&join(&folder, "package-lock.json")) {
                    Some(package_lock(&t, &relative(dir, &folder), specs))
                } else if let Some(t) = read(&join(&folder, "pnpm-lock.yaml")) {
                    Some(pnpm_lock(&t, &relative(dir, &folder), specs))
                } else {
                    read(&join(&folder, "yarn.lock")).map(|t| yarn_lock(&t, specs))
                }
            }
            Ecosystem::Pypi => read(&join(&folder, "uv.lock")).or_else(|| read(&join(&folder, "poetry.lock"))).map(|t| toml_lock(&t)),
        };
        if let Some(f) = found {
            return f;
        }
    }
    BTreeMap::new()
}

fn package_lock(text: &str, rel: &str, specs: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let Ok(doc) = serde_json::from_str::<Value>(text) else { return BTreeMap::new() };
    let mut out = BTreeMap::new();
    for name in specs.keys() {
        let mut keys = Vec::new();
        if !rel.is_empty() {
            keys.push(format!("{rel}/node_modules/{name}"));
        }
        keys.push(format!("node_modules/{name}"));
        let v = keys.iter().find_map(|k| doc["packages"][k]["version"].as_str()).or_else(|| doc["dependencies"][name]["version"].as_str());
        if let Some(v) = v {
            out.insert(name.clone(), v.to_owned());
        }
    }
    out
}

/// A pnpm version without its peer suffix: `18.3.1(react@18.3.1)`.
fn pnpm_version(v: &str) -> String {
    v.split('(').next().unwrap_or(v).trim().to_owned()
}

fn pnpm_lock(text: &str, rel: &str, specs: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let Ok(doc) = serde_saphyr::from_str::<Value>(text) else { return BTreeMap::new() };
    let importer = if rel.is_empty() { "." } else { rel };
    let mut out = BTreeMap::new();
    for name in specs.keys() {
        for group in ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"] {
            // v6 and later: importers; v5: the root keys.
            let entry = &doc["importers"][importer][group][name];
            let entry = if entry.is_null() && importer == "." { &doc[group][name] } else { entry };
            let v = entry["version"].as_str().or(entry.as_str());
            if let Some(v) = v.filter(|v| !v.starts_with("link:")) {
                out.insert(name.clone(), pnpm_version(v));
                break;
            }
        }
    }
    out
}

fn unquote(s: &str) -> &str {
    s.trim().trim_matches('"')
}

/// yarn classic (`"react@^18.3.1":` / `version "18.3.1"`) and berry
/// (`"react@npm:^18.3.1":` / `version: 18.3.1`).
fn yarn_lock(text: &str, specs: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut current: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            current = line.trim_end_matches(':').split(", ").map(|d| unquote(d).to_owned()).collect();
            continue;
        }
        let t = line.trim();
        let Some(v) = t.strip_prefix("version:").or_else(|| t.strip_prefix("version ")) else { continue };
        let v = unquote(v);
        for (name, spec) in specs {
            let plain = format!("{name}@{spec}");
            let berry = format!("{name}@npm:{spec}");
            if current.iter().any(|d| *d == plain || *d == berry) {
                out.entry(name.clone()).or_insert_with(|| v.to_owned());
            }
        }
    }
    out
}

/// `uv.lock` and `poetry.lock`: `[[package]]` tables with `name` and
/// `version`. A name locked twice (a resolution fork) keeps its first.
fn toml_lock(text: &str) -> BTreeMap<String, String> {
    let Ok(doc) = toml::from_str::<toml::Table>(text) else { return BTreeMap::new() };
    let mut out = BTreeMap::new();
    for p in doc.get("package").and_then(|p| p.as_array()).into_iter().flatten() {
        if let (Some(n), Some(v)) = (p.get("name").and_then(|n| n.as_str()), p.get("version").and_then(|v| v.as_str())) {
            out.entry(normalize(n)).or_insert_with(|| v.to_owned());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn specs(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    #[test]
    fn every_lockfile_gives_the_installed_version() {
        let s = specs(&[("react", "^18.3.1")]);
        let files: BTreeMap<&str, &str> = [
            ("a/package-lock.json", r#"{"packages":{"node_modules/react":{"version":"18.3.1"}}}"#),
            ("b/pnpm-lock.yaml", "lockfileVersion: '9.0'\nimporters:\n  .:\n    dependencies:\n      react:\n        specifier: ^18.3.1\n        version: 18.3.1(x@1.0.0)\n"),
            ("c/yarn.lock", "# yarn\n\n\"react@^18.3.1\", \"react@^18.0.0\":\n  version \"18.3.1\"\n"),
            ("d/yarn.lock", "\"react@npm:^18.3.1\":\n  version: 18.3.1\n"),
            ("e/uv.lock", "[[package]]\nname = \"Requests\"\nversion = \"2.31.0\"\n"),
        ]
        .into_iter()
        .collect();
        let read = |p: &str| files.get(p).map(|t| t.to_string());
        for dir in ["a", "b", "c", "d"] {
            assert_eq!(installed(Ecosystem::Npm, dir, &s, &read).get("react").map(String::as_str), Some("18.3.1"), "{dir}");
        }
        assert_eq!(installed(Ecosystem::Pypi, "e/sub", &BTreeMap::new(), &read).get("requests").map(String::as_str), Some("2.31.0"));
        assert!(installed(Ecosystem::Npm, "z", &s, &read).is_empty());
    }

    #[test]
    fn a_workspace_reads_the_root_lockfile_under_its_own_folder() {
        let s = specs(&[("react", "^18.0.0")]);
        let lock = r#"{"packages":{"node_modules/react":{"version":"18.2.0"},"packages/web/node_modules/react":{"version":"18.3.1"}}}"#;
        let read = |p: &str| (p == "package-lock.json").then(|| lock.to_owned());
        assert_eq!(installed(Ecosystem::Npm, "packages/web", &s, &read)["react"], "18.3.1");
        assert_eq!(installed(Ecosystem::Npm, "", &s, &read)["react"], "18.2.0");
    }
}
