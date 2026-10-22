//! Output for `rules --for`, `config --print`, `match --explain`,
//! `fix --trace` and `--timings`: text for people, JSON for the APIs and
//! MCP (SR-CLI-MCP decisions 4 and 5).

use crate::SCHEMA_VERSION;
use codelaw_config::Traced;
use codelaw_engine::{FileTrace, MatchReport, RuleCard, TimingReport};
use codelaw_fix::TraceEvent;
use serde_json::{Value, json};

// ── rules ────────────────────────────────────────────────────────────────

/// `rules [--for <file>]`: one line per rule.
pub fn rule_cards_text(cards: &[RuleCard]) -> String {
    let mut out = String::new();
    for c in cards {
        let preview = if c.preview { "  (preview)" } else { "" };
        out.push_str(&format!("{:<40} {:<6} {:<8} {}{preview}\n", c.id, c.severity, c.fix.name(), c.source));
    }
    out
}

/// `rules [--for <file>] --format json`, and `rulesFor` / `rules_for`.
pub fn rule_cards_json(cards: &[RuleCard], file: Option<&str>) -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "codelaw_version": codelaw_core::VERSION,
        "file": file,
        "rules": cards.iter().map(|c| json!({
            "id": c.id,
            "title": c.title,
            "severity": c.severity,
            "fix": c.fix.name(),
            "source": c.source,
            "preview": c.preview,
        })).collect::<Vec<_>>(),
    })
}

// ── config ───────────────────────────────────────────────────────────────

fn compact(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// `config --print`: `key = value  # source`, aligned.
pub fn config_text(values: &[Traced], files: &[String]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# config files: {}\n", if files.is_empty() { "none (every value is a default)".to_owned() } else { files.join(", ") }));
    let lines: Vec<(String, &str)> = values.iter().map(|t| (format!("{} = {}", t.key, compact(&t.value)), t.source.as_str())).collect();
    let width = lines.iter().map(|(l, _)| l.chars().count()).max().unwrap_or(0).min(72);
    for (l, s) in lines {
        let pad = width.saturating_sub(l.chars().count());
        out.push_str(&format!("{l}{}  # {s}\n", " ".repeat(pad)));
    }
    out
}

pub fn config_json(values: &[Traced], files: &[String], file: Option<&str>) -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "codelaw_version": codelaw_core::VERSION,
        "file": file,
        "config_files": files,
        "values": values.iter().map(|t| json!({ "key": t.key, "value": t.value, "source": t.source })).collect::<Vec<_>>(),
    })
}

// ── match --explain ─────────────────────────────────────────────────────

fn mark(holds: Option<bool>) -> &'static str {
    match holds {
        Some(true) => "✔",
        Some(false) => "✖",
        None => "?",
    }
}

/// `match <id> <file>`: the matches; with `explain`, every candidate node
/// and each clause.
pub fn match_text(report: &MatchReport, explain: bool) -> String {
    let mut out = String::new();
    let mut shown = 0;
    for (start, end, n) in &report.nodes {
        if !explain && !n.matched {
            continue;
        }
        shown += 1;
        let verdict = if n.matched { "match" } else { "no match" };
        out.push_str(&format!("{}:{}:{}-{}:{}  {}  {verdict}\n", report.file, start.line, start.column, end.line, end.column, n.kind));
        if explain {
            for c in &n.clauses {
                let err = c.error.as_deref().map(|e| format!("  ({e})")).unwrap_or_default();
                out.push_str(&format!("  {} {}{err}\n", mark(c.holds), c.label));
            }
            if let Some(ok) = n.constraints {
                out.push_str(&format!("  {} constraints\n", mark(Some(ok))));
            }
            for p in &n.predicates {
                let err = p.error.as_deref().map(|e| format!("  ({e})")).unwrap_or_default();
                out.push_str(&format!("  {} where {}{err}\n", mark(p.holds), p.label));
            }
        }
        for (k, v) in &n.captures {
            let v: String = v.chars().take(60).collect();
            out.push_str(&format!("  {k} = {v:?}\n"));
        }
    }
    let matched = report.nodes.iter().filter(|(_, _, n)| n.matched).count();
    if explain {
        out.push_str(&format!("{} on {}: {matched} matches among {shown} candidate nodes\n", report.rule, report.file));
    } else {
        out.push_str(&format!("{} on {}: {matched} matches\n", report.rule, report.file));
    }
    out
}

pub fn match_json(report: &MatchReport) -> Value {
    let clause = |c: &codelaw_match::ClauseResult| json!({ "clause": c.label, "holds": c.holds, "error": c.error });
    json!({
        "schema_version": SCHEMA_VERSION,
        "codelaw_version": codelaw_core::VERSION,
        "rule": report.rule,
        "file": report.file,
        "language": report.language.name(),
        "nodes": report.nodes.iter().map(|(s, e, n)| json!({
            "kind": n.kind,
            "range": {
                "start": { "line": s.line, "column": s.column, "offset": n.start },
                "end": { "line": e.line, "column": e.column, "offset": n.end },
            },
            "matched": n.matched,
            "clauses": n.clauses.iter().map(clause).collect::<Vec<_>>(),
            "constraints": n.constraints,
            "predicates": n.predicates.iter().map(clause).collect::<Vec<_>>(),
            "captures": n.captures,
        })).collect::<Vec<_>>(),
    })
}

// ── fix --trace ─────────────────────────────────────────────────────────

fn event_json(e: &TraceEvent) -> Value {
    match e {
        TraceEvent::Offered { rule, edits } => json!({ "event": "offered", "rule": rule, "edits": edits }),
        TraceEvent::Duplicate { rule } => json!({ "event": "duplicate", "rule": rule }),
        TraceEvent::Deferred { rule, behind } => json!({ "event": "deferred", "rule": rule, "behind": behind }),
        TraceEvent::Applied { rule, edits } => json!({ "event": "applied", "rule": rule, "edits": edits }),
        TraceEvent::Refused { rule, why } => json!({ "event": "refused", "rule": rule, "why": why.note() }),
    }
}

fn why_text(why: codelaw_fix::Failure) -> &'static str {
    match why {
        codelaw_fix::Failure::VerifyFailed => "it would add a syntax error",
        codelaw_fix::Failure::Conflict => "the fixes undo each other (the file came back to an earlier state)",
        codelaw_fix::Failure::NoConvergence => "still offered after the last pass",
    }
}

/// `fix --trace`: each pass of each file.
pub fn trace_text(traces: &[FileTrace]) -> String {
    let mut out = String::new();
    for t in traces {
        out.push_str(&format!("trace {}\n", t.file));
        for (i, pass) in t.trace.passes.iter().enumerate() {
            out.push_str(&format!("  pass {}\n", i + 1));
            for e in pass {
                let line = match e {
                    TraceEvent::Offered { rule, edits } => format!("offered   {rule} ({edits} edits)"),
                    TraceEvent::Duplicate { rule } => format!("duplicate {rule}: the same edits as a fix already taken"),
                    TraceEvent::Deferred { rule, behind } => format!("deferred  {rule}: waits for the next pass, behind {behind}"),
                    TraceEvent::Applied { rule, edits } => format!("applied   {rule} ({edits} edits), verified"),
                    TraceEvent::Refused { rule, why } => format!("refused   {rule}: {}; reported as manual", why_text(*why)),
                };
                out.push_str(&format!("    {line}\n"));
            }
        }
        let end = match t.trace.end {
            codelaw_fix::LoopEnd::Fixpoint => "fixpoint: nothing more to fix",
            codelaw_fix::LoopEnd::Conflict => "stopped: the fixes conflict",
            codelaw_fix::LoopEnd::MaxPasses => "stopped: did not converge",
        };
        out.push_str(&format!("  {end}{}\n", if t.changed { "" } else { " (the file is unchanged)" }));
    }
    out
}

pub fn trace_json(traces: &[FileTrace]) -> Value {
    traces
        .iter()
        .map(|t| {
            json!({
                "file": t.file,
                "changed": t.changed,
                "end": t.trace.end.name(),
                "passes": t.trace.passes.iter().map(|p| p.iter().map(event_json).collect::<Vec<_>>()).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>()
        .into()
}

// ── --timings ───────────────────────────────────────────────────────────

fn ms(nanos: u64) -> String {
    format!("{:.1} ms", nanos as f64 / 1e6)
}

/// `--timings`: the phases, then the 20 slowest rules. Phase and rule times
/// are summed over the worker threads.
pub fn timings_text(t: &TimingReport) -> String {
    let mut out = format!("timings: {} wall · {} threads · phase and rule times summed over threads\n", ms(t.wall_nanos), t.threads);
    for p in &t.phases {
        out.push_str(&format!("  {:<10} {:>10}  ×{}\n", p.phase, ms(p.nanos), p.count));
    }
    out.push_str(&format!("  {:<10} {} hits · {} misses\n", "cache", t.cache_hits, t.cache_misses));
    if !t.rules.is_empty() {
        let total: u64 = t.rules.iter().map(|r| r.nanos).sum::<u64>().max(1);
        out.push_str("  slowest rules\n");
        for r in t.rules.iter().take(20) {
            out.push_str(&format!("    {:<40} {:>10}  {:>5.1}%  ×{}\n", r.rule, ms(r.nanos), r.nanos as f64 * 100.0 / total as f64, r.calls));
        }
    }
    out
}

pub fn timings_json(t: &TimingReport) -> Value {
    json!({
        "wall_ms": t.wall_nanos as f64 / 1e6,
        "threads": t.threads,
        "phases": t.phases.iter().map(|p| json!({ "phase": p.phase, "ms": p.nanos as f64 / 1e6, "count": p.count })).collect::<Vec<_>>(),
        "cache": { "hits": t.cache_hits, "misses": t.cache_misses },
        "rules": t.rules.iter().map(|r| json!({ "rule": r.rule, "ms": r.nanos as f64 / 1e6, "calls": r.calls })).collect::<Vec<_>>(),
    })
}
