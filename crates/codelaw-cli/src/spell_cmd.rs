//! `codelaw spell learn` (SR-SPELL decision 7, W-112): run
//! `spell/unknown-word` over the project, whatever the config turns on, and
//! add each word it reports to the words file — lower case, sorted, after
//! the lines already there, which are never changed. A word
//! `spell/suspicious-word` would report (a known typo, or one edit from a
//! common word) is held back and listed for review instead.

use crate::fail;
use clap::Subcommand;
use codelaw_config::RuleSetting;
use codelaw_core::template;
use codelaw_engine::{EngineOptions, Project};
use codelaw_native::spell::{Suspicion, suspicion};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Subcommand)]
pub enum SpellAction {
    /// Add every word no dictionary knows to the words file
    /// (`.codelaw/spell/words.txt`); list likely typos for review instead.
    Learn,
}

const RULE: &str = "spell/unknown-word";

/// What one `learn` did.
pub struct Learned {
    /// The words file, as the report names it.
    pub file: String,
    /// The words added, lower case and sorted.
    pub added: Vec<String>,
    /// Each word held back, sorted, with why.
    pub review: Vec<(String, String)>,
}

impl Learned {
    /// The review list first, then what was added (the hazard: a large
    /// repo adds many words, and the likely typos must not scroll away).
    pub fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !self.review.is_empty() {
            out.push(format!("{} likely typos were not added; fix them, or add them to {} by hand if they are right:", self.review.len(), self.file));
            out.extend(self.review.iter().map(|(w, why)| format!("  {w}: {why}")));
        }
        match self.added.len() {
            0 => out.push(format!("no new words; {} is up to date", self.file)),
            n => {
                out.push(format!("added {n} word{} to {}:", if n == 1 { "" } else { "s" }, self.file));
                out.extend(self.added.iter().map(|w| format!("  {w}")));
            }
        }
        out
    }
}

pub fn spell(action: SpellAction) -> ExitCode {
    match action {
        SpellAction::Learn => match learn(Path::new(".")) {
            Ok(l) => {
                l.lines().iter().for_each(|l| println!("{l}"));
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
    }
}

/// Learn the words of the project whose config governs `start`.
pub fn learn(start: &Path) -> Result<Learned, String> {
    let mut config = codelaw_config::find_config(start).map_err(|e| e.to_string())?;
    let Some(root) = config.files.first_mut() else {
        return Err("no codelaw config here; run `codelaw init` first".into());
    };
    // `spell/unknown-word` runs whatever the config says: the pack is
    // loaded, its preview rules allowed, the rule on. Only it runs.
    root.raw.preview = Some(true);
    if !root.raw.extends.iter().any(|e| e.starts_with("codelaw:spell/")) {
        root.raw.extends.push("codelaw:spell/recommended".into());
    }
    root.raw.rules.insert(RULE.into(), RuleSetting::Severity("warn".into()));
    let file = config.spell_words_file();
    let options = EngineOptions { rules: Some(vec![RULE.into()]), no_cache: true, ..EngineOptions::from_env() };
    let mut project = Project::from_config(config, options).map_err(|e| format!("{e}\n`spell learn` needs the spell pack: install @codelaw/rules-spell or codelaw-rules-spell"))?;
    let report = project.check(&[]).map_err(|e| e.to_string())?;
    let data = project.rules.get(RULE).map(|r| r.data.clone()).unwrap_or_default();

    let mut added = BTreeSet::new();
    let mut review = BTreeMap::new();
    for d in report.diagnostics.iter().filter(|d| d.rule == RULE) {
        // The rule's message starts with the word as written.
        let Some(word) = d.message.strip_prefix('`').and_then(|m| m.split('`').next()) else { continue };
        let word = word.to_ascii_lowercase();
        match suspicion(&word, &data) {
            Some(Suspicion::Typo(fixes)) => {
                review.insert(word, format!("a known typo of {}", fixes.split('|').map(|f| format!("`{f}`")).collect::<Vec<_>>().join(" or ")));
            }
            Some(Suspicion::Near(near)) => {
                review.insert(word, format!("one edit from the common word `{near}`"));
            }
            None => {
                added.insert(word);
            }
        }
    }
    let shown = codelaw_core::normalize_path(file.strip_prefix(&project.config.root_dir).unwrap_or(&file));
    let added: Vec<String> = added.into_iter().collect();
    write(&file, &added).map_err(|e| format!("{shown}: {e}"))?;
    Ok(Learned { file: shown, added, review: review.into_iter().collect() })
}

/// Append `words` to the words file, keeping every line in it; a missing
/// file starts from its template (L10).
fn write(file: &PathBuf, words: &[String]) -> std::io::Result<()> {
    let mut text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => template::text("init/.codelaw/spell/words.txt").to_owned(),
        Err(e) => return Err(e),
    };
    if words.is_empty() && file.is_file() {
        return Ok(());
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    for w in words {
        text.push_str(w);
        text.push('\n');
    }
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(file, text)
}
