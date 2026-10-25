//! Configuration: three homes, one format (SR-CONFIG).
//!
//! `codelaw.toml`, then `[tool.codelaw]` in `pyproject.toml`, then the
//! `"codelaw"` key of `package.json` (decision 1). Validation is strict
//! (decision 5) and the resolved config is a pure function of its inputs
//! (decision 9). Every value is traceable to its source (decision 7).

mod raw;
mod resolve;
mod trace;

pub use raw::{RawConfig, RawOverride, RawProfile, RuleSetting};
pub use resolve::{Config, ConfigFile, DEFAULT_EXCLUDES, Home, Layer, Profile, ProfileChoice, Remap, RuleConfig, find_config, load_file};
pub use trace::{TraceEnv, Traced};

/// `codelaw.schema.json`: every key a config may set, each naming the one
/// record that defines it (decision 6). Shipped in npm `codelaw` and printed
/// by `codelaw config --schema`; `tests/schema.rs` holds it to the records
/// and to what the loader accepts.
pub const SCHEMA: &str = include_str!("../codelaw.schema.json");

use std::fmt;
use std::path::PathBuf;

/// A configuration error: exit 2, naming the file, the key and the nearest
/// valid key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub file: Option<PathBuf>,
    pub message: String,
}

impl ConfigError {
    pub fn new(file: Option<PathBuf>, message: impl Into<String>) -> Self {
        ConfigError { file, message: message.into() }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.file {
            Some(file) => write!(f, "{}: {}", codelaw_core::normalize_path(file), self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for ConfigError {}

/// "did you mean …?" for an unknown name among the valid ones.
pub fn did_you_mean<'a>(unknown: &str, valid: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    valid
        .into_iter()
        .map(|v| (strsim::jaro_winkler(unknown, v), v))
        .filter(|(score, _)| *score > 0.8)
        .max_by(|a, b| a.0.total_cmp(&b.0).then_with(|| b.1.cmp(a.1)))
        .map(|(_, v)| v)
}

/// Rewrites serde's "unknown field `x`, expected one of `a`, `b`" into the
/// SR-CONFIG decision 5 shape.
pub(crate) fn explain_serde_error(message: &str) -> String {
    let first_line = message.lines().find(|l| l.contains("unknown field")).unwrap_or(message);
    if let Some(rest) = first_line.split("unknown field `").nth(1)
        && let Some((unknown, tail)) = rest.split_once('`')
    {
        let expected: Vec<&str> = tail.split('`').skip(1).step_by(2).collect();
        return match did_you_mean(unknown, expected.iter().copied()) {
            Some(near) => format!("unknown key `{unknown}`; did you mean `{near}`?"),
            None if !expected.is_empty() => {
                format!("unknown key `{unknown}`; valid keys are {}", expected.iter().map(|e| format!("`{e}`")).collect::<Vec<_>>().join(", "))
            }
            None => format!("unknown key `{unknown}`"),
        };
    }
    message.trim().to_owned()
}
