//! `codelaw exceptions` and the MCP `list_exceptions` (SR-BASELINE decision
//! 11): every exception the project holds, in one list. A read-only report
//! over the files; it lints nothing and reads the clock never (L5
//! decision 2): the day is the as-of date.

use crate::project::{Source, suppressions};
use crate::{AsOf, EngineError, Project};
use codelaw_baseline::Date;

/// Where an exception is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExceptionKind {
    /// An `[[allow]]` of `.codelaw/baseline/<rule>.toml`.
    Baseline,
    /// A `[[deprecated]]` of `.codelaw/deprecations.toml`, with the
    /// baseline entries of its symbol folded into it.
    Deprecation,
    /// A `codelaw-ignore` comment, one per rule it names.
    Inline,
}

impl ExceptionKind {
    pub fn name(self) -> &'static str {
        match self {
            ExceptionKind::Baseline => "baseline",
            ExceptionKind::Deprecation => "deprecation",
            ExceptionKind::Inline => "inline",
        }
    }
}

/// One exception.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exception {
    pub kind: ExceptionKind,
    /// The rule it excepts; `graph/no-deprecated-usage` for a deprecation.
    pub rule: String,
    /// The files it covers, project-relative. One for a baseline entry
    /// and an inline ignore; the files of the folded entries for a
    /// deprecation.
    pub files: Vec<String>,
    /// The deprecated symbol, or the symbol of a baseline entry.
    pub symbol: Option<String>,
    /// The first day it no longer holds.
    pub expires: Option<Date>,
    pub added_by: Option<String>,
    pub reason: Option<String>,
    /// The uses it covers: the entry's `count`; 1 for an inline ignore.
    pub uses: usize,
    /// Where it is written: the file holding it, and the 1-based line.
    pub at: String,
    pub line: u32,
}

/// Every exception of a project, and the as-of date they are judged on.
#[derive(Clone, Debug)]
pub struct Exceptions {
    pub as_of: AsOf,
    /// Sorted by `expires` (none last), rule, file, line.
    pub items: Vec<Exception>,
    /// What could not be read, to print on stderr.
    pub warnings: Vec<String>,
    /// The email `git config user.email` names, for `--mine`.
    pub git_email: Option<String>,
}

impl Project {
    /// Collect every exception: the baseline entries, the deprecations,
    /// and the inline `codelaw-ignore` comments of every discovered file.
    pub fn exceptions(&mut self) -> Result<Exceptions, EngineError> {
        let mut items = Vec::new();
        let registry = self.deprecations.as_ref().map(|d| d.registry.clone()).unwrap_or_default();
        let mut folded: std::collections::BTreeMap<String, Vec<(&str, &codelaw_baseline::Entry)>> = Default::default();
        for b in self.baselines.rules.values() {
            let at = codelaw_baseline::file::rel_path(&b.rule);
            for e in &b.entries {
                if let Some(sym) = e.symbol.as_deref().filter(|s| registry.find(s).is_some()) {
                    folded.entry(sym.to_owned()).or_default().push((&b.rule, e));
                    continue;
                }
                items.push(Exception {
                    kind: ExceptionKind::Baseline,
                    rule: b.rule.clone(),
                    files: vec![e.file.clone()],
                    symbol: e.symbol.clone(),
                    expires: self.entry_expires(e),
                    added_by: e.added_by.clone(),
                    reason: e.reason.clone(),
                    uses: e.count,
                    at: at.clone(),
                    line: e.line,
                });
            }
        }
        for d in &registry.deprecated {
            let entries = folded.remove(&d.symbol).unwrap_or_default();
            let mut files: Vec<String> = entries.iter().map(|(_, e)| e.file.clone()).collect();
            files.sort();
            files.dedup();
            items.push(Exception {
                kind: ExceptionKind::Deprecation,
                rule: codelaw_baseline::DEPRECATED_RULE.to_owned(),
                files,
                symbol: Some(d.symbol.clone()),
                expires: d.policy.expires,
                added_by: d.added_by.clone(),
                reason: d.reason.clone(),
                uses: entries.iter().map(|(_, e)| e.count).sum(),
                at: codelaw_baseline::REGISTRY_REL.to_owned(),
                line: d.line,
            });
        }
        let mut warnings = Vec::new();
        for f in self.files(&[])? {
            let Ok(read) = self.read_entry(&f) else { continue };
            if !read.text.contains(crate::suppress::MARKER) {
                continue;
            }
            let src = Source { rel: f.rel.clone(), abs: f.abs.clone(), language: Some(f.language), text: read.text, reported: true };
            let index = codelaw_core::LineIndex::new(&src.text);
            for s in suppressions(&src) {
                // A comment whose keys cannot be read covers nothing: it is
                // not an exception, and `check` already reports it.
                if s.dated.bad.is_some() {
                    continue;
                }
                let line = index.position(s.start).line;
                for rule in &s.rules {
                    items.push(Exception {
                        kind: ExceptionKind::Inline,
                        rule: rule.clone(),
                        files: vec![f.rel.clone()],
                        symbol: None,
                        expires: s.dated.expires,
                        added_by: s.dated.by.clone(),
                        reason: s.reason.clone(),
                        uses: 1,
                        at: f.rel.clone(),
                        line,
                    });
                }
            }
        }
        items.sort_by(|a, b| {
            let key = |e: &Exception| (e.expires.is_none(), e.expires.map(|d| d.days()), e.rule.clone(), e.files.first().cloned(), e.at.clone(), e.line);
            key(a).cmp(&key(b))
        });
        let as_of = self.as_of().clone();
        if let Some(why) = as_of.date.is_none().then_some(&as_of.source) {
            warnings.push(format!("no as-of date ({why}): days left are not shown"));
        }
        Ok(Exceptions { as_of, items, warnings, git_email: self.git_user_email() })
    }
}
