use serde::Deserialize;
use std::collections::BTreeMap;

/// The keys of one config file, exactly as written. Every section is
/// strict: an unknown key is a load error (SR-CONFIG decision 5).
///
/// The sections owned by records whose features are not built yet
/// (`[format]`, `[graph]`, `[baseline]`, `[adapters]`, `[agents]`,
/// `[daemon]`, `[lsp]`) are accepted as tables and not yet validated.
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
    #[serde(default)]
    pub overrides: Vec<RawOverride>,
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
    pub adapters: Option<toml::Table>,
    pub agents: Option<toml::Table>,
    pub daemon: Option<toml::Table>,
    pub lsp: Option<toml::Table>,
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

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOverride {
    pub files: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub rules: BTreeMap<String, RuleSetting>,
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

/// `[lint]`: SR-LINTER decision 3.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLint {
    pub tagged_templates: Option<bool>,
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
