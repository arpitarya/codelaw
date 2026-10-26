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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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
    #[serde(default, deserialize_with = "de_data")]
    pub data: Option<serde_json::Value>,
}

/// `data` is omitted from JSON when absent, but the cache's codec is
/// positional (a skipped field shifts every later one) and cannot carry a
/// `serde_json::Value` (it is self-describing): so a non-human-readable
/// serializer always gets the field, as the value's JSON text.
impl Serialize for Diagnostic {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let readable = s.is_human_readable();
        let n = 13 + usize::from(!readable || self.data.is_some());
        let mut st = s.serialize_struct("Diagnostic", n)?;
        st.serialize_field("rule", &self.rule)?;
        st.serialize_field("severity", &self.severity)?;
        st.serialize_field("file", &self.file)?;
        st.serialize_field("start_byte", &self.start_byte)?;
        st.serialize_field("end_byte", &self.end_byte)?;
        st.serialize_field("start", &self.start)?;
        st.serialize_field("end", &self.end)?;
        st.serialize_field("message", &self.message)?;
        st.serialize_field("fix", &self.fix)?;
        st.serialize_field("edits", &self.edits)?;
        st.serialize_field("steps", &self.steps)?;
        st.serialize_field("source", &self.source)?;
        st.serialize_field("note", &self.note)?;
        if readable {
            if self.data.is_some() {
                st.serialize_field("data", &self.data)?;
            }
        } else {
            st.serialize_field("data", &self.data.as_ref().map(|v| v.to_string()))?;
        }
        st.end()
    }
}

fn de_data<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<serde_json::Value>, D::Error> {
    if d.is_human_readable() {
        Option::<serde_json::Value>::deserialize(d)
    } else {
        Option::<String>::deserialize(d)?.map(|t| serde_json::from_str(&t).map_err(serde::de::Error::custom)).transpose()
    }
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
