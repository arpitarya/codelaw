//! Rule files, rule packs and the rule set (SR-RULE-FORMAT, SR-RULE-PACKS).
//!
//! A rule is one Markdown file. It is parsed and checked here — the
//! template (SR-RULE-FORMAT decisions 2–5) and the L4 fix obligations —
//! before any engine sees it. Compiling its `codelaw` block is the
//! engines' job.

mod engine_rules;
mod markdown;
mod pack;
mod rule;
mod ruleset;

pub use engine_rules::{ENGINE_RULES, EngineRule, engine_rule};
pub use pack::{ExtendsSpec, FIRST_PARTY_PACKS, FirstPartyPack, PackManifest, PackSearch, Preset, RulePack, first_party, first_party_npm, first_party_pypi, load_pack, load_pack_from, load_rules_dir, namespace_pack};
pub use rule::{
    Block, BlockKind, Case, Example, ExampleKind, KNOWN_SKILLS, LanguageExamples, OptionSpec, RESERVED_NAMESPACES,
    RuleFile, Scope, Snippet, Stability, parse_cases, parse_rule,
};
pub use ruleset::{LoadOptions, LoadedRule, RuleSet, RuleSource};

use std::fmt;
use std::path::PathBuf;

/// A rule, pack or preset that cannot be loaded: exit 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
    pub message: String,
}

impl LoadError {
    pub fn new(file: Option<PathBuf>, message: impl Into<String>) -> Self {
        LoadError { file, line: None, message: message.into() }
    }

    pub fn at(file: &std::path::Path, line: usize, message: impl Into<String>) -> Self {
        LoadError { file: Some(file.to_owned()), line: Some(line), message: message.into() }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.file, self.line) {
            (Some(file), Some(line)) => write!(f, "{}:{line}: {}", codelaw_core::normalize_path(file), self.message),
            (Some(file), None) => write!(f, "{}: {}", codelaw_core::normalize_path(file), self.message),
            _ => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for LoadError {}
