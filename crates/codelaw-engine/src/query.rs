//! What a rule set says about itself: every rule, the rules that apply to
//! one file, and why one rule is or isn't active (SR-CLI-MCP decision 5).
//! `codelaw rules`, `explain` and the MCP tools `list_rules`,
//! `rules_for_path`, `explain_rule` and `trace_rule` all call these, so
//! both surfaces stay thin (decision 6).

use crate::{EngineError, Project};
use codelaw_config::RuleConfig;
use codelaw_core::{Diagnostic, FixClass, Language};
use codelaw_rules::LoadedRule;
use std::path::{Path, PathBuf};

/// A severity as config spells it; `off` when a rule does not run.
pub fn severity_name(cfg: &Option<RuleConfig>) -> String {
    cfg.as_ref().and_then(|c| c.severity).map(|s| s.name().to_owned()).unwrap_or_else(|| "off".into())
}

/// One rule in a listing: the card `rules_for_path` returns (SR-CLI-MCP
/// decision 7, about 60 tokens).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleRow {
    pub id: String,
    pub title: String,
    /// The default severity in a listing; the effective one for a file.
    pub severity: String,
    pub fix: FixClass,
    pub source: String,
    pub preview: bool,
}

/// Why a rule is or isn't active (`rules --why`, MCP `trace_rule`).
#[derive(Clone, Debug)]
pub struct RuleWhy {
    pub id: String,
    pub title: String,
    pub source: String,
    pub path: String,
    pub shadows: Vec<String>,
    pub enabled_by: Option<String>,
    pub default: String,
    pub preview: bool,
    pub for_file: Option<WhyForFile>,
}

/// The file half of [`RuleWhy`].
#[derive(Clone, Debug)]
pub struct WhyForFile {
    /// Relative to the project root.
    pub file: String,
    /// Each config layer that sets the rule, and the value it sets.
    pub set_by: Vec<(String, String)>,
    /// The rule's language and `files` globs admit the file.
    pub applies: bool,
    /// The effective severity there; `off` when it does not apply.
    pub severity: String,
    /// What the rule reports in the file, when the file exists.
    pub diagnostics: Vec<Diagnostic>,
}

fn row(r: &LoadedRule, severity: String) -> RuleRow {
    RuleRow { id: r.rule.id.clone(), title: r.rule.title.clone(), severity, fix: r.rule.fix, source: r.source.label(), preview: r.preview_blocked }
}

impl Project {
    /// Every rule, with its default severity, in id order.
    pub fn list_rules(&self) -> Vec<RuleRow> {
        self.compiled.iter().map(|c| row(&c.loaded, severity_name(&c.loaded.default))).collect()
    }

    /// A file as the engine sees it: absolute, relative to the root, and
    /// its language.
    fn locate(&self, file: &Path) -> Result<(PathBuf, String, Language), EngineError> {
        let abs = std::fs::canonicalize(file).unwrap_or_else(|_| {
            if file.is_absolute() { file.to_path_buf() } else { std::env::current_dir().map(|d| d.join(file)).unwrap_or_else(|_| file.to_path_buf()) }
        });
        let rel = codelaw_core::normalize_path(abs.strip_prefix(self.root()).unwrap_or(&abs));
        let Some(language) = Language::from_path(&abs) else {
            return Err(EngineError::Usage(format!("`{}` is not a file codelaw checks (no known extension)", codelaw_core::normalize_path(file))));
        };
        Ok((abs, rel, language))
    }

    /// The rules that run on `file`, with their effective severity; rules
    /// that are off there are left out.
    pub fn rules_for(&self, file: &Path) -> Result<Vec<RuleRow>, EngineError> {
        let (abs, rel, language) = self.locate(file)?;
        let layers = self.config.layers_for(&abs);
        Ok(self
            .compiled
            .iter()
            .filter(|c| c.applies_to_language(language) && c.applies_to_path(&rel))
            .map(|c| row(&c.loaded, severity_name(&self.rules.effective(&c.loaded, &layers))))
            .filter(|r| r.severity != "off")
            .collect())
    }

    /// A rule of this project by id, or an error naming the nearest id.
    pub fn find_rule(&self, id: &str) -> Result<&LoadedRule, EngineError> {
        self.rules.get(id).ok_or_else(|| {
            let near = codelaw_config::did_you_mean(id, self.rules.rules.iter().map(|r| r.rule.id.as_str()))
                .map(|n| format!("; did you mean `{n}`?"))
                .unwrap_or_default();
            EngineError::Usage(format!("unknown rule `{id}`{near}"))
        })
    }

    /// Why `id` is active, from which source, and — with a file — what
    /// config sets it there and what it reports.
    pub fn rule_why(&mut self, id: &str, file: Option<&Path>) -> Result<RuleWhy, EngineError> {
        let r = self.find_rule(id)?.clone();
        let mut why = RuleWhy {
            id: id.to_owned(),
            title: r.rule.title.clone(),
            source: r.source.label(),
            path: codelaw_core::normalize_path(&r.rule.path),
            shadows: r.shadows.clone(),
            enabled_by: r.enabled_by.clone(),
            default: severity_name(&r.default),
            preview: r.preview_blocked,
            for_file: None,
        };
        let Some(file) = file else { return Ok(why) };
        let (abs, rel, language) = self.locate(file)?;
        let layers = self.config.layers_for(&abs);
        let set_by =
            layers.iter().filter(|l| l.rules.contains_key(id)).map(|l| (l.source.clone(), l.rules[id].severity().unwrap_or("(options)").to_owned())).collect();
        let applies = self.compiled.iter().find(|c| c.id() == id).is_some_and(|c| c.applies_to_language(language) && c.applies_to_path(&rel));
        let severity = if applies { severity_name(&self.rules.effective(&r, &layers)) } else { "off".into() };
        let diagnostics = if applies && abs.is_file() { self.check(&[abs])?.diagnostics.into_iter().filter(|d| d.rule == id).collect() } else { Vec::new() };
        why.for_file = Some(WhyForFile { file: rel, set_by, applies, severity, diagnostics });
        Ok(why)
    }

    /// The project that governs the first path (its folder for a file), or
    /// the current folder when there is none — what every command loads.
    pub fn load_for(paths: &[PathBuf], options: crate::EngineOptions) -> Result<Project, EngineError> {
        let start = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
        let start = if start.is_file() { start.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")) } else { start };
        let start = if start.as_os_str().is_empty() { PathBuf::from(".") } else { start };
        Project::load(&start, options)
    }

    /// Check `code` as if it were the text of `file`, which need not exist
    /// (MCP `check_snippet`): the `check` report of that one file.
    pub fn check_snippet(&self, file: &Path, code: &str) -> Result<crate::CheckReport, EngineError> {
        let (_, rel, language) = self.locate(file)?;
        Ok(crate::CheckReport { diagnostics: self.check_source(&rel, language, code), skipped: Vec::new(), files_checked: 1, warnings: self.warnings.clone() })
    }

    /// Fix `code` as if it were the text of `file`, applying `auto` fixes
    /// (and the format step, once `fix` has one); nothing is written.
    pub fn fix_snippet(&self, file: &Path, code: &str) -> Result<crate::FileFix, EngineError> {
        let (_, rel, language) = self.locate(file)?;
        Ok(self.fix_source(&rel, language, code, false))
    }
}

impl crate::EngineOptions {
    /// The environment's options narrowed by `--lang` and `--rule`.
    pub fn with_filters(languages: &[String], rules: &[String]) -> Result<crate::EngineOptions, String> {
        let mut o = crate::EngineOptions::from_env();
        if !languages.is_empty() {
            let mut set = std::collections::BTreeSet::new();
            for l in languages {
                match Language::from_name(l) {
                    Some(lang) => {
                        set.insert(lang);
                    }
                    None => {
                        let near = codelaw_config::did_you_mean(l, Language::ALL.iter().map(|x| x.name()))
                            .map(|n| format!("; did you mean `{n}`?"))
                            .unwrap_or_default();
                        return Err(format!("unknown language `{l}` in --lang{near}"));
                    }
                }
            }
            o.languages = Some(set);
        }
        if !rules.is_empty() {
            o.rules = Some(rules.to_vec());
        }
        Ok(o)
    }
}
