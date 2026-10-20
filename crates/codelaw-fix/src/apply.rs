//! The apply loop (SR-ARCHITECTURE decision 5) and the verifier
//! (L4 decision 5): gather fixes, drop overlaps, apply, re-parse, repeat to
//! a fixpoint — at most ten passes — and never let a fix add a syntax
//! error.

use codelaw_cache::hash_bytes;
use codelaw_core::{Language, TextEdit};
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// The most passes the loop runs.
pub const MAX_PASSES: usize = 10;

/// One fix on offer: a rule's edits for one diagnostic (or one group).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub rule: String,
    pub edits: Vec<TextEdit>,
}

impl Candidate {
    /// A key that survives the text shifting around it: the rule, and what
    /// each edit removes and inserts.
    pub fn key(&self, text: &str) -> String {
        let mut k = self.rule.clone();
        for e in &self.edits {
            k.push('\0');
            k.push_str(text.get(e.start..e.end).unwrap_or_default());
            k.push('\u{1}');
            k.push_str(&e.text);
        }
        k
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Failure {
    /// The fix would add a syntax error.
    VerifyFailed,
    /// Fixes undo each other: the file came back to an earlier state.
    Conflict,
    /// Still offering fixes after the last pass.
    NoConvergence,
}

impl Failure {
    pub fn note(self) -> &'static str {
        match self {
            Failure::VerifyFailed => "fix-verify-failed",
            Failure::Conflict => "fix-conflict",
            Failure::NoConvergence => "fix-did-not-converge",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LoopOutcome {
    pub text: String,
    pub passes: usize,
    /// Fixes applied, per rule.
    pub applied: BTreeMap<String, usize>,
    /// Rules whose fixes were refused, with why. Their diagnostics are
    /// reported as `manual`.
    pub failed: BTreeSet<(String, Failure)>,
    /// Keys of refused candidates, for the caller to mark.
    pub refused: BTreeSet<String>,
}

/// Whether two edits touch: overlapping ranges, or an insertion at or
/// inside the other's range. Touching edits from two fixes are not applied
/// in the same pass.
fn touches(a: &TextEdit, b: &TextEdit) -> bool {
    (a.start < b.end && b.start < a.end)
        || (a.start == a.end && b.start <= a.start && a.start <= b.end)
        || (b.start == b.end && a.start <= b.start && b.start <= a.end)
}

pub fn overlaps(a: &[TextEdit], b: &[TextEdit]) -> bool {
    a.iter().any(|x| b.iter().any(|y| touches(x, y)))
}

/// Apply edits sorted by position. Edits of one fix may touch end to start
/// but not overlap.
pub fn apply_edits(text: &str, edits: &[TextEdit]) -> Result<String, String> {
    let mut sorted: Vec<&TextEdit> = edits.iter().collect();
    sorted.sort_by_key(|e| (e.start, e.end));
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for e in sorted {
        if e.start < cursor || e.end < e.start || e.end > text.len() {
            return Err(format!("edits overlap or fall outside the file at byte {}", e.start));
        }
        if !text.is_char_boundary(e.start) || !text.is_char_boundary(e.end) {
            return Err(format!("an edit splits a character at byte {}", e.start));
        }
        out.push_str(&text[cursor..e.start]);
        out.push_str(&e.text);
        cursor = e.end;
    }
    out.push_str(&text[cursor..]);
    Ok(out)
}

fn errors(language: Language, text: &str) -> usize {
    codelaw_lang::syntax_error_count(&codelaw_lang::parse(language, text))
}

/// Run fixes to a fixpoint. `collect` is called with the current text and
/// returns the candidates on offer, in a deterministic order; candidates
/// whose key is in `refused` must not be offered again. Syntax errors are
/// counted in the host grammar only; the engine counts embedded regions
/// too, through [`fix_loop_with`].
pub fn fix_loop(language: Language, original: &str, collect: impl FnMut(&str, &BTreeSet<String>) -> Vec<Candidate>) -> LoopOutcome {
    fix_loop_with(original, |t| errors(language, t), collect)
}

/// [`fix_loop`] with the syntax-error count supplied: a fix is kept only if
/// `errors` of the result is no more than `errors` of the original. The
/// loop ends only when the text offers no fix, which is the idempotence
/// check of SR-ARCHITECTURE decision 5: one more run changes nothing.
pub fn fix_loop_with(
    original: &str,
    errors: impl Fn(&str) -> usize,
    mut collect: impl FnMut(&str, &BTreeSet<String>) -> Vec<Candidate>,
) -> LoopOutcome {
    let base_errors = errors(original);
    let mut out = LoopOutcome { text: original.to_owned(), ..Default::default() };
    let mut seen: HashSet<[u8; 32]> = HashSet::new();
    seen.insert(hash_bytes(original.as_bytes()).0);

    while out.passes < MAX_PASSES {
        let candidates: Vec<Candidate> =
            collect(&out.text, &out.refused).into_iter().filter(|c| !out.refused.contains(&c.key(&out.text))).collect();
        if candidates.is_empty() {
            return out;
        }
        out.passes += 1;

        // One copy of identical fixes (a group fix is offered by every
        // diagnostic in it), then greedily the ones that don't touch.
        let mut selected: Vec<&Candidate> = Vec::new();
        for c in &candidates {
            if selected.iter().any(|s| s.edits == c.edits) {
                continue;
            }
            if c.edits.is_empty() || selected.iter().any(|s| overlaps(&s.edits, &c.edits)) {
                continue;
            }
            selected.push(c);
        }
        let all: Vec<TextEdit> = selected.iter().flat_map(|c| c.edits.iter().cloned()).collect();
        let next = match apply_edits(&out.text, &all) {
            Ok(t) if errors(&t) <= base_errors => Some(t),
            _ => None,
        };
        let next = match next {
            Some(t) => {
                for c in &selected {
                    *out.applied.entry(c.rule.clone()).or_default() += 1;
                }
                t
            }
            None => {
                // Find the culprits one at a time; keep the first good fix.
                let mut kept = None;
                for c in &selected {
                    match apply_edits(&out.text, &c.edits) {
                        Ok(t) if errors(&t) <= base_errors => {
                            *out.applied.entry(c.rule.clone()).or_default() += 1;
                            kept = Some(t);
                            break;
                        }
                        _ => {
                            out.refused.insert(c.key(&out.text));
                            out.failed.insert((c.rule.clone(), Failure::VerifyFailed));
                        }
                    }
                }
                match kept {
                    Some(t) => t,
                    None => continue,
                }
            }
        };
        if !seen.insert(hash_bytes(next.as_bytes()).0) {
            for c in &selected {
                out.refused.insert(c.key(&out.text));
                out.failed.insert((c.rule.clone(), Failure::Conflict));
            }
            return out;
        }
        out.text = next;
    }
    for c in collect(&out.text, &out.refused) {
        let key = c.key(&out.text);
        if !out.refused.contains(&key) {
            out.refused.insert(key);
            out.failed.insert((c.rule.clone(), Failure::NoConvergence));
        }
    }
    out
}
