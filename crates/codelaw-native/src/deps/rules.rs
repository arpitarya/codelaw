//! `deps/outdated` and `deps/stale-snapshot` (named `data/outdated-dependency`
//! and `data/stale-deps-snapshot` until W-128): a manifest's dependencies
//! against the committed registry snapshot. Neither reads a registry; both
//! read only files (L5 decision 3). `deps/vulnerable` and
//! `deps/incomplete-scan` are in `vulnerable.rs`.
//!
//! Every `deps` rule names `[deps]` as its config section, so its keys
//! arrive as options: `severity`, `min_release_age`, `prerelease`,
//! `ignore`, `max_snapshot_age`, `advisory_sources` and `agent`
//! (SR-RULE-PACKS decision 15).

use super::npm::Range;
use super::pep440::{Clause, Pep440, requirement};
use super::snapshot::{Package, SNAPSHOT_PATH, Snapshot, days, parse_days};
use super::{Dep, Ecosystem, folder_of, lock, manifest_deps};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, Severity, TextEdit};
use codelaw_lang::SyntaxNode;
use semver::Version;
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) const MANIFESTS: &[Language] = &[Language::Json, Language::Toml];

pub(super) type Options = BTreeMap<String, Value>;

/// The kinds a finding is graded by, each `[deps] severity` key: five
/// for `deps/outdated`, four advisory severities for `deps/vulnerable`.
const GRADES: [(&str, Option<Severity>); 10] = [
    ("major", Some(Severity::Warn)),
    ("minor", Some(Severity::Info)),
    ("patch", Some(Severity::Info)),
    ("deprecated", Some(Severity::Error)),
    ("yanked", Some(Severity::Error)),
    ("lockfile", Some(Severity::Info)),
    ("critical", Some(Severity::Error)),
    ("high", Some(Severity::Error)),
    ("moderate", Some(Severity::Warn)),
    ("low", Some(Severity::Info)),
];

/// A grade's severity: `[deps] severity`'s, or the default; `None` is off.
pub(super) fn severity(options: &Options, grade: &str) -> Option<Severity> {
    let default = GRADES.iter().find(|(g, _)| *g == grade).and_then(|(_, s)| *s);
    match options.get("severity").and_then(|s| s.get(grade)).and_then(Value::as_str) {
        Some("off") => None,
        Some(s) => Severity::parse(s).or(default),
        None => default,
    }
}

/// The grade a message ends with: `… (major)`.
fn grade_of(message: &str) -> Option<&str> {
    let open = message.rfind(" (")?;
    message[open + 2..].strip_suffix(')')
}

pub(super) struct Settings {
    pub min_release_age: i64,
    pub prerelease: bool,
    pub max_snapshot_age: i64,
}

pub(super) fn settings(options: &Options) -> Settings {
    Settings {
        min_release_age: options.get("min_release_age").and_then(parse_days).unwrap_or(0),
        prerelease: options.get("prerelease").and_then(Value::as_bool).unwrap_or(false),
        max_snapshot_age: options.get("max_snapshot_age").and_then(parse_days).unwrap_or(14),
    }
}

/// Whether `[deps] ignore` holds `key` on the as-of date: a string entry
/// always, a table entry until its `expires` (the first day it no longer
/// holds, as a dated exception's: SR-BASELINE decision 9). An entry naming
/// an `advisory` holds only that advisory: see [`ignored_advisory`].
pub(super) fn ignored(options: &Options, key: &str, as_of: &dyn Fn() -> Option<i64>) -> bool {
    ignore_entries(options, key, as_of).any(|advisory| advisory.is_none())
}

/// Whether `[deps] ignore` holds this advisory of `key` (by any of its
/// ids), or the whole package.
pub(super) fn ignored_advisory(options: &Options, key: &str, ids: &[&str], as_of: &dyn Fn() -> Option<i64>) -> bool {
    ignore_entries(options, key, as_of).any(|advisory| advisory.is_none_or(|a| ids.contains(&a)))
}

/// The entries that hold `key` on the as-of date: `None` for the whole
/// package, else the advisory named.
fn ignore_entries<'o>(options: &'o Options, key: &'o str, as_of: &'o dyn Fn() -> Option<i64>) -> impl Iterator<Item = Option<&'o str>> + 'o {
    options.get("ignore").and_then(Value::as_array).into_iter().flatten().filter_map(move |e| match e {
        Value::String(s) => (s == key).then_some(None),
        Value::Object(o) => {
            let holds = o.get("package").and_then(Value::as_str) == Some(key)
                && match o.get("expires").and_then(Value::as_str).and_then(days) {
                    Some(exp) => as_of().is_none_or(|today| today < exp),
                    None => true,
                };
            holds.then(|| o.get("advisory").and_then(Value::as_str))
        }
        _ => None,
    })
}

/// Whether a version published on `date` is old enough on `fetched`.
fn old_enough(date: &str, fetched: i64, age: i64) -> bool {
    match days(date) {
        Some(d) => fetched - d >= age,
        // The registry gave no date: nothing to hold it back by.
        None => true,
    }
}

/// `major`, `minor` or `patch`: the first release component that differs.
fn grade(from: [u64; 3], to: [u64; 3]) -> &'static str {
    if from[0] != to[0] {
        "major"
    } else if from[1] != to[1] {
        "minor"
    } else {
        "patch"
    }
}

/// The release components of `target` cut to `n`, as text.
fn cut(release: &[u64], n: usize) -> String {
    (0..n).map(|i| release.get(i).copied().unwrap_or(0).to_string()).collect::<Vec<_>>().join(".")
}

pub struct OutdatedDependency;

impl OutdatedDependency {
    fn npm(&self, dep: &Dep, pkg: &Package, installed: Option<&String>, s: &Settings, fetched: i64) -> Option<NativeFinding> {
        let range = Range::parse(&dep.spec)?;
        let installed_v = installed.and_then(|v| Version::parse(v).ok());
        let current = installed_v.clone().unwrap_or_else(|| range.base());
        let target = pkg
            .versions
            .iter()
            .filter(|(v, date)| !pkg.deprecated.contains_key(*v) && old_enough(date, fetched, s.min_release_age))
            .filter_map(|(v, _)| Version::parse(v).ok())
            .filter(|v| s.prerelease || v.pre.is_empty())
            .max();
        let what = format!("`{}` {}", dep.name, dep.spec);
        let bump = |t: &Version| -> Vec<TextEdit> {
            if range.allows(t) || !matches!(range.op.as_str(), "^" | "~" | ">=" | "=" | "") {
                return Vec::new();
            }
            let text = if !t.pre.is_empty() || range.parts.len() == 3 { t.to_string() } else { cut(&[t.major, t.minor, t.patch], range.parts.len()) };
            vec![TextEdit::new(dep.start + range.start, dep.start + range.end, text)]
        };
        let newer = target.clone().filter(|t| *t > current);
        // The version in use: the installed one, or the one the range names exactly.
        let in_use = installed.cloned().or_else(|| (range.parts.len() == 3 && matches!(range.op.as_str(), "" | "=")).then(|| range.base().to_string()));
        if let Some(msg) = in_use.as_ref().and_then(|v| pkg.deprecated.get(v)) {
            let later = newer.as_ref().map(|t| format!("; latest {t}")).unwrap_or_default();
            let edits = newer.as_ref().map(bump).unwrap_or_default();
            let v = in_use.as_deref().unwrap_or_default();
            return Some(NativeFinding { start: dep.start, end: dep.end, message: format!("{what} — {v} is deprecated: {msg}{later} (deprecated)"), edits });
        }
        let t = newer?;
        if range.allows(&t) {
            let have = installed_v?;
            return Some(NativeFinding {
                start: dep.start,
                end: dep.end,
                message: format!("{what} allows {t}, but the lockfile has {have}: run your install to pick it up (lockfile)"),
                edits: Vec::new(),
            });
        }
        let g = grade([current.major, current.minor, current.patch], [t.major, t.minor, t.patch]);
        let have = installed.map(|v| format!("installed {v}, ")).unwrap_or_default();
        Some(NativeFinding { start: dep.start, end: dep.end, message: format!("{what} — {have}latest {t} ({g})"), edits: bump(&t) })
    }

    fn pypi(&self, dep: &Dep, pkg: &Package, installed: Option<&String>, s: &Settings, fetched: i64) -> Option<NativeFinding> {
        let req = requirement(&dep.spec)?;
        let installed_v = installed.and_then(|v| Pep440::parse(v));
        // The clause the fix moves: a pin or a lower bound.
        let base = req.clauses.iter().position(|c| matches!(c.op.as_str(), "==" | "===" | ">=" | "~=" | ">") && !c.wildcard);
        let base_v = base.and_then(|i| Pep440::parse(&req.clauses[i].version));
        let current = installed_v.clone().or(base_v)?;
        let yanked = |v: &Pep440| pkg.yanked.keys().any(|y| Pep440::parse(y).as_ref() == Some(v));
        let target = pkg
            .versions
            .iter()
            .filter(|(_, date)| old_enough(date, fetched, s.min_release_age))
            .filter_map(|(v, _)| Pep440::parse(v))
            .filter(|v| !yanked(v) && (s.prerelease || !v.is_prerelease()))
            .max();
        let allows = |clauses: &[Clause], v: &Pep440| clauses.iter().all(|c| c.allows(v));
        let what = format!("`{}`", dep.spec);
        let bump = |t: &Pep440| -> Vec<TextEdit> {
            let Some(i) = base else { return Vec::new() };
            let c = &req.clauses[i];
            let written = Pep440::parse(&c.version).map(|v| v.release.len()).unwrap_or(1);
            let text = match c.op.as_str() {
                _ if t.is_prerelease() => t.to_string(),
                "==" | "===" => t.to_string(),
                ">=" => cut(&t.release, written),
                "~=" => cut(&t.release, written.max(2)),
                _ => return Vec::new(),
            };
            let mut moved = req.clauses.clone();
            moved[i].version = text.clone();
            if allows(&req.clauses, t) || !allows(&moved, t) {
                return Vec::new();
            }
            vec![TextEdit::new(dep.start + c.start, dep.start + c.end, text)]
        };
        let newer = target.clone().filter(|t| *t > current);
        // The version in use: the installed one, or the one a `==` pins.
        let pinned = base.filter(|i| matches!(req.clauses[*i].op.as_str(), "==" | "===")).and_then(|i| Pep440::parse(&req.clauses[i].version));
        let in_use = installed_v.clone().or(pinned);
        if let Some(have) = &in_use
            && let Some((v, why)) = pkg.yanked.iter().find(|(y, _)| Pep440::parse(y).as_ref() == Some(have))
        {
            let why = if why.is_empty() { String::new() } else { format!(": {why}") };
            let later = newer.as_ref().map(|t| format!("; latest {t}")).unwrap_or_default();
            let edits = newer.as_ref().map(bump).unwrap_or_default();
            return Some(NativeFinding { start: dep.start, end: dep.end, message: format!("{what} — {v} is yanked{why}{later} (yanked)"), edits });
        }
        let t = newer?;
        if allows(&req.clauses, &t) {
            let have = installed?;
            return Some(NativeFinding {
                start: dep.start,
                end: dep.end,
                message: format!("{what} allows {t}, but the lockfile has {have}: run your install to pick it up (lockfile)"),
                edits: Vec::new(),
            });
        }
        let g = if current.epoch != t.epoch { "major" } else { grade([current.part(0), current.part(1), current.part(2)], [t.part(0), t.part(1), t.part(2)]) };
        let have = installed.map(|v| format!("installed {v}, ")).unwrap_or_default();
        Some(NativeFinding { start: dep.start, end: dep.end, message: format!("{what} — {have}latest {t} ({g})"), edits: bump(&t) })
    }
}

/// The as-of date in days, read only when a rule needs it.
pub(super) fn as_of_days(ctx: &NativeContext<'_>) -> Option<i64> {
    ctx.file.as_of().as_deref().and_then(days)
}

impl NativeRule for OutdatedDependency {
    fn id(&self) -> &'static str {
        "deps/outdated"
    }
    fn languages(&self) -> &'static [Language] {
        MANIFESTS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("deps")
    }
    fn severity_of(&self, finding: &NativeFinding, options: &Options) -> Option<Severity> {
        grade_of(&finding.message).and_then(|g| severity(options, g))
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let path = ctx.file.path();
        let deps = manifest_deps(ctx.language, path, node.get_inner_node(), ctx.source);
        if deps.is_empty() {
            return;
        }
        let Some(snap) = ctx.file.read(SNAPSHOT_PATH).and_then(|t| Snapshot::parse(&t)) else { return };
        let Some(fetched) = days(&snap.fetched) else { return };
        let s = settings(ctx.options);
        let dir = folder_of(path);
        let read = |p: &str| ctx.file.read(p);
        let mut installed: BTreeMap<Ecosystem, BTreeMap<String, String>> = BTreeMap::new();
        for eco in [Ecosystem::Npm, Ecosystem::Pypi] {
            let specs: BTreeMap<String, String> = deps
                .iter()
                .filter(|d| d.ecosystem == eco)
                .map(|d| (if eco == Ecosystem::Npm { d.name.clone() } else { d.key["pypi:".len()..].to_owned() }, d.spec.clone()))
                .collect();
            if !specs.is_empty() {
                installed.insert(eco, lock::installed(eco, dir, &specs, &read));
            }
        }
        let as_of = std::cell::OnceCell::new();
        let today = || *as_of.get_or_init(|| as_of_days(ctx));
        for dep in &deps {
            let Some(pkg) = snap.packages.get(&dep.key) else { continue };
            if ignored(ctx.options, &dep.key, &today) {
                continue;
            }
            let lock_key = if dep.ecosystem == Ecosystem::Npm { dep.name.as_str() } else { &dep.key["pypi:".len()..] };
            let have = installed.get(&dep.ecosystem).and_then(|m| m.get(lock_key));
            let found = match dep.ecosystem {
                Ecosystem::Npm => self.npm(dep, pkg, have, &s, fetched),
                Ecosystem::Pypi => self.pypi(dep, pkg, have, &s, fetched),
            };
            if let Some(f) = found.filter(|f| grade_of(&f.message).is_some_and(|g| severity(ctx.options, g).is_some())) {
                out.push(f);
            }
        }
    }
}

/// The one file a per-project finding sits on: the root `package.json`,
/// else the root `pyproject.toml`.
pub(super) fn project_manifest(ctx: &NativeContext<'_>) -> bool {
    match ctx.file.path() {
        "package.json" => true,
        "pyproject.toml" => !ctx.file.exists("package.json"),
        _ => false,
    }
}

pub struct StaleDepsSnapshot;

impl NativeRule for StaleDepsSnapshot {
    fn id(&self) -> &'static str {
        "deps/stale-snapshot"
    }
    fn languages(&self) -> &'static [Language] {
        MANIFESTS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("deps")
    }
    fn check(&self, _node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        // One finding per project: on the root `package.json`, else the
        // root `pyproject.toml`.
        if !project_manifest(ctx) {
            return;
        }
        let Some(text) = ctx.file.read(SNAPSHOT_PATH) else { return };
        let end = ctx.source.find('\n').unwrap_or(ctx.source.len());
        let Some(snap) = Snapshot::parse(&text).filter(|s| days(&s.fetched).is_some()) else {
            out.push(NativeFinding {
                start: 0,
                end,
                message: format!("`{SNAPSHOT_PATH}` cannot be read: run `codelaw deps refresh` to write it again"),
                edits: Vec::new(),
            });
            return;
        };
        let Some(today_s) = ctx.file.as_of() else { return };
        let (Some(today), Some(fetched)) = (days(&today_s), days(&snap.fetched)) else { return };
        let max = settings(ctx.options).max_snapshot_age;
        let age = today - fetched;
        if age > max {
            out.push(NativeFinding {
                start: 0,
                end,
                message: format!(
                    "The dependency snapshot was fetched on {}, {age} days before the as-of date {today_s}; `[deps] max_snapshot_age` is {max} days: run `codelaw deps refresh`",
                    snap.fetched
                ),
                edits: Vec::new(),
            });
        }
    }
}

#[cfg(test)]
mod tests;
