//! The dependency facts an agent is told before it writes (W-128;
//! SR-AGENT-SURFACES, SR-CLI-MCP decision 5), read from the committed
//! snapshot only — never the network (L5 decision 3):
//!
//! - [`Project::check_dependency`]: one package, for `codelaw deps check`
//!   and MCP `check_dependency`;
//! - [`Project::dependency_notes`]: for `rules_for_path`, the vulnerable or
//!   major-behind packages a manifest declares or a source file imports.

use crate::Project;
use codelaw_core::Language;
use codelaw_native::deps::advisory::{Target, Ver};
use codelaw_native::deps::pep440::normalize;
use codelaw_native::deps::snapshot::{SNAPSHOT_PATH, Snapshot, days};
use codelaw_native::deps::{Ecosystem, assess, check_dependency, lock, manifest_deps};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

impl Project {
    fn deps_snapshot(&self) -> Option<Snapshot> {
        std::fs::read_to_string(self.root().join(SNAPSHOT_PATH)).ok().and_then(|t| Snapshot::parse(&t))
    }

    fn as_of_days(&self) -> Option<i64> {
        self.options.as_of.as_deref().or(self.options.as_of_env.as_deref()).and_then(days)
    }

    /// What the snapshot says of one package: its advisories, the version
    /// to move to, its latest and licence.
    pub fn check_dependency(&self, eco: Ecosystem, name: &str, version: Option<&str>) -> Value {
        let options = self.config.section_options("deps");
        check_dependency(self.deps_snapshot().as_ref(), &options, eco, name, version, self.as_of_days())
    }

    /// The packages `file` brings in that are vulnerable or a major
    /// version behind: every dependency of a manifest, or the ones a
    /// JavaScript, TypeScript or Python file imports (looked up in the
    /// nearest `package.json` / `pyproject.toml` and its lockfile).
    pub fn dependency_notes(&self, file: &Path) -> Vec<Value> {
        let Some(snap) = self.deps_snapshot() else { return Vec::new() };
        let Some(fetched) = days(&snap.fetched) else { return Vec::new() };
        let (abs, rel) = self.locate_path(file);
        let name = rel.rsplit('/').next().unwrap_or(&rel).to_owned();
        let read = |p: &str| std::fs::read_to_string(self.root().join(p)).ok();
        // The nearest manifests, from the file's folder up.
        let mut folder = codelaw_native::deps::folder_of(&rel).to_owned();
        let mut declared: BTreeMap<String, (Ecosystem, String, Option<String>)> = BTreeMap::new();
        loop {
            for (manifest, lang) in [("package.json", Language::Json), ("pyproject.toml", Language::Toml)] {
                let path = if folder.is_empty() { manifest.to_owned() } else { format!("{folder}/{manifest}") };
                let Some(text) = read(&path) else { continue };
                let tree = codelaw_lang::parse(lang, &text);
                let deps = manifest_deps(lang, &path, tree.root().get_inner_node(), &text);
                for eco in [Ecosystem::Npm, Ecosystem::Pypi] {
                    let specs: BTreeMap<String, String> = deps.iter().filter(|d| d.ecosystem == eco).map(|d| (lock_key(eco, &d.name), d.spec.clone())).collect();
                    let installed = lock::installed(eco, &folder, &specs, &read);
                    for d in deps.iter().filter(|d| d.ecosystem == eco) {
                        let k = lock_key(eco, &d.name);
                        declared.entry(d.key.clone()).or_insert((eco, d.name.clone(), installed.get(&k).cloned().or_else(|| written(&d.spec))));
                    }
                }
            }
            if folder.is_empty() || !declared.is_empty() {
                break;
            }
            folder = folder.rsplit_once('/').map(|(a, _)| a.to_owned()).unwrap_or_default();
        }
        let wanted: Vec<String> = if name == "package.json" || name == "pyproject.toml" {
            declared.keys().cloned().collect()
        } else {
            let Ok(text) = std::fs::read_to_string(&abs) else { return Vec::new() };
            let lang = Language::from_path(&abs);
            imports(lang, &text).into_iter().filter(|k| declared.contains_key(k)).collect()
        };
        let options = self.config.section_options("deps");
        let as_of = self.as_of_days();
        let as_of_f = || as_of;
        let mut out = Vec::new();
        for key in wanted {
            let (eco, name, version) = &declared[&key];
            let (Some(pkg), Some(version)) = (snap.packages.get(&key), version) else { continue };
            let vulnerable = assess(*eco, &key, pkg, version, &options, fetched, &as_of_f);
            let behind = match (pkg.latest.as_deref().and_then(|l| Ver::parse(*eco, l)), Ver::parse(*eco, version)) {
                (Some(l), Some(v)) => l.major() > v.major(),
                _ => false,
            };
            if vulnerable.is_none() && !behind {
                continue;
            }
            let next = match &vulnerable {
                Some(a) => {
                    let to = match &a.target {
                        Target::Clean { version: t, .. } => format!("use {t}"),
                        Target::Unverified { fixed } => format!("fixed in {fixed}, not verified clean"),
                        Target::None => "no fixed version is known".into(),
                    };
                    format!("{name} {version} is vulnerable ({}, {}): {to}", a.advisories[0].id, a.severity())
                }
                None => format!("{name} {version} is a major version behind its latest, {}", pkg.latest.as_deref().unwrap_or_default()),
            };
            out.push(json!({
                "ecosystem": eco.prefix(),
                "package": name,
                "version": version,
                "vulnerable": vulnerable.is_some(),
                "severity": vulnerable.as_ref().map(|a| a.severity().to_owned()),
                "advisories": vulnerable.as_ref().map(|a| a.advisories.iter().map(|x| x.id.clone()).collect::<Vec<_>>()).unwrap_or_default(),
                "target": match vulnerable.as_ref().map(|a| &a.target) { Some(Target::Clean { version, .. }) => json!(version), _ => Value::Null },
                "latest": pkg.latest,
                "major_behind": behind,
                "next": next,
            }));
        }
        out
    }
}

fn lock_key(eco: Ecosystem, name: &str) -> String {
    match eco {
        Ecosystem::Npm => name.to_owned(),
        Ecosystem::Pypi => normalize(name),
    }
}

/// The version a requirement starts from.
fn written(spec: &str) -> Option<String> {
    if let Some(r) = codelaw_native::deps::npm::Range::parse(spec) {
        return Some(r.base().to_string());
    }
    let req = codelaw_native::deps::pep440::requirement(spec)?;
    req.clauses.iter().find(|c| matches!(c.op.as_str(), "==" | "===" | ">=" | "~=") && !c.wildcard).map(|c| c.version.clone())
}

/// The package keys a source file imports: npm bare specifiers in
/// `import … from`, `import(…)`, `require(…)`; Python's top-level modules.
fn imports(lang: Option<Language>, text: &str) -> Vec<String> {
    let mut out = Vec::new();
    match lang {
        Some(l) if l.is_js_family() => {
            for (i, _) in text.match_indices(['\'', '"']) {
                let before = text[..i].trim_end();
                if !(before.ends_with("from") || before.ends_with("require(") || before.ends_with("import(") || before.ends_with("import")) {
                    continue;
                }
                let q = &text[i..i + 1];
                let Some(end) = text[i + 1..].find(q) else { continue };
                let spec = &text[i + 1..i + 1 + end];
                if spec.starts_with('.') || spec.starts_with('/') || spec.contains(':') || spec.is_empty() {
                    continue;
                }
                let mut parts = spec.split('/');
                let name = match parts.next() {
                    Some(s) if s.starts_with('@') => format!("{s}/{}", parts.next().unwrap_or_default()),
                    Some(s) => s.to_owned(),
                    None => continue,
                };
                out.push(format!("npm:{name}"));
            }
        }
        Some(Language::Python) => {
            for line in text.lines() {
                let t = line.trim_start();
                let module = t.strip_prefix("import ").or_else(|| t.strip_prefix("from ")).and_then(|r| r.split([' ', '.', ',']).next());
                if let Some(m) = module.filter(|m| !m.is_empty()) {
                    out.push(format!("pypi:{}", normalize(m)));
                }
            }
        }
        _ => {}
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_name_packages() {
        let js = "import x from 'lodash/fp';\nconst y = require(\"@acme/ui/button\");\nimport './local';\nawait import('chalk');";
        assert_eq!(imports(Some(Language::Typescript), js), ["npm:@acme/ui", "npm:chalk", "npm:lodash"]);
        assert_eq!(imports(Some(Language::Python), "import Jinja2\nfrom requests.adapters import X\n"), ["pypi:jinja2", "pypi:requests"]);
    }
}
