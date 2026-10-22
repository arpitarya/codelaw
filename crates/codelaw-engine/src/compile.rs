//! Compile once, dispatch by kind (SR-ARCHITECTURE decision 15): every
//! loaded rule becomes a matcher per grammar, with the node kinds it can
//! match, before any file is read.
//!
//! The compiled set is cached in memory by the hash of everything it is
//! compiled from (SR-CACHE decision 1's first row, decision 8's tier), so a
//! warm process (`lsp`, `mcp`, `daemon`) compiles a rule set once, however
//! many times it loads the project (SR-ARCHITECTURE decision 21).

use codelaw_cache::{Key, KeyBuilder};
use codelaw_config::Config;
use codelaw_core::Language;
use codelaw_fix::{DataStep, FixSpec, parse_data_fix, parse_fix};
use codelaw_lang::{AgLanguage, Grammar};
use codelaw_match::{CompiledMatch, CompiledTarget};
use codelaw_native::NativeRule;
use codelaw_rules::{BlockKind, LoadError, LoadedRule, RuleSet};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::sync::{Arc, Mutex};

pub enum Matcher {
    /// One compiled `match:` per grammar the rule's languages parse with.
    Match(Vec<(Grammar, CompiledMatch)>),
    Native(&'static dyn NativeRule),
    /// A `scope: project` rule (SR-PROJECT-GRAPH decision 6).
    Graph(codelaw_graph::GraphRule),
    /// `target:` + `assert:` on the data tree of a JSON, YAML or TOML file
    /// (SR-RULE-FORMAT decision 3e); it runs once per file, not per node.
    Target(CompiledTarget),
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
            Matcher::Graph(_) => None,
            // Dispatched to no node: the unit runs it on its data tree.
            Matcher::Target(_) => matches!(grammar, Grammar::Json | Grammar::Yaml | Grammar::Toml).then_some(Some(Vec::new())),
            Matcher::Unsupported(_) => None,
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

/// A compiled rule set and the warnings compiling it gave.
pub(crate) type Compiled = (Arc<Vec<CompiledRule>>, Vec<String>);

/// How many compiled rule sets a process keeps: one per project it serves,
/// most recently used first.
const KEPT: usize = 8;

static COMPILED: Mutex<Vec<(Key, Compiled)>> = Mutex::new(Vec::new());

/// The compiled rule set's key: every input `compile_all` reads, and every
/// field of a loaded rule it copies into the result (SR-CACHE decision 1).
/// The rule set's fingerprint covers each rule file's bytes, its source and
/// default, and every pack's data tables (decision 15c); the config's
/// covers the config files; the version (in every key) and the grammars
/// are the code.
pub(crate) fn rule_set_key(rules: &RuleSet, config: &Config) -> Key {
    let mut k = KeyBuilder::new("compiled-rules");
    k.push(rules.fingerprint.0);
    k.push(config.fingerprint().0);
    k.push(codelaw_lang::GRAMMARS_FINGERPRINT);
    for r in &rules.rules {
        k.push(codelaw_core::normalize_path(&r.rule.path));
        k.push(r.enabled_by.as_deref().unwrap_or_default());
        k.push(r.shadows.join("\0"));
        k.push(format!("{}{}", r.external_allowed, r.preview_blocked));
    }
    k.key()
}

/// The compiled rule set, from the in-memory cache when this process has
/// compiled the same inputs before. A failed compile is never cached.
pub(crate) fn compile_cached(rules: &RuleSet, config: &Config) -> Result<Compiled, Vec<LoadError>> {
    let key = rule_set_key(rules, config);
    let mut kept = COMPILED.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(i) = kept.iter().position(|(k, _)| *k == key) {
        let hit = kept.remove(i);
        let out = hit.1.clone();
        kept.insert(0, hit);
        return Ok(out);
    }
    // Compiled under the lock, so two loads of one project compile it once.
    let (compiled, warnings) = compile_all(rules, config)?;
    let out = (Arc::new(compiled), warnings);
    kept.insert(0, (key, out.clone()));
    kept.truncate(KEPT);
    Ok(out)
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
            BlockKind::Graph(g) => match codelaw_graph::compile(g, rule.block.fix.as_ref()) {
                Ok(c) => Matcher::Graph(c),
                Err(e) => {
                    fail(e);
                    continue;
                }
            },
            BlockKind::Target { .. } => match codelaw_match::compile_target(rule, &loaded.data) {
                Ok(t) => {
                    for l in &rule.languages {
                        if !matches!(l, Language::Json | Language::Yaml | Language::Toml) {
                            fail(format!("`target:` rules read JSON, YAML and TOML, not {l}"));
                        }
                    }
                    Matcher::Target(t)
                }
                Err(e) => {
                    fail(e);
                    continue;
                }
            },
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
            // A graph rule's fix compiled with its query.
            Some(_) if matches!(matcher, Matcher::Graph(_)) => None,
            Some(_) if is_target => None,
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
            Matcher::Graph(_) | Matcher::Target(_) | Matcher::Unsupported(_) => false,
        } || matches!(&fix, Some(FixSpec::Steps(s)) if s.iter().any(|p| matches!(p, codelaw_fix::Primitive::RenameSymbolLocal { .. })));
        out.push(CompiledRule { loaded: loaded.clone(), matcher, fix, data_fix, needs_semantic, files, exclude });
    }
    // One message per problem, not one per language that shares a grammar.
    errors.dedup_by(|a, b| a.message == b.message && a.file == b.file);
    if errors.is_empty() { Ok((out, warnings)) } else { Err(errors) }
}
