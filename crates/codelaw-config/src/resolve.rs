use crate::raw::{RawConfig, RawDepsIgnore, RawOverride, RawProfile, RuleSetting};
use crate::{ConfigError, did_you_mean, explain_serde_error};
use codelaw_cache::{Key, KeyBuilder};
use codelaw_core::Language;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Which of the three homes a config came from (SR-CONFIG decision 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Home {
    CodelawToml,
    Pyproject,
    PackageJson,
}

impl Home {
    pub fn file_name(self) -> &'static str {
        match self {
            Home::CodelawToml => "codelaw.toml",
            Home::Pyproject => "pyproject.toml",
            Home::PackageJson => "package.json",
        }
    }
}

/// An override's `files` and `exclude` globs.
pub(crate) type OverrideGlobs = (GlobSet, GlobSet);

/// One config file, parsed and validated.
#[derive(Clone, Debug)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub dir: PathBuf,
    pub home: Home,
    pub raw: RawConfig,
    /// The config text, hashed into every cache key.
    pub text: String,
    include: Option<GlobSet>,
    exclude: GlobSet,
    pub(crate) overrides: Vec<(GlobSet, GlobSet)>,
    /// Each profile's `[[overrides]]` globs, by profile name.
    pub(crate) profile_overrides: BTreeMap<String, Vec<(GlobSet, GlobSet)>>,
}

/// One layer of rule settings that applies to a file.
pub struct Layer<'a> {
    pub source: String,
    pub rules: &'a BTreeMap<String, RuleSetting>,
}

/// The profile a run uses and where the choice came from (SR-CONFIG
/// decision 10): `ci` from `CI=true`, `local` by default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// `--profile`, `CODELAW_PROFILE`, `CI=<value>`, `<surface> default`
    /// or `default`.
    pub source: String,
}

impl Default for Profile {
    fn default() -> Self {
        Profile { name: "local".into(), source: "default".into() }
    }
}

impl Profile {
    /// The report header's line: `profile ci (CI=true)`.
    pub fn line(&self) -> String {
        format!("profile {} ({})", self.name, self.source)
    }
}

/// What picks the profile (SR-CONFIG decision 10): `--profile` >
/// `CODELAW_PROFILE` > the surface's own default > `CI` set and not `""`,
/// `false` or `0` → `ci` > `local`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileChoice {
    /// `--profile <name>`.
    pub flag: Option<String>,
    /// `CODELAW_PROFILE`.
    pub env: Option<String>,
    /// `CI`.
    pub ci: Option<String>,
    /// A surface's default, which takes the place of the `CI` check: the
    /// profile and the surface, e.g. `("ci", "codelaw hook")`.
    pub surface: Option<(&'static str, &'static str)>,
}

impl ProfileChoice {
    pub fn resolve(&self) -> Profile {
        let given = |v: &Option<String>| v.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(str::to_owned);
        if let Some(name) = given(&self.flag) {
            return Profile { name, source: "--profile".into() };
        }
        if let Some(name) = given(&self.env) {
            return Profile { name, source: "CODELAW_PROFILE".into() };
        }
        if let Some((name, surface)) = self.surface {
            return Profile { name: name.into(), source: format!("{surface} default") };
        }
        match self.ci.as_deref() {
            Some(v) if !v.is_empty() && !v.eq_ignore_ascii_case("false") && v != "0" => Profile { name: "ci".into(), source: format!("CI={v}") },
            _ => Profile::default(),
        }
    }
}

/// What `remap` does to the findings of one file (SR-CONFIG decision 11):
/// for `error`, `warn` and `info`, the severity it becomes, `None` being
/// `off`. A severity no matching override names is kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Remap([Option<Option<codelaw_core::Severity>>; 3]);

fn severity_slot(s: codelaw_core::Severity) -> usize {
    match s {
        codelaw_core::Severity::Error => 0,
        codelaw_core::Severity::Warn => 1,
        codelaw_core::Severity::Info => 2,
    }
}

impl Remap {
    /// Lay `table` over this one: later wins per key.
    fn merge(&mut self, table: &BTreeMap<String, String>) {
        for (from, to) in table {
            if let Some(from) = codelaw_core::Severity::parse(from) {
                self.0[severity_slot(from)] = Some(codelaw_core::Severity::parse(to));
            }
        }
    }

    /// Whether nothing changes.
    pub fn is_identity(&self) -> bool {
        self.0.iter().all(Option::is_none)
    }

    /// The severity `s` becomes; `None` is `off`.
    pub fn apply(&self, s: codelaw_core::Severity) -> Option<codelaw_core::Severity> {
        self.0[severity_slot(s)].unwrap_or(Some(s))
    }
}

/// A rule's settings, resolved for one file.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleConfig {
    /// `None` is `off`.
    pub severity: Option<codelaw_core::Severity>,
    pub options: BTreeMap<String, serde_json::Value>,
}

fn glob_set(patterns: &[String], file: &Path) -> Result<GlobSet, ConfigError> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        let glob = GlobBuilder::new(p.trim_start_matches("./"))
            .literal_separator(true)
            .build()
            .map_err(|e| ConfigError::new(Some(file.to_owned()), format!("invalid glob `{p}`: {e}")))?;
        b.add(glob);
    }
    b.build().map_err(|e| ConfigError::new(Some(file.to_owned()), e.to_string()))
}

fn validate_severity(file: &Path, id: &str, setting: &RuleSetting) -> Result<(), ConfigError> {
    if let Some(s) = setting.severity()
        && !matches!(s, "error" | "warn" | "info" | "off")
    {
        let near = did_you_mean(s, ["error", "warn", "info", "off"]).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
        return Err(ConfigError::new(
            Some(file.to_owned()),
            format!("rule `{id}`: invalid severity `{s}`{near} (one of error, warn, info, off)"),
        ));
    }
    Ok(())
}

const REMAP_KEYS: [&str; 3] = ["error", "warn", "info"];
const REMAP_VALUES: [&str; 4] = ["error", "warn", "info", "off"];

/// An `[[overrides]]` entry's severities and `remap` (SR-CONFIG decision 11).
fn validate_override(file: &Path, what: &str, o: &RawOverride) -> Result<(), ConfigError> {
    for (id, s) in &o.rules {
        validate_severity(file, id, s)?;
    }
    for (from, to) in o.remap.iter().flatten() {
        if !REMAP_KEYS.contains(&from.as_str()) {
            let near = did_you_mean(from, REMAP_KEYS).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
            return Err(ConfigError::new(Some(file.to_owned()), format!("{what}: `remap` key `{from}`{near} (one of error, warn, info)")));
        }
        if !REMAP_VALUES.contains(&to.as_str()) {
            let near = did_you_mean(to, REMAP_VALUES).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
            return Err(ConfigError::new(
                Some(file.to_owned()),
                format!("{what}: `remap` maps `{from}` to `{to}`{near} (one of error, warn, info, off)"),
            ));
        }
    }
    Ok(())
}

fn override_globs(overrides: &[RawOverride], path: &Path) -> Result<Vec<(GlobSet, GlobSet)>, ConfigError> {
    overrides.iter().map(|o| Ok((glob_set(&o.files, path)?, glob_set(&o.exclude, path)?))).collect()
}

impl ConfigFile {
    fn new(path: PathBuf, home: Home, raw: RawConfig, text: String) -> Result<ConfigFile, ConfigError> {
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        if let Some(langs) = &raw.languages {
            for name in langs {
                if Language::from_name(name).is_none() {
                    let near = did_you_mean(name, Language::ALL.iter().map(|l| l.name()))
                        .map(|n| format!("; did you mean `{n}`?"))
                        .unwrap_or_default();
                    return Err(ConfigError::new(Some(path), format!("unknown language `{name}` in `languages`{near}")));
                }
            }
        }
        if let Some(req) = &raw.requires {
            let parsed = semver::VersionReq::parse(req)
                .map_err(|e| ConfigError::new(Some(path.clone()), format!("`requires = \"{req}\"` is not a version range: {e}")))?;
            let ours = semver::Version::parse(codelaw_core::VERSION).expect("engine version is semver");
            if !parsed.matches(&ours) {
                return Err(ConfigError::new(
                    Some(path),
                    format!("requires codelaw {req}, but this is {}; upgrade codelaw", codelaw_core::VERSION),
                ));
            }
        }
        for (id, s) in &raw.rules {
            validate_severity(&path, id, s)?;
        }
        for (i, o) in raw.overrides.iter().enumerate() {
            validate_override(&path, &format!("override #{}", i + 1), o)?;
        }
        for (name, p) in &raw.profiles {
            for (id, s) in &p.rules {
                validate_severity(&path, id, s)?;
            }
            for (i, o) in p.overrides.iter().enumerate() {
                validate_override(&path, &format!("profile `{name}` override #{}", i + 1), o)?;
            }
        }
        for e in raw.deps.ignore.iter().flatten() {
            let p = e.package();
            let named = p.strip_prefix("npm:").or_else(|| p.strip_prefix("pypi:")).is_some_and(|n| !n.is_empty());
            if !named {
                return Err(ConfigError::new(Some(path), format!("`[deps] ignore`: `{p}` names no package; write `npm:<name>` or `pypi:<name>`")));
            }
            if let RawDepsIgnore::Dated(d) = e
                && let Some(x) = d.expires.as_deref().filter(|x| !is_date(x))
            {
                return Err(ConfigError::new(Some(path), format!("`[deps] ignore`: `expires = \"{x}\"` is not a date; write it as YYYY-MM-DD")));
            }
        }
        let include = if raw.include.is_empty() { None } else { Some(glob_set(&raw.include, &path)?) };
        let exclude = glob_set(&raw.exclude, &path)?;
        let overrides = override_globs(&raw.overrides, &path)?;
        let profile_overrides =
            raw.profiles.iter().map(|(name, p)| Ok((name.clone(), override_globs(&p.overrides, &path)?))).collect::<Result<_, ConfigError>>()?;
        Ok(ConfigFile { path, dir, home, raw, text, include, exclude, overrides, profile_overrides })
    }

    /// The path of `file` relative to this config's folder, if under it.
    pub(crate) fn relative<'p>(&self, file: &'p Path) -> Option<&'p Path> {
        file.strip_prefix(&self.dir).ok()
    }
}

/// Load the config home of one folder, if it has one. A folder with more
/// than one home uses the first and warns about the rest.
pub fn load_file(dir: &Path, warnings: &mut Vec<String>) -> Result<Option<ConfigFile>, ConfigError> {
    let mut found: Vec<ConfigFile> = Vec::new();
    for home in [Home::CodelawToml, Home::Pyproject, Home::PackageJson] {
        let path = dir.join(home.file_name());
        let Ok(text) = fs::read_to_string(&path) else { continue };
        let raw = match home {
            Home::CodelawToml => Some(
                toml::from_str::<RawConfig>(&text)
                    .map_err(|e| ConfigError::new(Some(path.clone()), toml_error(&text, &e)))?,
            ),
            // A `pyproject.toml` or `package.json` is a home only if it
            // configures codelaw. One that doesn't parse is a config error
            // only when it could: it mentions `codelaw` (SR-CONFIG decision 1).
            Home::Pyproject => {
                let doc: toml::Table = match toml::from_str(&text) {
                    Ok(d) => d,
                    Err(_) if !text.contains("codelaw") => continue,
                    Err(e) => return Err(ConfigError::new(Some(path.clone()), toml_error(&text, &e))),
                };
                match doc.get("tool").and_then(|t| t.get("codelaw")) {
                    Some(v) => Some(v.clone().try_into::<RawConfig>().map_err(|e| {
                        ConfigError::new(Some(path.clone()), format!("[tool.codelaw]: {}", explain_serde_error(&e.to_string())))
                    })?),
                    None => None,
                }
            }
            Home::PackageJson => {
                let doc: serde_json::Value = match serde_json::from_str(&text) {
                    Ok(d) => d,
                    Err(_) if !text.contains("\"codelaw\"") => continue,
                    Err(e) => return Err(ConfigError::new(Some(path.clone()), format!("invalid JSON: {e}"))),
                };
                match doc.get("codelaw") {
                    Some(v) => Some(serde_json::from_value::<RawConfig>(v.clone()).map_err(|e| {
                        ConfigError::new(Some(path.clone()), format!("\"codelaw\": {}", explain_serde_error(&e.to_string())))
                    })?),
                    None => None,
                }
            }
        };
        if let Some(raw) = raw {
            found.push(ConfigFile::new(path, home, raw, text)?);
        }
    }
    let mut found = found.into_iter();
    let first = found.next();
    for other in found {
        if let Some(f) = &first {
            warnings.push(format!(
                "{} is ignored: {} in the same folder wins (SR-CONFIG decision 1)",
                codelaw_core::normalize_path(&other.path),
                f.home.file_name()
            ));
        }
    }
    Ok(first)
}

fn toml_error(text: &str, e: &toml::de::Error) -> String {
    let what = explain_serde_error(e.message());
    match e.span() {
        Some(span) => {
            let pos = codelaw_core::LineIndex::new(text).position(span.start);
            format!("{}:{}: {what}", pos.line, pos.column)
        }
        None => what,
    }
}

/// The resolved configuration of one project.
#[derive(Clone, Debug)]
pub struct Config {
    /// The project root: the folder of the topmost config, or of the
    /// nearest `root = true` one (SR-CONFIG decision 2).
    pub root_dir: PathBuf,
    /// Root config first, then nested configs, outermost first.
    pub files: Vec<ConfigFile>,
    pub warnings: Vec<String>,
    /// The profile this run uses (SR-CONFIG decision 10); `local` until
    /// [`Config::select_profile`] says otherwise.
    pub profile: Profile,
}

/// Folders and files the walk never enters, before any config.
pub const DEFAULT_EXCLUDES: &[&str] = &[".git", "node_modules", ".venv", ".codelaw/cache"];

/// Keys a nested config may not set yet; they belong to the root config.
const ROOT_ONLY: &[&str] = &[
    "extends", "rules_dir", "preview", "allow_pack_fixers", "[files]", "[cache]", "[fix]", "[lint]", "[markdown]", "[daemon]", "[lsp]", "[profiles]",
    "[deps]",
];

fn root_only_keys(raw: &RawConfig) -> Vec<&'static str> {
    let set = [
        !raw.extends.is_empty(),
        raw.rules_dir.is_some(),
        raw.preview.is_some(),
        !raw.allow_pack_fixers.is_empty(),
        raw.files != Default::default(),
        raw.cache != Default::default(),
        raw.fix != Default::default(),
        raw.lint != Default::default(),
        raw.markdown != Default::default(),
        raw.daemon != Default::default(),
        raw.lsp != Default::default(),
        !raw.profiles.is_empty(),
        raw.deps != Default::default(),
    ];
    ROOT_ONLY.iter().zip(set).filter(|(_, s)| *s).map(|(k, _)| *k).collect()
}

/// Find the configuration that governs `start` by walking up to the
/// project root. With no config anywhere, `start` is the root and every
/// value is a default.
pub fn find_config(start: &Path) -> Result<Config, ConfigError> {
    let start = fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    let start_dir = if start.is_file() { start.parent().map(Path::to_path_buf).unwrap_or_default() } else { start };
    let mut warnings = Vec::new();
    let mut chain = Vec::new();
    let mut dir = Some(start_dir.as_path());
    while let Some(d) = dir {
        if let Some(f) = load_file(d, &mut warnings)? {
            let is_root = f.raw.root;
            chain.push(f);
            if is_root {
                break;
            }
        }
        dir = d.parent();
    }
    chain.reverse();
    let root_dir = chain.first().map(|f| f.dir.clone()).unwrap_or(start_dir);
    let config = Config { root_dir, files: chain, warnings, profile: Profile::default() };
    config.check_nested()?;
    Ok(config)
}

impl Config {
    /// A config with every default, rooted at `root_dir`.
    pub fn defaults(root_dir: PathBuf) -> Config {
        Config { root_dir, files: Vec::new(), warnings: Vec::new(), profile: Profile::default() }
    }

    /// A config from TOML text, as if it were `root_dir/codelaw.toml`.
    pub fn from_toml(root_dir: PathBuf, text: &str) -> Result<Config, ConfigError> {
        let path = root_dir.join("codelaw.toml");
        let raw = toml::from_str::<RawConfig>(text).map_err(|e| ConfigError::new(Some(path.clone()), toml_error(text, &e)))?;
        let file = ConfigFile::new(path, Home::CodelawToml, raw, text.to_owned())?;
        Ok(Config { root_dir, files: vec![file], warnings: Vec::new(), profile: Profile::default() })
    }

    fn check_nested(&self) -> Result<(), ConfigError> {
        for f in self.files.iter().skip(1) {
            let keys = root_only_keys(&f.raw);
            if !keys.is_empty() {
                return Err(ConfigError::new(
                    Some(f.path.clone()),
                    format!("{} can only be set in the root config ({}) in this version", keys.join(", "), codelaw_core::normalize_path(&self.files[0].path)),
                ));
            }
        }
        Ok(())
    }

    /// Add a config found below the root during discovery. A nested
    /// `root = true` config is a separate project: the caller skips its
    /// files and the returned flag says so.
    pub fn add_nested(&mut self, file: ConfigFile) -> Result<bool, ConfigError> {
        if self.files.iter().any(|f| f.path == file.path) {
            return Ok(true);
        }
        if file.raw.root {
            self.warnings.push(format!(
                "{} is a separate project (root = true); run codelaw from {} to check it",
                codelaw_core::normalize_path(&file.path),
                codelaw_core::normalize_path(&file.dir)
            ));
            return Ok(false);
        }
        self.files.push(file);
        self.files.sort_by_key(|f| f.dir.components().count());
        self.check_nested()?;
        Ok(true)
    }

    /// Pick the profile (SR-CONFIG decision 10). A name the root config
    /// does not define is an error, except `ci` and `local`, which change
    /// nothing when absent.
    pub fn select_profile(&mut self, choice: &ProfileChoice) -> Result<(), ConfigError> {
        let profile = choice.resolve();
        let defined = self.root_raw().map(|r| &r.profiles);
        if !matches!(profile.name.as_str(), "ci" | "local") && defined.is_none_or(|d| !d.contains_key(&profile.name)) {
            let names: Vec<&str> = defined.into_iter().flat_map(|d| d.keys().map(String::as_str)).collect();
            let near = did_you_mean(&profile.name, names.iter().copied()).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
            let have = if names.is_empty() { "no profiles".to_owned() } else { names.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ") };
            return Err(ConfigError::new(
                self.root_file().map(Path::to_path_buf),
                format!("profile `{}` (from {}) is not defined{near}; the config defines {have}", profile.name, profile.source),
            ));
        }
        self.profile = profile;
        Ok(())
    }

    /// Whether the root config defines any profile: then every report
    /// names the one in use.
    pub fn profile_in_play(&self) -> bool {
        self.root_raw().is_some_and(|r| !r.profiles.is_empty())
    }

    /// The root config and the profile in use, when it defines it.
    pub(crate) fn active_profile(&self) -> Option<(&ConfigFile, &RawProfile, &[OverrideGlobs])> {
        let root = self.files.first()?;
        let p = root.raw.profiles.get(&self.profile.name)?;
        Some((root, p, root.profile_overrides.get(&self.profile.name).map(Vec::as_slice).unwrap_or_default()))
    }

    fn root_raw(&self) -> Option<&RawConfig> {
        self.files.first().map(|f| &f.raw)
    }

    pub fn extends(&self) -> &[String] {
        self.root_raw().map(|r| r.extends.as_slice()).unwrap_or_default()
    }

    pub fn root_file(&self) -> Option<&Path> {
        self.files.first().map(|f| f.path.as_path())
    }

    /// The folder `extends` resolves from: the root config's folder.
    pub fn config_dir(&self) -> &Path {
        self.files.first().map(|f| f.dir.as_path()).unwrap_or(&self.root_dir)
    }

    pub fn rules_dir(&self) -> PathBuf {
        let rel = self.root_raw().and_then(|r| r.rules_dir.clone()).unwrap_or_else(|| ".codelaw/rules".into());
        self.config_dir().join(rel)
    }

    pub fn preview(&self) -> bool {
        self.root_raw().and_then(|r| r.preview).unwrap_or(false)
    }

    pub fn allow_pack_fixers(&self) -> &[String] {
        self.root_raw().map(|r| r.allow_pack_fixers.as_slice()).unwrap_or_default()
    }

    pub fn follow_symlinks(&self) -> bool {
        self.root_raw().and_then(|r| r.files.follow_symlinks).unwrap_or(false)
    }

    pub fn lint_generated(&self) -> bool {
        self.root_raw().and_then(|r| r.files.lint_generated).unwrap_or(false)
    }

    pub fn generated_markers(&self) -> Vec<String> {
        let mut m = vec!["@generated".to_owned()];
        if let Some(extra) = self.root_raw().and_then(|r| r.files.generated_markers.clone()) {
            m.extend(extra);
        }
        m
    }

    /// Default 1 MiB (SR-ARCHITECTURE decision 10).
    pub fn max_file_size(&self) -> u64 {
        self.root_raw().and_then(|r| r.files.max_file_size).map(|s| s.0).unwrap_or(1 << 20)
    }

    pub fn cache_enabled(&self) -> bool {
        self.root_raw().and_then(|r| r.cache.enabled).unwrap_or(true)
    }

    /// `CODELAW_CACHE_DIR`, else `[cache] dir`, else `.codelaw/cache/`
    /// under the project root (SR-CACHE decision 3).
    pub fn cache_dir(&self, env: Option<&str>) -> PathBuf {
        if let Some(e) = env.filter(|e| !e.is_empty()) {
            return PathBuf::from(e);
        }
        match self.root_raw().and_then(|r| r.cache.dir.clone()) {
            Some(d) => self.config_dir().join(d),
            None => self.root_dir.join(".codelaw").join("cache"),
        }
    }

    /// Default 512 MB (SR-CACHE decision 6).
    pub fn cache_max_size(&self) -> u64 {
        self.root_raw().and_then(|r| r.cache.max_size).map(|s| s.0).unwrap_or(512_000_000)
    }

    /// Default 5 s (SR-ARCHITECTURE decision 18).
    pub fn external_timeout(&self) -> Duration {
        self.root_raw().and_then(|r| r.fix.external_timeout).map(|d| d.0).unwrap_or(Duration::from_secs(5))
    }

    /// `[adapters] enabled`; `None` is "the default adapters" (SR-LINTER
    /// decision 2). The names are checked against the manifests by the
    /// engine.
    pub fn adapters_enabled(&self) -> Option<&[String]> {
        self.root_raw().and_then(|r| r.adapters.enabled.as_deref())
    }

    /// Default 60 s per run of an adapter (SR-LINTER decision 2).
    pub fn adapter_timeout(&self) -> Duration {
        self.root_raw().and_then(|r| r.adapters.timeout).map(|d| d.0).unwrap_or(Duration::from_secs(60))
    }

    /// Default 30 minutes (SR-ARCHITECTURE decision 22).
    pub fn daemon_idle_timeout(&self) -> Duration {
        self.root_raw().and_then(|r| r.daemon.idle_timeout).map(|d| d.0).unwrap_or(Duration::from_secs(30 * 60))
    }

    /// How many closed files' syntax trees a warm process keeps, most
    /// recently touched first; default 256 (SR-ARCHITECTURE decision 24).
    pub fn lsp_tree_cache(&self) -> usize {
        self.root_raw().and_then(|r| r.lsp.tree_cache).unwrap_or(256)
    }

    pub fn lint_code_blocks(&self) -> bool {
        self.root_raw().and_then(|r| r.markdown.lint_code_blocks).unwrap_or(false)
    }

    /// The keys of a root config section as JSON options, for a native
    /// rule that reads one (`[deps]`: SR-RULE-PACKS decision 15).
    pub fn section_options(&self, name: &str) -> BTreeMap<String, serde_json::Value> {
        let value = match (name, self.root_raw()) {
            ("deps", Some(r)) => serde_json::to_value(&r.deps).ok(),
            _ => None,
        };
        match value {
            Some(serde_json::Value::Object(m)) => m.into_iter().collect(),
            _ => BTreeMap::new(),
        }
    }

    pub fn tagged_templates(&self) -> bool {
        self.root_raw().and_then(|r| r.lint.tagged_templates).unwrap_or(false)
    }

    /// The configs that govern `file` (absolute), outermost first.
    fn chain_for<'a>(&'a self, file: &'a Path) -> impl Iterator<Item = &'a ConfigFile> + 'a {
        self.files.iter().filter(move |f| file.starts_with(&f.dir))
    }

    /// The languages switched on for `file`: the innermost config that sets
    /// `languages` wins. `None` is all of them.
    pub fn languages_for(&self, file: &Path) -> Option<BTreeSet<Language>> {
        self.chain_for(file)
            .filter_map(|f| f.raw.languages.as_ref())
            .last()
            .map(|names| names.iter().filter_map(|n| Language::from_name(n)).collect())
    }

    /// `include` and `exclude` of every governing config.
    pub fn is_included(&self, file: &Path) -> bool {
        self.chain_for(file).all(|f| {
            let Some(rel) = f.relative(file) else { return true };
            f.include.as_ref().is_none_or(|inc| inc.is_match(rel)) && !f.exclude.is_match(rel)
        })
    }

    /// The `[[overrides]]` that match `file`, in precedence order: each
    /// config's, outermost first and in file order, then the profile's
    /// (SR-CONFIG decision 4); each with whether it is the profile's and
    /// its index in its list.
    pub(crate) fn overrides_for<'a>(&'a self, file: &Path) -> Vec<(&'a ConfigFile, bool, usize, &'a RawOverride)> {
        let mut out = Vec::new();
        let mut matching = |f: &'a ConfigFile, overrides: &'a [RawOverride], globs: &'a [(GlobSet, GlobSet)], in_profile: bool| {
            let Some(rel) = f.relative(file) else { return };
            for (i, (o, (files, exclude))) in overrides.iter().zip(globs).enumerate() {
                if files.is_match(rel) && !exclude.is_match(rel) {
                    out.push((f, in_profile, i, o));
                }
            }
        };
        for f in self.files.iter().filter(|f| file.starts_with(&f.dir)) {
            matching(f, &f.raw.overrides, &f.overrides, false);
        }
        if let Some((f, p, globs)) = self.active_profile() {
            matching(f, &p.overrides, globs, true);
        }
        out
    }

    /// `<config> override #n`, or `<config> profile <name> override #n`.
    pub(crate) fn override_label(&self, f: &ConfigFile, in_profile: bool, i: usize) -> String {
        let path = codelaw_core::normalize_path(&f.path);
        if in_profile { format!("{path} profile {} override #{}", self.profile.name, i + 1) } else { format!("{path} override #{}", i + 1) }
    }

    /// The rule-setting layers for `file`, lowest precedence first: each
    /// config's `[rules]`, outermost first, then the profile's `rules`,
    /// then the matching `[[overrides]]` in file order, then the profile's
    /// (SR-CONFIG decision 4).
    pub fn layers_for<'a>(&'a self, file: &'a Path) -> Vec<Layer<'a>> {
        let mut layers = Vec::new();
        for f in self.chain_for(file) {
            layers.push(Layer { source: codelaw_core::normalize_path(&f.path), rules: &f.raw.rules });
        }
        if let Some((f, p, _)) = self.active_profile().filter(|(f, _, _)| file.starts_with(&f.dir)) {
            layers.push(Layer { source: format!("{} profile {}", codelaw_core::normalize_path(&f.path), self.profile.name), rules: &p.rules });
        }
        for (f, in_profile, i, o) in self.overrides_for(file) {
            layers.push(Layer { source: self.override_label(f, in_profile, i), rules: &o.rules });
        }
        layers
    }

    /// Whether any override, a profile's included, sets `remap`: without
    /// one the stage costs nothing.
    pub fn has_remap(&self) -> bool {
        self.files.iter().any(|f| f.raw.overrides.iter().chain(f.raw.profiles.values().flat_map(|p| &p.overrides)).any(|o| o.remap.is_some()))
    }

    /// What `remap` does to the findings in `file` (SR-CONFIG decision 11):
    /// every matching override's table, later winning per key.
    pub fn remap_for(&self, file: &Path) -> Remap {
        let mut r = Remap::default();
        for (_, _, _, o) in self.overrides_for(file) {
            if let Some(t) = &o.remap {
                r.merge(t);
            }
        }
        r
    }

    /// Every rule id named anywhere in the config, with the file naming it.
    pub fn named_rule_ids(&self) -> Vec<(String, PathBuf)> {
        let mut out = Vec::new();
        for f in &self.files {
            for id in f.raw.rules.keys() {
                out.push((id.clone(), f.path.clone()));
            }
            for o in &f.raw.overrides {
                for id in o.rules.keys() {
                    out.push((id.clone(), f.path.clone()));
                }
            }
            for p in f.raw.profiles.values() {
                for id in p.rules.keys().chain(p.overrides.iter().flat_map(|o| o.rules.keys())) {
                    out.push((id.clone(), f.path.clone()));
                }
            }
        }
        out
    }

    /// The config's part of every cache key (SR-CONFIG decision 9).
    pub fn fingerprint(&self) -> Key {
        let mut k = KeyBuilder::new("config");
        for f in &self.files {
            k.push(codelaw_core::normalize_path(f.path.strip_prefix(&self.root_dir).unwrap_or(&f.path)));
            k.push(&f.text);
        }
        // The profile picks severities, so it is part of the key: a warm
        // `hook` (ci) and an editor (local) never share a verdict.
        if self.profile_in_play() {
            k.push(format!("profile {}", self.profile.name));
        }
        k.key()
    }
}

/// `YYYY-MM-DD`, with a month and a day in range.
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && [&s[..4], &s[5..7], &s[8..]].iter().all(|p| p.bytes().all(|c| c.is_ascii_digit()))
        && (1..=12).contains(&s[5..7].parse::<u8>().unwrap_or(0))
        && (1..=31).contains(&s[8..].parse::<u8>().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("codelaw-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        fs::canonicalize(d).unwrap()
    }

    #[test]
    fn unknown_key_names_the_nearest() {
        let err = Config::from_toml(PathBuf::from("/x"), "extend = []\n").unwrap_err();
        assert!(err.message.contains("unknown key `extend`; did you mean `extends`?"), "{}", err.message);
        let err = Config::from_toml(PathBuf::from("/x"), "[files]\nmax_file_sise = 1\n").unwrap_err();
        assert!(err.message.contains("did you mean `max_file_size`?"), "{}", err.message);
    }

    #[test]
    fn bad_severity_and_language() {
        let err = Config::from_toml(PathBuf::from("/x"), "[rules]\n\"a/b\" = \"eror\"\n").unwrap_err();
        assert!(err.message.contains("did you mean `error`?"), "{}", err.message);
        let err = Config::from_toml(PathBuf::from("/x"), "languages = [\"pyhton\"]\n").unwrap_err();
        assert!(err.message.contains("did you mean `python`?"), "{}", err.message);
    }

    #[test]
    fn requires_refuses_an_older_engine() {
        let err = Config::from_toml(PathBuf::from("/x"), "requires = \">=99.0\"\n").unwrap_err();
        assert!(err.message.contains("upgrade codelaw"), "{}", err.message);
    }

    #[test]
    fn homes_in_priority_order() {
        let d = temp("homes");
        fs::write(d.join("package.json"), r#"{"name":"x","codelaw":{"extends":["codelaw:js/recommended"]}}"#).unwrap();
        let c = find_config(&d).unwrap();
        assert_eq!(c.extends(), ["codelaw:js/recommended"]);
        fs::write(d.join("pyproject.toml"), "[tool.codelaw]\nextends = [\"codelaw:py/recommended\"]\n").unwrap();
        let c = find_config(&d).unwrap();
        assert_eq!(c.extends(), ["codelaw:py/recommended"]);
        assert_eq!(c.warnings.len(), 1, "{:?}", c.warnings);
        fs::write(d.join("pyproject.toml"), "[project]\nname = \"x\"\n").unwrap();
        let c = find_config(&d).unwrap();
        assert_eq!(c.extends(), ["codelaw:js/recommended"]);
        assert!(c.warnings.is_empty());
    }

    #[test]
    fn a_broken_manifest_without_codelaw_is_not_a_home() {
        let d = temp("broken");
        fs::write(d.join("package.json"), "{ \"name\": \"fixture\", oops }").unwrap();
        fs::write(d.join("pyproject.toml"), "[project\nname = 1").unwrap();
        let c = find_config(&d).unwrap();
        assert!(c.files.is_empty(), "neither file configures codelaw");
        fs::write(d.join("package.json"), "{ \"codelaw\": { oops } }").unwrap();
        assert!(find_config(&d).unwrap_err().message.contains("invalid JSON"));
        fs::remove_file(d.join("package.json")).unwrap();
        fs::write(d.join("pyproject.toml"), "[tool.codelaw\nextends = []").unwrap();
        assert!(find_config(&d).is_err());
    }

    #[test]
    fn walk_up_layers_and_overrides() {
        let d = temp("walk");
        fs::write(
            d.join("codelaw.toml"),
            "exclude = [\"dist/**\"]\n[rules]\n\"a/x\" = \"warn\"\n[[overrides]]\nfiles = [\"pkg/**/*.ts\"]\nrules = { \"a/x\" = \"off\" }\n",
        )
        .unwrap();
        fs::create_dir_all(d.join("pkg/src")).unwrap();
        fs::write(d.join("pkg/codelaw.toml"), "languages = [\"typescript\"]\n[rules]\n\"a/y\" = \"error\"\n").unwrap();
        let c = find_config(&d.join("pkg/src")).unwrap();
        assert_eq!(c.root_dir, d);
        assert_eq!(c.files.len(), 2);
        let file = d.join("pkg/src/a.ts");
        let layers = c.layers_for(&file);
        assert_eq!(layers.len(), 3);
        assert!(layers[2].source.ends_with("override #1"));
        assert_eq!(c.languages_for(&file), Some([Language::Typescript].into()));
        assert_eq!(c.languages_for(&d.join("b.ts")), None);
        assert!(!c.is_included(&d.join("dist/a.js")));
        assert!(c.is_included(&d.join("src/a.js")));
    }

    #[test]
    fn the_profile_choice_in_order() {
        let pick = |flag: Option<&str>, env: Option<&str>, ci: Option<&str>, surface| {
            let p = ProfileChoice { flag: flag.map(Into::into), env: env.map(Into::into), ci: ci.map(Into::into), surface }.resolve();
            p.line()
        };
        assert_eq!(pick(None, None, None, None), "profile local (default)");
        assert_eq!(pick(None, None, Some("true"), None), "profile ci (CI=true)");
        for not_ci in ["", "false", "False", "0"] {
            assert_eq!(pick(None, None, Some(not_ci), None), "profile local (default)", "CI={not_ci}");
        }
        assert_eq!(pick(None, Some("staging"), Some("true"), None), "profile staging (CODELAW_PROFILE)");
        assert_eq!(pick(Some("local"), Some("staging"), Some("true"), None), "profile local (--profile)");
        // A surface's default takes the place of the CI check, not of the flag or env.
        assert_eq!(pick(None, None, None, Some(("ci", "codelaw hook"))), "profile ci (codelaw hook default)");
        assert_eq!(pick(None, None, Some("true"), Some(("local", "LSP"))), "profile local (LSP default)");
        assert_eq!(pick(None, Some("ci"), None, Some(("local", "LSP"))), "profile ci (CODELAW_PROFILE)");
    }

    #[test]
    fn profile_layers_and_remap_in_precedence_order() {
        let text = "[rules]\n\"a/x\" = \"warn\"\n\n[[overrides]]\nfiles = [\"s/**\"]\nrules = { \"a/x\" = \"info\" }\nremap = { error = \"warn\", info = \"off\" }\n\n[profiles.ci]\nrules = { \"a/x\" = \"error\" }\noverrides = [{ files = [\"s/**\"], remap = { error = \"info\" } }]\n";
        let mut c = Config::from_toml(PathBuf::from("/x"), text).unwrap();
        let file = PathBuf::from("/x/s/a.ts");
        let sources = |c: &Config| c.layers_for(&file).into_iter().map(|l| l.source).collect::<Vec<_>>();
        assert_eq!(sources(&c), ["/x/codelaw.toml", "/x/codelaw.toml override #1"], "`local` is not defined: no change");
        let local_key = c.fingerprint();
        c.select_profile(&ProfileChoice { ci: Some("true".into()), ..Default::default() }).unwrap();
        assert_eq!(sources(&c), ["/x/codelaw.toml", "/x/codelaw.toml profile ci", "/x/codelaw.toml override #1", "/x/codelaw.toml profile ci override #1"]);
        assert_ne!(c.fingerprint(), local_key, "the profile is part of every cache key");
        use codelaw_core::Severity::*;
        let r = c.remap_for(&file);
        assert_eq!((r.apply(Error), r.apply(Warn), r.apply(Info)), (Some(Info), Some(Warn), None), "the later override wins per key");
        assert!(c.remap_for(&PathBuf::from("/x/t/a.ts")).is_identity());
        assert!(c.has_remap());
        let err = c.select_profile(&ProfileChoice { flag: Some("cis".into()), ..Default::default() }).unwrap_err();
        assert!(err.message.contains("profile `cis` (from --profile) is not defined; did you mean `ci`?"), "{}", err.message);
        let err = Config::from_toml(PathBuf::from("/x"), "[profiles.ci]\noverrides = [{ files = [\"a\"], remap = { eror = \"warn\" } }]\n").unwrap_err();
        assert!(err.message.contains("profile `ci` override #1: `remap` key `eror`; did you mean `error`?"), "{}", err.message);
    }

    #[test]
    fn nested_config_cannot_set_root_keys() {
        let d = temp("nested");
        fs::write(d.join("codelaw.toml"), "").unwrap();
        fs::create_dir_all(d.join("pkg")).unwrap();
        fs::write(d.join("pkg/codelaw.toml"), "extends = [\"codelaw:js/recommended\"]\n").unwrap();
        let err = find_config(&d.join("pkg")).unwrap_err();
        assert!(err.message.contains("can only be set in the root config"), "{}", err.message);
    }
}

