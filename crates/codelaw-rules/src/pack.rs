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

/// One first-party pack and where it is published (L6 decision 11,
/// SR-RULE-PACKS decision 2). A pack is published on a registry only when
/// that registry's name is `Some`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstPartyPack {
    /// The folder under `packs/` and the name `extends` resolves to.
    pub short: &'static str,
    /// `name` in its `codelaw-pack.toml`: the npm name when it is on npm,
    /// else the PyPI name.
    pub manifest: &'static str,
    pub npm: Option<&'static str>,
    pub pypi: Option<&'static str>,
}

impl FirstPartyPack {
    /// The Python module a PyPI copy installs as.
    pub fn module(&self) -> Option<String> {
        self.pypi.map(|d| d.replace('-', "_"))
    }

    /// The name on one registry: `npm` or `pypi`.
    pub fn on(&self, registry: &str) -> Option<&'static str> {
        match registry {
            "npm" => self.npm,
            "pypi" => self.pypi,
            _ => None,
        }
    }
}

const fn both(short: &'static str, npm: &'static str, pypi: &'static str) -> FirstPartyPack {
    FirstPartyPack { short, manifest: npm, npm: Some(npm), pypi: Some(pypi) }
}

const fn npm_only(short: &'static str, npm: &'static str) -> FirstPartyPack {
    FirstPartyPack { short, manifest: npm, npm: Some(npm), pypi: None }
}

const fn pypi_only(short: &'static str, pypi: &'static str) -> FirstPartyPack {
    FirstPartyPack { short, manifest: pypi, npm: None, pypi: Some(pypi) }
}

/// The fifteen first-party packs: thirteen on npm, nine on PyPI (the
/// decider's rulings of 2026-10-24: W-31's split, and W-32's two npm-only
/// library packs; and of 2026-10-25, W-53's four security packs).
/// `scripts/package.py` reads this table, so it is the one list of what is
/// published where.
pub const FIRST_PARTY_PACKS: &[FirstPartyPack] = &[
    both("js", "@codelaw/rules-js", "codelaw-rules-js"),
    npm_only("ts", "@codelaw/rules-ts"),
    both("react", "@codelaw/rules-react", "codelaw-rules-react"),
    npm_only("node", "@codelaw/rules-node"),
    npm_only("stencil", "@codelaw/rules-stencil"),
    both("html", "@codelaw/rules-html", "codelaw-rules-html"),
    both("css", "@codelaw/rules-css", "codelaw-rules-css"),
    pypi_only("python", "codelaw-rules-python"),
    both("data", "@codelaw/rules-data", "codelaw-rules-data"),
    npm_only("tanstack-query", "@codelaw/rules-tanstack-query"),
    npm_only("redux-toolkit", "@codelaw/rules-redux-toolkit"),
    both("security-js", "@codelaw/rules-security-js", "codelaw-rules-security-js"),
    npm_only("security-ts", "@codelaw/rules-security-ts"),
    both("security-html", "@codelaw/rules-security-html", "codelaw-rules-security-html"),
    pypi_only("security-python", "codelaw-rules-security-python"),
];

/// Every namespace a first-party pack owns, in [`namespace_pack`]'s order.
pub const FIRST_PARTY_NAMESPACES: &[&str] = &[
    "js", "ts", "react", "node", "stencil", "html", "css", "py", "data", "md", "json", "yaml", "toml", "tanstack-query", "redux-toolkit",
    "sec-js", "sec-ts", "sec-html", "sec-py",
];

/// The fixed namespace-to-pack map (decision 4).
pub fn namespace_pack(namespace: &str) -> Option<&'static str> {
    Some(match namespace {
        "js" => "js",
        "ts" => "ts",
        "react" => "react",
        "node" => "node",
        "stencil" => "stencil",
        "html" => "html",
        "css" => "css",
        "py" => "python",
        "data" | "md" | "json" | "yaml" | "toml" => "data",
        "tanstack-query" => "tanstack-query",
        "redux-toolkit" => "redux-toolkit",
        "sec-js" => "security-js",
        "sec-ts" => "security-ts",
        "sec-html" => "security-html",
        "sec-py" => "security-python",
        _ => return None,
    })
}

/// The first-party pack with this short name.
pub fn first_party(short: &str) -> &'static FirstPartyPack {
    FIRST_PARTY_PACKS.iter().find(|p| p.short == short).expect("a first-party pack")
}

/// The first-party pack published under this npm name.
pub fn first_party_npm(name: &str) -> Option<&'static FirstPartyPack> {
    FIRST_PARTY_PACKS.iter().find(|p| p.npm == Some(name))
}

/// The first-party pack published under this PyPI name, in any spelling
/// PEP 503 treats as the same.
pub fn first_party_pypi(name: &str) -> Option<&'static FirstPartyPack> {
    let norm = name.to_ascii_lowercase().replace(['_', '.'], "-");
    FIRST_PARTY_PACKS.iter().find(|p| p.pypi == Some(norm.as_str()))
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
                let near = codelaw_config::did_you_mean(ns, FIRST_PARTY_NAMESPACES.iter().copied()).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
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
    /// First-party presets this one switches on first, as `codelaw:` specs
    /// (SR-RULE-PACKS decision 3).
    #[serde(default)]
    extends: Vec<String>,
    #[serde(default)]
    rules: BTreeMap<String, RuleSetting>,
}

#[derive(Clone, Debug)]
pub struct Preset {
    pub name: String,
    pub description: Option<String>,
    /// `codelaw:<namespace>/<preset>` entries, resolved as `extends`
    /// entries before this preset's own `[rules]`.
    pub extends: Vec<String>,
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

    /// The npm copy first, then the Python one (decision 4), each only on
    /// a registry the pack is published on.
    pub fn find_first_party(&self, short: &str) -> Option<PathBuf> {
        let p = first_party(short);
        p.npm.and_then(|n| self.find_npm(n)).or_else(|| p.pypi.and_then(|d| self.find_pypi(d)))
    }

    /// The error for a missing first-party pack, naming every install it
    /// has, and saying so when it is on one registry only.
    pub fn missing_first_party(spec: &str, short: &str) -> String {
        let p = first_party(short);
        match (p.npm, p.pypi) {
            (Some(npm), Some(pypi)) => {
                format!("`{spec}` needs the rule pack {npm} (npm) or {pypi} (PyPI); install it with `npm i -D {npm}` or `pip install {pypi}`")
            }
            (Some(npm), None) => format!("`{spec}` needs the rule pack {npm}, which is published on npm only; install it with `npm i -D {npm}`"),
            (None, Some(pypi)) => format!("`{spec}` needs the rule pack {pypi}, which is published on PyPI only; install it with `pip install {pypi}`"),
            (None, None) => unreachable!("every first-party pack is on a registry"),
        }
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
    // A folder is entered once, by its real path, so a symlink cycle in
    // `data/` ends instead of looping forever.
    let mut entered = std::collections::BTreeSet::new();
    while let Some(d) = stack.pop() {
        if !entered.insert(fs::canonicalize(&d).unwrap_or_else(|_| d.clone())) {
            continue;
        }
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
/// as a whole (decision 13). It is first-party if its manifest names a
/// first-party pack; [`load_pack_from`] also asks where it was found.
pub fn load_pack(dir: &Path) -> Result<RulePack, Vec<LoadError>> {
    load_pack_from(dir, true)
}

/// Load a pack found through `extends`. `first_party_origin` says whether
/// the spec could only resolve to a first-party install (`codelaw:`, or
/// `npm:`/`pypi:` naming a first-party package): a pack reached any other
/// way is third-party even if its manifest copies a first-party name, so it
/// cannot use `native:` or the reserved namespaces (decision 10).
pub fn load_pack_from(dir: &Path, first_party_origin: bool) -> Result<RulePack, Vec<LoadError>> {
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
    let first_party = FIRST_PARTY_PACKS.iter().find(|p| p.manifest == manifest.name).map(|p| p.short).filter(|_| first_party_origin);

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
                    for spec in &p.extends {
                        match ExtendsSpec::parse(spec) {
                            Ok(ExtendsSpec::FirstParty { preset, .. }) if preset == *name => {
                                errors.push(LoadError::new(Some(path.clone()), format!("preset `{name}` extends itself")));
                            }
                            Ok(ExtendsSpec::FirstParty { .. }) => {}
                            Ok(_) => errors.push(LoadError::new(
                                Some(path.clone()),
                                format!("preset `{name}` extends `{spec}`: a preset extends only first-party presets, `codelaw:<namespace>/<preset>`"),
                            )),
                            Err(e) => errors.push(LoadError::new(Some(path.clone()), format!("preset `{name}`: {e}"))),
                        }
                    }
                    presets.insert(name.clone(), Preset { name: name.clone(), description: p.description, extends: p.extends, rules: p.rules });
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
            Ok(ExtendsSpec::FirstParty { pack: "react", preset: "react/recommended".into() })
        );
        assert_eq!(ExtendsSpec::parse("codelaw:ts/recommended"), Ok(ExtendsSpec::FirstParty { pack: "ts", preset: "ts/recommended".into() }));
        assert_eq!(ExtendsSpec::parse("codelaw:py/recommended"), Ok(ExtendsSpec::FirstParty { pack: "python", preset: "py/recommended".into() }));
        assert_eq!(ExtendsSpec::parse("codelaw:md/recommended"), Ok(ExtendsSpec::FirstParty { pack: "data", preset: "md/recommended".into() }));
        assert!(ExtendsSpec::parse("codelaw:jss/recommended").unwrap_err().contains("did you mean `js`"));
        assert_eq!(
            ExtendsSpec::parse("codelaw:sec-py/recommended"),
            Ok(ExtendsSpec::FirstParty { pack: "security-python", preset: "sec-py/recommended".into() })
        );
        assert!(ExtendsSpec::parse("codelaw:sec-jss/recommended").unwrap_err().contains("did you mean `sec-js`"));
        assert_eq!(ExtendsSpec::parse("npm:@acme/x"), Ok(ExtendsSpec::Npm("@acme/x".into())));
        assert!(ExtendsSpec::parse("github:acme/x").is_err());
    }

    #[test]
    fn missing_pack_names_both_installs() {
        let m = PackSearch::missing_first_party("codelaw:js/recommended", "js");
        assert!(m.contains("npm i -D @codelaw/rules-js") && m.contains("pip install codelaw-rules-js"), "{m}");
    }

    #[test]
    fn a_one_registry_pack_names_only_that_registry() {
        let m = PackSearch::missing_first_party("codelaw:ts/recommended", "ts");
        assert!(m.contains("npm only") && m.contains("npm i -D @codelaw/rules-ts") && !m.contains("pip"), "{m}");
        let m = PackSearch::missing_first_party("codelaw:py/recommended", "python");
        assert!(m.contains("PyPI only") && m.contains("pip install codelaw-rules-python") && !m.contains("npm"), "{m}");
    }

    #[test]
    fn thirteen_packs_on_npm_and_nine_on_pypi() {
        // The decider's rulings of 2026-10-24 and 2026-10-25 (L6 decision 11;
        // W-31, W-32, W-53).
        let npm: Vec<&str> = FIRST_PARTY_PACKS.iter().filter(|p| p.npm.is_some()).map(|p| p.short).collect();
        let pypi: Vec<&str> = FIRST_PARTY_PACKS.iter().filter(|p| p.pypi.is_some()).map(|p| p.short).collect();
        assert_eq!(FIRST_PARTY_PACKS.len(), 15);
        assert_eq!(
            npm,
            ["js", "ts", "react", "node", "stencil", "html", "css", "data", "tanstack-query", "redux-toolkit", "security-js", "security-ts", "security-html"]
        );
        assert_eq!(pypi, ["js", "react", "html", "css", "python", "data", "security-js", "security-html", "security-python"]);
        for p in FIRST_PARTY_PACKS {
            assert_eq!(Some(p.manifest), p.npm.or(p.pypi), "{}", p.short);
            if let Some(n) = p.npm {
                assert_eq!(n, format!("@codelaw/rules-{}", p.short));
            }
            if let Some(d) = p.pypi {
                assert_eq!(d, format!("codelaw-rules-{}", p.short));
            }
        }
        // Every namespace a first-party pack owns maps to a pack in the table.
        for ns in FIRST_PARTY_NAMESPACES {
            let short = namespace_pack(ns).unwrap();
            assert!(FIRST_PARTY_PACKS.iter().any(|p| p.short == short), "{ns}");
            assert!(RESERVED_NAMESPACES.contains(ns), "{ns} is reserved");
        }
        // Every pack owns at least one namespace.
        for p in FIRST_PARTY_PACKS {
            assert!(FIRST_PARTY_NAMESPACES.iter().any(|ns| namespace_pack(ns) == Some(p.short)), "{}", p.short);
        }
    }

    #[test]
    fn every_pack_folder_loads_as_its_first_party_pack() {
        let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs");
        let mut folders: Vec<String> = fs::read_dir(&packs).unwrap().flatten().filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        folders.sort();
        let mut table: Vec<&str> = FIRST_PARTY_PACKS.iter().map(|p| p.short).collect();
        table.sort();
        assert_eq!(folders, table, "packs/ and FIRST_PARTY_PACKS name the same packs");
        let mut loaded = BTreeMap::new();
        for p in FIRST_PARTY_PACKS {
            let pack = load_pack(&packs.join(p.short)).unwrap_or_else(|e| panic!("{}: {e:?}", p.short));
            assert_eq!(pack.manifest.name, p.manifest);
            assert!(pack.first_party, "{}", p.short);
            loaded.insert(p.short, pack);
        }
        // A preset extends only presets that exist, in packs published on
        // every registry its own pack is (decision 3): `sec-ts` on npm needs
        // `sec-js` on npm, `sec-html` on both needs `html` on both.
        for p in FIRST_PARTY_PACKS {
            for preset in loaded[p.short].presets.values() {
                for spec in &preset.extends {
                    let Ok(ExtendsSpec::FirstParty { pack: other, preset: name }) = ExtendsSpec::parse(spec) else {
                        panic!("{}: `{spec}` is not a first-party preset", preset.name)
                    };
                    let o = first_party(other);
                    assert!(p.npm.is_none() || o.npm.is_some(), "{} extends {spec}, which is not on npm", preset.name);
                    assert!(p.pypi.is_none() || o.pypi.is_some(), "{} extends {spec}, which is not on PyPI", preset.name);
                    assert!(loaded[other].presets.contains_key(&name), "{} extends {spec}, which {other} does not have", preset.name);
                }
            }
        }
    }

    #[test]
    fn a_preset_extends_only_first_party_presets() {
        let dir = std::env::temp_dir().join(format!("codelaw-preset-extends-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("presets/acme")).unwrap();
        fs::write(dir.join("codelaw-pack.toml"), "name = \"acme\"\nversion = \"1.0.0\"\nmin_codelaw = \"0.1.0\"\nlanguages = [\"javascript\"]\npresets = [\"acme/a\", \"acme/b\"]\n").unwrap();
        fs::write(dir.join("presets/acme/a.toml"), "extends = [\"npm:@acme/other\"]\n").unwrap();
        fs::write(dir.join("presets/acme/b.toml"), "extends = [\"codelaw:acme/b\"]\n").unwrap();
        let e: Vec<String> = load_pack(&dir).unwrap_err().into_iter().map(|e| e.message).collect();
        assert!(e.iter().any(|m| m.contains("extends only first-party presets")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("no first-party pack has the namespace `acme`")), "{e:?}");
        let _ = fs::remove_dir_all(&dir);
    }
}
