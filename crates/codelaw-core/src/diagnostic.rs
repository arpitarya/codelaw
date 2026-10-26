use crate::text::Position;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warn,
    Info,
}

impl Severity {
    pub fn parse(s: &str) -> Option<Severity> {
        Some(match s {
            "error" => Severity::Error,
            "warn" => Severity::Warn,
            "info" => Severity::Info,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Info => "info",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The three fix classes of L4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixClass {
    Auto,
    Suggest,
    Manual,
}

impl FixClass {
    pub fn parse(s: &str) -> Option<FixClass> {
        Some(match s {
            "auto" => FixClass::Auto,
            "suggest" => FixClass::Suggest,
            "manual" => FixClass::Manual,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            FixClass::Auto => "auto",
            FixClass::Suggest => "suggest",
            FixClass::Manual => "manual",
        }
    }
}

impl fmt::Display for FixClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An LSP-style text edit over byte offsets.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextEdit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl TextEdit {
    pub fn new(start: usize, end: usize, text: impl Into<String>) -> Self {
        TextEdit { start, end, text: text.into() }
    }

    pub fn delete(start: usize, end: usize) -> Self {
        TextEdit { start, end, text: String::new() }
    }

    pub fn insert(at: usize, text: impl Into<String>) -> Self {
        TextEdit { start: at, end: at, text: text.into() }
    }
}

/// One finding. Every diagnostic carries its fix (L4 decision 4): the class,
/// the edits for `auto` and `suggest`, the steps for `manual`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub rule: String,
    pub severity: Severity,
    pub file: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start: Position,
    pub end: Position,
    pub message: String,
    pub fix: FixClass,
    pub edits: Vec<TextEdit>,
    pub steps: Vec<String>,
    /// Where the rule came from: `local`, `engine`, or `<pack>@<version>`.
    pub source: String,
    /// Why a fix was not offered, e.g. `fix-conflict` or `fix-verify-failed`.
    pub note: Option<String>,
    /// Structured facts about the finding, for `--format json` and SARIF: a
    /// size rule's measured value and its limit (`{ "lines": 412, "max": 300 }`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Diagnostic {
    /// The L5 decision 2 order: path, line, column, rule.
    pub fn sort_key(&self) -> (&str, Position, &str, usize, &str) {
        (&self.file, self.start, &self.rule, self.end_byte, &self.message)
    }
}

/// A file that was not checked, and why (SR-ARCHITECTURE decision 26).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Skipped {
    pub file: String,
    pub reason: String,
}
