//! The queries behind `rules --for`, `config --print`, `match --explain`
//! and `fix --trace` (SR-CLI-MCP decision 1, SR-RULE-AUTHORING decision 3).
//! Every surface — CLI, MCP, the npm and Python APIs — calls these.

use crate::compile::Matcher;
use crate::discover::FileEntry;
use crate::fix::FixOptions;
use crate::lint::{RunMode, lint_text};
use crate::{EngineError, Project};
use codelaw_config::{TraceEnv, Traced};
use codelaw_core::{FixClass, Language, LineIndex, Position};
use codelaw_fix::{LoopTrace, fix_loop_traced};
use codelaw_lang::Grammar;
use codelaw_match::{Explainer, NodeExplanation};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// One rule as `rules --for` and MCP `rules_for_path` show it: a card under
/// about 60 tokens (SR-CLI-MCP decision 7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleCard {
    pub id: String,
    pub title: String,
    /// `error`, `warn`, `info` or `off`.
    pub severity: String,
    pub fix: FixClass,
    /// `local`, or the pack it comes from.
    pub source: String,
    pub preview: bool,
}

/// `match <id> <file> [--explain]`.
#[derive(Clone, Debug)]
pub struct MatchReport {
    pub rule: String,
    pub file: String,
    pub language: Language,
    /// Each candidate node, with its positions.
    pub nodes: Vec<(Position, Position, NodeExplanation)>,
}

/// One file's `fix --trace`.
#[derive(Clone, Debug)]
pub struct FileTrace {
    pub file: String,
    pub trace: LoopTrace,
    pub changed: bool,
}

fn severity_name(s: Option<codelaw_core::Severity>) -> String {
    s.map(|s| s.name().to_owned()).unwrap_or_else(|| "off".into())
}

impl Project {
    /// A path as the engine sees it: absolute, and relative to the root.
    fn locate_path(&self, file: &Path) -> (PathBuf, String) {
        let abs = std::fs::canonicalize(file).unwrap_or_else(|_| self.root().join(file));
        let rel = codelaw_core::normalize_path(abs.strip_prefix(self.root()).unwrap_or(&abs));
        (abs, rel)
    }

    /// The rules that apply to `file` with the severity they run at, or —
    /// with no file — every rule with its default severity.
    pub fn rule_cards(&self, file: Option<&Path>) -> Result<Vec<RuleCard>, EngineError> {
        let target = match file {
            Some(f) => {
                let (abs, rel) = self.locate_path(f);
                let Some(language) = Language::from_path(&abs) else {
                    return Err(EngineError::Usage(format!("{rel}: not a file codelaw checks (no known extension)")));
                };
                Some((abs, rel, language))
            }
            None => None,
        };
        let layers = target.as_ref().map(|(abs, _, _)| self.config.layers_for(abs));
        let mut out = Vec::new();
        for c in &self.compiled {
            let r = &c.loaded;
            let severity = match (&target, &layers) {
                (Some((_, rel, language)), Some(layers)) => {
                    if !c.applies_to_language(*language) || !c.applies_to_path(rel) {
                        continue;
                    }
                    let s = severity_name(self.rules.effective(r, layers).and_then(|e| e.severity));
                    if s == "off" {
                        continue;
                    }
                    s
                }
                _ => severity_name(r.default.as_ref().and_then(|d| d.severity)),
            };
            out.push(RuleCard {
                id: r.rule.id.clone(),
                title: r.rule.title.clone(),
                severity,
                fix: r.rule.fix,
                source: r.source.label(),
                preview: r.preview_blocked,
            });
        }
        Ok(out)
    }

    /// `config --print [--for <file>]`: the config's values (SR-CONFIG
    /// decision 7), then every rule's severity, each with its source.
    pub fn config_trace(&self, file: Option<&Path>, env: &TraceEnv) -> Vec<Traced> {
        let abs = file.map(|f| self.locate_path(f).0);
        let mut out = self.config.trace(abs.as_deref(), env);
        let layers = self.config.layers_for(abs.as_deref().unwrap_or(self.root()));
        for c in &self.compiled {
            let r = &c.loaded;
            let id = &r.rule.id;
            let effective = self.rules.effective(r, &layers);
            let sources = self.config.rule_setting_sources(abs.as_deref(), id);
            let source = if r.preview_blocked {
                "preview rule: runs only with `preview = true`".to_owned()
            } else if let Some((s, _)) = sources.last() {
                s.clone()
            } else if let Some(by) = &r.enabled_by {
                format!("preset {by}")
            } else {
                "default: not in any extended preset".to_owned()
            };
            let severity = severity_name(effective.as_ref().and_then(|e| e.severity));
            let value = match effective.filter(|e| !e.options.is_empty()) {
                Some(e) => json!({ "severity": severity, "options": e.options }),
                None => Value::String(severity),
            };
            out.push(Traced { key: format!("rules.{}", serde_json::to_string(id).expect("a string")), value, source });
        }
        out
    }

    /// Which clause of a `match:` rule held or failed on each candidate
    /// node of `file`, and what was captured (`match --explain`).
    pub fn explain_match(&self, id: &str, file: &Path) -> Result<MatchReport, EngineError> {
        let Some(c) = self.compiled.iter().find(|c| c.id() == id) else {
            let near = codelaw_config::did_you_mean(id, self.compiled.iter().map(|c| c.id()))
                .map(|n| format!("; did you mean `{n}`?"))
                .unwrap_or_default();
            return Err(EngineError::Usage(format!("unknown rule `{id}`{near}")));
        };
        let (abs, rel) = self.locate_path(file);
        let language = Language::from_path(&abs).ok_or_else(|| EngineError::Usage(format!("{rel}: not a file codelaw checks")))?;
        if !matches!(c.matcher, Matcher::Match(_)) {
            return Err(EngineError::Usage(format!("`{id}` is not a `match:` rule; `match --explain` explains `match:` rules")));
        }
        if !c.applies_to_language(language) {
            let langs: Vec<&str> = c.loaded.rule.languages.iter().map(|l| l.name()).collect();
            return Err(EngineError::Usage(format!("`{id}` does not apply to {} ({rel}); it is for {}", language.name(), langs.join(", "))));
        }
        let grammar = Grammar::for_language(language);
        let explainer = Explainer::new(&c.loaded.rule, grammar, &c.loaded.data).map_err(|e| EngineError::Usage(format!("`{id}` on {}: {e}", language.name())))?;
        let text = std::fs::read_to_string(&abs).map_err(|e| EngineError::Usage(format!("{rel}: {e}")))?;
        let text = text.strip_prefix('\u{feff}').map(str::to_owned).unwrap_or(text);
        let root = codelaw_lang::parse(language, &text);
        let semantic = if c.needs_semantic { codelaw_semantic::build(language, &root, &text) } else { None };
        let index = LineIndex::new(&text);
        let nodes = explainer.explain(&root, semantic.as_ref()).into_iter().map(|n| (index.position(n.start), index.position(n.end), n)).collect();
        Ok(MatchReport { rule: id.to_owned(), file: rel, language, nodes })
    }

    /// `fix --trace`: each pass of the apply loop for every file a fix
    /// would change, computed in memory; nothing is written.
    pub fn trace_fix(&mut self, paths: &[PathBuf], options: &FixOptions) -> Result<Vec<FileTrace>, EngineError> {
        let files: Vec<FileEntry> = self.files(paths)?;
        let mut out = Vec::new();
        for entry in &files {
            let Ok(file) = self.read_entry(entry) else { continue };
            let mut trace = Some(LoopTrace::default());
            let outcome = fix_loop_traced(
                entry.language,
                &file.text,
                |current, refused| {
                    lint_text(self, entry, current, &RunMode { external_fixers: true, refused: Some(refused) })
                        .candidates
                        .into_iter()
                        .filter(|(_, class)| *class == FixClass::Auto || (options.unsafe_fixes && *class == FixClass::Suggest))
                        .map(|(c, _)| c)
                        .collect()
                },
                &mut trace,
            );
            let trace = trace.expect("traced");
            if !trace.passes.is_empty() {
                out.push(FileTrace { file: entry.rel.clone(), changed: outcome.text != file.text, trace });
            }
        }
        Ok(out)
    }
}
