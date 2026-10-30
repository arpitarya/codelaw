//! Vulnerability advisories in the snapshot (W-128; SR-RULE-PACKS decision
//! 15): the ranges they affect, the merge of every enabled source's records
//! (the safer answer wins), and the version a vulnerable dependency moves
//! to — the lowest **known-clean** one.
//!
//! A range is a comparator set — `>=1.0.0 <1.2.3`, `<=2.0`, `=1.5.0`, `*` —
//! read in the ecosystem's order (semver for npm, PEP 440 for PyPI). A
//! version is affected when any range of any advisory holds it. A version is
//! known-clean only where a source **covered** it (the package's `scanned`)
//! and no advisory affects it; a version no source covered is never offered.

use super::Ecosystem;
use super::pep440::Pep440;
use super::snapshot::{Advisory, Package, days};
use semver::Version;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// The advisory severities, worst first, and the word for none given.
pub const SEVERITIES: [&str; 4] = ["critical", "high", "moderate", "low"];

/// An advisory no source grades counts as `high`: the safer answer.
pub const UNGRADED: &str = "high";

/// A severity word as the snapshot keeps it: one of [`SEVERITIES`], from
/// any source's spelling (`HIGH`, `MEDIUM`, npm's `info`); `None` when the
/// source gives none.
pub fn severity_word(s: &str) -> Option<&'static str> {
    match s.trim().to_ascii_lowercase().as_str() {
        "critical" => Some("critical"),
        "high" => Some("high"),
        "moderate" | "medium" => Some("moderate"),
        "low" | "info" | "none" => Some("low"),
        _ => None,
    }
}

/// Worse first: `critical` < `high` < `moderate` < `low`.
pub fn severity_rank(s: &str) -> usize {
    SEVERITIES.iter().position(|x| *x == s).unwrap_or(1)
}

/// One version of an ecosystem, ordered as that ecosystem orders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ver {
    Npm(Version),
    Pypi(Pep440),
}

impl Ver {
    pub fn parse(eco: Ecosystem, text: &str) -> Option<Ver> {
        match eco {
            Ecosystem::Npm => npm_version(text).map(Ver::Npm),
            Ecosystem::Pypi => Pep440::parse(text).map(Ver::Pypi),
        }
    }

    /// The first release component: what "the same major" means.
    pub fn major(&self) -> (u64, u64) {
        match self {
            Ver::Npm(v) => (0, v.major),
            Ver::Pypi(v) => (v.epoch, v.part(0)),
        }
    }

    pub fn is_prerelease(&self) -> bool {
        match self {
            Ver::Npm(v) => !v.pre.is_empty(),
            Ver::Pypi(v) => v.is_prerelease(),
        }
    }
}

impl PartialOrd for Ver {
    fn partial_cmp(&self, other: &Ver) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ver {
    fn cmp(&self, other: &Ver) -> Ordering {
        match (self, other) {
            (Ver::Npm(a), Ver::Npm(b)) => a.cmp(b),
            (Ver::Pypi(a), Ver::Pypi(b)) => a.cmp(b),
            (Ver::Npm(_), Ver::Pypi(_)) => Ordering::Less,
            (Ver::Pypi(_), Ver::Npm(_)) => Ordering::Greater,
        }
    }
}

/// An npm version, padded when a range writes fewer than three parts
/// (`1.2` is `1.2.0`); a leading `v` or `=` is dropped.
fn npm_version(text: &str) -> Option<Version> {
    let t = text.trim().trim_start_matches(['v', '=']);
    if let Ok(v) = Version::parse(t) {
        return Some(v);
    }
    let core = t.split(['-', '+']).next().unwrap_or(t);
    let parts: Vec<u64> = core.split('.').map(|p| p.parse().ok()).collect::<Option<Vec<_>>>()?;
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    Some(Version::new(parts[0], parts.get(1).copied().unwrap_or(0), parts.get(2).copied().unwrap_or(0)))
}

const OPS: [&str; 5] = [">=", "<=", ">", "<", "="];

/// The comparators of one set: `>=1.0.0 <1.2.3`, with `>= 1.0.0` read as
/// one comparator.
fn comparators(range: &str) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    let mut pending: Option<&'static str> = None;
    for tok in range.split_whitespace() {
        if let Some(op) = pending.take() {
            out.push((op, tok.to_owned()));
            continue;
        }
        let op = OPS.iter().find(|o| tok.starts_with(**o)).copied();
        match op {
            Some(o) if tok.len() == o.len() => pending = Some(o),
            Some(o) => out.push((o, tok[o.len()..].to_owned())),
            None => out.push(("=", tok.to_owned())),
        }
    }
    out
}

/// Whether one comparator set holds `v`. `*` (or an empty set) holds every
/// version; a bound that does not parse holds too — the safer answer.
pub fn range_holds(eco: Ecosystem, range: &str, v: &Ver) -> bool {
    range.split("||").any(|alt| {
        let alt = alt.trim();
        if alt.is_empty() || alt == "*" {
            return true;
        }
        comparators(alt).into_iter().all(|(op, bound)| {
            if bound == "*" || (op == ">=" && bound == "0") {
                return true;
            }
            let Some(b) = Ver::parse(eco, &bound) else { return true };
            match op {
                ">=" => *v >= b,
                ">" => *v > b,
                "<=" => *v <= b,
                "<" => *v < b,
                _ => *v == b,
            }
        })
    })
}

/// Whether an advisory affects `v`.
pub fn affects(eco: Ecosystem, a: &Advisory, v: &Ver) -> bool {
    a.affected.iter().any(|r| range_holds(eco, r, v))
}

/// OSV's `ranges[].events` as comparator sets: `introduced` opens one,
/// `fixed` closes it below, `last_affected` closes it at.
pub fn ranges_from_events(events: &[(String, String)]) -> Vec<String> {
    let mut out = Vec::new();
    let mut open: Option<String> = None;
    for (kind, v) in events {
        match kind.as_str() {
            "introduced" => open = Some(v.clone()),
            "fixed" | "last_affected" | "limit" => {
                let lo = open.take().filter(|o| o != "0").map(|o| format!(">={o} ")).unwrap_or_default();
                let op = if kind == "last_affected" { "<=" } else { "<" };
                out.push(format!("{lo}{op}{v}"));
            }
            _ => {}
        }
    }
    if let Some(o) = open {
        out.push(if o == "0" { "*".to_owned() } else { format!(">={o}") });
    }
    out
}

/// The id a merged record goes by: a GHSA id first, then PYSEC, then CVE,
/// then any other; the lowest of its kind.
fn primary(ids: &BTreeSet<String>) -> String {
    let rank = |id: &str| ["GHSA-", "PYSEC-", "CVE-"].iter().position(|p| id.starts_with(p)).unwrap_or(3);
    ids.iter().min_by(|a, b| rank(a).cmp(&rank(b)).then(a.cmp(b))).cloned().unwrap_or_default()
}

/// Merge one package's advisories from every source into one record per
/// vulnerability: records meet when their ids or aliases meet. The merged
/// record holds every range (a version is affected if any source says
/// so), every fixed version, the worst severity, every id and every
/// source; withdrawn records never arrive here.
pub fn merge(eco: Ecosystem, records: Vec<Advisory>) -> Vec<Advisory> {
    // Union-find over the records, by shared ids.
    let n = records.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while p[r] != r {
            r = p[r];
        }
        let mut i = i;
        while p[i] != r {
            let next = p[i];
            p[i] = r;
            i = next;
        }
        r
    }
    let mut owner: BTreeMap<String, usize> = BTreeMap::new();
    for (i, a) in records.iter().enumerate() {
        for id in std::iter::once(&a.id).chain(a.aliases.iter()) {
            match owner.get(id) {
                Some(&j) => {
                    let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                    if ri != rj {
                        parent[ri] = rj;
                    }
                }
                None => {
                    owner.insert(id.clone(), i);
                }
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut out = Vec::new();
    for members in groups.values() {
        let mut ids = BTreeSet::new();
        let mut affected = BTreeSet::new();
        let mut fixed = BTreeSet::new();
        let mut sources = BTreeSet::new();
        let mut severity: Option<&str> = None;
        let mut urls: BTreeMap<String, String> = BTreeMap::new();
        for &i in members {
            let a = &records[i];
            ids.insert(a.id.clone());
            ids.extend(a.aliases.iter().cloned());
            affected.extend(a.affected.iter().cloned());
            fixed.extend(a.fixed.iter().cloned());
            sources.extend(a.sources.iter().cloned());
            if let Some(s) = SEVERITIES.iter().find(|s| **s == a.severity) {
                severity = Some(match severity {
                    Some(have) if severity_rank(have) <= severity_rank(s) => have,
                    _ => s,
                });
            }
            if let Some(u) = &a.url {
                urls.entry(a.id.clone()).or_insert_with(|| u.clone());
            }
        }
        let id = primary(&ids);
        ids.remove(&id);
        let url = urls.get(&id).or_else(|| urls.values().next()).cloned();
        let mut fixed: Vec<String> = fixed.into_iter().collect();
        fixed.sort_by(|a, b| match (Ver::parse(eco, a), Ver::parse(eco, b)) {
            (Some(x), Some(y)) => x.cmp(&y),
            _ => a.cmp(b),
        });
        fixed.dedup();
        out.push(Advisory {
            id,
            aliases: ids.into_iter().collect(),
            severity: severity.unwrap_or("").to_owned(),
            affected: affected.into_iter().collect(),
            fixed,
            url,
            sources: sources.into_iter().collect(),
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// The severity a merged record is graded by: its own, or [`UNGRADED`].
pub fn graded(a: &Advisory) -> &str {
    if a.severity.is_empty() { UNGRADED } else { &a.severity }
}

/// Whether a source covered `v` of this package: `*` covers every version.
pub fn covered(pkg: &Package, version: &str, eco: Ecosystem) -> bool {
    let v = Ver::parse(eco, version);
    pkg.scanned.values().any(|vs| vs.iter().any(|s| s == "*" || s == version || (v.is_some() && Ver::parse(eco, s) == v)))
}

/// What a vulnerable dependency can move to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// The lowest known-clean version; `major` when it is outside the
    /// major in use.
    Clean { version: String, major: bool },
    /// No covered clean version; the highest fixed version an advisory
    /// names, which no source verified.
    Unverified { fixed: String },
    /// No version is known to fix it.
    None,
}

/// The filters the target obeys: `[deps] min_release_age` on the
/// snapshot's day, and `[deps] prerelease`.
pub struct TargetRules {
    pub fetched: i64,
    pub min_release_age: i64,
    pub prerelease: bool,
}

/// The upgrade target: the lowest version above the one in use that every
/// advisory leaves clean, that a source covered, that is neither
/// deprecated nor yanked and is old enough — inside the major in use
/// first, so a security fix never forces a major when the major has one.
pub fn target(eco: Ecosystem, pkg: &Package, advisories: &[&Advisory], in_use: &Ver, rules: &TargetRules) -> Target {
    let old_enough = |date: &str| match days(date) {
        Some(d) => rules.fetched - d >= rules.min_release_age,
        None => true,
    };
    let withdrawn = |v: &str| pkg.deprecated.contains_key(v) || pkg.yanked.contains_key(v);
    let mut clean: Vec<(Ver, &String)> = pkg
        .versions
        .iter()
        .filter(|(v, date)| !withdrawn(v) && old_enough(date))
        .filter_map(|(v, _)| Ver::parse(eco, v).map(|p| (p, v)))
        .filter(|(p, _)| p > in_use && (rules.prerelease || !p.is_prerelease()))
        .filter(|(p, v)| covered(pkg, v, eco) && !advisories.iter().any(|a| affects(eco, a, p)))
        .collect();
    clean.sort_by(|a, b| a.0.cmp(&b.0));
    if let Some((_, v)) = clean.iter().find(|(p, _)| p.major() == in_use.major()) {
        return Target::Clean { version: (*v).clone(), major: false };
    }
    if let Some((_, v)) = clean.first() {
        return Target::Clean { version: (*v).clone(), major: true };
    }
    let fixed = advisories.iter().flat_map(|a| a.fixed.iter()).filter_map(|f| Ver::parse(eco, f).map(|p| (p, f))).filter(|(p, _)| p > in_use).max_by(|a, b| a.0.cmp(&b.0));
    match fixed {
        Some((_, f)) => Target::Unverified { fixed: f.clone() },
        None => Target::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adv(id: &str, aliases: &[&str], sev: &str, affected: &[&str], fixed: &[&str], source: &str) -> Advisory {
        Advisory {
            id: id.into(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            severity: sev.into(),
            affected: affected.iter().map(|s| s.to_string()).collect(),
            fixed: fixed.iter().map(|s| s.to_string()).collect(),
            url: None,
            sources: vec![source.into()],
        }
    }

    fn npm(v: &str) -> Ver {
        Ver::parse(Ecosystem::Npm, v).unwrap()
    }

    #[test]
    fn ranges_hold_what_they_say() {
        let e = Ecosystem::Npm;
        assert!(range_holds(e, "<4.17.21", &npm("4.17.20")) && !range_holds(e, "<4.17.21", &npm("4.17.21")));
        assert!(range_holds(e, ">=1.0.0 <1.2.3 || >=2.0.0 <2.1.1", &npm("2.1.0")));
        assert!(!range_holds(e, ">=1.0.0 <1.2.3 || >=2.0.0 <2.1.1", &npm("1.5.0")));
        assert!(range_holds(e, ">= 1.0.0 < 1.2", &npm("1.1.9")));
        assert!(range_holds(e, "*", &npm("9.9.9")) && range_holds(e, "=1.5.0", &npm("1.5.0")));
        let p = |s: &str| Ver::parse(Ecosystem::Pypi, s).unwrap();
        assert!(range_holds(Ecosystem::Pypi, ">=2.0 <2.31.0", &p("2.30")) && !range_holds(Ecosystem::Pypi, ">=2.0 <2.31.0", &p("2.31")));
        let ev = |v: &[(&str, &str)]| ranges_from_events(&v.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect::<Vec<_>>());
        assert_eq!(ev(&[("introduced", "0"), ("fixed", "4.17.21")]), ["<4.17.21"]);
        assert_eq!(ev(&[("introduced", "1.0"), ("last_affected", "1.4"), ("introduced", "2.0")]), [">=1.0 <=1.4", ">=2.0"]);
        assert_eq!(ev(&[("introduced", "0")]), ["*"]);
    }

    #[test]
    fn merge_keeps_the_safer_answer() {
        let a = adv("GHSA-aaaa", &[], "moderate", &["<1.2.0"], &["1.2.0"], "registry");
        let b = adv("PYSEC-1", &["GHSA-aaaa", "CVE-2024-1"], "high", &["<1.3.0"], &["1.3.0"], "osv");
        let c = adv("GHSA-bbbb", &[], "", &[">=2.0.0 <2.0.5"], &["2.0.5"], "osv");
        let m = merge(Ecosystem::Npm, vec![a, b, c]);
        assert_eq!(m.len(), 2);
        let one = &m[0];
        assert_eq!(one.id, "GHSA-aaaa");
        assert_eq!(one.aliases, ["CVE-2024-1", "PYSEC-1"]);
        assert_eq!(one.severity, "high", "the highest severity");
        assert_eq!(one.fixed, ["1.2.0", "1.3.0"]);
        assert_eq!(one.sources, ["osv", "registry"]);
        // Affected if any source says so: 1.2.5 is outside the registry's range, inside OSV's.
        assert!(affects(Ecosystem::Npm, one, &npm("1.2.5")));
        assert_eq!(graded(&m[1]), "high", "an ungraded record is graded high");
    }

    fn package(versions: &[&str], scanned: &[(&str, &[&str])]) -> Package {
        Package {
            versions: versions.iter().map(|v| (v.to_string(), "2020-01-01".to_string())).collect(),
            scanned: scanned.iter().map(|(s, vs)| (s.to_string(), vs.iter().map(|v| v.to_string()).collect())).collect(),
            ..Package::default()
        }
    }

    #[test]
    fn the_target_is_known_clean_and_in_the_major_first() {
        let rules = TargetRules { fetched: days("2026-10-29").unwrap(), min_release_age: 0, prerelease: false };
        let a = adv("GHSA-x", &[], "high", &["<1.2.3"], &["1.2.3"], "osv");
        let b = adv("GHSA-y", &[], "low", &[">=1.2.3 <1.2.5"], &["1.2.5"], "osv");
        let pkg = package(&["1.2.0", "1.2.3", "1.2.4", "1.2.5", "1.3.0", "2.0.0"], &[("osv", &["*"])]);
        let t = target(Ecosystem::Npm, &pkg, &[&a, &b], &npm("1.2.0"), &rules);
        assert_eq!(t, Target::Clean { version: "1.2.5".into(), major: false }, "the lowest clean version against every advisory");
        // Nothing clean in the major: the lowest above it, marked a major.
        let c = adv("GHSA-z", &[], "high", &["<2.0.0"], &["2.0.0"], "osv");
        assert_eq!(target(Ecosystem::Npm, &pkg, &[&c], &npm("1.2.0"), &rules), Target::Clean { version: "2.0.0".into(), major: true });
    }

    #[test]
    fn a_version_no_source_covered_is_never_offered() {
        // The registry source of PyPI covers only the versions it queried:
        // 2.0.1 (the fixed version) is yanked, 2.0.2 was never queried.
        let rules = TargetRules { fetched: days("2026-10-29").unwrap(), min_release_age: 0, prerelease: false };
        let p = |s: &str| Ver::parse(Ecosystem::Pypi, s).unwrap();
        let a = adv("PYSEC-9", &[], "", &[">=2.0 <2.0.1"], &["2.0.1"], "registry");
        let mut pkg = package(&["2.0", "2.0.1", "2.0.2"], &[("registry", &["2.0", "2.0.1"])]);
        pkg.yanked.insert("2.0.1".into(), "broken".into());
        assert_eq!(target(Ecosystem::Pypi, &pkg, &[&a], &p("2.0"), &rules), Target::Unverified { fixed: "2.0.1".into() });
        // Once a source covers 2.0.2, it is offered.
        pkg.scanned.get_mut("registry").unwrap().push("2.0.2".into());
        assert_eq!(target(Ecosystem::Pypi, &pkg, &[&a], &p("2.0"), &rules), Target::Clean { version: "2.0.2".into(), major: false });
        // No fixed version at all.
        let b = adv("PYSEC-10", &[], "", &[">=2.0"], &[], "registry");
        assert_eq!(target(Ecosystem::Pypi, &package(&["2.0"], &[]), &[&b], &p("2.0"), &rules), Target::None);
    }
}
