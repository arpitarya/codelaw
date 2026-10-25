//! The burn-down (SR-BASELINE decision 10): how many violations each rule's
//! committed baseline allowed at each commit that touched the baseline
//! folder. A pure function of the snapshots it is given (L5): the engine
//! reads them from git, nothing here runs git or a lint.

use crate::date::Date;
use crate::file::Baselines;
use std::collections::{BTreeMap, BTreeSet};

/// How points are grouped: every commit, or the last commit of each day or
/// ISO week (Monday first), by the commit's own date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum By {
    Commit,
    Day,
    Week,
}

impl By {
    pub fn parse(s: &str) -> Result<By, String> {
        match s {
            "commit" => Ok(By::Commit),
            "day" => Ok(By::Day),
            "week" => Ok(By::Week),
            other => Err(format!("unknown --by `{other}` (commit, day or week)")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            By::Commit => "commit",
            By::Day => "day",
            By::Week => "week",
        }
    }
}

/// The baseline at one commit: findings allowed, per rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub id: String,
    pub date: Date,
    pub counts: BTreeMap<String, u64>,
}

impl Snapshot {
    pub fn new(id: &str, date: Date, baselines: &Baselines) -> Snapshot {
        let counts = baselines.rules.iter().map(|(rule, b)| (rule.clone(), b.entries.iter().map(|e| e.count as u64).sum())).collect();
        Snapshot { id: id.to_owned(), date, counts }
    }
}

/// Keep the last of each day or week; snapshots come oldest first.
pub fn bucket(snaps: Vec<Snapshot>, by: By) -> Vec<Snapshot> {
    let key = |s: &Snapshot| match by {
        By::Commit => None,
        By::Day => Some(s.date.days()),
        // 1970-01-01 was a Thursday: shifting by 3 starts weeks on Monday.
        By::Week => Some((s.date.days() + 3).div_euclid(7)),
    };
    let mut out: Vec<Snapshot> = Vec::new();
    for s in snaps {
        match (out.last(), key(&s)) {
            (Some(last), Some(k)) if key(last) == Some(k) => *out.last_mut().expect("last") = s,
            _ => out.push(s),
        }
    }
    out
}

/// One rule's counts at every point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub rule: String,
    pub values: Vec<u64>,
}

impl Row {
    pub fn start(&self) -> u64 {
        self.values.first().copied().unwrap_or(0)
    }
    pub fn now(&self) -> u64 {
        self.values.last().copied().unwrap_or(0)
    }
    pub fn change(&self) -> i64 {
        self.now() as i64 - self.start() as i64
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    pub by: By,
    /// `(commit, date)`, oldest first.
    pub points: Vec<(String, Date)>,
    pub rows: Vec<Row>,
    pub total: Row,
}

/// Build the series. `opening` is the state just before the window (it is
/// never merged into a bucket); `rule` keeps one rule.
pub fn build(opening: Option<Snapshot>, commits: Vec<Snapshot>, by: By, rule: Option<&str>) -> History {
    let mut snaps: Vec<Snapshot> = opening.into_iter().collect();
    snaps.extend(bucket(commits, by));
    let rules: BTreeSet<&String> = snaps.iter().flat_map(|s| s.counts.keys()).filter(|r| rule.is_none_or(|x| x == r.as_str())).collect();
    let rows: Vec<Row> = rules.into_iter().map(|r| Row { rule: r.clone(), values: snaps.iter().map(|s| s.counts.get(r).copied().unwrap_or(0)).collect() }).collect();
    let total = Row { rule: "total".into(), values: (0..snaps.len()).map(|i| rows.iter().map(|r| r.values[i]).sum()).collect() };
    History { by, points: snaps.iter().map(|s| (s.id.clone(), s.date)).collect(), rows, total }
}

const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const SPARK_WIDTH: usize = 24;

/// Bars scaled to the series' own maximum (zero is the lowest bar); a long
/// series is sampled evenly, keeping its first and last point.
pub fn spark(values: &[u64]) -> String {
    let picked: Vec<u64> = if values.len() <= SPARK_WIDTH {
        values.to_vec()
    } else {
        (0..SPARK_WIDTH).map(|i| values[i * (values.len() - 1) / (SPARK_WIDTH - 1)]).collect()
    };
    let max = picked.iter().copied().max().unwrap_or(0);
    picked.iter().map(|&v| BARS[((v * 7 + max / 2).checked_div(max).unwrap_or(0)) as usize]).collect()
}

fn signed(n: i64, minus: &str) -> String {
    match n {
        0 => "0".into(),
        n if n < 0 => format!("{minus}{}", -n),
        n => format!("+{n}"),
    }
}

fn short(id: &str) -> &str {
    id.get(..7).unwrap_or(id)
}

impl History {
    fn all(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().chain(std::iter::once(&self.total))
    }

    fn header(&self) -> String {
        match (self.points.first(), self.points.last()) {
            (Some((a, da)), Some((z, dz))) => format!("{} points ({}), {da} ({}) to {dz} ({})", self.points.len(), self.by.name(), short(a), short(z)),
            _ => "no commit touched the baseline in that window".into(),
        }
    }

    /// The table, with a sparkline per rule.
    pub fn text(&self) -> String {
        let mut out = format!("{}\n", self.header());
        if self.points.is_empty() {
            return out;
        }
        let cells: Vec<[String; 5]> = self
            .all()
            .map(|r| [r.rule.clone(), r.start().to_string(), r.now().to_string(), signed(r.change(), "−"), spark(&r.values)])
            .collect();
        let head = ["rule", "start", "now", "change", "trend"];
        let width = |c: usize| cells.iter().map(|r| r[c].chars().count()).chain([head[c].len()]).max().unwrap_or(0);
        let w: Vec<usize> = (0..4).map(width).collect();
        let line = |r: &[String; 5]| {
            let pad = |c: usize| " ".repeat(w[c] - r[c].chars().count());
            format!("{}{}  {}{}  {}{}  {}{}  {}\n", r[0], pad(0), pad(1), r[1], pad(2), r[2], pad(3), r[3], r[4])
        };
        out.push_str(line(&head.map(str::to_owned)).trim_end());
        out.push('\n');
        for c in &cells {
            out.push_str(line(c).trim_end());
            out.push('\n');
        }
        out
    }

    /// A GitHub-flavoured table for a PR comment.
    pub fn markdown(&self) -> String {
        let mut out = format!("{}\n", self.header());
        if self.points.is_empty() {
            return out;
        }
        out.push_str("\n| rule | start | now | change | trend |\n|---|--:|--:|--:|---|\n");
        for r in self.all() {
            out.push_str(&format!("| `{}` | {} | {} | {} | {} |\n", r.rule, r.start(), r.now(), signed(r.change(), "−"), spark(&r.values)));
        }
        out
    }

    /// Long format for a dashboard: one line per point and rule, then total.
    pub fn csv(&self) -> String {
        let mut out = String::from("date,commit,rule,count\n");
        for (i, (id, date)) in self.points.iter().enumerate() {
            for r in self.all() {
                out.push_str(&format!("{date},{id},{},{}\n", r.rule, r.values[i]));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(id: &str, day: &str, counts: &[(&str, u64)]) -> Snapshot {
        Snapshot { id: id.into(), date: Date::parse(day).unwrap(), counts: counts.iter().map(|(r, n)| ((*r).to_owned(), *n)).collect() }
    }

    #[test]
    fn the_table_matches_the_spec() {
        let h = build(
            None,
            vec![
                snap("aaaaaaaa", "2026-01-01", &[("graph/no-deprecated-usage", 212), ("react/no-context", 41)]),
                snap("bbbbbbbb", "2026-03-01", &[("graph/no-deprecated-usage", 97), ("react/no-context", 38)]),
            ],
            By::Commit,
            None,
        );
        let t = h.text();
        let lines: Vec<&str> = t.lines().collect();
        assert_eq!(lines[1], "rule                       start  now  change  trend");
        assert_eq!(lines[2], "graph/no-deprecated-usage    212   97    −115  █▄");
        assert_eq!(lines[4], "total                        253  135    −118  █▅");
    }

    #[test]
    fn spark_scales_to_the_maximum_and_samples_long_series() {
        assert_eq!(spark(&[0, 0]), "▁▁");
        assert_eq!(spark(&[8, 4, 0]), "█▅▁");
        let long: Vec<u64> = (0..100).collect();
        let s = spark(&long);
        assert_eq!(s.chars().count(), SPARK_WIDTH);
        assert!(s.starts_with('▁') && s.ends_with('█'));
    }

    #[test]
    fn buckets_keep_the_last_of_the_day_or_week() {
        let s = vec![snap("1", "2026-01-05", &[("r", 5)]), snap("2", "2026-01-05", &[("r", 4)]), snap("3", "2026-01-11", &[("r", 3)]), snap("4", "2026-01-12", &[("r", 2)])];
        let ids = |by| bucket(s.clone(), by).into_iter().map(|x| x.id).collect::<Vec<_>>();
        assert_eq!(ids(By::Commit), ["1", "2", "3", "4"]);
        assert_eq!(ids(By::Day), ["2", "3", "4"]);
        // 2026-01-05 is a Monday and the 11th the Sunday ending its week.
        assert_eq!(ids(By::Week), ["3", "4"]);
    }
}
