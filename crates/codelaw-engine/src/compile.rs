//! Compile once, dispatch by kind (SR-ARCHITECTURE decision 15): every
//! loaded rule becomes a matcher per grammar, with the node kinds it can
//! match, before any file is read.

use codelaw_config::Config;
use codelaw_core::Language;
use codelaw_fix::{DataStep, FixSpec, parse_data_fix, parse_fix};
use codelaw_lang::{AgLanguage, Grammar};
use codelaw_match::{CompiledMatch, CompiledTarget};
use codelaw_native::NativeRule;
use codelaw_rules::{BlockKind, LoadError, LoadedRule, RuleSet};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

pub enum Matcher {
    /// One compiled `match:` per grammar the rule's languages parse with.
    Match(Vec<(Grammar, CompiledMatch)>),
    Native(&'static dyn NativeRule),
<<<<<<< HEAD
    /// A `scope: project` rule (SR-PROJECT-GRAPH decision 6).
    Graph(codelaw_graph::GraphRule),
=======
    /// `target:` + `assert:` on the data tree of a JSON, YAML or TOML file
    /// (SR-RULE-FORMAT decision 3e); it runs once per file, not per node.
    Target(CompiledTarget),
>>>>>>> origin/wip/W-12-structured-rules
    /// A form this version cannot run yet; the reason is warned once.
    Unsupported(&'static str),
}

pub struct CompiledRule {
    pub loaded: LoadedRule,
    pub matcher: Matcher,
    pub fix: Option<FixSpec>,
    /// A `target:` rule's fix steps.
    pub data_fix: Option<Vec<DataStep>>,
    pub needs_semantic: bool,
    files: Option<GlobSet>,
    exclude: GlobSet,
}

impl CompiledRule {
    pub fn id(&self) -> &str {
        &self.loaded.rule.id
    }

    /// A rule's `files` and `exclude` globs are relative to the repo root
    /// (SR-ARCHITECTURE decision 11).
    pub fn applies_to_path(&self, rel: &str) -> bool {
        self.files.as_ref().is_none_or(|f| f.is_match(rel)) && !self.exclude.is_match(rel)
    }

    pub fn applies_to_language(&self, language: Language) -> bool {
        self.loaded.rule.languages.contains(&language)
    }

    /// The kinds to dispatch on for one grammar: `None` when the rule has no
    /// matcher for it, `Some(None)` for every kind.
    pub fn kinds_for(&self, grammar: Grammar) -> Option<Option<Vec<u16>>> {
        match &self.matcher {
            Matcher::Match(per) => per.iter().find(|(g, _)| *g == grammar).map(|(_, m)| m.kinds()),
            Matcher::Native(n) => Some(Some(kind_ids(grammar, n.kinds()))),
<<<<<<< HEAD
            Matcher::Graph(_) | Matcher::Unsupported(_) => None,
=======
            // Dispatched to no node: the unit runs it on its data tree.
            Matcher::Target(_) => matches!(grammar, Grammar::Json | Grammar::Yaml | Grammar::Toml).then_some(Some(Vec::new())),
            Matcher::Unsupported(_) => None,
>>>>>>> origin/wip/W-12-structured-rules
        }
    }

    pub fn compiled_match(&self, grammar: Grammar) -> Option<&CompiledMatch> {
        match &self.matcher {
            Matcher::Match(per) => per.iter().find(|(g, _)| *g == grammar).map(|(_, m)| m),
            _ => None,
        }
    }
}

/// Node kind names to the grammar's ids; an unknown name matches nothing.
fn kind_ids(grammar: Grammar, kinds: &[&str]) -> Vec<u16> {
    kinds.iter().map(|k| grammar.kind_to_id(k)).filter(|id| *id != 0).collect()
}

fn globs(patterns: &[String]) -> Result<GlobSet, String> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        b.add(GlobBuilder::new(p.trim_start_matches("./")).literal_separator(true).build().map_err(|e| format!("invalid glob `{p}`: {e}"))?);
    }
    b.build().map_err(|e| e.to_string())
}

/// Compile every loaded rule. A rule that fails refuses the run (exit 2),
/// naming the rule (SR-RULE-PACKS decision 13).
pub fn compile_all(rules: &RuleSet, _config: &Config) -> Result<(Vec<CompiledRule>, Vec<String>), Vec<LoadError>> {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for loaded in &rules.rules {
        let rule = &loaded.rule;
        let path = Some(rule.path.clone());
        let mut fail = |msg: String| errors.push(LoadError::new(path.clone(), format!("rule `{}`: {msg}", rule.id)));
        let matcher = match &rule.block.kind {
            BlockKind::Match { .. } => {
                // A kind must exist in at least one of the rule's grammars;
                // where it is missing it matches nothing.
                let grammars: Vec<Grammar> = rule.languages.iter().flat_map(|l| Grammar::all_for_language(*l).iter().copied()).collect();
                for k in codelaw_match::kinds_named(rule) {
                    if !grammars.iter().any(|g| codelaw_match::grammar_has_kind(*g, &k)) {
                        let langs: Vec<&str> = rule.languages.iter().map(|l| l.name()).collect();
                        fail(format!("`kind: {k}` is not a node kind of {}; `codelaw tree` shows the kinds", langs.join(", ")));
                    }
                }
                // Each declared language needs one grammar the rule compiles
                // for; Markdown has two (block and inline), and a rule about
                // links compiles only for the inline one.
                let mut per: Vec<(Grammar, CompiledMatch)> = Vec::new();
                for language in &rule.languages {
                    let mut first_error = None;
                    let mut any = false;
                    for g in Grammar::all_for_language(*language) {
                        if per.iter().any(|(x, _)| x == g) {
                            any = true;
                            continue;
                        }
                        match codelaw_match::compile(rule, *g, &loaded.data) {
                            Ok(m) => {
                                per.push((*g, m));
                                any = true;
                            }
                            Err(e) => {
                                first_error.get_or_insert(e);
                            }
                        }
                    }
                    if !any && let Some(e) = first_error {
                        fail(e);
                    }
                }
                per.sort_by_key(|(g, _)| *g);
                Matcher::Match(per)
            }
            BlockKind::Native(id) => match codelaw_native::lookup(id) {
                Some(n) => {
                    for l in &rule.languages {
                        if !n.languages().contains(l) {
                            fail(format!("`native: {id}` does not support {l}"));
                        }
                    }
                    Matcher::Native(n)
                }
                None => {
                    fail(format!("`native: {id}` is not an engine rule in codelaw {}", codelaw_core::VERSION));
                    continue;
                }
            },
<<<<<<< HEAD
            BlockKind::Graph(g) => match codelaw_graph::compile(g, rule.block.fix.as_ref()) {
                Ok(c) => Matcher::Graph(c),
=======
            BlockKind::Graph(_) => Matcher::Unsupported("`graph:` rules need the project graph (SR-PROJECT-GRAPH), which this version does not build yet"),
            BlockKind::Target { .. } => match codelaw_match::compile_target(rule, &loaded.data) {
                Ok(t) => {
                    for l in &rule.languages {
                        if !matches!(l, Language::Json | Language::Yaml | Language::Toml) {
                            fail(format!("`target:` rules read JSON, YAML and TOML, not {l}"));
                        }
                    }
                    Matcher::Target(t)
                }
>>>>>>> origin/wip/W-12-structured-rules
                Err(e) => {
                    fail(e);
                    continue;
                }
            },
<<<<<<< HEAD
            BlockKind::Target { .. } => Matcher::Unsupported("`target:` + `assert:` rules are not built yet in this version"),
=======
>>>>>>> origin/wip/W-12-structured-rules
        };
        if let Matcher::Unsupported(why) = &matcher
            && loaded.default.is_some()
        {
            warnings.push(format!("rule `{}` does not run: {why}", rule.id));
        }
        let is_target = matches!(matcher, Matcher::Target(_));
        let data_fix = match (&rule.block.fix, is_target) {
            (Some(v), true) => match parse_data_fix(v, &loaded.data) {
                Ok(f) => Some(f),
                Err(e) => {
                    fail(format!("invalid `fix:`: {e}"));
                    None
                }
            },
            _ => None,
        };
        let fix = match &rule.block.fix {
<<<<<<< HEAD
            // A graph rule's fix compiled with its query.
            Some(_) if matches!(matcher, Matcher::Graph(_)) => None,
=======
            Some(_) if is_target => None,
>>>>>>> origin/wip/W-12-structured-rules
            Some(v) => match parse_fix(v) {
                Ok(f) => Some(f),
                Err(e) => {
                    fail(format!("invalid `fix:`: {e}"));
                    None
                }
            },
            None => None,
        };
        let files = if rule.files.is_empty() { None } else { globs(&rule.files).map_err(&mut fail).ok() };
        let exclude = globs(&rule.exclude).map_err(&mut fail).unwrap_or_else(|_| GlobSet::empty());
        let needs_semantic = match &matcher {
            Matcher::Match(per) => per.iter().any(|(_, m)| m.needs_semantic()),
            Matcher::Native(n) => n.needs_semantic(),
<<<<<<< HEAD
            Matcher::Graph(_) | Matcher::Unsupported(_) => false,
=======
            Matcher::Target(_) | Matcher::Unsupported(_) => false,
>>>>>>> origin/wip/W-12-structured-rules
        } || matches!(&fix, Some(FixSpec::Steps(s)) if s.iter().any(|p| matches!(p, codelaw_fix::Primitive::RenameSymbolLocal { .. })));
        out.push(CompiledRule { loaded: loaded.clone(), matcher, fix, data_fix, needs_semantic, files, exclude });
    }
    // One message per problem, not one per language that shares a grammar.
    errors.dedup_by(|a, b| a.message == b.message && a.file == b.file);
    if errors.is_empty() { Ok((out, warnings)) } else { Err(errors) }
}
