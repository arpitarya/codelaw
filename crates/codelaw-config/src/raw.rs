use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The keys of one config file, exactly as written. Every section is
/// strict: an unknown key is a load error (SR-CONFIG decision 5).
///
/// The sections owned by records whose features are not built yet
/// (`[format]`, `[graph]`, `[baseline]`) are accepted as tables and not
/// yet validated.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    #[serde(default)]
    pub extends: Vec<String>,
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub rules_dir: Option<String>,
    #[serde(default)]
    pub rules: BTreeMap<String, RuleSetting>,
    /// SR-CONFIG decision 3: `[scope.<name>]`, a named group of files.
    #[serde(default)]
    pub scope: ScopeMap,
    /// Deprecated spelling of `scope` (one minor release): each entry
    /// loads as `scope.override-<n>`.
    #[serde(default)]
    pub overrides: Vec<RawOverride>,
    /// SR-CONFIG decision 10: `[profile.<name>]`, root-only.
    #[serde(default)]
    pub profile: BTreeMap<String, RawProfile>,
    /// Deprecated spelling of `profile` (one minor release).
    #[serde(default)]
    pub profiles: BTreeMap<String, RawProfile>,
    #[serde(default)]
    pub root: bool,
    pub requires: Option<String>,
    /// SR-RULE-PACKS decision 9: preview rules run only with this set.
    pub preview: Option<bool>,
    /// SR-RULE-PACKS decision 12.
    #[serde(default)]
    pub allow_pack_fixers: Vec<String>,
    #[serde(default)]
    pub files: RawFiles,
    #[serde(default)]
    pub cache: RawCache,
    #[serde(default)]
    pub fix: RawFix,
    #[serde(default)]
    pub lint: RawLint,
    #[serde(default)]
    pub markdown: RawMarkdown,
    pub format: Option<toml::Table>,
    pub graph: Option<toml::Table>,
    pub baseline: Option<toml::Table>,
    /// SR-BASELINE decision 6: the policy of a symbol marked deprecated in
    /// code that the registry does not list. Its values are checked when
    /// the project loads.
    pub deprecation: Option<RawDeprecation>,
    /// SR-BASELINE decision 9: dated exceptions. Root config only.
    pub exceptions: Option<RawExceptions>,
    /// SR-PROJECT-GRAPH decision 9 (W-71): the test rules of new code —
    /// the lcov report and the rules' options; `[coverage.ignore]` is
    /// SR-BASELINE decision 12's. Root config only.
    pub coverage: Option<RawCoverage>,
    /// SR-RULE-PACKS decision 15: the dependency rules. Root config only.
    #[serde(default)]
    pub deps: RawDeps,
    /// SR-PROJECT-GRAPH decision 8 (W-63): CSS Modules. Root config only.
    #[serde(default)]
    pub css: RawCss,
    /// SR-SPELL decision 7 (W-103): the spell pack's locale and accepted
    /// words. Root config only.
    #[serde(default)]
    pub spell: RawSpell,
    #[serde(default)]
    pub adapters: RawAdapters,
    #[serde(default)]
    pub agents: RawAgents,
    #[serde(default)]
    pub daemon: RawDaemon,
    #[serde(default)]
    pub lsp: RawLsp,
}

/// A rule's severity, or its severity and options.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RuleSetting {
    Severity(String),
    Full {
        severity: Option<String>,
        #[serde(default)]
        options: BTreeMap<String, serde_json::Value>,
    },
}

impl RuleSetting {
    pub fn severity(&self) -> Option<&str> {
        match self {
            RuleSetting::Severity(s) => Some(s),
            RuleSetting::Full { severity, .. } => severity.as_deref(),
        }
    }

    pub fn options(&self) -> Option<&BTreeMap<String, serde_json::Value>> {
        match self {
            RuleSetting::Severity(_) => None,
            RuleSetting::Full { options, .. } => Some(options),
        }
    }
}

/// `[scope.<name>]`: SR-CONFIG decision 3. A preset may ship one, and a
/// project's of the same name extends it, so `files` may be omitted there.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawScope {
    pub files: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    #[serde(default)]
    pub rules: BTreeMap<String, RuleSetting>,
    /// SR-CONFIG decision 11.
    pub remap: Option<BTreeMap<String, String>>,
}

/// The `scope` table in the order the file wrote it: a later scope wins
/// (SR-CONFIG decision 4). A TOML file keeps its order; a `pyproject.toml`
/// is read sorted by name.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ScopeMap(pub Vec<(String, RawScope)>);

impl<'de> Deserialize<'de> for ScopeMap {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = ScopeMap;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a table of named scopes")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut m: A) -> Result<ScopeMap, A::Error> {
                let mut out: Vec<(String, RawScope)> = Vec::new();
                while let Some((k, v)) = m.next_entry::<String, RawScope>()? {
                    if out.iter().any(|(n, _)| *n == k) {
                        return Err(serde::de::Error::custom(format!("duplicate scope `{k}`")));
                    }
                    out.push((k, v));
                }
                Ok(ScopeMap(out))
            }
        }
        d.deserialize_map(V)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOverride {
    pub files: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub rules: BTreeMap<String, RuleSetting>,
    /// SR-CONFIG decision 11: each severity of a finding in these files
    /// mapped to another, or to `off`.
    pub remap: Option<BTreeMap<String, String>>,
}

/// `[profile.<name>]`: SR-CONFIG decision 10. The same shapes as the top
/// level's `[rules]` and `[scope.<name>]`.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProfile {
    #[serde(default)]
    pub rules: BTreeMap<String, RuleSetting>,
    #[serde(default)]
    pub scope: ScopeMap,
    /// Deprecated spelling of `scope`.
    #[serde(default)]
    pub overrides: Vec<RawOverride>,
}

/// `[files]`: SR-ARCHITECTURE decisions 7 and 10.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFiles {
    pub follow_symlinks: Option<bool>,
    pub generated_markers: Option<Vec<String>>,
    pub lint_generated: Option<bool>,
    pub max_file_size: Option<Size>,
}

/// `[cache]`: SR-CACHE decisions 3, 6 and 9.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCache {
    pub enabled: Option<bool>,
    pub dir: Option<String>,
    pub max_size: Option<Size>,
}

/// `[fix]`: SR-ARCHITECTURE decision 18.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFix {
    pub external_timeout: Option<Duration>,
}

/// `[adapters]`: SR-LINTER decision 2.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAdapters {
    pub enabled: Option<Vec<String>>,
    pub timeout: Option<Duration>,
}

/// `[deprecation]`: SR-BASELINE decision 6.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDeprecation {
    pub until: Option<String>,
    pub expires: Option<String>,
    pub warn_from: Option<String>,
    pub warn_days: Option<i64>,
    pub on_touch: Option<String>,
}

/// `[exceptions]`: SR-BASELINE decision 9.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawExceptions {
    pub require_expiry: Option<bool>,
    pub warn_days: Option<i64>,
}

/// `[coverage]`: SR-PROJECT-GRAPH decision 9. Values are checked when the
/// project loads.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCoverage {
    /// The lcov report, relative to the config file; `--coverage` wins.
    pub report: Option<String>,
    /// `all` (default) or `public`.
    pub scope: Option<String>,
    /// Least line coverage per function, 0–100 (default 60).
    pub min: Option<i64>,
    /// `function` (default) or `patch`.
    pub mode: Option<String>,
    /// No report is an error, not a skip.
    pub require_report: Option<bool>,
    /// `skip` (default) or `all`.
    pub on_no_change_set: Option<String>,
    /// A regex: function names that are never judged.
    pub ignore_names: Option<String>,
    /// `[coverage.ignore]`: what every `.codelawignore` line must carry.
    pub ignore: Option<RawCoverageIgnore>,
}

/// `[coverage.ignore]`: SR-BASELINE decision 12.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCoverageIgnore {
    pub require_expiry: Option<bool>,
    pub require_on_touch: Option<bool>,
    pub on_touch: Option<String>,
    pub on_touch_scope: Option<String>,
}

/// `[css]`: which stylesheets are CSS Modules, and how the bundler maps class
/// names to `styles.x`. The `css-modules/*` rules and the graph read it; it
/// serialises to the JSON the native rules read as options.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawCss {
    /// Globs of module stylesheets, besides `*.module.css` and `*.module.scss`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<String>>,
    /// css-loader's `localsConvention`: `asIs` (default), `camelCase`,
    /// `camelCaseOnly`, `dashes` or `dashesOnly`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules_convention: Option<String>,
}

/// `[spell]`: the spell pack's options (SR-SPELL decision 7). The `spell/*`
/// rules read it as their options, so it serialises to the JSON they read.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawSpell {
    /// `en-US` or `en-GB`: `spell/locale-variant` reports the other
    /// locale's spellings. Unset, both are accepted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Words the project accepts: never a known typo, an unknown word or a
    /// locale variant; `spell/suspicious-word` checks each one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<String>>,
}

/// The values of `[spell] locale`.
pub const SPELL_LOCALES: &[&str] = &["en-US", "en-GB"];

/// The values of `[css] modules_convention`.
pub const CSS_MODULES_CONVENTIONS: &[&str] = &["asIs", "camelCase", "camelCaseOnly", "dashes", "dashesOnly"];

/// `[deps]`: SR-RULE-PACKS decision 15. The dependency rules read it as
/// their options, so it serialises to the JSON they read.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawDeps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<RawDepsSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_release_age: Option<Days>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<RawDepsIgnore>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_snapshot_age: Option<Days>,
}

/// `[deps] severity`: a severity, or `off`, per kind of finding.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawDepsSeverity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major: Option<GradeSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<GradeSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<GradeSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<GradeSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yanked: Option<GradeSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile: Option<GradeSeverity>,
}

/// `error`, `warn`, `info` or `off`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GradeSeverity(pub String);

impl<'de> Deserialize<'de> for GradeSeverity {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if matches!(s.as_str(), "error" | "warn" | "info" | "off") {
            return Ok(GradeSeverity(s));
        }
        Err(serde::de::Error::custom(format!("invalid severity `{s}` (one of error, warn, info, off)")))
    }
}

/// One `[deps] ignore` entry: `"npm:typescript"`, or a dated one.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawDepsIgnore {
    Package(String),
    Dated(RawDatedIgnore),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawDatedIgnore {
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RawDepsIgnore {
    pub fn package(&self) -> &str {
        match self {
            RawDepsIgnore::Package(p) => p,
            RawDepsIgnore::Dated(d) => &d.package,
        }
    }
}

/// A number of days: an integer, or a string like `"7d"` or `"2w"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Days(pub u64);

impl<'de> Deserialize<'de> for Days {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Either {
            N(u64),
            S(String),
        }
        let parsed = match Either::deserialize(d)? {
            Either::N(n) => Some(n),
            Either::S(s) => {
                let s = s.trim();
                if let Some(n) = s.strip_suffix('d') {
                    n.trim().parse().ok()
                } else if let Some(n) = s.strip_suffix('w') {
                    n.trim().parse::<u64>().ok().map(|n| n * 7)
                } else {
                    None
                }
            }
        };
        parsed.map(Days).ok_or_else(|| serde::de::Error::custom("invalid number of days; write e.g. 7, \"7d\" or \"2w\""))
    }
}

/// `[daemon]`: SR-ARCHITECTURE decision 22.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDaemon {
    pub idle_timeout: Option<Duration>,
}

/// `[lsp]`: SR-ARCHITECTURE decision 24.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLsp {
    pub tree_cache: Option<usize>,
}

/// `[lint]`: SR-LINTER decision 3.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLint {
    pub tagged_templates: Option<bool>,
    /// Other tools' line suppressions read as `codelaw-ignore` (SR-LINTER
    /// decision 6; W-94): `eslint`, `ruff`, `biome`.
    pub ignore_aliases: Option<Vec<String>>,
}

/// The values `[lint] ignore_aliases` takes.
pub const IGNORE_ALIASES: &[&str] = &["eslint", "ruff", "biome"];

/// `[agents]`: SR-AGENT-SURFACES decision 6. The names are checked
/// against the agents of the matrix by `codelaw-agents`, which knows them.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAgents {
    pub targets: Option<Vec<String>>,
    pub command: Option<String>,
}

/// `[markdown]`: L3 decision 8.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMarkdown {
    pub lint_code_blocks: Option<bool>,
}

/// A byte size: an integer, or a string like `"1 MiB"` or `"512MB"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub u64);

impl<'de> Deserialize<'de> for Size {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Either {
            N(u64),
            S(String),
        }
        match Either::deserialize(d)? {
            Either::N(n) => Ok(Size(n)),
            Either::S(s) => parse_size(&s).map(Size).ok_or_else(|| {
                serde::de::Error::custom(format!("invalid size `{s}`; write e.g. 1048576, \"1 MiB\" or \"512MB\""))
            }),
        }
    }
}

fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let n: u64 = s[..split].parse().ok()?;
    let mult = match s[split..].trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "kb" => 1_000,
        "kib" => 1 << 10,
        "mb" => 1_000_000,
        "mib" => 1 << 20,
        "gb" => 1_000_000_000,
        "gib" => 1 << 30,
        _ => return None,
    };
    n.checked_mul(mult)
}

/// A duration: integer seconds, or a string like `"5s"` or `"500ms"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration(pub std::time::Duration);

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Either {
            N(u64),
            S(String),
        }
        let parsed = match Either::deserialize(d)? {
            Either::N(n) => Some(std::time::Duration::from_secs(n)),
            Either::S(s) => {
                let s = s.trim();
                if let Some(ms) = s.strip_suffix("ms") {
                    ms.trim().parse().ok().map(std::time::Duration::from_millis)
                } else if let Some(sec) = s.strip_suffix('s') {
                    sec.trim().parse().ok().map(std::time::Duration::from_secs)
                } else {
                    None
                }
            }
        };
        parsed.map(Duration).ok_or_else(|| serde::de::Error::custom("invalid duration; write e.g. 5, \"5s\" or \"500ms\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(parse_size("1 MiB"), Some(1 << 20));
        assert_eq!(parse_size("512MB"), Some(512_000_000));
        assert_eq!(parse_size("12"), Some(12));
        assert_eq!(parse_size("1 parsec"), None);
    }
}
