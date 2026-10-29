//! What a diagnostic tells the editor beyond its message (W-137,
//! SR-EDITOR decision 8): the one `data` shape every published diagnostic
//! carries, and the hover card built from it.
//!
//! Nothing here judges code. The verdict — rule, severity, message, fix —
//! is the engine's; this module only *names* it for the editor: which kind
//! of finding it is (a vulnerable dependency, an expired TODO, a focused
//! test…), which of the three views it belongs to, and the few facts a badge
//! or a card shows (safe version, latest version, due date, ticket). The
//! facts come from the rule's own `data` first and, for a rule that has
//! none yet, from the words of its message; a rule this module has never
//! heard of is a plain `code` finding, so a new rule never breaks the
//! editor.
//!
//! The shape (every key but `fix`, `source`, `rule`, `kind`, `view`,
//! `title` and `badge` is optional):
//!
//! ```json
//! { "fix": "auto", "source": "deps@0.5.0", "rule": "deps/vulnerable",
//!   "kind": "vulnerable", "view": "manifest", "grade": "high",
//!   "title": "lodash 4.17.15 is vulnerable", "badge": "HIGH · 3 advisories → 4.17.21",
//!   "package": "lodash", "version": "4.17.15", "safe": "4.17.21", "latest": "4.17.21",
//!   "advisories": ["GHSA-35jh-r3h4-6jhm"], "more": 2,
//!   "due": "2026-10-01", "days": -29, "ticket": "PAY-388", "tag": "TODO", "flag": "new-checkout",
//!   "baseline": true, "fixable": true, "facts": { "…": "the rule's own data, verbatim" } }
//! ```

use codelaw_core::{Diagnostic, FixClass, Language, Severity};
use codelaw_rules::RuleFile;
use serde_json::{Map, Value, json};

/// The kinds a finding is shown as. `code` is everything else.
pub const KINDS: &[&str] = &[
    "vulnerable",
    "deprecated",
    "licence",
    "outdated",
    "runtime",
    "scan",
    "expired",
    "due",
    "ticket",
    "flag",
    "flag-expired",
    "focused",
    "skipped",
    "complexity",
    "ownership",
    "security",
    "code",
];

/// The files whose findings follow `codelaw.view.manifests`.
const MANIFESTS: &[&str] = &[
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "pyproject.toml",
    "uv.lock",
    "poetry.lock",
    "requirements.txt",
    ".nvmrc",
    ".node-version",
    ".python-version",
    ".tool-versions",
];

/// Whether a project-relative path is a manifest, a lockfile or a runtime
/// pin (`codelaw.view.manifests`).
pub fn is_manifest(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    MANIFESTS.contains(&name)
}

/// The kind of a finding, from its rule id and, where one rule reports
/// several kinds, its message. Tolerant: an unknown rule is `code` (or
/// `security` in a security pack).
pub fn kind_of(rule: &str, message: &str) -> &'static str {
    let name = rule.rsplit('/').next().unwrap_or(rule);
    let m = message.to_ascii_lowercase();
    let has = |s: &str| name.contains(s);
    if has("vulnerab") {
        "vulnerable"
    } else if has("incomplete-scan") || has("stale-snapshot") || has("stale-deps-snapshot") {
        "scan"
    } else if has("licen") {
        "licence"
    } else if has("outdated") {
        match grade_suffix(message) {
            Some("deprecated" | "yanked") => "deprecated",
            _ => "outdated",
        }
    } else if has("deprecated-dep") {
        "deprecated"
    } else if has("engines") || has("runtime") || has("python-version") || has("node-version") {
        "runtime"
    } else if has("flag") {
        if m.contains("expired") { "flag-expired" } else { "flag" }
    } else if has("todo") || has("fixme") || (has("marker") && !has("flag")) {
        if m.contains(" expired") {
            "expired"
        } else if m.contains("expires in") || m.contains("due in") {
            "due"
        } else {
            "ticket"
        }
    } else if has("focused") || has("no-only") || has("exclusive") {
        "focused"
    } else if has("skip") {
        "skipped"
    } else if has("complex") || has("max-depth") || has("cognitive") {
        "complexity"
    } else if has("codeowners") || has("owner") {
        "ownership"
    } else if rule.starts_with("sec-") || rule.starts_with("security") {
        "security"
    } else {
        "code"
    }
}

/// Which of the three view settings a finding follows: `manifest` for any
/// finding in a manifest, `marker` for TODO/FIXME/SKIP comments and skipped
/// tests, `code` for the rest.
pub fn view_of(rel: &str, kind: &str) -> &'static str {
    if is_manifest(rel) {
        "manifest"
    } else if matches!(kind, "expired" | "due" | "ticket" | "skipped") {
        "marker"
    } else {
        "code"
    }
}

/// The grade a message ends with, `… (major)`.
fn grade_suffix(message: &str) -> Option<&str> {
    let open = message.rfind(" (")?;
    message[open + 2..].strip_suffix(')').filter(|g| !g.contains(' '))
}

/// The first `` `word` `` of a message, without the backticks and anything
/// from a space or `(` on.
fn first_quoted(message: &str) -> Option<&str> {
    let start = message.find('`')? + 1;
    let end = start + message[start..].find('`')?;
    let q = &message[start..end];
    Some(q.split([' ', '(']).next().unwrap_or(q)).filter(|s| !s.is_empty())
}

/// The word after `marker` in a message: `latest 4.17.21` → `4.17.21`.
fn word_after<'m>(message: &'m str, marker: &str) -> Option<&'m str> {
    let rest = &message[message.find(marker)? + marker.len()..];
    let w = rest.split([' ', ',', ';', ')', '(']).next()?.trim_end_matches(['.', ':']);
    (!w.is_empty()).then_some(w)
}

/// The first `YYYY-MM-DD` in a text.
fn first_date(text: &str) -> Option<String> {
    let b = text.as_bytes();
    (0..b.len().saturating_sub(9)).find_map(|i| {
        let s = text.get(i..i + 10)?;
        codelaw_engine::Date::parse(s).is_ok().then(|| s.to_owned())
    })
}

/// The first ticket-shaped word (`PAY-412`) in a text.
fn first_ticket(text: &str) -> Option<String> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')).find_map(|w| {
        let (head, num) = w.split_once('-')?;
        let ok = head.len() >= 2
            && head.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && head.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            && !num.is_empty()
            && num.chars().all(|c| c.is_ascii_digit());
        ok.then(|| w.to_owned())
    })
}

/// A number written just before `unit` (`29 days ago` → 29).
fn number_before(text: &str, unit: &str) -> Option<i64> {
    let at = text.find(unit)?;
    let head = text[..at].trim_end();
    let digits: String = head.chars().rev().take_while(char::is_ascii_digit).collect::<Vec<_>>().into_iter().rev().collect();
    digits.parse().ok()
}

fn plural(n: i64, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// What a diagnostic's facts are, for its `data`.
pub struct Facts<'a> {
    pub diagnostic: &'a Diagnostic,
    /// The project-relative path of the file it is shown in.
    pub rel: &'a str,
    /// Whether the project's baseline holds it.
    pub baseline: bool,
    /// The as-of date, in days since 1970 (a due date's distance).
    pub as_of: Option<i64>,
    pub rule: Option<&'a RuleFile>,
}

/// The `data` of one diagnostic: the shape in the module doc.
pub fn data(f: &Facts<'_>) -> Value {
    let d = f.diagnostic;
    let kind = kind_of(&d.rule, &d.message);
    let mut o = Map::new();
    o.insert("fix".into(), json!(d.fix.name()));
    o.insert("source".into(), json!(d.source));
    o.insert("rule".into(), json!(d.rule));
    o.insert("kind".into(), json!(kind));
    o.insert("view".into(), json!(view_of(f.rel, kind)));
    if matches!(d.fix, FixClass::Auto | FixClass::Suggest) && !d.edits.is_empty() {
        o.insert("fixable".into(), json!(true));
    }
    if f.baseline {
        o.insert("baseline".into(), json!(true));
    }
    let facts = d.data.as_ref().and_then(Value::as_object);
    let fact = |k: &str| facts.and_then(|m| m.get(k)).and_then(Value::as_str).map(str::to_owned);
    let msg = d.message.as_str();
    match kind {
        "vulnerable" => {
            // `deps/vulnerable`: `` `lodash` 4.17.15 is vulnerable — GHSA-… (high, CVE-…) and 2 more; upgrade to 4.17.21 ``.
            let pkg = fact("package").or_else(|| first_quoted(msg).map(str::to_owned));
            let version = fact("version").or_else(|| msg.split_once("` ").and_then(|(_, r)| r.split(' ').next()).map(str::to_owned));
            let advisory = fact("advisory").or_else(|| msg.split_once(" — ").and_then(|(_, r)| r.split([' ', ';']).next()).map(str::to_owned));
            let grade = msg.split_once(" — ").and_then(|(_, r)| r.split_once(" (")).and_then(|(_, r)| r.split([',', ')', ' ']).next()).map(str::to_ascii_lowercase);
            let safe = fact("target").or_else(|| word_after(msg, "upgrade to ").map(str::to_owned));
            let more = number_before(msg, "more").unwrap_or(0);
            put(&mut o, "package", pkg);
            put(&mut o, "version", version);
            put(&mut o, "grade", grade);
            put(&mut o, "safe", safe);
            if let Some(a) = advisory {
                o.insert("advisories".into(), json!([a]));
            }
            if more > 0 {
                o.insert("more".into(), json!(more));
            }
        }
        "outdated" | "deprecated" => {
            put(&mut o, "package", fact("package").or_else(|| first_quoted(msg).map(str::to_owned)));
            put(&mut o, "grade", grade_suffix(msg).map(str::to_owned));
            put(&mut o, "version", fact("version").or_else(|| word_after(msg, "installed ").map(str::to_owned)));
            put(&mut o, "latest", fact("latest").or_else(|| word_after(msg, "latest ").map(str::to_owned)));
            if grade_suffix(msg) == Some("lockfile") {
                put(&mut o, "latest", word_after(msg, "allows ").map(str::to_owned));
                put(&mut o, "version", word_after(msg, "lockfile has ").map(str::to_owned));
            }
        }
        "expired" | "due" | "ticket" | "flag" | "flag-expired" | "skipped" => {
            let due = fact("due").or_else(|| fact("expires")).or_else(|| first_date(msg));
            let days = match (&due, f.as_of) {
                (Some(s), Some(today)) => codelaw_engine::Date::parse(s).ok().map(|d| d.days() - today),
                _ => number_before(msg, "day").map(|n| if msg.contains(" ago") || kind.ends_with("expired") || kind == "expired" { -n } else { n }),
            };
            put(&mut o, "due", due);
            if let Some(n) = days {
                o.insert("days".into(), json!(n));
            }
            put(&mut o, "ticket", fact("ticket").or_else(|| first_ticket(msg)));
            if kind.starts_with("flag") {
                put(&mut o, "flag", fact("flag").or_else(|| first_quoted(msg).map(str::to_owned)));
                put(&mut o, "owner", fact("owner").or_else(|| msg.split_whitespace().find(|w| w.starts_with('@')).map(|w| w.trim_end_matches([',', ';', ')', '.']).to_owned())));
            } else {
                let tag = first_quoted(msg).map(|q| q.chars().take_while(|c| c.is_ascii_uppercase()).collect::<String>()).filter(|t| !t.is_empty());
                put(&mut o, "tag", tag);
            }
        }
        _ => {}
    }
    o.insert("title".into(), json!(title(kind, &o, d, f.rule)));
    o.insert("badge".into(), json!(badge(kind, &o, d)));
    if let Some(v) = &d.data {
        o.insert("facts".into(), v.clone());
    }
    Value::Object(o)
}

fn put(o: &mut Map<String, Value>, key: &str, v: Option<String>) {
    if let Some(v) = v.filter(|v| !v.is_empty()) {
        o.insert(key.into(), json!(v));
    }
}

fn s<'a>(o: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    o.get(key).and_then(Value::as_str)
}

fn days(o: &Map<String, Value>) -> Option<i64> {
    o.get("days").and_then(Value::as_i64)
}

/// The card's title: plain words, the facts first.
fn title(kind: &str, o: &Map<String, Value>, d: &Diagnostic, rule: Option<&RuleFile>) -> String {
    let pkg = s(o, "package").unwrap_or("This dependency");
    let ver = s(o, "version").map(|v| format!(" {v}")).unwrap_or_default();
    let tag = s(o, "tag").unwrap_or("TODO");
    let flag = s(o, "flag").unwrap_or("");
    let fallback = || {
        rule.map(|r| r.title.clone())
            .or_else(|| codelaw_rules::engine_rule(&d.rule).map(|e| e.title.to_owned()))
            .unwrap_or_else(|| d.rule.clone())
    };
    match (kind, days(o)) {
        ("vulnerable", _) => format!("{pkg}{ver} is vulnerable"),
        ("deprecated", _) if s(o, "grade") == Some("yanked") => format!("{pkg}{ver} was yanked"),
        ("deprecated", _) => format!("{pkg}{ver} is deprecated"),
        ("outdated", _) => match s(o, "latest") {
            Some(l) => format!("{pkg} {l} is out"),
            None => format!("{pkg} is out of date"),
        },
        ("expired", Some(n)) => format!("{tag} expired {}", ago(-n)),
        ("expired", None) => format!("{tag} past its date"),
        ("due", Some(n)) => format!("{tag} due in {}", plural(n, "day", "days")),
        ("ticket", _) => format!("{tag} needs a ticket"),
        ("flag", Some(n)) => format!("Flag {flag} expires in {}", plural(n, "day", "days")),
        ("flag-expired", Some(n)) => format!("Flag {flag} expired {}", ago(-n)),
        ("flag" | "flag-expired", _) if !flag.is_empty() => format!("Flag {flag}"),
        ("focused", _) => "Only this test will run".into(),
        ("skipped", _) => "Skipped test needs a reason and a date".into(),
        ("scan", _) => "The dependency scan is incomplete".into(),
        _ => fallback(),
    }
}

fn ago(n: i64) -> String {
    if n == 0 { "today".into() } else { format!("{} ago", plural(n, "day", "days")) }
}

/// The badge's words: short, with a sign before the words, so colour is
/// never the only signal.
fn badge(kind: &str, o: &Map<String, Value>, d: &Diagnostic) -> String {
    let n = days(o);
    match kind {
        "vulnerable" => {
            let grade = s(o, "grade").unwrap_or("vulnerable").to_ascii_uppercase();
            let count = 1 + o.get("more").and_then(Value::as_i64).unwrap_or(0);
            let safe = s(o, "safe").map(|v| format!(" → {v}")).unwrap_or_default();
            format!("{grade} · {}{safe}", plural(count, "advisory", "advisories"))
        }
        "deprecated" => {
            let what = if s(o, "grade") == Some("yanked") { "yanked" } else { "deprecated" };
            s(o, "latest").map_or_else(|| what.to_owned(), |l| format!("{what} → {l}"))
        }
        "outdated" => match (s(o, "latest"), s(o, "grade")) {
            (Some(l), Some("lockfile")) => format!("↑ {l} · lockfile"),
            (Some(l), Some(g)) => format!("↑ {l} · {g}"),
            (Some(l), None) => format!("↑ {l}"),
            _ => "outdated".into(),
        },
        "licence" => "licence denied".into(),
        "runtime" => "runtime mismatch".into(),
        "scan" => "scan incomplete".into(),
        "expired" => n.map_or_else(|| "expired".into(), |n| format!("expired {}", short_ago(-n))),
        "due" => n.map_or_else(|| "due soon".into(), |n| format!("due in {n} d")),
        "ticket" => "needs a ticket".into(),
        "flag" => n.map_or_else(|| "flag".into(), |n| format!("flag · expires in {n} d")),
        "flag-expired" => n.map_or_else(|| "flag expired".into(), |n| format!("flag expired {}", short_ago(-n))),
        "focused" => "focused test — the rest are off".into(),
        "skipped" => "skip needs a reason and a date".into(),
        "complexity" => match d.data.as_ref().and_then(|v| Some((v.get("max")?.as_i64()?, v.as_object()?.iter().find(|(k, _)| *k != "max")?.1.as_i64()?))) {
            Some((max, got)) => format!("{got} · max {max}"),
            None => "too complex".into(),
        },
        "ownership" => "no owner".into(),
        _ => d.rule.clone(),
    }
}

fn short_ago(n: i64) -> String {
    if n == 0 { "today".into() } else { format!("{n} d ago") }
}

/// The fix chip: what the person has to do.
pub fn fix_chip(d: &Diagnostic) -> &'static str {
    if matches!(d.fix, FixClass::Auto | FixClass::Suggest) && !d.edits.is_empty() { "one-click fix" } else { "needs your call" }
}

/// The first sentence of a text.
fn sentence(text: &str) -> &str {
    let t = text.trim();
    let mut from = 0;
    while let Some(i) = t[from..].find(". ") {
        let at = from + i;
        // `e.g. x`, `i.e. y` and a version `1. 2` do not end a sentence.
        let word = t[..at].rsplit(' ').next().unwrap_or("");
        if word.len() > 2 && !word.contains('.') {
            return &t[..=at];
        }
        from = at + 2;
    }
    t
}

/// The hover card of one diagnostic (board K): a title with its severity
/// and fix chips; one sentence on why; an example — the fix as before →
/// after, the rule's Don't/Do, or the facts as rows; a meta line. Portable
/// markdown: an editor with no extension shows it as it is, and the VS Code
/// extension adds an icon and the action links.
pub fn card(d: &Diagnostic, data: &Value, rule: Option<&RuleFile>, language: Option<Language>, line: Option<(&str, String)>) -> String {
    let o = data.as_object().cloned().unwrap_or_default();
    let kind = s(&o, "kind").unwrap_or("code");
    let sev = match d.severity {
        Severity::Error => "error",
        Severity::Warn => "warn",
        Severity::Info => "info",
    };
    let grade = s(&o, "grade").filter(|_| kind == "vulnerable").unwrap_or(sev);
    let mut chips = vec![format!("`{grade}`")];
    if o.get("baseline").is_some() {
        chips.push("`baseline`".into());
    }
    chips.push(format!("`{}`", fix_chip(d)));
    let mut out = format!("**{}** &nbsp; {}\n\n", s(&o, "title").unwrap_or(&d.rule), chips.join(" "));
    // Why: the finding's own words for a fact-bearing kind, the rule's Why
    // for code.
    let why = match (kind, rule) {
        ("code" | "security" | "focused" | "complexity", Some(r)) if !r.why.is_empty() => sentence(&r.why).to_owned(),
        _ => d.message.clone(),
    };
    out.push_str(&why);
    out.push_str("\n\n");
    // The example.
    let mut rows: Vec<(&str, String)> = Vec::new();
    for (key, label) in [("safe", "Safe version"), ("latest", "Latest"), ("ticket", "Ticket"), ("due", "Due"), ("owner", "Owner")] {
        if let Some(v) = s(&o, key) {
            rows.push((label, v.to_owned()));
        }
    }
    if let Some(n) = o.get("more").and_then(Value::as_i64) {
        rows.push(("Also", plural(n, "more advisory", "more advisories")));
    }
    let fence = language.map(|l| l.name()).unwrap_or("");
    if let Some((before, after)) = line.filter(|(b, a)| b.trim() != a.trim()) {
        out.push_str(&format!("```{fence}\n{}\n```\n→\n```{fence}\n{}\n```\n\n", before.trim(), after.trim()));
    } else if !rows.is_empty() {
        out.push_str("| | |\n|---|---|\n");
        for (k, v) in &rows {
            out.push_str(&format!("| {k} | **{v}** |\n"));
        }
        out.push('\n');
    } else if let Some((dont, doo)) = rule.zip(language).and_then(|(r, l)| example(r, l)) {
        out.push_str(&format!("Don't\n```{fence}\n{dont}\n```\nDo\n```{fence}\n{doo}\n```\n\n"));
    }
    // The meta line.
    let mut meta = vec![format!("`{}`", d.rule)];
    if let Some(sec) = rule.and_then(|r| r.security.as_ref()) {
        meta.extend(sec.cwe.iter().map(|c| format!("CWE-{c}")));
    }
    meta.push(if o.get("baseline").is_some() { "in the baseline".into() } else { "new".into() });
    meta.push(format!("fix: `{}`", d.fix));
    out.push_str(&meta.join(" · "));
    if d.fix == FixClass::Manual && !d.steps.is_empty() {
        out.push_str("\n\n");
        for (i, step) in d.steps.iter().enumerate() {
            out.push_str(&format!("{}. {step}\n", i + 1));
        }
    }
    out.trim_end().to_owned()
}

/// The rule's first Don't and Do for a language, a few lines each.
fn example(rule: &RuleFile, language: Language) -> Option<(String, String)> {
    let ex = rule.examples.iter().find(|e| e.language == language)?;
    let short = |code: &str| code.lines().take(4).collect::<Vec<_>>().join("\n");
    Some((short(&ex.donts.first()?.code), short(&ex.dos.first()?.code)))
}

#[cfg(test)]
#[path = "../tests/unit/card.rs"]
mod tests;
