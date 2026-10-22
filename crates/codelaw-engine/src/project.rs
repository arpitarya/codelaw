//! The project stage of `check` (SR-ARCHITECTURE decision 4.4): read
//! every file of the project root once for its graph facts (cached by
//! content, SR-PROJECT-GRAPH decision 5), link the graph and run the
//! `scope: project` rules, then their suppressions.

use crate::compile::{CompiledRule, Matcher};
use crate::lint::comment_ranges;
use crate::{EngineError, Project, suppress};
use codelaw_cache::KeyBuilder;
use codelaw_core::{Diagnostic, FixClass, Language, LineIndex, Severity, TextEdit};
use codelaw_graph::{Graph, GraphConfig, GraphInput, Manifest, is_manifest};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One file of the project as the graph sees it.
#[derive(Clone)]
pub(crate) struct Source {
    pub rel: String,
    pub abs: PathBuf,
    pub language: Option<Language>,
    pub text: String,
}

impl Project {
    /// The active project rules (`--rule` applied).
    pub(crate) fn graph_rules(&self) -> Vec<(usize, &CompiledRule)> {
        let only = self.options.rules.as_ref();
        self.compiled
            .iter()
            .enumerate()
            .filter(|(_, r)| matches!(r.matcher, Matcher::Graph(_)))
            .filter(|(_, r)| only.is_none_or(|o| o.iter().any(|id| id == r.id())))
            .filter(|(_, r)| self.rules.effective(&r.loaded, &[]).is_some() || self.config.named_rule_ids().iter().any(|(id, _)| id == r.id()))
            .collect()
    }

    pub(crate) fn graph_config(&self) -> Result<GraphConfig, EngineError> {
        let table = self.config.files.first().and_then(|f| f.raw.graph.as_ref());
        GraphConfig::from_table(table).map_err(|e| EngineError::Config(codelaw_config::ConfigError::new(self.config.root_file().map(Path::to_path_buf), e)))
    }

    /// Every file of the project root the graph reads, from disk.
    pub(crate) fn graph_sources(&mut self) -> Result<Vec<Source>, EngineError> {
        let files = crate::discover::discover(self, &[])?.0;
        let mut out: Vec<Source> = Vec::new();
        let mut dirs: BTreeSet<PathBuf> = BTreeSet::new();
        let root = self.config.root_dir.clone();
        for f in files {
            let mut d = f.abs.parent().map(Path::to_path_buf);
            while let Some(dir) = d {
                if !dir.starts_with(&root) || !dirs.insert(dir.clone()) {
                    break;
                }
                d = dir.parent().map(Path::to_path_buf);
            }
            let name = f.abs.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_owned();
            if !codelaw_graph::reads(f.language) && !is_manifest(&name) {
                // Other files only exist for resolution.
                out.push(Source { rel: f.rel, abs: f.abs, language: None, text: String::new() });
                continue;
            }
            if let Ok(file) = self.read_entry(&f) {
                out.push(Source { rel: f.rel, abs: f.abs, language: Some(f.language), text: file.text });
            }
        }
        // Manifests, whatever `languages` says: resolution needs them.
        let known: BTreeSet<String> = out.iter().map(|s| s.rel.clone()).collect();
        for dir in dirs {
            for name in ["package.json", "tsconfig.json", "jsconfig.json", "pyproject.toml", "setup.py", "setup.cfg"] {
                let abs = dir.join(name);
                let rel = crate::discover::rel_path(&abs, &root);
                if known.contains(&rel) || !abs.is_file() {
                    continue;
                }
                if let Ok(text) = std::fs::read_to_string(&abs) {
                    out.push(Source { rel, abs, language: None, text });
                }
            }
        }
        out.sort_by(|a, b| a.rel.cmp(&b.rel));
        Ok(out)
    }

    /// Link the graph over a set of sources.
    pub(crate) fn build_graph(&self, sources: &[Source], config: GraphConfig) -> Graph {
        let facts: Vec<Option<codelaw_graph::FileFacts>> = self.pool.install(|| {
            sources
                .par_iter()
                .map(|s| {
                    let language = s.language.filter(|l| codelaw_graph::reads(*l))?;
                    let key = KeyBuilder::new("graph-facts").part(codelaw_graph::FACTS_VERSION).part(language.name()).part(&s.text).part(codelaw_lang::GRAMMARS_FINGERPRINT).key();
                    if let Some(f) = self.cache.get(&key) {
                        return Some(f);
                    }
                    let f = std::panic::catch_unwind(|| codelaw_graph::extract(language, &s.text)).ok().flatten()?;
                    self.cache.put(&key, &f);
                    Some(f)
                })
                .collect()
        });
        let mut inputs = Vec::new();
        let mut manifests = Vec::new();
        for (s, f) in sources.iter().zip(facts) {
            let name = s.rel.rsplit('/').next().unwrap_or_default();
            if is_manifest(name) {
                manifests.push(Manifest { path: s.rel.clone(), text: s.text.clone() });
            }
            inputs.push(GraphInput { path: s.rel.clone(), facts: f });
        }
        Graph::build(inputs, &manifests, config)
    }

    /// The project graph of the whole root (`codelaw graph …`).
    pub fn graph(&mut self) -> Result<Graph, EngineError> {
        let config = self.graph_config()?;
        let sources = self.graph_sources()?;
        Ok(self.build_graph(&sources, config))
    }

    /// Run every active project rule over the sources.
    pub(crate) fn run_graph_rules(&self, sources: &[Source], config: GraphConfig) -> Vec<Diagnostic> {
        let rules = self.graph_rules();
        if rules.is_empty() {
            return Vec::new();
        }
        let graph = self.build_graph(sources, config);
        let by_rel: BTreeMap<&str, &Source> = sources.iter().map(|s| (s.rel.as_str(), s)).collect();
        let mut per_file: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
        for (_, rule) in rules {
            let Matcher::Graph(g) = &rule.matcher else { continue };
            let languages = &rule.loaded.rule.languages;
            let template = rule.loaded.rule.block.message.clone().unwrap_or_else(|| rule.loaded.rule.title.clone());
            for f in codelaw_graph::run(g, &graph, languages) {
                if !rule.applies_to_path(&f.file) {
                    continue;
                }
                let Some(src) = by_rel.get(f.file.as_str()) else { continue };
                let layers = self.config.layers_for(&src.abs);
                let Some(cfg) = self.rules.effective(&rule.loaded, &layers) else { continue };
                let Some(mut severity) = cfg.severity else { continue };
                let index = LineIndex::new(&src.text);
                let mut message = f.render(&template);
                let mut note = None;
                if let Some(why) = &f.uncertain {
                    severity = Severity::Info;
                    message = format!("{message} (uncertain: {why})");
                    note = Some("certainty: uncertain".to_owned());
                }
                let declared = rule.loaded.rule.fix;
                let (class, edits) = if declared == FixClass::Manual {
                    (FixClass::Manual, Vec::new())
                } else if f.edits.is_empty() {
                    note.get_or_insert_with(|| "fix-unavailable".into());
                    (FixClass::Manual, Vec::new())
                } else {
                    (declared, f.edits.clone())
                };
                let steps = if class == FixClass::Manual {
                    if rule.loaded.rule.steps.is_empty() {
                        vec![format!("Make the change by hand; `codelaw explain {}` shows what the rule requires.", rule.id())]
                    } else {
                        rule.loaded.rule.steps.clone()
                    }
                } else {
                    Vec::new()
                };
                per_file.entry(f.file.clone()).or_default().push(Diagnostic {
                    rule: rule.id().to_owned(),
                    severity,
                    file: f.file.clone(),
                    start_byte: f.span.0,
                    end_byte: f.span.1,
                    start: index.position(f.span.0),
                    end: index.position(f.span.1),
                    message,
                    fix: class,
                    edits,
                    steps,
                    source: rule.loaded.source.label(),
                    note,
                });
            }
        }
        let mut out = Vec::new();
        for (file, mut diags) in per_file {
            let src = by_rel[file.as_str()];
            if let Some(language) = src.language {
                // `codelaw-ignore graph/…: reason` works as for file rules.
                let root = codelaw_lang::parse(language, &src.text);
                let comments = comment_ranges(&root, &src.text);
                let index = LineIndex::new(&src.text);
                let sups = suppress::find(&src.text, &comments, &index);
                diags.retain(|d| !sups.iter().any(|s| s.reason.is_some() && s.target_line == d.start.line && s.rules.contains(&d.rule)));
            }
            out.extend(diags);
        }
        out
    }

    /// Apply project-rule edits (auto, and suggest with `unsafe_fixes`)
    /// to the sources, at most `passes` times; the changed texts by path.
    pub(crate) fn fix_graph(&self, sources: &mut [Source], config: &GraphConfig, unsafe_fixes: bool, passes: usize) -> BTreeMap<String, BTreeMap<String, usize>> {
        let mut applied: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for _ in 0..passes {
            let diags = self.run_graph_rules(sources, config.clone());
            let mut edits: BTreeMap<String, Vec<(String, Vec<TextEdit>)>> = BTreeMap::new();
            for d in diags {
                if d.edits.is_empty() || !(d.fix == FixClass::Auto || (unsafe_fixes && d.fix == FixClass::Suggest)) {
                    continue;
                }
                edits.entry(d.file.clone()).or_default().push((d.rule.clone(), d.edits));
            }
            if edits.is_empty() {
                break;
            }
            let mut changed = false;
            for s in sources.iter_mut() {
                let Some(list) = edits.remove(&s.rel) else { continue };
                let mut flat: Vec<(TextEdit, String)> = list.into_iter().flat_map(|(r, es)| es.into_iter().map(move |e| (e, r.clone()))).collect();
                flat.sort_by(|a, b| a.0.start.cmp(&b.0.start).then(a.0.end.cmp(&b.0.end)));
                let mut text = String::with_capacity(s.text.len());
                let mut at = 0;
                let mut count: BTreeMap<String, usize> = BTreeMap::new();
                for (e, rule) in flat {
                    if e.start < at || e.end > s.text.len() {
                        continue;
                    }
                    text.push_str(&s.text[at..e.start]);
                    text.push_str(&e.text);
                    at = e.end;
                    *count.entry(rule).or_default() += 1;
                }
                text.push_str(&s.text[at..]);
                // The verifier's first check: no new syntax error.
                if let Some(l) = s.language {
                    let before = codelaw_lang::syntax_error_count(&codelaw_lang::parse(l, &s.text));
                    let after = codelaw_lang::syntax_error_count(&codelaw_lang::parse(l, &text));
                    if after > before {
                        continue;
                    }
                }
                if text != s.text {
                    s.text = text;
                    changed = true;
                    let a = applied.entry(s.rel.clone()).or_default();
                    for (r, n) in count {
                        *a.entry(r).or_default() += n;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        applied
    }

    /// Check an in-memory set of files as one project: file rules on each,
    /// then project rules over all of them (the rule tester's virtual
    /// repo, SR-PROJECT-GRAPH decision 6).
    pub fn check_files(&self, files: &[(String, Language, String)]) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for (rel, language, text) in files {
            out.extend(self.check_source_in(rel, *language, text, crate::files::Tree::Memory(files)).into_iter().filter(|d| d.rule != "codelaw/no-rules"));
        }
        let sources = self.virtual_sources(files);
        out.extend(self.run_graph_rules(&sources, self.graph_config().unwrap_or_default()));
        out.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        out
    }

    /// Fix an in-memory set of files; nothing is written.
    pub fn fix_files(&self, files: &[(String, Language, String)], unsafe_fixes: bool) -> Vec<(String, String)> {
        let mut texts: Vec<(String, Language, String)> =
            files.iter().map(|(rel, l, t)| (rel.clone(), *l, self.fix_source_in(rel, *l, t, unsafe_fixes, crate::files::Tree::Memory(files)).fixed)).collect();
        let mut sources = self.virtual_sources(&texts);
        self.fix_graph(&mut sources, &self.graph_config().unwrap_or_default(), unsafe_fixes, 10);
        for (rel, _, text) in texts.iter_mut() {
            if let Some(s) = sources.iter().find(|s| &s.rel == rel) {
                *text = s.text.clone();
            }
        }
        texts.into_iter().map(|(r, _, t)| (r, t)).collect()
    }

    fn virtual_sources(&self, files: &[(String, Language, String)]) -> Vec<Source> {
        let mut v: Vec<Source> = files
            .iter()
            .map(|(rel, language, text)| Source { rel: rel.clone(), abs: self.config.root_dir.join(rel), language: Some(*language), text: text.clone() })
            .collect();
        v.sort_by(|a, b| a.rel.cmp(&b.rel));
        v
    }
}

/// Whether a project-relative path is inside one of the checked paths.
pub(crate) fn within(rel: &str, scopes: &[String]) -> bool {
    scopes.is_empty() || scopes.iter().any(|s| s.is_empty() || s == "." || rel == s || rel.starts_with(&format!("{s}/")))
}

/// The checked paths, relative to the project root.
pub(crate) fn scopes(root: &Path, paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|p| {
            let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
            crate::discover::rel_path(&abs, root)
        })
        .collect()
}
