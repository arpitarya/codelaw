//! The committed snapshot of registry facts, `.codelaw/deps-snapshot.json`
//! (SR-RULE-PACKS decision 15). `codelaw deps refresh` writes it; the rules
//! only read it, so `check` never needs the network (L5 decision 3).
//!
//! Every map is a `BTreeMap`, so the bytes are sorted and a refresh with no
//! registry change writes the same file.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Where the snapshot lives, relative to the project root.
pub const SNAPSHOT_PATH: &str = ".codelaw/deps-snapshot.json";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The day the registries were read, `YYYY-MM-DD`.
    pub fetched: String,
    /// The registry each ecosystem (or npm scope) was read from, without
    /// credentials.
    #[serde(default)]
    pub registries: BTreeMap<String, String>,
    /// `npm:<name>` or `pypi:<normalised name>`.
    #[serde(default)]
    pub packages: BTreeMap<String, Package>,
    /// Each advisory source the refresh read, by its `[deps]
    /// advisory_sources` name, and how that went (W-128).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sources: BTreeMap<String, SourceState>,
}

/// How one advisory source answered the refresh.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceState {
    /// `registry` or `osv`.
    pub kind: String,
    /// `ok`, or `unavailable` when the source could not be read.
    pub state: String,
    /// The refresh's day.
    pub on: String,
    /// Why it was unavailable: the first failed request, without credentials.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SourceState {
    pub fn unavailable(&self) -> bool {
        self.state != "ok"
    }
}

/// One advisory of one package, merged from every source that reported it.
/// It keeps ids, ranges, versions, a severity, a URL and the source names —
/// never an advisory's text (OSV and GHSA data is CC-BY 4.0).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advisory {
    /// A GHSA id when one is known, else PYSEC, else CVE, else the source's.
    pub id: String,
    /// Every other id the sources gave it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// `critical`, `high`, `moderate`, `low`, or `""` when no source grades it.
    #[serde(default)]
    pub severity: String,
    /// Comparator sets of the versions it affects (`>=1.0.0 <1.2.3`, `*`).
    pub affected: Vec<String>,
    /// The versions the sources name as fixing it, lowest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The `[deps] advisory_sources` names that reported it.
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    /// The highest stable version the registry has; none for a transitive
    /// package the refresh did not need the versions of.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest: Option<String>,
    /// Each version kept, with the day it was published (`""` when the
    /// registry does not say).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub versions: BTreeMap<String, String>,
    /// npm: versions marked deprecated, with the registry's message.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub deprecated: BTreeMap<String, String>,
    /// PyPI: versions yanked, with the reason (`""` when none is given).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub yanked: BTreeMap<String, String>,
    /// The advisories that affect a version in use, merged across sources.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub advisories: Vec<Advisory>,
    /// Per advisory source, the versions it covered — `*` for every
    /// version: a version is known-clean only where one covered it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scanned: BTreeMap<String, Vec<String>>,
    /// The licence the registry names, per version in use (an SPDX
    /// expression where the registry gives one; W-129 reads it).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub licence: BTreeMap<String, String>,
}

impl Snapshot {
    pub fn parse(text: &str) -> Option<Snapshot> {
        serde_json::from_str(text).ok()
    }

    /// The file's bytes: two-space JSON, keys sorted, one final newline.
    pub fn to_text(&self) -> String {
        let mut s = serde_json::to_string_pretty(self).expect("a snapshot serialises");
        s.push('\n');
        s
    }
}

/// Days since 1970-01-01 of a `YYYY-MM-DD` date (or the date part of an
/// RFC 3339 timestamp).
pub fn days(date: &str) -> Option<i64> {
    let d = date.get(..10)?;
    let b = d.as_bytes();
    if b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i64 = d[..4].parse().ok()?;
    let m: i64 = d[5..7].parse().ok()?;
    let day: i64 = d[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&day) {
        return None;
    }
    // Howard Hinnant's days_from_civil.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// The `YYYY-MM-DD` date `n` days after 1970-01-01.
pub fn date_of(n: i64) -> String {
    let z = n + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

/// A number of days from config: an integer, or `"7d"` / `"2w"`.
pub fn parse_days(v: &serde_json::Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        return (n >= 0).then_some(n);
    }
    let s = v.as_str()?.trim();
    if let Some(n) = s.strip_suffix('d') {
        return n.trim().parse().ok().filter(|n: &i64| *n >= 0);
    }
    if let Some(n) = s.strip_suffix('w') {
        return n.trim().parse::<i64>().ok().filter(|n| *n >= 0).map(|n| n * 7);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_round_trip() {
        assert_eq!(days("1970-01-01"), Some(0));
        assert_eq!(days("2026-10-25T09:00:00Z"), days("2026-10-25"));
        for d in ["2024-02-29", "2026-10-25", "1999-12-31", "2000-03-01"] {
            assert_eq!(date_of(days(d).unwrap()), d);
        }
        assert_eq!(parse_days(&serde_json::json!("7d")), Some(7));
        assert_eq!(parse_days(&serde_json::json!("2w")), Some(14));
        assert_eq!(parse_days(&serde_json::json!(3)), Some(3));
        assert_eq!(parse_days(&serde_json::json!("7 days")), None);
    }
}
