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
    /// SR-RULE-PACKS decision 16 (W-132): `data/codeowners-coverage`. Root
    /// config only.
    #[serde(default)]
    pub codeowners: RawCodeowners,
    /// SR-SPELL decision 7 (W-103): the spell pack's locale and accepted
    /// words. Root config only.
    #[serde(default)]
    pub spell: RawSpell,
<<<<<<< HEAD
    /// SR-CONFIG decision 11a (W-131): the `*/todo-marker` rules' tags, ticket
    /// and date. Root config only.
    #[serde(default)]
    pub todo: RawTodo,
    /// SR-CONFIG decision 11a (W-131): `[tests.skip]`. Root config only.
    #[serde(default)]
    pub tests: RawTests,
=======
    /// SR-PROJECT-GRAPH decision 11 (W-130): feature-flag expiry — the
    /// accessors, the SDK presets, the inline markers' policy and the
    /// warning window. Its presence turns the flag rules on. Root config only.
    pub flags: Option<RawFlags>,
>>>>>>> 7a0a398a (W-130 wip)
    #[serde(default)]
    pub adapters: RawAdapters,
    #[serde(default)]
    pub agents: RawAgents,
    #[serde(default)]
    pub daemon: RawDaemon,
    #[serde(default)]
    pub lsp: RawLsp,
    /// SR-AGENT-SURFACES decision 10 (W-133): the thresholds and platform
    /// of the `codelaw-rules-from-reviews` skill. Validated here; nothing
    /// in the engine reads it. Root config only.
    #[serde(default)]
    pub reviews: RawReviews,
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

/// `[codeowners]`: where CODEOWNERS is and what it may leave out. The
/// `data/codeowners-coverage` rule reads it as its options, so it
/// serialises to the JSON the rule reads.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawCodeowners {
    /// The CODEOWNERS file, from the project root; by default the first of
    /// `.github/CODEOWNERS`, `CODEOWNERS`, `docs/CODEOWNERS` and
    /// `.gitlab/CODEOWNERS` that exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// A pattern that names no owner is meant (default `false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unowned: Option<bool>,
    /// Gitignore-style patterns of paths that need no owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
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
    /// More accepted words, one per line, `#` comments, merged with
    /// `words`; by default [`SPELL_WORDS_FILE`], relative to the config's
    /// folder (W-112). Not a rule option: the loader reads the file.
    #[serde(skip_serializing)]
    pub words_file: Option<String>,
}

<<<<<<< HEAD
/// `[todo]`: what a TODO-style comment must carry (SR-CONFIG decision 11a,
/// W-131). The `*/todo-marker` rules read it as their options, so it
/// serialises to the JSON they read.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawTodo {
    /// The words that open a marker; `["TODO", "FIXME", "HACK"]` by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// A ticket is required; `true` by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_ticket: Option<bool>,
    /// A date is required; `false` by default. A date that is written is
    /// enforced either way.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_date: Option<bool>,
    /// A regular expression a whole ticket must match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_pattern: Option<String>,
    /// How many days before its date a marker is warned about; 14 by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warn_days: Option<u32>,
}

/// `[tests]`: the tables about tests (SR-CONFIG decision 11a, W-131).
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTests {
    #[serde(default)]
    pub skip: RawTestsSkip,
}

/// `[tests.skip]`: what a skipped test must carry. The `*/skipped-test`
/// rules read it as their options.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawTestsSkip {
    /// A reason is required; `true` by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_reason: Option<bool>,
    /// A `SKIP(…, date)` expiry is required; `true` by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_expiry: Option<bool>,
    /// A ticket is required; `false` by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_ticket: Option<bool>,
    /// A regular expression a whole ticket must match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_pattern: Option<String>,
    /// How many days before its expiry a skip is warned about; 14 by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warn_days: Option<u32>,
}

/// The ticket pattern when `ticket_pattern` is not set (`ABC-123`).
pub const DEFAULT_TICKET_PATTERN: &str = "[A-Z][A-Z0-9]+-\\d+";
=======
/// `[flags]`: SR-PROJECT-GRAPH decision 11. The flag rules read it as their
/// options, so it serialises to the JSON they read.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawFlags {
    /// SDK presets: the accessor patterns codelaw ships for each.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presets: Option<Vec<String>>,
    /// Accessor patterns: a call with one `$NAME` slot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessors: Option<Vec<String>>,
    /// `allow` (default) or `registry-only`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<String>,
    /// Days before a flag's `expires` that its reads warn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warn_days: Option<Days>,
}

/// The values of `[flags] presets` (`codelaw_graph::flags::PRESETS`).
pub const FLAG_PRESETS: &[&str] = &["launchdarkly", "unleash", "split", "openfeature", "growthbook", "flagsmith"];

/// The values of `[flags] inline`.
pub const FLAG_INLINE: &[&str] = &["allow", "registry-only"];
>>>>>>> 7a0a398a (W-130 wip)

/// Where `[spell] words_file` points when it is not set: beside the rules
/// folder. A missing file there is no error; a missing file the config
/// names is.
pub const SPELL_WORDS_FILE: &str = ".codelaw/spell/words.txt";

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

/// `[reviews]`: SR-AGENT-SURFACES decision 10 (W-133). The skill
/// `codelaw-rules-from-reviews` reads it; the engine only validates it,
/// so a typo fails the load (SR-CONFIG decision 5) and nothing fetches
/// (L5).
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawReviews {
    pub platform: Option<String>,
    pub url: Option<String>,
    pub min_repeats: Option<u32>,
    pub min_reviewers: Option<u32>,
    pub window: Option<String>,
}

/// The values of `[reviews] platform`.
pub const REVIEW_PLATFORMS: &[&str] = &["github", "gitlab", "bitbucket", "bitbucket-datacenter", "azure"];

/// `[reviews] window`: a whole number and a unit, `d`, `w`, `m` (months)
/// or `y` — `"6m"`, `"90d"`.
pub fn is_review_window(s: &str) -> bool {
    let Some(unit) = s.chars().last() else { return false };
    let n = &s[..s.len() - unit.len_utf8()];
    matches!(unit, 'd' | 'w' | 'm' | 'y') && !n.is_empty() && !n.starts_with('0') && n.chars().all(|c| c.is_ascii_digit())
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

    #[test]
    fn review_windows() {
        for ok in ["6m", "90d", "2w", "1y", "12m"] {
            assert!(is_review_window(ok), "{ok}");
        }
        for bad in ["", "m", "0m", "06m", "6", "6 months", "6h", "-6m"] {
            assert!(!is_review_window(bad), "{bad}");
        }
    }
}
