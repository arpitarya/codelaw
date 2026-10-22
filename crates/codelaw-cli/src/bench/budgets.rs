//! `perf/budgets.toml` and the gate over it (L8 decisions 1, 4, 6 and 7).
//!
//! One `[operation.<name>]` table per L8 operation. **A budget is set only
//! when its `budget` key is a number**, and a set budget also names
//! `set_by`, `set_on` and the `measured_p95` it came from (decision 4).
//! `budget = "unset"` is the only other value it may hold. A `proposed`
//! value is a proposal awaiting the decider: the gate prints it and never
//! compares against it. So a file of proposals reports every p95 and fails
//! nothing (decision 6), and the first ruling that writes a number into
//! `budget` turns the gate on for that operation, with no change to CI.

use std::collections::BTreeMap;
use std::path::Path;

/// One operation's entry.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub unit: String,
    /// `Some` only when the decider has set it.
    pub budget: Option<f64>,
    pub proposed: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Budgets {
    pub machine: String,
    pub operations: BTreeMap<String, Entry>,
}

pub fn load(path: &Path) -> Result<Budgets, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", codelaw_core::normalize_path(path)))?;
    parse(&text).map_err(|e| format!("{}: {e}", codelaw_core::normalize_path(path)))
}

fn number(v: &toml::Value) -> Option<f64> {
    match v {
        toml::Value::Integer(i) => Some(*i as f64),
        toml::Value::Float(f) => Some(*f),
        _ => None,
    }
}

pub fn parse(text: &str) -> Result<Budgets, String> {
    let doc: toml::Table = text.parse().map_err(|e: toml::de::Error| e.message().to_owned())?;
    let machine = doc.get("reference").and_then(|r| r.get("machine")).and_then(|m| m.as_str()).unwrap_or("unset").to_owned();
    let mut operations = BTreeMap::new();
    let ops = doc.get("operation").and_then(|o| o.as_table()).ok_or("no [operation.<name>] tables")?;
    for (name, v) in ops {
        let t = v.as_table().ok_or_else(|| format!("operation.{name} is not a table"))?;
        let unit = t.get("unit").and_then(|u| u.as_str()).ok_or_else(|| format!("operation.{name}: `unit` is missing (\"ms\" or \"MiB\")"))?;
        if unit != "ms" && unit != "MiB" {
            return Err(format!("operation.{name}: unit `{unit}` is not \"ms\" or \"MiB\""));
        }
        let budget = match t.get("budget") {
            Some(toml::Value::String(s)) if s == "unset" => None,
            Some(v) => match number(v) {
                Some(n) if n > 0.0 => {
                    // A set budget carries its provenance (decision 4).
                    for key in ["set_by", "set_on", "measured_p95"] {
                        if !t.contains_key(key) {
                            return Err(format!("operation.{name}: a set budget names `{key}` (L8 decision 4)"));
                        }
                    }
                    Some(n)
                }
                _ => return Err(format!("operation.{name}: `budget` is a positive number set by the decider, or \"unset\"")),
            },
            None => return Err(format!("operation.{name}: `budget` is missing; it is \"unset\" until the decider sets it")),
        };
        let proposed = t.get("proposed").and_then(number);
        operations.insert(name.clone(), Entry { unit: unit.to_owned(), budget, proposed });
    }
    Ok(Budgets { machine, operations })
}

/// One operation's verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Within a set budget.
    Within,
    /// Over a set budget: the change does not merge.
    Over,
    /// No budget set: reported, never failed (decision 6).
    Unset,
}

pub fn verdict(entry: Option<&Entry>, p95: f64) -> Verdict {
    match entry.and_then(|e| e.budget) {
        Some(b) if p95 > b => Verdict::Over,
        Some(_) => Verdict::Within,
        None => Verdict::Unset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROPOSALS: &str = "[reference]\nmachine = \"unset\"\n\n[operation.startup]\nunit = \"ms\"\nbudget = \"unset\"\nproposed = 20\n";

    #[test]
    fn a_proposal_is_reported_and_never_fails() {
        let b = parse(PROPOSALS).unwrap();
        let e = &b.operations["startup"];
        assert_eq!(e.budget, None);
        assert_eq!(e.proposed, Some(20.0));
        assert_eq!(verdict(Some(e), 1e9), Verdict::Unset);
    }

    #[test]
    fn a_set_budget_fails_a_p95_over_it() {
        let set = "[operation.startup]\nunit = \"ms\"\nbudget = 20\nset_by = \"Arpit\"\nset_on = 2026-11-01\nmeasured_p95 = 14.2\n";
        let b = parse(set).unwrap();
        let e = &b.operations["startup"];
        assert_eq!(verdict(Some(e), 19.9), Verdict::Within);
        assert_eq!(verdict(Some(e), 20.1), Verdict::Over);
    }

    #[test]
    fn a_number_without_its_provenance_is_refused() {
        let bare = "[operation.startup]\nunit = \"ms\"\nbudget = 20\n";
        assert!(parse(bare).unwrap_err().contains("set_by"));
        let word = "[operation.startup]\nunit = \"ms\"\nbudget = \"20\"\n";
        assert!(parse(word).is_err());
        let missing = "[operation.startup]\nunit = \"ms\"\n";
        assert!(parse(missing).unwrap_err().contains("missing"));
    }

    #[test]
    fn the_repo_file_parses_and_names_every_operation() {
        let text = include_str!("../../../../perf/budgets.toml");
        let b = parse(text).unwrap();
        for op in super::super::OPERATIONS {
            assert!(b.operations.contains_key(op.name), "perf/budgets.toml has no [operation.{}]", op.name);
        }
    }
}
