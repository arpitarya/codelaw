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

/// The lockfiles of each ecosystem, in the order [`installed`] reads them.
pub const NPM_LOCKFILES: [&str; 3] = ["package-lock.json", "pnpm-lock.yaml", "yarn.lock"];
pub const PYPI_LOCKFILES: [&str; 2] = ["uv.lock", "poetry.lock"];

/// The lockfile `installed` reads for the manifest in `dir`: its path and
/// text, nearest first.
pub fn lockfile_for(eco: Ecosystem, dir: &str, read: &dyn Fn(&str) -> Option<String>) -> Option<(String, String)> {
    let names: &[&str] = match eco {
        Ecosystem::Npm => &NPM_LOCKFILES,
        Ecosystem::Pypi => &PYPI_LOCKFILES,
    };
    for folder in folders(dir) {
        for n in names {
            let p = join(&folder, n);
            if let Some(t) = read(&p) {
                return Some((p, t));
            }
        }
    }
    None
}

/// One package version a lockfile installs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Locked {
    /// The registry's name: npm's, or PyPI's normalised.
    pub name: String,
    pub version: String,
    /// Whether a manifest the lockfile covers names it directly at this
    /// version (a root or workspace dependency).
    pub direct: bool,
}

/// Every package version a lockfile installs, sorted, each once. `file`
/// is the lockfile's name (`package-lock.json`, `pnpm-lock.yaml`,
/// `yarn.lock`, `uv.lock`, `poetry.lock`).
pub fn all_locked(file: &str, text: &str) -> Vec<Locked> {
    let name = file.rsplit('/').next().unwrap_or(file);
    let mut out: Vec<Locked> = match name {
        "package-lock.json" => package_lock_all(text),
        "pnpm-lock.yaml" => pnpm_lock_all(text),
        "yarn.lock" => yarn_lock_all(text),
        "uv.lock" | "poetry.lock" => toml_lock_all(text),
        _ => Vec::new(),
    };
    out.sort();
    // A pair both direct and not is direct.
    out.dedup_by(|b, a| {
        if a.name == b.name && a.version == b.version {
            a.direct |= b.direct;
            true
        } else {
            false
        }
    });
    out
}

/// The package a `package-lock.json` key installs: the part after its last
/// `node_modules/`; `None` for the root and workspace folders.
pub fn package_lock_name(key: &str) -> Option<&str> {
    let i = key.rfind("node_modules/")?;
    Some(&key[i + "node_modules/".len()..])
}

fn package_lock_all(text: &str) -> Vec<Locked> {
    let Ok(doc) = serde_json::from_str::<Value>(text) else { return Vec::new() };
    let mut out = Vec::new();
    if let Some(packages) = doc["packages"].as_object() {
        // Direct: each root or workspace entry's dependency names, at the
        // copy that entry resolves (its own `node_modules`, else the root's).
        let mut direct: Vec<(String, String)> = Vec::new();
        for (key, entry) in packages.iter().filter(|(k, _)| package_lock_name(k).is_none()) {
            for group in ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"] {
                for name in entry[group].as_object().into_iter().flat_map(|m| m.keys()) {
                    let own = if key.is_empty() { format!("node_modules/{name}") } else { format!("{key}/node_modules/{name}") };
                    let at = if packages.contains_key(&own) { own } else { format!("node_modules/{name}") };
                    if let Some(v) = packages.get(&at).and_then(|e| e["version"].as_str()) {
                        direct.push((name.clone(), v.to_owned()));
                    }
                }
            }
        }
        for (key, entry) in packages {
            let Some(name) = package_lock_name(key) else { continue };
            if entry["link"].as_bool() == Some(true) {
                continue;
            }
            let name = entry["name"].as_str().unwrap_or(name);
            if let Some(v) = entry["version"].as_str() {
                let d = direct.iter().any(|(n, dv)| n == name && dv == v);
                out.push(Locked { name: name.to_owned(), version: v.to_owned(), direct: d });
            }
        }
        return out;
    }
    // lockfileVersion 1: nested `dependencies`.
    fn walk(deps: &Value, out: &mut Vec<Locked>) {
        for (name, e) in deps.as_object().into_iter().flatten() {
            if let Some(v) = e["version"].as_str().filter(|v| !v.contains(':')) {
                out.push(Locked { name: name.clone(), version: v.to_owned(), direct: false });
            }
            walk(&e["dependencies"], out);
        }
    }
    walk(&doc["dependencies"], &mut out);
    out
}

/// A pnpm `packages` key's name and version: `/a@1.0.0` (v6), `a@1.0.0`
/// (v9), `/a/1.0.0` (v5), each maybe with a peer suffix.
pub fn pnpm_key(key: &str) -> Option<(String, String)> {
    let k = key.trim_start_matches('/');
    let k = k.split('(').next().unwrap_or(k);
    // v5: `name/version` (`@scope/name/version`), the version maybe with `_peer@x`.
    let slash = if k.starts_with('@') { k.find('/').and_then(|i| k[i + 1..].find('/').map(|j| i + 1 + j)) } else { k.find('/') };
    let (name, version) = match slash.filter(|i| k[i + 1..].starts_with(|c: char| c.is_ascii_digit())) {
        Some(i) => (&k[..i], k[i + 1..].split('_').next().unwrap_or("")),
        None => {
            let i = k.get(1..)?.find('@')? + 1;
            (&k[..i], &k[i + 1..])
        }
    };
    (!name.is_empty() && !version.is_empty()).then(|| (name.to_owned(), version.to_owned()))
}

fn pnpm_lock_all(text: &str) -> Vec<Locked> {
    let Ok(doc) = serde_saphyr::from_str::<Value>(text) else { return Vec::new() };
    let mut direct: Vec<(String, String)> = Vec::new();
    let mut importer = |deps: &Value| {
        for group in ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"] {
            for (name, e) in deps[group].as_object().into_iter().flatten() {
                if let Some(v) = e["version"].as_str().or(e.as_str()).filter(|v| !v.starts_with("link:")) {
                    direct.push((name.clone(), pnpm_version(v)));
                }
            }
        }
    };
    match doc["importers"].as_object() {
        Some(imps) => imps.values().for_each(&mut importer),
        None => importer(&doc),
    }
    let mut out = Vec::new();
    for key in doc["packages"].as_object().into_iter().flat_map(|m| m.keys()) {
        if let Some((name, version)) = pnpm_key(key) {
            let d = direct.iter().any(|(n, v)| *n == name && *v == version);
            out.push(Locked { name, version, direct: d });
        }
    }
    out
}

/// A yarn descriptor's package name: `react@^18`, `@a/b@npm:^1`.
fn yarn_name(descriptor: &str) -> Option<&str> {
    let i = descriptor[1.min(descriptor.len())..].find('@')? + 1;
    Some(&descriptor[..i])
}

fn yarn_lock_all(text: &str) -> Vec<Locked> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            let first = line.trim_end_matches(':').split(", ").next().map(unquote).unwrap_or_default();
            name = yarn_name(first).map(str::to_owned).filter(|n| n != "__metadata");
            continue;
        }
        let t = line.trim();
        let Some(v) = t.strip_prefix("version:").or_else(|| t.strip_prefix("version ")) else { continue };
        if let Some(n) = name.take() {
            let v = unquote(v);
            // Berry's workspace entries are `0.0.0-use.local`.
            if !v.contains("use.local") {
                out.push(Locked { name: n, version: v.to_owned(), direct: false });
            }
        }
    }
    out
}

fn toml_lock_all(text: &str) -> Vec<Locked> {
    let Ok(doc) = toml::from_str::<toml::Table>(text) else { return Vec::new() };
    let mut out = Vec::new();
    for p in doc.get("package").and_then(|p| p.as_array()).into_iter().flatten() {
        // uv's own project is `source = { editable = "." }` or virtual.
        let local = p.get("source").and_then(|s| s.as_table()).is_some_and(|s| s.contains_key("editable") || s.contains_key("virtual"));
        if let (Some(n), Some(v), false) = (p.get("name").and_then(|n| n.as_str()), p.get("version").and_then(|v| v.as_str()), local) {
            out.push(Locked { name: normalize(n), version: v.to_owned(), direct: false });
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
    fn every_lockfile_lists_what_it_installs() {
        let pl = r#"{"lockfileVersion":3,"packages":{"":{"dependencies":{"a":"^1"}},"node_modules/a":{"version":"1.0.0"},"node_modules/b":{"version":"2.0.0"},"node_modules/a/node_modules/b":{"version":"1.5.0"},"packages/w":{"devDependencies":{"c":"^3"}},"packages/w/node_modules/c":{"version":"3.1.0"},"node_modules/w":{"link":true}}}"#;
        let got: Vec<(String, String, bool)> = all_locked("package-lock.json", pl).into_iter().map(|l| (l.name, l.version, l.direct)).collect();
        assert_eq!(got, [("a".into(), "1.0.0".into(), true), ("b".into(), "1.5.0".into(), false), ("b".into(), "2.0.0".into(), false), ("c".into(), "3.1.0".into(), true)]);
        let pnpm = "lockfileVersion: '9.0'\nimporters:\n  .:\n    dependencies:\n      a:\n        specifier: ^1\n        version: 1.0.0\npackages:\n  a@1.0.0:\n    resolution: {integrity: x}\n  '@s/b@2.0.0':\n    resolution: {integrity: y}\n";
        let got: Vec<(String, String, bool)> = all_locked("pnpm-lock.yaml", pnpm).into_iter().map(|l| (l.name, l.version, l.direct)).collect();
        assert_eq!(got, [("@s/b".into(), "2.0.0".into(), false), ("a".into(), "1.0.0".into(), true)]);
        assert_eq!(pnpm_key("/@s/b@2.0.0(react@18.0.0)"), Some(("@s/b".into(), "2.0.0".into())));
        assert_eq!(pnpm_key("/a/1.0.0_react@18"), Some(("a".into(), "1.0.0".into())));
        let yarn = "\"@s/b@^2\", \"@s/b@^2.0\":\n  version \"2.0.0\"\n\nminimist@^1.2.0:\n  version \"1.2.5\"\n";
        let got: Vec<(String, String)> = all_locked("yarn.lock", yarn).into_iter().map(|l| (l.name, l.version)).collect();
        assert_eq!(got, [("@s/b".into(), "2.0.0".into()), ("minimist".into(), "1.2.5".into())]);
        let uv = "[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = { editable = \".\" }\n\n[[package]]\nname = \"Jinja2\"\nversion = \"3.1.2\"\n";
        let got: Vec<(String, String)> = all_locked("uv.lock", uv).into_iter().map(|l| (l.name, l.version)).collect();
        assert_eq!(got, [("jinja2".into(), "3.1.2".into())]);
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
