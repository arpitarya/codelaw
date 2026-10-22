//! The queries behind `config --print` and `match --explain` (SR-CLI-MCP
//! decision 1, SR-RULE-AUTHORING decision 3; SR-CONFIG decision 7). The
//! rule listings are `query.rs`'s; `fix --trace` is the fix pipeline's own
//! loop (`fix.rs`).

use crate::compile::Matcher;
use crate::{EngineError, Project};
use codelaw_config::{TraceEnv, Traced};
use codelaw_core::{Language, LineIndex, Position};
use codelaw_lang::Grammar;
use codelaw_match::{Explainer, NodeExplanation};
use serde_json::{Value, json};
use std::path::Path;

/// `match <id> <file> [--explain]`.
#[derive(Clone, Debug)]
pub struct MatchReport {
    pub rule: String,
    pub file: String,
    pub language: Language,
    /// Each candidate node, with its positions.
    pub nodes: Vec<(Position, Position, NodeExplanation)>,
}

impl Project {
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
            let severity = crate::severity_name(&effective);
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
        let (abs, rel, language) = self.locate(file)?;
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
}
