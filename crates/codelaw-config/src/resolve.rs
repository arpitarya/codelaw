use crate::raw::{RawConfig, RawDepsIgnore, RawProfile, RuleSetting};
use crate::scope::{self, Scope, glob_set};
pub use crate::scope::PresetScope;
use crate::{ConfigError, did_you_mean, explain_serde_error};
use codelaw_cache::{Key, KeyBuilder};
use codelaw_core::Language;
use globset::GlobSet;
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
    /// The preset scopes (root config only), then this config's, in
    /// precedence order (SR-CONFIG decision 4).
    pub(crate) scopes: Vec<Scope>,
    /// Each profile's scopes, by profile name.
    pub(crate) profile_scopes: BTreeMap<String, Vec<Scope>>,
    /// `[profile.*]` and the deprecated `[profiles.*]`, by name.
    pub profiles: BTreeMap<String, RawProfile>,
    pub(crate) alias_profiles: BTreeSet<String>,
    /// Deprecated spellings this file uses: one warning each.
    pub deprecations: Vec<String>,
    /// The words of `[spell] words_file` (or of the default file, when it
    /// is there), in file order: SR-SPELL decision 7, W-112.
    pub spell_file_words: Vec<String>,
}

/// Where a scope's settings are listed by `codelaw config --scopes`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeInfo {
    pub name: String,
    pub files: Vec<String>,
    pub exclude: Vec<String>,
    /// `codelaw.toml:7`, or the preset that ships it.
    pub source: String,
    /// The profile it belongs to, if any.
    pub profile: Option<String>,
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

/// A scope's severities and `remap` (SR-CONFIG decision 11).
fn validate_scope(file: &Path, what: &str, rules: &BTreeMap<String, RuleSetting>, remap: Option<&BTreeMap<String, String>>) -> Result<(), ConfigError> {
    for (id, s) in rules {
        validate_severity(file, id, s)?;
    }
    for (from, to) in remap.into_iter().flatten() {
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

/// A TOML table reads back sorted by name; the scopes apply in the order
/// the file wrote them (SR-CONFIG decision 4), so put them back by the
/// position of each one's key. A JSON home is already in order.
fn order_scopes(raw: &mut RawConfig, home: Home, text: &str) {
    if home == Home::PackageJson {
        return;
    }
    let sort = |scopes: &mut crate::raw::ScopeMap, base: &[&str]| {
        if scopes.0.len() > 1 {
            scopes.0.sort_by_key(|(name, _)| {
                let mut path = base.to_vec();
                path.push(name);
                crate::trace::key_offset(home, text, &path).unwrap_or(usize::MAX)
            });
        }
    };
    sort(&mut raw.scope, &["scope"]);
    for (name, p) in raw.profile.iter_mut() {
        sort(&mut p.scope, &["profile", name, "scope"]);
    }
    for (name, p) in raw.profiles.iter_mut() {
        sort(&mut p.scope, &["profiles", name, "scope"]);
    }
}

impl ConfigFile {
    fn new(path: PathBuf, home: Home, raw: RawConfig, text: String) -> Result<ConfigFile, ConfigError> {
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let mut raw = raw;
        order_scopes(&mut raw, home, &text);
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
        for (name, sc) in &raw.scope.0 {
            validate_scope(&path, &format!("scope `{name}`"), &sc.rules, sc.remap.as_ref())?;
        }
        for (i, o) in raw.overrides.iter().enumerate() {
            validate_scope(&path, &format!("override #{}", i + 1), &o.rules, o.remap.as_ref())?;
        }
        let mut profiles = raw.profile.clone();
        let mut alias_profiles = BTreeSet::new();
        for (name, p) in &raw.profiles {
            if profiles.contains_key(name) {
                return Err(ConfigError::new(
                    Some(path),
                    format!("profile `{name}` is defined under both `[profile.{name}]` and the deprecated `[profiles.{name}]`; keep `[profile.{name}]`"),
                ));
            }
            profiles.insert(name.clone(), p.clone());
            alias_profiles.insert(name.clone());
        }
        for (name, p) in &profiles {
            for (id, s) in &p.rules {
                validate_severity(&path, id, s)?;
            }
            for (sname, sc) in &p.scope.0 {
                validate_scope(&path, &format!("profile `{name}` scope `{sname}`"), &sc.rules, sc.remap.as_ref())?;
            }
            for (i, o) in p.overrides.iter().enumerate() {
                validate_scope(&path, &format!("profile `{name}` override #{}", i + 1), &o.rules, o.remap.as_ref())?;
            }
        }
        let mut deprecations = Vec::new();
        let at = codelaw_core::normalize_path(&path);
        if !raw.overrides.is_empty() || profiles.values().any(|p| !p.overrides.is_empty()) {
            deprecations.push(format!(
                "{at}: `[[overrides]]` is deprecated and goes in the next minor release; write `[scope.<name>]` (each entry loads as `scope.override-<n>`)"
            ));
        }
        if !alias_profiles.is_empty() {
            deprecations.push(format!("{at}: `[profiles.<name>]` is deprecated and goes in the next minor release; write `[profile.<name>]`"));
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
        if let Some(a) = raw.lint.ignore_aliases.iter().flatten().find(|a| !crate::raw::IGNORE_ALIASES.contains(&a.as_str())) {
            return Err(ConfigError::new(
                Some(path),
                format!("`[lint] ignore_aliases`: `{a}` is not one of {}", crate::raw::IGNORE_ALIASES.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ")),
            ));
        }
        if let Some(c) = raw.css.modules_convention.as_deref().filter(|c| !crate::raw::CSS_MODULES_CONVENTIONS.contains(c)) {
            return Err(ConfigError::new(
                Some(path),
                format!("`[css] modules_convention`: `{c}` is not one of {}", crate::raw::CSS_MODULES_CONVENTIONS.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ")),
            ));
        }
        if let Some(l) = raw.spell.locale.as_deref().filter(|l| !crate::raw::SPELL_LOCALES.contains(l)) {
            return Err(ConfigError::new(
                Some(path),
                format!("`[spell] locale`: `{l}` is not one of {}", crate::raw::SPELL_LOCALES.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ")),
            ));
        }
        if let Some(w) = raw.spell.words.iter().flatten().find(|w| w.is_empty() || w.chars().any(char::is_whitespace)) {
            return Err(ConfigError::new(Some(path), format!("`[spell] words`: `{w}` is not one word")));
        }
<<<<<<< HEAD
        check_reviews(&raw.reviews, &path)?;
        check_markers(&raw, &path)?;
=======
        if let Some(f) = &raw.flags {
            if let Some(p) = f.presets.iter().flatten().find(|p| !crate::raw::FLAG_PRESETS.contains(&p.as_str())) {
                return Err(ConfigError::new(
                    Some(path),
                    format!("`[flags] presets`: `{p}` is not one of {}", crate::raw::FLAG_PRESETS.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ")),
                ));
            }
            if let Some(i) = f.inline.as_deref().filter(|i| !crate::raw::FLAG_INLINE.contains(i)) {
                return Err(ConfigError::new(Some(path), format!("`[flags] inline`: `{i}` is not one of `allow`, `registry-only`")));
            }
        }
>>>>>>> 7a0a398a (W-130 wip)
        let spell_file_words = read_words_file(&path, &dir, raw.spell.words_file.as_deref())?;
        if let Some(globs) = &raw.css.modules {
            glob_set(globs, &path).map_err(|e| ConfigError::new(Some(path.clone()), format!("`[css] modules`: {}", e.message)))?;
        }
        let include = if raw.include.is_empty() { None } else { Some(glob_set(&raw.include, &path)?) };
        let exclude = glob_set(&raw.exclude, &path)?;
        let built = scope::build(&path, &raw, &profiles, &alias_profiles, &[], raw.extends.is_empty())?;
        Ok(ConfigFile {
            path,
            dir,
            home,
            raw,
            text,
            include,
            exclude,
            scopes: built.scopes,
            profile_scopes: built.profile_scopes,
            profiles,
            alias_profiles,
            deprecations,
            spell_file_words,
        })
    }

    /// Rebuild the scopes with the presets' in front (the root config).
    fn with_preset_scopes(&mut self, presets: &[PresetScope]) -> Result<(), ConfigError> {
        let built = scope::build(&self.path, &self.raw, &self.profiles, &self.alias_profiles, presets, true)?;
        self.scopes = built.scopes;
        self.profile_scopes = built.profile_scopes;
        Ok(())
    }

    /// Every rule table the project wrote in a scope or profile, to check
    /// the ids against the rule set.
    pub fn scope_rule_tables(&self) -> Vec<&BTreeMap<String, RuleSetting>> {
        let mut out: Vec<&BTreeMap<String, RuleSetting>> = self.raw.scope.0.iter().map(|(_, s)| &s.rules).chain(self.raw.overrides.iter().map(|o| &o.rules)).collect();
        for p in self.profiles.values() {
            out.push(&p.rules);
            out.extend(p.scope.0.iter().map(|(_, s)| &s.rules));
            out.extend(p.overrides.iter().map(|o| &o.rules));
        }
        out
    }

    /// The path of `file` relative to this config's folder, if under it.
    pub(crate) fn relative<'p>(&self, file: &'p Path) -> Option<&'p Path> {
        file.strip_prefix(&self.dir).ok()
    }
}

/// `[reviews]` (SR-AGENT-SURFACES decision 10): the values the skill
/// would otherwise misread, refused at load.
fn check_reviews(r: &crate::raw::RawReviews, path: &Path) -> Result<(), ConfigError> {
    let err = |m: String| Err(ConfigError::new(Some(path.to_path_buf()), m));
    if let Some(p) = r.platform.as_deref().filter(|p| !crate::raw::REVIEW_PLATFORMS.contains(p)) {
        return err(format!("`[reviews] platform`: `{p}` is not one of {}", crate::raw::REVIEW_PLATFORMS.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ")));
    }
    if let Some(u) = &r.url {
        if !(u.starts_with("https://") || u.starts_with("http://")) || u.len() <= "https://".len() || u.chars().any(char::is_whitespace) {
            return err(format!("`[reviews] url`: `{u}` is not an http(s) URL"));
        }
        if r.platform.is_none() {
            return err("`[reviews] url` needs `[reviews] platform`: a self-hosted URL does not say which platform it is".into());
        }
    }
    for (key, v) in [("min_repeats", r.min_repeats), ("min_reviewers", r.min_reviewers)] {
        if v == Some(0) {
            return err(format!("`[reviews] {key}`: 0 is not allowed; the least is 1"));
        }
    }
    if let Some(w) = r.window.as_deref().filter(|w| !crate::raw::is_review_window(w)) {
        return err(format!("`[reviews] window`: `{w}` is not a window; write e.g. \"6m\", \"90d\", \"2w\" or \"1y\""));
    }
    Ok(())
}

/// `[todo]` and `[tests.skip]` (SR-CONFIG decision 11a): tags are plain words and
/// a ticket pattern is a regular expression, so a typo fails on a laptop as
/// it would in CI.
fn check_markers(raw: &RawConfig, path: &Path) -> Result<(), ConfigError> {
    let fail = |m: String| Err(ConfigError::new(Some(path.to_path_buf()), m));
    if let Some(tags) = &raw.todo.tags {
        if tags.is_empty() {
            return fail("`[todo] tags`: name at least one tag".into());
        }
        if let Some(t) = tags.iter().find(|t| t.is_empty() || !t.chars().all(|c| c.is_alphanumeric() || c == '_')) {
            return fail(format!("`[todo] tags`: `{t}` is not one word of letters, digits and `_`"));
        }
    }
    for (table, pattern) in [("[todo]", &raw.todo.ticket_pattern), ("[tests.skip]", &raw.tests.skip.ticket_pattern)] {
        if let Some(p) = pattern
            && let Err(e) = regex::Regex::new(p)
        {
            return fail(format!("`{table} ticket_pattern`: `{p}` is not a regular expression: {e}"));
        }
    }
    Ok(())
}

/// The words of a `[spell] words_file` text: one per line, blank lines and
/// lines starting with `#` skipped; a line holding whitespace inside is an
/// error naming its line (W-112).
pub fn words_file_words(text: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let w = line.trim();
        if w.is_empty() || w.starts_with('#') {
            continue;
        }
        if w.chars().any(char::is_whitespace) {
            return Err(format!("line {}: `{w}` is not one word", i + 1));
        }
        out.push(w.to_owned());
    }
    Ok(out)
}

/// Read `[spell] words_file`, relative to the config's folder: missing at
/// the default path is no error, missing at a path the config sets is
/// (SR-SPELL decision 7).
fn read_words_file(config: &Path, dir: &Path, set: Option<&str>) -> Result<Vec<String>, ConfigError> {
    let file = dir.join(set.unwrap_or(crate::raw::SPELL_WORDS_FILE));
    let shown = codelaw_core::normalize_path(&file);
    let text = match fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && set.is_none() => return Ok(Vec::new()),
        Err(e) => return Err(ConfigError::new(Some(config.to_path_buf()), format!("`[spell] words_file`: {shown}: {e}"))),
    };
    words_file_words(&text).map_err(|e| ConfigError::new(Some(config.to_path_buf()), format!("`[spell] words_file`: {shown}: {e}")))
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
    "extends", "rules_dir", "preview", "allow_pack_fixers", "[files]", "[cache]", "[fix]", "[lint]", "[markdown]", "[daemon]", "[lsp]", "[profile]",
<<<<<<< HEAD
    "[deps]", "[css]", "[coverage]", "[spell]", "[reviews]", "[todo]", "[tests]", "[codeowners]",
=======
    "[deps]", "[css]", "[coverage]", "[spell]", "[flags]",
>>>>>>> 7a0a398a (W-130 wip)
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
        !raw.profile.is_empty() || !raw.profiles.is_empty(),
        raw.deps != Default::default(),
        raw.css != Default::default(),
        raw.coverage.is_some(),
        raw.spell != Default::default(),
<<<<<<< HEAD
        raw.reviews != Default::default(),
        raw.todo != Default::default(),
        raw.tests != Default::default(),
        raw.codeowners != Default::default(),
=======
        raw.flags.is_some(),
>>>>>>> 7a0a398a (W-130 wip)
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
    warnings.extend(chain.iter().flat_map(|f| f.deprecations.iter().cloned()));
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
        Ok(Config { root_dir, warnings: file.deprecations.clone(), files: vec![file], profile: Profile::default() })
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
        self.warnings.extend(file.deprecations.iter().cloned());
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
        let defined = self.files.first().map(|f| &f.profiles);
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
        self.files.first().is_some_and(|f| !f.profiles.is_empty())
    }

    /// The root config and the profile in use, when it defines it.
    pub(crate) fn active_profile(&self) -> Option<(&ConfigFile, &RawProfile, &[Scope])> {
        let root = self.files.first()?;
        let p = root.profiles.get(&self.profile.name)?;
        Some((root, p, root.profile_scopes.get(&self.profile.name).map(Vec::as_slice).unwrap_or_default()))
    }

    /// Put the scopes the presets ship in front of the root config's
    /// (SR-CONFIG decision 4; SR-RULE-PACKS). A project scope of the same
    /// name extends the preset's.
    pub fn set_preset_scopes(&mut self, presets: &[PresetScope]) -> Result<(), ConfigError> {
        match self.files.first_mut() {
            Some(root) => root.with_preset_scopes(presets),
            None => Ok(()),
        }
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

    /// Where `[spell] words_file` points: the default path when unset.
    pub fn spell_words_file(&self) -> PathBuf {
        let rel = self.root_raw().and_then(|r| r.spell.words_file.clone()).unwrap_or_else(|| crate::raw::SPELL_WORDS_FILE.into());
        self.config_dir().join(rel)
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
            ("css", Some(r)) => serde_json::to_value(&r.css).ok(),
<<<<<<< HEAD
            ("todo", Some(r)) => serde_json::to_value(&r.todo).ok(),
            ("tests.skip", Some(r)) => serde_json::to_value(&r.tests.skip).ok(),
            ("codeowners", Some(r)) => serde_json::to_value(&r.codeowners).ok(),
=======
            ("flags", Some(r)) => r.flags.as_ref().and_then(|f| serde_json::to_value(f).ok()),
>>>>>>> 7a0a398a (W-130 wip)
            // `[spell] words` and the words file are one list (W-112).
            ("spell", Some(r)) => serde_json::to_value(&r.spell).ok().map(|mut v| {
                let file = self.files.first().map(|f| f.spell_file_words.as_slice()).unwrap_or_default();
                if !file.is_empty() {
                    let words: Vec<&String> = r.spell.words.iter().flatten().chain(file).collect();
                    v["words"] = serde_json::json!(words);
                }
                v
            }),
            _ => None,
        };
        match value {
            Some(serde_json::Value::Object(m)) => m.into_iter().collect(),
            _ => BTreeMap::new(),
        }
    }

    /// Whether the root config has a `[flags]` table: it turns the flag
    /// rules on (SR-PROJECT-GRAPH decision 11).
    pub fn flags_on(&self) -> bool {
        self.root_raw().is_some_and(|r| r.flags.is_some())
    }

    /// `[css] modules`: globs naming CSS Module stylesheets besides
    /// `*.module.css` and `*.module.scss`.
    pub fn css_module_globs(&self) -> Vec<String> {
        self.root_raw().and_then(|r| r.css.modules.clone()).unwrap_or_default()
    }

    /// `[css] modules_convention`: css-loader's `localsConvention`, `asIs` by default.
    pub fn css_modules_convention(&self) -> String {
        self.root_raw().and_then(|r| r.css.modules_convention.clone()).unwrap_or_else(|| "asIs".into())
    }

    pub fn tagged_templates(&self) -> bool {
        self.root_raw().and_then(|r| r.lint.tagged_templates).unwrap_or(false)
    }

    /// `[lint] ignore_aliases`: the tools whose line suppressions count as
    /// `codelaw-ignore`; none by default.
    pub fn ignore_aliases(&self) -> Vec<String> {
        self.root_raw().and_then(|r| r.lint.ignore_aliases.clone()).unwrap_or_default()
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

    /// The scopes that match `file`, in precedence order: each config's,
    /// outermost first and in order, then the profile's (SR-CONFIG
    /// decision 4).
    pub(crate) fn scopes_for<'a>(&'a self, file: &Path) -> Vec<(&'a ConfigFile, &'a Scope)> {
        let mut out = Vec::new();
        let mut matching = |f: &'a ConfigFile, scopes: &'a [Scope]| {
            let Some(rel) = f.relative(file) else { return };
            out.extend(scopes.iter().filter(|s| s.matches(rel)).map(|s| (f, s)));
        };
        for f in self.files.iter().filter(|f| file.starts_with(&f.dir)) {
            matching(f, &f.scopes);
        }
        if let Some((f, _, scopes)) = self.active_profile() {
            matching(f, scopes);
        }
        out
    }

    /// `<config> scope <name>`, `<config> profile <p> scope <name>`, or
    /// the preset that ships a scope the project does not write.
    pub(crate) fn scope_label(&self, f: &ConfigFile, s: &Scope) -> String {
        let path = codelaw_core::normalize_path(&f.path);
        match (&s.profile, &s.toml_path) {
            (Some(p), _) => format!("{path} profile {p} scope {}", s.name),
            (None, Some(_)) => format!("{path} scope {}", s.name),
            (None, None) => format!("{} scope {}", s.presets.join(", "), s.name),
        }
    }

    /// The rule-setting layers for `file`, lowest precedence first: each
    /// config's `[rules]`, outermost first, then the profile's `rules`,
    /// then the matching scopes in order, then the profile's (SR-CONFIG
    /// decision 4).
    pub fn layers_for<'a>(&'a self, file: &'a Path) -> Vec<Layer<'a>> {
        let mut layers = Vec::new();
        for f in self.chain_for(file) {
            layers.push(Layer { source: codelaw_core::normalize_path(&f.path), rules: &f.raw.rules });
        }
        if let Some((f, p, _)) = self.active_profile().filter(|(f, _, _)| file.starts_with(&f.dir)) {
            layers.push(Layer { source: format!("{} profile {}", codelaw_core::normalize_path(&f.path), self.profile.name), rules: &p.rules });
        }
        for (f, s) in self.scopes_for(file) {
            layers.push(Layer { source: self.scope_label(f, s), rules: &s.rules });
        }
        layers
    }

    /// Whether any scope, a profile's included, sets `remap`: without
    /// one the stage costs nothing.
    pub fn has_remap(&self) -> bool {
        self.files.iter().any(|f| f.scopes.iter().chain(f.profile_scopes.values().flatten()).any(|s| s.remap.is_some()))
    }

    /// What `remap` does to the findings in `file` (SR-CONFIG decision 11):
    /// every matching scope's table, later winning per key.
    pub fn remap_for(&self, file: &Path) -> Remap {
        let mut r = Remap::default();
        for (_, s) in self.scopes_for(file) {
            if let Some(t) = &s.remap {
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
            for t in f.scope_rule_tables() {
                for id in t.keys() {
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
            // The words file is config the text does not hold.
            if !f.spell_file_words.is_empty() {
                k.push(f.spell_file_words.join("\n"));
            }
        }
        // Scopes a preset ships are not in the config text.
        for f in self.files.iter().take(1) {
            for s in f.scopes.iter().filter(|s| !s.presets.is_empty()) {
                k.push(format!("scope {} {:?} {:?} {:?} {:?}", s.name, s.files, s.exclude, s.rules, s.remap));
            }
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
            "exclude = [\"dist/**\"]\n[rules]\n\"a/x\" = \"warn\"\n[scope.pkg]\nfiles = [\"pkg/**/*.ts\"]\nrules = { \"a/x\" = \"off\" }\n",
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
        assert!(layers[2].source.ends_with("scope pkg"));
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
        let text = "[rules]\n\"a/x\" = \"warn\"\n\n[scope.s]\nfiles = [\"s/**\"]\nrules = { \"a/x\" = \"info\" }\nremap = { error = \"warn\", info = \"off\" }\n\n[profile.ci]\nrules = { \"a/x\" = \"error\" }\nscope = { s = { remap = { error = \"info\" } } }\n";
        let mut c = Config::from_toml(PathBuf::from("/x"), text).unwrap();
        let file = PathBuf::from("/x/s/a.ts");
        let sources = |c: &Config| c.layers_for(&file).into_iter().map(|l| l.source).collect::<Vec<_>>();
        assert_eq!(sources(&c), ["/x/codelaw.toml", "/x/codelaw.toml scope s"], "`local` is not defined: no change");
        let local_key = c.fingerprint();
        c.select_profile(&ProfileChoice { ci: Some("true".into()), ..Default::default() }).unwrap();
        assert_eq!(sources(&c), ["/x/codelaw.toml", "/x/codelaw.toml profile ci", "/x/codelaw.toml scope s", "/x/codelaw.toml profile ci scope s"]);
        assert_ne!(c.fingerprint(), local_key, "the profile is part of every cache key");
        use codelaw_core::Severity::*;
        let r = c.remap_for(&file);
        assert_eq!((r.apply(Error), r.apply(Warn), r.apply(Info)), (Some(Info), Some(Warn), None), "the later override wins per key");
        assert!(c.remap_for(&PathBuf::from("/x/t/a.ts")).is_identity());
        assert!(c.has_remap());
        let err = c.select_profile(&ProfileChoice { flag: Some("cis".into()), ..Default::default() }).unwrap_err();
        assert!(err.message.contains("profile `cis` (from --profile) is not defined; did you mean `ci`?"), "{}", err.message);
        let err = Config::from_toml(PathBuf::from("/x"), "[profile.ci]\nscope = { a = { files = [\"a\"], remap = { eror = \"warn\" } } }\n").unwrap_err();
        assert!(err.message.contains("profile `ci` scope `a`: `remap` key `eror`; did you mean `error`?"), "{}", err.message);
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

    /// The `max` option of `js/max-lines` for `file`, folding the layers
    /// the way the rule set does.
    fn max_for(c: &Config, file: &str) -> Option<i64> {
        let path = PathBuf::from("/x").join(file);
        let mut max = None;
        for l in c.layers_for(&path) {
            if let Some(m) = l.rules.get("js/max-lines").and_then(|s| s.options()).and_then(|o| o.get("max")) {
                max = m.as_i64();
            }
        }
        max
    }

    fn preset_tests() -> Vec<PresetScope> {
        let raw: crate::raw::RawConfig =
            toml::from_str("[scope.tests]\nfiles = [\"**/*.test.*\", \"**/*.spec.*\"]\nrules = { \"js/max-lines\" = { options = { max = 300 } } }\n").unwrap();
        raw.scope.0.into_iter().map(|(name, scope)| PresetScope { preset: "codelaw:js/recommended".into(), name, scope }).collect()
    }

    const GOAL: &str = r#"
[rules]
"js/max-lines" = { severity = "error", options = { max = 100 } }

[scope.utils]
files = ["**/*.utils.ts", "**/*.utils.tsx"]
rules = { "js/max-lines" = { options = { max = 50 } } }

[scope.tests]
rules = { "js/max-lines" = { options = { max = 400 } } }

[profile.ci]
rules = { "js/no-console" = "error" }
"#;

    #[test]
    fn a_project_scope_extends_the_presets_of_the_same_name() {
        let mut c = Config::from_toml(PathBuf::from("/x"), &format!("extends = [\"codelaw:js/recommended\"]\n{GOAL}")).unwrap();
        // `tests` has no `files` of its own: it needs the preset.
        assert!(c.warnings.is_empty());
        c.set_preset_scopes(&preset_tests()).unwrap();
        assert_eq!(max_for(&c, "src/date.utils.ts"), Some(50));
        assert_eq!(max_for(&c, "src/a.test.ts"), Some(400), "the project's value wins, the preset's files stay");
        assert_eq!(max_for(&c, "src/b.spec.ts"), Some(400));
        assert_eq!(max_for(&c, "src/Orders.tsx"), Some(100));
        let names: Vec<(String, String)> = c.scope_list().into_iter().map(|s| (s.name, s.source)).collect();
        assert_eq!(names[0], ("tests".to_owned(), "codelaw.toml:10, extends codelaw:js/recommended".to_owned()));
        assert_eq!(names[1].0, "utils", "preset scopes first, then the project's in file order");
    }

    #[test]
    fn scope_names_are_checked() {
        for bad in ["default", "a b", "Utils", ""] {
            let text = format!("[scope.\"{bad}\"]\nfiles = [\"a\"]\n");
            let err = Config::from_toml(PathBuf::from("/x"), &text).unwrap_err();
            assert!(err.message.contains("scope name"), "{bad}: {}", err.message);
        }
        let err = Config::from_toml(PathBuf::from("/x"), "[scope.utils]\nrules = {}\n").unwrap_err();
        assert!(err.message.contains("scope `utils` has no `files`"), "{}", err.message);
    }

    #[test]
    fn the_flags_section_is_checked_and_read_as_options() {
        let c = Config::from_toml(PathBuf::from("/x"), "[flags]\npresets = [\"unleash\"]\naccessors = [\"isOn($NAME)\"]\nwarn_days = \"2w\"\n").unwrap();
        assert!(c.flags_on());
        let o = c.section_options("flags");
        assert_eq!(o["presets"], serde_json::json!(["unleash"]));
        assert_eq!(o["warn_days"], serde_json::json!(14));
        let err = Config::from_toml(PathBuf::from("/x"), "[flags]\npresets = [\"optimizely\"]\n").unwrap_err();
        assert!(err.to_string().contains("`[flags] presets`: `optimizely` is not one of"), "{err}");
        let err = Config::from_toml(PathBuf::from("/x"), "[flags]\ninline = \"never\"\n").unwrap_err();
        assert!(err.to_string().contains("`[flags] inline`: `never`"), "{err}");
        assert!(!Config::from_toml(PathBuf::from("/x"), "").unwrap().flags_on());
    }

    #[test]
    fn the_spell_section_is_checked_and_read_as_options() {
        let c = Config::from_toml(PathBuf::from("/x"), "[spell]\nlocale = \"en-GB\"\nwords = [\"kubectl\"]\n").unwrap();
        let o = c.section_options("spell");
        assert_eq!(o.get("locale"), Some(&serde_json::json!("en-GB")));
        assert_eq!(o.get("words"), Some(&serde_json::json!(["kubectl"])));
        let err = Config::from_toml(PathBuf::from("/x"), "[spell]\nlocale = \"en-AU\"\n").unwrap_err();
        assert!(err.to_string().contains("`[spell] locale`: `en-AU` is not one of `en-US`, `en-GB`"), "{err}");
        let err = Config::from_toml(PathBuf::from("/x"), "[spell]\nwords = [\"two words\"]\n").unwrap_err();
        assert!(err.to_string().contains("`[spell] words`: `two words` is not one word"), "{err}");
        assert!(Config::from_toml(PathBuf::from("/x"), "[spell]\nwordz = []\n").is_err());
    }

    /// W-112: the words file is merged with `[spell] words`; missing at the
    /// default path is silent, missing at a set path is an error naming it.
    #[test]
    fn the_spell_words_file_is_merged_with_the_inline_words() {
        let d = temp("spell-words-file");
        let at = |s: &str| Config::from_toml(d.clone(), s);
        // No file at the default path: no error, the inline words only.
        let c = at("[spell]\nwords = [\"kubectl\"]\n").unwrap();
        assert_eq!(c.section_options("spell").get("words"), Some(&serde_json::json!(["kubectl"])));
        assert_eq!(c.spell_words_file(), d.join(".codelaw/spell/words.txt"));
        let before = c.fingerprint();
        fs::create_dir_all(d.join(".codelaw/spell")).unwrap();
        fs::write(d.join(".codelaw/spell/words.txt"), "# accepted words\n\nnginx\n  Kubelet  \n").unwrap();
        let c = at("[spell]\nwords = [\"kubectl\"]\n").unwrap();
        assert_eq!(c.section_options("spell").get("words"), Some(&serde_json::json!(["kubectl", "nginx", "Kubelet"])));
        assert_ne!(c.fingerprint(), before, "the words file is part of every cache key");
        // The file alone, with no `[spell]` section.
        assert_eq!(at("").unwrap().section_options("spell").get("words"), Some(&serde_json::json!(["nginx", "Kubelet"])));
        // A set path is read relative to the config's folder, and must be there.
        fs::write(d.join("words.txt"), "terraform\n").unwrap();
        let c = at("[spell]\nwords_file = \"words.txt\"\n").unwrap();
        assert_eq!(c.section_options("spell").get("words"), Some(&serde_json::json!(["terraform"])));
        assert!(!c.section_options("spell").contains_key("words_file"), "not a rule option");
        let err = at("[spell]\nwords_file = \"missing.txt\"\n").unwrap_err();
        assert!(err.to_string().contains("`[spell] words_file`: ") && err.to_string().contains("missing.txt"), "{err}");
        fs::write(d.join("words.txt"), "ok\ntwo words\n").unwrap();
        let err = at("[spell]\nwords_file = \"words.txt\"\n").unwrap_err();
        assert!(err.to_string().contains("line 2: `two words` is not one word"), "{err}");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn the_old_spellings_load_with_one_warning_each() {
        let text = "[[overrides]]\nfiles = [\"a/**\"]\nrules = { \"js/max-lines\" = { options = { max = 7 } } }\n[[overrides]]\nfiles = [\"b/**\"]\n\n[profiles.ci]\nrules = { \"a/x\" = \"error\" }\n";
        let mut c = Config::from_toml(PathBuf::from("/x"), text).unwrap();
        assert_eq!(c.warnings.len(), 2, "{:?}", c.warnings);
        assert!(c.warnings[0].contains("`[[overrides]]` is deprecated") && c.warnings[0].contains("[scope.<name>]"));
        assert!(c.warnings[1].contains("`[profiles.<name>]` is deprecated") && c.warnings[1].contains("[profile.<name>]"));
        assert_eq!(max_for(&c, "a/x.ts"), Some(7));
        assert_eq!(c.scope_list().iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["override-1", "override-2"]);
        c.select_profile(&ProfileChoice { flag: Some("ci".into()), ..Default::default() }).unwrap();
        let both = "[profile.ci]\nrules = {}\n[profiles.ci]\nrules = {}\n";
        assert!(Config::from_toml(PathBuf::from("/x"), both).unwrap_err().message.contains("both"));
    }

    #[test]
    fn scopes_apply_in_the_order_the_file_wrote_them() {
        let text = "[scope.zeta]\nfiles = [\"a/**\"]\nrules = { \"js/max-lines\" = { options = { max = 1 } } }\n[scope.alpha]\nfiles = [\"a/**\"]\nrules = { \"js/max-lines\" = { options = { max = 2 } } }\n";
        let c = Config::from_toml(PathBuf::from("/x"), text).unwrap();
        assert_eq!(max_for(&c, "a/f.ts"), Some(2), "alpha is written last, so it wins");
        assert_eq!(c.scope_list().iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["zeta", "alpha"]);
    }
}
