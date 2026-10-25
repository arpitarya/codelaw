//! The ratchet (SR-BASELINE decision 5): between two revisions a baseline
//! may only shrink. `codelaw baseline check --since <ref>` fails on any
//! count that rose, entry that was added, deadline that moved later (or
//! was removed) and touch policy that got less strict; approval is the
//! platform's own `CODEOWNERS` review of `.codelaw/baseline/`.

use crate::file::{Baselines, Entry};
use std::collections::BTreeMap;

fn label(rule: &str, e: &Entry) -> String {
    match &e.symbol {
        Some(s) => format!("{rule}: {} ({s})", e.file),
        None => format!("{rule}: {}", e.file),
    }
}

/// Every way `after` is looser than `before`, sorted.
pub fn loosened(before: &Baselines, after: &Baselines) -> Vec<String> {
    let mut out = Vec::new();
    for (rule, b) in &after.rules {
        let old: BTreeMap<(&str, Option<&str>), &Entry> = before.rules.get(rule).map(|o| o.entries.iter().map(|e| (e.key(), e)).collect()).unwrap_or_default();
        for e in &b.entries {
            let Some(o) = old.get(&e.key()) else {
                out.push(format!("{}: entry added (count {})", label(rule, e), e.count));
                continue;
            };
            if e.count > o.count {
                out.push(format!("{}: count rose from {} to {}", label(rule, e), o.count, e.count));
            }
            // `expires` is compared, so an `until` rewritten as the
            // `expires` the day after is no change (decision 9).
            match (o.expires, e.expires) {
                (Some(a), Some(z)) if z > a => out.push(format!("{}: deadline extended, expires moved from {a} to {z}", label(rule, e))),
                (Some(a), None) => out.push(format!("{}: deadline removed (expires {a})", label(rule, e))),
                _ => {}
            }
            let strict = |x: Option<crate::OnTouch>| x.unwrap_or_default().strictness();
            if strict(e.on_touch) < strict(o.on_touch) {
                out.push(format!(
                    "{}: on_touch loosened from {} to {}",
                    label(rule, e),
                    o.on_touch.unwrap_or_default().name(),
                    e.on_touch.unwrap_or_default().name()
                ));
            }
        }
    }
    out.sort();
    out
}

/// One CODEOWNERS pattern as globs over a repository-relative path, with
/// gitignore's anchoring: a leading `/` or an inner `/` anchors it to the
/// root; a trailing `/` matches a folder's contents.
fn globs(pattern: &str) -> Vec<String> {
    let core = pattern.trim_end_matches('/');
    let anchored = pattern.starts_with('/') || core.trim_start_matches('/').contains('/');
    let core = core.trim_start_matches('/');
    if core.is_empty() {
        return vec!["**".into()];
    }
    let base = if anchored { core.to_owned() } else { format!("**/{core}") };
    vec![base.clone(), format!("{base}/**")]
}

/// Whether a CODEOWNERS file assigns owners to `path`: the last matching
/// line wins, and a line with no owners takes them away.
pub fn codeowners_covers(text: &str, path: &str) -> bool {
    let mut owned = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(pattern) = parts.next() else { continue };
        let owners = parts.take_while(|p| !p.starts_with('#')).count();
        let hit = globs(pattern).iter().any(|g| {
            globset::GlobBuilder::new(g).literal_separator(true).build().map(|g| g.compile_matcher().is_match(path)).unwrap_or(false)
        });
        if hit {
            owned = owners > 0;
        }
    }
    owned
}

/// Where GitHub, GitLab and Gitea look for CODEOWNERS, in their order.
pub const CODEOWNERS_PATHS: &[&str] = &[".github/CODEOWNERS", "CODEOWNERS", "docs/CODEOWNERS", ".gitlab/CODEOWNERS"];
