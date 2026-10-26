//! The semantic-model predicates of a `where:` block: `unused`,
//! `undeclared`, `shadowed`, `reassigned`, `imported-from`, `from_source`
//! and the rest, combined with `not`, `all` and `any`.

use crate::source::{SourcePath, from_source, parse_source};
use codelaw_rules::{CallItem, OptionSpec, parse_call_items, resolve_calls};
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
    /// The identifier is assigned to here.
    Write,
    /// The identifier declares a symbol.
    Declaration,
    /// The identifier declares a symbol its scope already declared.
    Redeclared,
    /// The identifier names a symbol declared as one of `kinds` (`var`,
    /// `let`, `const`, `function`, `class`, `param`, `import`,
    /// `catch-param`, `interface`, `type-alias`, `enum`, `namespace`,
    /// `type-param`, `variable`).
    Binding { kinds: Vec<String> },
    /// A reference to one of the language's own globals that the file does
    /// not declare.
    Builtin,
    /// The node is a request source, is built from one, or is a local name
    /// bound to one in the same function (`source.rs`; not taint
    /// analysis).
    FromSource { sources: Vec<SourcePath> },
    /// The node is a call to one of the rule's `call[]` option `option`
    /// (W-73); `default` is the rule's built-in list.
    Calls { option: String, default: Vec<CallItem> },
    /// The node is a function, or in one, that contains a call to one of
    /// the rule's `call[]` option `option` (W-73).
    FunctionCalls { option: String, default: Vec<CallItem> },
    Not(Box<Predicate>),
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
}

const NAMES: &[&str] = &[
    "unused", "undeclared", "shadowed", "reassigned", "imported-from", "write", "declaration", "redeclared", "binding", "builtin", "from_source",
    "calls", "function_calls", "not", "all", "any",
];

impl Predicate {
    /// Whether the predicate is about a call list: such an entry on a
    /// capture the matched branch did not bind is skipped, since that
    /// branch has no value to check (a rule whose `any:` binds `$V` in some
    /// branches only).
    pub fn skips_unbound(&self) -> bool {
        match self {
            Predicate::Calls { .. } | Predicate::FunctionCalls { .. } => true,
            Predicate::Not(p) => p.skips_unbound(),
            Predicate::All(ps) | Predicate::Any(ps) => ps.iter().all(Predicate::skips_unbound),
            _ => false,
        }
    }
}

const KINDS: &[&str] = &[
    "var", "let", "const", "function", "class", "param", "import", "catch-param", "interface", "type-alias", "enum", "namespace",
    "type-param", "variable",
];

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

fn parse(v: &Value, data: &BTreeMap<String, Arc<Value>>, opts: &BTreeMap<String, OptionSpec>) -> Result<Predicate, String> {
    match v {
        Value::String(s) => match s.as_str() {
            "unused" => Ok(Predicate::Unused),
            "undeclared" => Ok(Predicate::Undeclared { globals: BTreeSet::new() }),
            "shadowed" => Ok(Predicate::Shadowed { builtins: false }),
            "reassigned" => Ok(Predicate::Reassigned),
            "write" => Ok(Predicate::Write),
            "declaration" => Ok(Predicate::Declaration),
            "redeclared" => Ok(Predicate::Redeclared),
            "builtin" => Ok(Predicate::Builtin),
            "imported-from" => Err("`imported-from` names a module: `{ imported-from: react }`".into()),
            "binding" => Err("`binding` names declaration kinds: `{ binding: [const, let] }`".into()),
            "calls" => Err("`calls` names a `call[]` option of the rule: `{ calls: sanitizers }`".into()),
            "function_calls" => Err("`function_calls` names a `call[]` option of the rule: `{ function_calls: validators }`".into()),
            "from_source" => Err("`from_source` names its sources: `{ from_source: { sources: [req.query], tables: [sources/express] } }`".into()),
            other => Err(unknown(other)),
        },
        Value::Array(items) => Ok(Predicate::All(items.iter().map(|i| parse(i, data, opts)).collect::<Result<_, _>>()?)),
        Value::Object(map) => {
            if map.len() != 1 {
                return Err("a predicate object has exactly one key".into());
            }
            let (key, arg) = map.iter().next().expect("one key");
            match key.as_str() {
                "not" => Ok(Predicate::Not(Box::new(parse(arg, data, opts)?))),
                "all" | "any" => {
                    let Value::Array(items) = arg else { return Err(format!("`{key}` takes a list")) };
                    let list = items.iter().map(|i| parse(i, data, opts)).collect::<Result<Vec<_>, _>>()?;
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
                "calls" | "function_calls" => {
                    let option = arg.as_str().ok_or_else(|| format!("`{key}` names one `call[]` option: `{{ {key}: validators }}`"))?;
                    let Some(spec) = opts.get(option).filter(|s| s.kind == "call[]") else {
                        return Err(format!("`{key}: {option}` names no `call[]` option of this rule (declare `{option}: {{ type: \"call[]\" }}` in `options:`)"));
                    };
                    let default = spec.default.as_ref().map(parse_call_items).transpose()?.unwrap_or_default();
                    let option = option.to_owned();
                    Ok(if key == "calls" { Predicate::Calls { option, default } } else { Predicate::FunctionCalls { option, default } })
                }
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
                "from_source" => {
                    let mut names = Vec::new();
                    match arg {
                        Value::Object(o) => {
                            for (k, v) in o {
                                match k.as_str() {
                                    "sources" => names.extend(strings(v, "sources")?),
                                    "tables" => {
                                        for t in strings(v, "tables")? {
                                            names.extend(table_names(&t, data)?);
                                        }
                                    }
                                    other => return Err(format!("`from_source` takes `sources` and `tables`, not `{other}`")),
                                }
                            }
                        }
                        other => names.extend(strings(other, "from_source")?),
                    }
                    if names.is_empty() {
                        return Err("`from_source` names at least one source".into());
                    }
                    let sources = names
                        .iter()
                        .map(|n| parse_source(n).ok_or_else(|| format!("`from_source`: `{n}` is not a dotted access path like `req.query`")))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Predicate::FromSource { sources })
                }
                "binding" => {
                    let kinds = strings(arg, "binding")?;
                    for k in &kinds {
                        if !KINDS.contains(&k.as_str()) {
                            return Err(format!("`binding: {k}` is not a declaration kind (one of {})", KINDS.join(", ")));
                        }
                    }
                    Ok(Predicate::Binding { kinds })
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
pub fn parse_predicates(v: &Value, data: &BTreeMap<String, Arc<Value>>, opts: &BTreeMap<String, OptionSpec>) -> Result<Vec<(String, Predicate)>, String> {
    let Value::Object(map) = v else { return Err("`where:` maps captures (`$NAME`, `$$MATCH`) to predicates".into()) };
    let mut out = Vec::new();
    for (var, spec) in map {
        if !var.starts_with('$') {
            return Err(format!("`where:` key `{var}` must be a capture like `$NAME`, or `$$MATCH`"));
        }
        out.push((var.clone(), parse(spec, data, opts).map_err(|e| format!("`where: {var}`: {e}"))?));
    }
    Ok(out)
}

/// What a predicate is asked about: the captured node, and the source of
/// the tree it belongs to.
#[derive(Clone, Copy)]
pub struct Subject<'a> {
    pub start: usize,
    pub node: Option<tree_sitter::Node<'a>>,
    pub source: &'a str,
    /// The rule's options as the project resolved them, for `calls`.
    pub options: &'a BTreeMap<String, Value>,
}

impl Predicate {
    /// Whether the predicate holds for the captured node.
    pub fn holds(&self, model: &SemanticModel, subject: &Subject<'_>) -> bool {
        let start = subject.start;
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
            Predicate::Write => model.is_write_at(start),
            Predicate::Declaration => model.is_declaration_at(start),
            Predicate::Redeclared => model.is_redeclaration_at(start),
            Predicate::Binding { kinds } => model.symbol_at(start).is_some_and(|s| kinds.iter().any(|k| k == model.symbol(s).kind.name())),
            Predicate::Builtin => model.reference_at(start).is_some_and(|r| model.is_builtin_reference(r)),
            Predicate::FromSource { sources } => subject.node.is_some_and(|n| from_source(n, subject.source, model, sources)),
            Predicate::Calls { option, default } => subject
                .node
                .is_some_and(|n| crate::calls::is_call_to(n, subject.source, model, &resolve_calls(default, subject.options, option))),
            Predicate::FunctionCalls { option, default } => subject
                .node
                .is_some_and(|n| crate::calls::function_calls(n, subject.source, model, &resolve_calls(default, subject.options, option))),
            Predicate::Not(p) => !p.holds(model, subject),
            Predicate::All(ps) => ps.iter().all(|p| p.holds(model, subject)),
            Predicate::Any(ps) => ps.iter().any(|p| p.holds(model, subject)),
        }
    }
}
