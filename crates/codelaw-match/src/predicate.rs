//! The semantic-model predicates of a `where:` block: `unused`,
//! `undeclared`, `shadowed`, `reassigned` and `imported-from`, combined
//! with `not`, `all` and `any`.

use codelaw_semantic::SemanticModel;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Predicate {
    /// A declared name that is never read and not exported.
    Unused,
    /// A reference to a name nothing declares; `globals` are names the rule
    /// accepts besides the language's own.
    Undeclared { globals: BTreeSet<String> },
    /// A declaration that hides one in an enclosing scope; with `builtins`,
    /// hiding a language builtin counts.
    Shadowed { builtins: bool },
    /// A name written after its declaration.
    Reassigned,
    /// A name bound by an import from one of `modules`, optionally of one
    /// imported `name`.
    ImportedFrom { modules: Vec<String>, name: Option<String> },
    Not(Box<Predicate>),
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
}

const NAMES: &[&str] = &["unused", "undeclared", "shadowed", "reassigned", "imported-from", "not", "all", "any"];

fn unknown(name: &str) -> String {
    let near = NAMES
        .iter()
        .map(|n| (strsim_like(name, n), *n))
        .filter(|(s, _)| *s <= 3)
        .min()
        .map(|(_, n)| format!("; did you mean `{n}`?"))
        .unwrap_or_default();
    format!("unknown predicate `{name}`{near} (one of {})", NAMES.join(", "))
}

/// Edit distance, for "did you mean" without another dependency here.
fn strsim_like(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push((prev[j] + usize::from(ca != *cb)).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

fn strings(v: &Value, what: &str) -> Result<Vec<String>, String> {
    match v {
        Value::String(s) => Ok(vec![s.clone()]),
        Value::Array(a) => a.iter().map(|x| x.as_str().map(str::to_owned).ok_or_else(|| format!("`{what}` takes strings"))).collect(),
        _ => Err(format!("`{what}` takes a string or a list of strings")),
    }
}

/// Names in a data table: an array of strings, or the keys of an object.
fn table_names(name: &str, data: &BTreeMap<String, Arc<Value>>) -> Result<Vec<String>, String> {
    match data.get(name).map(|v| v.as_ref()) {
        Some(Value::Array(a)) => Ok(a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()),
        Some(Value::Object(o)) => Ok(o.keys().cloned().collect()),
        Some(_) => Err(format!("data table `{name}` is neither a list of names nor an object")),
        None => Err(format!("no data table `{name}` in the rule's pack (`data/{name}.json`)")),
    }
}

fn parse(v: &Value, data: &BTreeMap<String, Arc<Value>>) -> Result<Predicate, String> {
    match v {
        Value::String(s) => match s.as_str() {
            "unused" => Ok(Predicate::Unused),
            "undeclared" => Ok(Predicate::Undeclared { globals: BTreeSet::new() }),
            "shadowed" => Ok(Predicate::Shadowed { builtins: false }),
            "reassigned" => Ok(Predicate::Reassigned),
            "imported-from" => Err("`imported-from` names a module: `{ imported-from: react }`".into()),
            other => Err(unknown(other)),
        },
        Value::Array(items) => Ok(Predicate::All(items.iter().map(|i| parse(i, data)).collect::<Result<_, _>>()?)),
        Value::Object(map) => {
            if map.len() != 1 {
                return Err("a predicate object has exactly one key".into());
            }
            let (key, arg) = map.iter().next().expect("one key");
            match key.as_str() {
                "not" => Ok(Predicate::Not(Box::new(parse(arg, data)?))),
                "all" | "any" => {
                    let Value::Array(items) = arg else { return Err(format!("`{key}` takes a list")) };
                    let list = items.iter().map(|i| parse(i, data)).collect::<Result<Vec<_>, _>>()?;
                    Ok(if key == "all" { Predicate::All(list) } else { Predicate::Any(list) })
                }
                "imported-from" => match arg {
                    Value::Object(o) => {
                        for k in o.keys() {
                            if k != "module" && k != "name" {
                                return Err(format!("`imported-from` takes `module` and `name`, not `{k}`"));
                            }
                        }
                        let modules = strings(o.get("module").ok_or("`imported-from` needs `module`")?, "module")?;
                        let name = o.get("name").map(|n| n.as_str().map(str::to_owned).ok_or("`name` is a string")).transpose()?;
                        Ok(Predicate::ImportedFrom { modules, name })
                    }
                    other => Ok(Predicate::ImportedFrom { modules: strings(other, "imported-from")?, name: None }),
                },
                "undeclared" => {
                    let Value::Object(o) = arg else { return Err("`undeclared` takes `{ globals: [...], tables: [...] }`".into()) };
                    let mut globals = BTreeSet::new();
                    for (k, v) in o {
                        match k.as_str() {
                            "globals" => globals.extend(strings(v, "globals")?),
                            "tables" => {
                                for t in strings(v, "tables")? {
                                    globals.extend(table_names(&t, data)?);
                                }
                            }
                            other => return Err(format!("`undeclared` takes `globals` and `tables`, not `{other}`")),
                        }
                    }
                    Ok(Predicate::Undeclared { globals })
                }
                "shadowed" => {
                    let builtins = arg.get("builtins").and_then(Value::as_bool).ok_or("`shadowed` takes `{ builtins: true|false }`")?;
                    Ok(Predicate::Shadowed { builtins })
                }
                other => Err(unknown(other)),
            }
        }
        _ => Err("a predicate is a name, a list, or a one-key object".into()),
    }
}

/// Parse a `where:` block: capture name to predicate.
pub fn parse_predicates(v: &Value, data: &BTreeMap<String, Arc<Value>>) -> Result<Vec<(String, Predicate)>, String> {
    let Value::Object(map) = v else { return Err("`where:` maps captures (`$NAME`, `$$MATCH`) to predicates".into()) };
    let mut out = Vec::new();
    for (var, spec) in map {
        if !var.starts_with('$') {
            return Err(format!("`where:` key `{var}` must be a capture like `$NAME`, or `$$MATCH`"));
        }
        out.push((var.clone(), parse(spec, data).map_err(|e| format!("`where: {var}`: {e}"))?));
    }
    Ok(out)
}

impl Predicate {
    /// Whether the predicate holds for the identifier starting at `start`.
    pub fn holds(&self, model: &SemanticModel, start: usize) -> bool {
        match self {
            Predicate::Unused => model.declared_at(start).is_some_and(|s| model.is_unused(s)),
            Predicate::Undeclared { globals } => {
                model.reference_at(start).is_some_and(|r| model.is_undeclared(r, &|n| globals.contains(n)))
            }
            Predicate::Shadowed { builtins } => model.declared_at(start).is_some_and(|s| model.shadowed(s, *builtins)),
            Predicate::Reassigned => model.symbol_at(start).is_some_and(|s| model.is_reassigned(s)),
            Predicate::ImportedFrom { modules, name } => model
                .symbol_at(start)
                .is_some_and(|s| modules.iter().any(|m| model.imported_from(s, m, name.as_deref()))),
            Predicate::Not(p) => !p.holds(model, start),
            Predicate::All(ps) => ps.iter().all(|p| p.holds(model, start)),
            Predicate::Any(ps) => ps.iter().any(|p| p.holds(model, start)),
        }
    }
}
