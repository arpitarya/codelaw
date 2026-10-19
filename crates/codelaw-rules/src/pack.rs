//! Rule packs: layout, manifest, presets and where they are found
//! (SR-RULE-PACKS decisions 3–5 and 11).

use crate::LoadError;
use crate::rule::{RESERVED_NAMESPACES, RuleFile, parse_cases, parse_rule};
use codelaw_cache::{Key, KeyBuilder};
use codelaw_config::RuleSetting;
use codelaw_core::Language;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The five first-party packs: short name, npm name, PyPI name, module.
pub const FIRST_PARTY_PACKS: &[(&str, &str, &str, &str)] = &[
    ("js", "@codelaw/rules-js", "codelaw-rules-js", "codelaw_rules_js"),
    ("html", "@codelaw/rules-html", "codelaw-rules-html", "codelaw_rules_html"),
    ("css", "@codelaw/rules-css", "codelaw-rules-css", "codelaw_rules_css"),
    ("python", "@codelaw/rules-python", "codelaw-rules-python", "codelaw_rules_python"),
    ("data", "@codelaw/rules-data", "codelaw-rules-data", "codelaw_rules_data"),
];

/// The fixed namespace-to-pack map (decision 4).
pub fn namespace_pack(namespace: &str) -> Option<&'static str> {
    Some(match namespace {
        "js" | "ts" | "react" | "node" | "stencil" => "js",
        "html" => "html",
        "css" => "css",
        "py" => "python",
        "data" | "md" | "json" | "yaml" | "toml" => "data",
        _ => return None,
    })
}

fn first_party(short: &str) -> &'static (&'static str, &'static str, &'static str, &'static str) {
    FIRST_PARTY_PACKS.iter().find(|p| p.0 == short).expect("a first-party pack")
}

/// One `extends` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtendsSpec {
    /// `codelaw:<namespace>/<preset>`.
    FirstParty { pack: &'static str, preset: String },
    /// `npm:<package>`: every rule of the pack at its own severity.
    Npm(String),
    /// `pypi:<distribution>`.
    Pypi(String),
    /// `path:<folder>`, relative to the config file.
    Path(String),
}

impl ExtendsSpec {
    pub fn parse(spec: &str) -> Result<ExtendsSpec, String> {
        if let Some(rest) = spec.strip_prefix("codelaw:") {
            let Some((ns, name)) = rest.split_once('/').filter(|(a, b)| !a.is_empty() && !b.is_empty()) else {
                return Err(format!("`{spec}`: a first-party preset is `codelaw:<namespace>/<preset>`, e.g. `codelaw:js/recommended`"));
            };
            let Some(pack) = namespace_pack(ns) else {
                let known = ["js", "ts", "react", "node", "stencil", "html", "css", "py", "data", "md", "json", "yaml", "toml"];
                let near = codelaw_config::did_you_mean(ns, known).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
                return Err(format!("`{spec}`: no first-party pack has the namespace `{ns}`{near}"));
            };
            let _ = name;
            return Ok(ExtendsSpec::FirstParty { pack, preset: rest.to_owned() });
        }
        if let Some(rest) = spec.strip_prefix("npm:").filter(|r| !r.is_empty()) {
            return Ok(ExtendsSpec::Npm(rest.to_owned()));
        }
        if let Some(rest) = spec.strip_prefix("pypi:").filter(|r| !r.is_empty()) {
            return Ok(ExtendsSpec::Pypi(rest.to_owned()));
        }
        if let Some(rest) = spec.strip_prefix("path:").filter(|r| !r.is_empty()) {
            return Ok(ExtendsSpec::Path(rest.to_owned()));
        }
        Err(format!("`{spec}`: an `extends` entry starts with `codelaw:`, `npm:`, `pypi:` or `path:`"))
    }
}

/// `codelaw-pack.toml`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    pub min_codelaw: String,
    pub languages: Vec<String>,
    #[serde(default)]
    pub presets: Vec<String>,
    pub description: Option<String>,
}

/// `presets/<namespace>/<name>.toml`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PresetFile {
    description: Option<String>,
    #[serde(default)]
    rules: BTreeMap<String, RuleSetting>,
}

#[derive(Clone, Debug)]
pub struct Preset {
    pub name: String,
    pub description: Option<String>,
    pub rules: BTreeMap<String, RuleSetting>,
}

/// A loaded pack.
#[derive(Debug)]
pub struct RulePack {
    pub manifest: PackManifest,
    pub dir: PathBuf,
    pub first_party: bool,
    pub rules: Vec<RuleFile>,
    pub presets: BTreeMap<String, Preset>,
    /// `data/` tables, parsed once and shared read-only
    /// (SR-ARCHITECTURE decision 15c), keyed by their path under `data/`
    /// without the extension.
    pub data: Arc<BTreeMap<String, Arc<serde_json::Value>>>,
    pub data_hash: Key,
}

impl RulePack {
    pub fn label(&self) -> String {
        format!("{}@{}", self.manifest.name, self.manifest.version)
    }
}

/// Where packs are looked for (decision 4).
#[derive(Clone, Debug, Default)]
pub struct PackSearch {
    /// `node_modules` is searched from here upwards.
    pub config_dir: PathBuf,
    /// Python site-packages folders, in order.
    pub site_packages: Vec<PathBuf>,
}

fn site_packages_of(venv: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(venv.join("lib")) {
        let mut names: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("python"))
            .map(|e| e.path().join("site-packages"))
            .filter(|p| p.is_dir())
            .collect();
        names.sort();
        out.extend(names);
    }
    let windows = venv.join("Lib").join("site-packages");
    if windows.is_dir() {
        out.push(windows);
    }
    out
}

impl PackSearch {
    /// `CODELAW_SITE_PACKAGES` (set by the Python launcher), else
    /// `$VIRTUAL_ENV`, else `.venv/` at the project root.
    pub fn new(config_dir: &Path, project_root: &Path, site_packages_env: Option<&str>, virtual_env: Option<&str>) -> Self {
        let site_packages = if let Some(env) = site_packages_env.filter(|e| !e.is_empty()) {
            std::env::split_paths(env).collect()
        } else if let Some(venv) = virtual_env.filter(|e| !e.is_empty()) {
            site_packages_of(Path::new(venv))
        } else {
            site_packages_of(&project_root.join(".venv"))
        };
        PackSearch { config_dir: config_dir.to_owned(), site_packages }
    }

    pub fn find_npm(&self, package: &str) -> Option<PathBuf> {
        let mut dir = Some(self.config_dir.as_path());
        while let Some(d) = dir {
            let candidate = d.join("node_modules").join(package);
            if candidate.join("codelaw-pack.toml").is_file() {
                return Some(candidate);
            }
            dir = d.parent();
        }
        None
    }

    pub fn find_pypi(&self, distribution: &str) -> Option<PathBuf> {
        let module = distribution.replace(['-', '.'], "_").to_ascii_lowercase();
        self.site_packages.iter().map(|sp| sp.join(&module)).find(|p| p.join("codelaw-pack.toml").is_file())
    }

    /// The npm copy first, then the Python one (decision 4).
    pub fn find_first_party(&self, short: &str) -> Option<PathBuf> {
        let (_, npm, pypi, _) = first_party(short);
        self.find_npm(npm).or_else(|| self.find_pypi(pypi))
    }

    /// The error for a missing first-party pack, naming both installs.
    pub fn missing_first_party(spec: &str, short: &str) -> String {
        let (_, npm, pypi, _) = first_party(short);
        format!("`{spec}` needs the rule pack {npm} (npm) or {pypi} (PyPI); install it with `npm i -D {npm}` or `pip install {pypi}`")
    }
}

fn read_rules(dir: &Path, errors: &mut Vec<LoadError>) -> Vec<RuleFile> {
    let mut rules = Vec::new();
    let Ok(namespaces) = fs::read_dir(dir) else { return rules };
    let mut namespaces: Vec<_> = namespaces.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    namespaces.sort();
    for ns_dir in namespaces {
        let ns = ns_dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let Ok(files) = fs::read_dir(&ns_dir) else { continue };
        let mut files: Vec<_> = files.flatten().map(|e| e.path()).collect();
        files.sort();
        for file in files {
            if file.is_dir() {
                errors.push(LoadError::new(
                    Some(file.clone()),
                    "rule files live at `<namespace>/<name>.md`, one folder deep (SR-RULE-FORMAT decision 1)",
                ));
                continue;
            }
            let name = file.file_name().unwrap_or_default().to_string_lossy().into_owned();
            if !name.ends_with(".md") || name.ends_with(".cases.md") || name.eq_ignore_ascii_case("README.md") {
                continue;
            }
            let expected = format!("{ns}/{}", name.trim_end_matches(".md"));
            let text = match fs::read_to_string(&file) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(LoadError::new(Some(file.clone()), format!("cannot read: {e}")));
                    continue;
                }
            };
            match parse_rule(&file, &text, Some(&expected)) {
                Ok(mut rule) => {
                    let cases_path = file.with_file_name(format!("{}.cases.md", name.trim_end_matches(".md")));
                    if let Ok(cases_text) = fs::read_to_string(&cases_path) {
                        match parse_cases(&cases_path, &cases_text) {
                            Ok(cases) => {
                                rule.cases = cases;
                                rule.hash = KeyBuilder::new("rule").part(rule.hash.0).part(&cases_text).key();
                            }
                            Err(e) => errors.extend(e),
                        }
                    }
                    rules.push(rule);
                }
                Err(e) => errors.extend(e),
            }
        }
    }
    rules
}

/// Load the rule files of a folder: local rules or a pack's `rules/`.
pub fn load_rules_dir(dir: &Path) -> Result<Vec<RuleFile>, Vec<LoadError>> {
    let mut errors = Vec::new();
    let rules = read_rules(dir, &mut errors);
    if errors.is_empty() { Ok(rules) } else { Err(errors) }
}

fn read_data(dir: &Path, errors: &mut Vec<LoadError>) -> (BTreeMap<String, Arc<serde_json::Value>>, Key) {
    let mut out = BTreeMap::new();
    let mut key = KeyBuilder::new("data");
    let mut stack = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "json") {
                files.push(p);
            }
        }
    }
    files.sort();
    for f in files {
        let rel = codelaw_core::normalize_path(f.strip_prefix(dir).unwrap_or(&f));
        let name = rel.trim_end_matches(".json").to_owned();
        match fs::read_to_string(&f) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(v) => {
                    key.push(&rel);
                    key.push(&text);
                    out.insert(name, Arc::new(v));
                }
                Err(e) => errors.push(LoadError::new(Some(f.clone()), format!("invalid JSON data table: {e}"))),
            },
            Err(e) => errors.push(LoadError::new(Some(f.clone()), format!("cannot read: {e}"))),
        }
    }
    (out, key.key())
}

/// Load a pack from its folder. A pack with any failing rule is refused
/// as a whole (decision 13).
pub fn load_pack(dir: &Path) -> Result<RulePack, Vec<LoadError>> {
    let manifest_path = dir.join("codelaw-pack.toml");
    let text = fs::read_to_string(&manifest_path)
        .map_err(|e| vec![LoadError::new(Some(manifest_path.clone()), format!("cannot read the pack manifest: {e}"))])?;
    let manifest: PackManifest = toml::from_str(&text)
        .map_err(|e| vec![LoadError::new(Some(manifest_path.clone()), format!("invalid pack manifest: {}", e.message()))])?;

    let mut errors = Vec::new();
    let ours = semver::Version::parse(codelaw_core::VERSION).expect("engine version is semver");
    match semver::Version::parse(&manifest.min_codelaw) {
        Ok(min) if min > ours => {
            return Err(vec![LoadError::new(
                Some(manifest_path),
                format!(
                    "{}@{} needs codelaw {} or newer, but this is {}; upgrade codelaw (SR-RULE-PACKS decision 5)",
                    manifest.name, manifest.version, manifest.min_codelaw, codelaw_core::VERSION
                ),
            )]);
        }
        Ok(_) => {}
        Err(e) => errors.push(LoadError::new(Some(manifest_path.clone()), format!("`min_codelaw = \"{}\"`: {e}", manifest.min_codelaw))),
    }
    for l in &manifest.languages {
        if Language::from_name(l).is_none() {
            errors.push(LoadError::new(Some(manifest_path.clone()), format!("unknown language `{l}` in `languages`")));
        }
    }
    let first_party = FIRST_PARTY_PACKS.iter().find(|p| p.1 == manifest.name).map(|p| p.0);

    let rules = read_rules(&dir.join("rules"), &mut errors);
    for rule in &rules {
        let ns = rule.namespace();
        match first_party {
            Some(short) => {
                if ns != "graph" && namespace_pack(ns) != Some(short) {
                    errors.push(LoadError::new(
                        Some(rule.path.clone()),
                        format!("namespace `{ns}` does not belong to {} (SR-RULE-PACKS decision 4)", manifest.name),
                    ));
                }
                if rule.block.external_fixer().is_some() {
                    errors.push(LoadError::new(Some(rule.path.clone()), "first-party packs ship no external fixers (decision 12)"));
                }
            }
            None => {
                if RESERVED_NAMESPACES.contains(&ns) {
                    errors.push(LoadError::new(
                        Some(rule.path.clone()),
                        format!("namespace `{ns}` is reserved for the first-party packs (SR-RULE-FORMAT decision 1)"),
                    ));
                }
                if rule.native_id().is_some() {
                    errors.push(LoadError::new(Some(rule.path.clone()), "`native:` is accepted only in first-party packs (decision 10)"));
                }
            }
        }
    }

    let mut presets = BTreeMap::new();
    for name in &manifest.presets {
        let path = dir.join("presets").join(format!("{name}.toml"));
        match fs::read_to_string(&path) {
            Ok(text) => match toml::from_str::<PresetFile>(&text) {
                Ok(p) => {
                    for id in p.rules.keys() {
                        if !rules.iter().any(|r| &r.id == id) {
                            errors.push(LoadError::new(Some(path.clone()), format!("preset `{name}` names `{id}`, which is not in this pack")));
                        }
                    }
                    presets.insert(name.clone(), Preset { name: name.clone(), description: p.description, rules: p.rules });
                }
                Err(e) => errors.push(LoadError::new(Some(path.clone()), format!("invalid preset: {}", e.message()))),
            },
            Err(_) => errors.push(LoadError::new(Some(path.clone()), format!("the manifest lists preset `{name}` but this file is missing"))),
        }
    }

    let (data, data_hash) = read_data(&dir.join("data"), &mut errors);
    if !errors.is_empty() {
        let label = format!("{}@{}", manifest.name, manifest.version);
        let mut out = vec![LoadError::new(Some(dir.to_owned()), format!("the pack {label} is refused: {} rule file problem(s) below", errors.len()))];
        out.extend(errors);
        return Err(out);
    }
    Ok(RulePack { manifest, dir: dir.to_owned(), first_party: first_party.is_some(), rules, presets, data: Arc::new(data), data_hash })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extends_specs() {
        assert_eq!(
            ExtendsSpec::parse("codelaw:react/recommended"),
            Ok(ExtendsSpec::FirstParty { pack: "js", preset: "react/recommended".into() })
        );
        assert_eq!(ExtendsSpec::parse("codelaw:md/recommended"), Ok(ExtendsSpec::FirstParty { pack: "data", preset: "md/recommended".into() }));
        assert!(ExtendsSpec::parse("codelaw:jss/recommended").unwrap_err().contains("did you mean `js`"));
        assert_eq!(ExtendsSpec::parse("npm:@acme/x"), Ok(ExtendsSpec::Npm("@acme/x".into())));
        assert!(ExtendsSpec::parse("github:acme/x").is_err());
    }

    #[test]
    fn missing_pack_names_both_installs() {
        let m = PackSearch::missing_first_party("codelaw:js/recommended", "js");
        assert!(m.contains("npm i -D @codelaw/rules-js") && m.contains("pip install codelaw-rules-js"), "{m}");
    }
}
