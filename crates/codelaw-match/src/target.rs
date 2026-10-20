//! `target:` + `assert:` rules (SR-RULE-FORMAT decision 3e): paths into a
//! JSON, YAML or TOML file's data tree, and the assertions that must hold
//! at each one.
//!
//! ```yaml
//! target: [dependencies, devDependencies]   # one path or a list
//! assert: { sorted: true }
//! message: "Sort `$PATH` by name"
//! fix: [sort-keys]
//! ```

use crate::data::{DataTree, NodeId, Value};
use crate::schema::{Schema, SchemaError};
use codelaw_rules::{BlockKind, RuleFile};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::Arc;

/// One segment of a path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seg {
    Key(String),
    Index(usize),
    /// `*` or `[*]`: every member of an object, every item of an array.
    Any,
}

/// One concrete step of a resolved path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    Key(String),
    Index(usize),
}

/// A path. `$` is the document root; a path without it starts there too,
/// except inside `path:`, where it starts at the target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub absolute: bool,
    pub segs: Vec<Seg>,
}

impl Path {
    pub fn parse(text: &str) -> Result<Path, String> {
        let bad = |why: &str| format!("invalid path `{text}`: {why}");
        let mut s = text.trim();
        let absolute = s.starts_with('$');
        if absolute {
            s = &s[1..];
            if !(s.is_empty() || s.starts_with('.') || s.starts_with('[')) {
                return Err(bad("`$` is followed by `.` or `[`"));
            }
        }
        let mut segs = Vec::new();
        let b = s.as_bytes();
        let mut i = 0;
        let mut expect_key = !absolute;
        while i < b.len() {
            match b[i] {
                b'.' => {
                    i += 1;
                    expect_key = true;
                    if i >= b.len() {
                        return Err(bad("it ends with `.`"));
                    }
                }
                b'[' => {
                    let end = s[i..].find(']').map(|e| i + e).ok_or_else(|| bad("a `[` is not closed"))?;
                    let inner = s[i + 1..end].trim();
                    if inner == "*" {
                        segs.push(Seg::Any);
                    } else if let Some(q) = inner.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
                        segs.push(Seg::Key(q.to_owned()));
                    } else {
                        segs.push(Seg::Index(inner.parse().map_err(|_| bad("`[…]` holds an index, `*` or a quoted key"))?));
                    }
                    i = end + 1;
                    expect_key = false;
                }
                _ if !expect_key => return Err(bad("keys are separated by `.`")),
                b'"' => {
                    let end = s[i + 1..].find('"').map(|e| i + 1 + e).ok_or_else(|| bad("a `\"` is not closed"))?;
                    segs.push(Seg::Key(s[i + 1..end].to_owned()));
                    i = end + 1;
                    expect_key = false;
                }
                _ => {
                    let end = s[i..].find(['.', '[']).map(|e| i + e).unwrap_or(s.len());
                    let key = &s[i..end];
                    segs.push(if key == "*" { Seg::Any } else { Seg::Key(key.to_owned()) });
                    i = end;
                    expect_key = false;
                }
            }
        }
        if segs.is_empty() && !absolute {
            return Err(bad("it is empty; the root is `$`"));
        }
        Ok(Path { absolute, segs })
    }
}

/// Where a path lands: an existing value, or the place a missing key would
/// go — under `anchor`, the deepest value that exists, as the keys
/// `missing`.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub steps: Vec<Step>,
    pub node: Option<NodeId>,
    pub anchor: NodeId,
    pub missing: Vec<String>,
}

impl Slot {
    /// The path as people read it: `scripts.test`, `jobs.build.steps[0]`.
    pub fn display(&self) -> String {
        display_steps(&self.steps)
    }

    /// The last key of the path, if it ends in one.
    pub fn key(&self) -> Option<&str> {
        match self.steps.last() {
            Some(Step::Key(k)) => Some(k),
            _ => None,
        }
    }
}

pub fn display_steps(steps: &[Step]) -> String {
    let mut out = String::new();
    for s in steps {
        match s {
            Step::Key(k) => {
                if !out.is_empty() {
                    out.push('.');
                }
                if k.is_empty() || k.contains(['.', ' ', '[', ']', '"']) {
                    out.push_str(&format!("\"{k}\""));
                } else {
                    out.push_str(k);
                }
            }
            Step::Index(i) => out.push_str(&format!("[{i}]")),
        }
    }
    if out.is_empty() { "$".into() } else { out }
}

/// Every slot a path reaches from `start`, whose steps are `prefix`.
pub fn resolve(tree: &DataTree, start: NodeId, prefix: &[Step], path: &Path) -> Vec<Slot> {
    let mut out = Vec::new();
    walk(tree, start, prefix.to_vec(), &path.segs, &mut out);
    out
}

fn walk(tree: &DataTree, node: NodeId, steps: Vec<Step>, segs: &[Seg], out: &mut Vec<Slot>) {
    let Some(seg) = segs.first() else {
        out.push(Slot { steps, node: Some(node), anchor: node, missing: Vec::new() });
        return;
    };
    match (seg, &tree.node(node).value) {
        (Seg::Key(k), Value::Object(_)) => match tree.member(node, k) {
            Some(m) => {
                let mut s = steps;
                s.push(Step::Key(k.clone()));
                walk(tree, m.value, s, &segs[1..], out);
            }
            None => {
                // A missing key is a slot only when the rest of the path is
                // keys too: that is where it would be written.
                let keys: Option<Vec<String>> = segs.iter().map(|s| if let Seg::Key(k) = s { Some(k.clone()) } else { None }).collect();
                if let Some(missing) = keys {
                    let mut s = steps;
                    s.extend(missing.iter().cloned().map(Step::Key));
                    out.push(Slot { steps: s, node: None, anchor: node, missing });
                }
            }
        },
        (Seg::Index(i), Value::Array(items)) => {
            if let Some(&item) = items.get(*i) {
                let mut s = steps;
                s.push(Step::Index(*i));
                walk(tree, item, s, &segs[1..], out);
            }
        }
        (Seg::Any, Value::Object(ms)) => {
            for m in ms {
                let mut s = steps.clone();
                s.push(Step::Key(m.key.clone()));
                walk(tree, m.value, s, &segs[1..], out);
            }
        }
        (Seg::Any, Value::Array(items)) => {
            for (i, item) in items.iter().enumerate() {
                let mut s = steps.clone();
                s.push(Step::Index(i));
                walk(tree, *item, s, &segs[1..], out);
            }
        }
        _ => {}
    }
}

/// The value at concrete steps from a document root.
pub fn lookup(tree: &DataTree, root: NodeId, steps: &[Step]) -> Option<NodeId> {
    let mut cur = root;
    for s in steps {
        cur = match (s, &tree.node(cur).value) {
            (Step::Key(k), Value::Object(_)) => tree.member(cur, k)?.value,
            (Step::Index(i), Value::Array(items)) => *items.get(*i)?,
            _ => return None,
        };
    }
    Some(cur)
}

/// How `sorted:` and `sort-keys` order things.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SortSpec {
    /// Keys that come first, in this order; the rest follow sorted.
    pub first: Vec<String>,
}

impl SortSpec {
    pub fn parse(v: &serde_json::Value, what: &str) -> Result<SortSpec, String> {
        match v {
            serde_json::Value::Bool(true) | serde_json::Value::Null => Ok(SortSpec::default()),
            serde_json::Value::Object(o) => {
                for k in o.keys() {
                    if k != "first" {
                        return Err(format!("`{what}` takes `first`, not `{k}`"));
                    }
                }
                let first = match o.get("first") {
                    Some(serde_json::Value::Array(a)) => {
                        a.iter().map(|x| x.as_str().map(str::to_owned).ok_or(format!("`{what}.first` is a list of keys"))).collect::<Result<_, _>>()?
                    }
                    None => Vec::new(),
                    _ => return Err(format!("`{what}.first` is a list of keys")),
                };
                Ok(SortSpec { first })
            }
            _ => Err(format!("`{what}` is `true` or `{{ first: [<key>…] }}`")),
        }
    }

    /// The sort key of a name: listed keys by their place, then the rest by
    /// code point.
    pub fn key<'a>(&self, name: &'a str) -> (usize, &'a str) {
        (self.first.iter().position(|f| f == name).unwrap_or(self.first.len()), name)
    }

    /// The names a container sorts by: an object's keys, or an array's items
    /// when every one is a string.
    pub fn names(tree: &DataTree, node: NodeId) -> Option<Vec<String>> {
        match &tree.node(node).value {
            Value::Object(ms) => Some(ms.iter().map(|m| m.key.clone()).collect()),
            Value::Array(items) => items.iter().map(|i| if let Value::String(s) = &tree.node(*i).value { Some(s.clone()) } else { None }).collect(),
            _ => None,
        }
    }

    pub fn is_sorted(&self, names: &[String]) -> bool {
        names.windows(2).all(|w| self.key(&w[0]) <= self.key(&w[1]))
    }
}

/// A JSON type name, as `type:` and JSON Schema spell them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JType {
    String,
    Number,
    Integer,
    Boolean,
    Null,
    Object,
    Array,
}

impl JType {
    pub fn parse(s: &str) -> Option<JType> {
        Some(match s {
            "string" => JType::String,
            "number" => JType::Number,
            "integer" => JType::Integer,
            "boolean" => JType::Boolean,
            "null" => JType::Null,
            "object" => JType::Object,
            "array" => JType::Array,
            _ => return None,
        })
    }

    pub fn of(v: &Value) -> JType {
        match v {
            Value::Null => JType::Null,
            Value::Bool(_) => JType::Boolean,
            Value::Number(n) if n.integer || (n.value.fract() == 0.0 && n.value.is_finite()) => JType::Integer,
            Value::Number(_) => JType::Number,
            Value::String(_) => JType::String,
            Value::Array(_) => JType::Array,
            Value::Object(_) => JType::Object,
        }
    }

    /// Whether a value of type `self` is a `want`: an integer is a number.
    pub fn is(self, want: JType) -> bool {
        self == want || (self == JType::Integer && want == JType::Number)
    }

    pub fn name(self) -> &'static str {
        match self {
            JType::String => "a string",
            JType::Number => "a number",
            JType::Integer => "an integer",
            JType::Boolean => "a boolean",
            JType::Null => "null",
            JType::Object => "an object",
            JType::Array => "an array",
        }
    }
}

/// An assertion (`assert:`, a fix's `when:`). A map of several keys holds
/// when every one holds. Every check but `present: true` holds where the
/// value is missing.
#[derive(Clone, Debug)]
pub enum Pred {
    Present(bool),
    Type(Vec<JType>),
    Equals(serde_json::Value),
    OneOf(Vec<serde_json::Value>),
    Matches(Regex),
    Sorted(SortSpec),
    Schema(Arc<Schema>),
    Path(Vec<(Path, Pred)>),
    Not(Box<Pred>),
    All(Vec<Pred>),
    Any(Vec<Pred>),
}

pub const PREDICATES: &[&str] = &["present", "type", "equals", "one-of", "matches", "sorted", "schema", "path", "not", "all", "any"];

pub type Tables = BTreeMap<String, Arc<serde_json::Value>>;

impl Pred {
    pub fn parse(v: &serde_json::Value, tables: &Tables) -> Result<Pred, String> {
        let serde_json::Value::Object(o) = v else {
            return Err("an assertion is a map: `{ present: true }`, `{ sorted: true }`, …".into());
        };
        if o.is_empty() {
            return Err("an assertion names at least one check".into());
        }
        let mut all = Vec::new();
        for (k, x) in o {
            all.push(Self::one(k, x, tables)?);
        }
        Ok(if all.len() == 1 { all.pop().expect("one") } else { Pred::All(all) })
    }

    fn one(k: &str, x: &serde_json::Value, tables: &Tables) -> Result<Pred, String> {
        use serde_json::Value as J;
        Ok(match k {
            "present" => Pred::Present(x.as_bool().ok_or("`present:` is `true` or `false`")?),
            "type" => {
                let names: Vec<&str> = match x {
                    J::String(s) => vec![s.as_str()],
                    J::Array(a) => a.iter().map(|t| t.as_str().ok_or("`type:` is a type name or a list of them")).collect::<Result<_, _>>()?,
                    _ => return Err("`type:` is a type name or a list of them".into()),
                };
                Pred::Type(
                    names
                        .into_iter()
                        .map(|n| JType::parse(n).ok_or(format!("unknown type `{n}` (string, number, integer, boolean, null, object, array)")))
                        .collect::<Result<_, _>>()?,
                )
            }
            "equals" => Pred::Equals(x.clone()),
            "one-of" => Pred::OneOf(x.as_array().ok_or("`one-of:` is a list of values")?.clone()),
            "matches" => {
                let s = x.as_str().ok_or("`matches:` is a regular expression")?;
                Pred::Matches(Regex::new(s).map_err(|e| format!("`matches:` is not a valid regular expression: {e}"))?)
            }
            "sorted" => Pred::Sorted(SortSpec::parse(x, "sorted")?),
            "schema" => {
                let name = x.as_str().ok_or("`schema:` names a data table, like `schemas/package`")?;
                let table = tables.get(name).ok_or(format!("`schema: {name}` names no data table of this pack"))?;
                Pred::Schema(Arc::new(Schema::new(table.clone())))
            }
            "path" => {
                let J::Object(o) = x else { return Err("`path:` maps a path to an assertion: `{ extends: { present: true } }`".into()) };
                let mut out = Vec::new();
                for (p, a) in o {
                    out.push((Path::parse(p)?, Pred::parse(a, tables)?));
                }
                Pred::Path(out)
            }
            "not" => Pred::Not(Box::new(Pred::parse(x, tables)?)),
            "all" | "any" => {
                let items = x.as_array().ok_or(format!("`{k}:` is a list of assertions"))?;
                let preds = items.iter().map(|i| Pred::parse(i, tables)).collect::<Result<Vec<_>, _>>()?;
                if k == "all" { Pred::All(preds) } else { Pred::Any(preds) }
            }
            other => {
                let near = PREDICATES.iter().min_by_key(|p| strsim::levenshtein(p, other)).filter(|p| strsim::levenshtein(p, other) <= 2);
                return Err(format!(
                    "unknown assertion `{other}`{} (one of {})",
                    near.map(|p| format!("; did you mean `{p}`?")).unwrap_or_default(),
                    PREDICATES.join(", ")
                ));
            }
        })
    }

    /// Whether the assertion holds at `slot` of the document `root`. Schema
    /// errors are collected into `errors`.
    pub fn holds(&self, tree: &DataTree, root: NodeId, slot: &Slot, errors: &mut Vec<SchemaError>) -> bool {
        let node = slot.node;
        match self {
            Pred::Present(want) => node.is_some() == *want,
            Pred::Not(p) => !p.holds(tree, root, slot, &mut Vec::new()),
            Pred::All(ps) => {
                let mut ok = true;
                for p in ps {
                    ok &= p.holds(tree, root, slot, errors);
                }
                ok
            }
            Pred::Any(ps) => {
                let mut scratch = Vec::new();
                let ok = ps.iter().any(|p| p.holds(tree, root, slot, &mut scratch));
                if !ok {
                    errors.extend(scratch);
                }
                ok
            }
            Pred::Path(pairs) => {
                let mut ok = true;
                for (path, p) in pairs {
                    let slots = if path.absolute {
                        resolve(tree, root, &[], path)
                    } else {
                        match node {
                            Some(n) => resolve(tree, n, &slot.steps, path),
                            // Below a missing value everything is missing.
                            None => {
                                let keys: Option<Vec<String>> =
                                    path.segs.iter().map(|s| if let Seg::Key(k) = s { Some(k.clone()) } else { None }).collect();
                                match keys {
                                    Some(keys) => {
                                        let mut steps = slot.steps.clone();
                                        steps.extend(keys.iter().cloned().map(Step::Key));
                                        let mut missing = slot.missing.clone();
                                        missing.extend(keys);
                                        vec![Slot { steps, node: None, anchor: slot.anchor, missing }]
                                    }
                                    None => Vec::new(),
                                }
                            }
                        }
                    };
                    for s in &slots {
                        ok &= p.holds(tree, root, s, errors);
                    }
                }
                ok
            }
            _ => {
                let Some(n) = node else { return true };
                let v = &tree.node(n).value;
                match self {
                    Pred::Type(ts) => ts.iter().any(|t| JType::of(v).is(*t)),
                    Pred::Equals(want) => json_eq(&tree.to_json(n), want),
                    Pred::OneOf(options) => {
                        let have = tree.to_json(n);
                        options.iter().any(|o| json_eq(&have, o))
                    }
                    Pred::Matches(re) => matches!(v, Value::String(s) if re.is_match(s)),
                    Pred::Sorted(spec) => SortSpec::names(tree, n).is_none_or(|names| spec.is_sorted(&names)),
                    Pred::Schema(schema) => {
                        let before = errors.len();
                        schema.validate(tree, n, &slot.steps, errors);
                        errors.len() == before
                    }
                    _ => unreachable!("handled above"),
                }
            }
        }
    }
}

/// JSON equality with numbers compared by value: `1` equals `1.0`.
pub fn json_eq(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value as J;
    match (a, b) {
        (J::Number(x), J::Number(y)) => x.as_f64() == y.as_f64(),
        (J::Array(x), J::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(a, b)| json_eq(a, b)),
        (J::Object(x), J::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| json_eq(v, w))),
        _ => a == b,
    }
}

/// A compiled `target:` + `assert:` rule.
#[derive(Clone, Debug)]
pub struct CompiledTarget {
    pub paths: Vec<Path>,
    pub assert: Pred,
}

/// One place an assertion failed.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetFinding {
    pub document: NodeId,
    pub slot: Slot,
    pub start: usize,
    pub end: usize,
    /// The schema's reason, when a `schema:` check failed (`$ERROR`).
    pub error: Option<String>,
}

/// Compile a rule's `target:` and `assert:`.
pub fn compile_target(rule: &RuleFile, tables: &Tables) -> Result<CompiledTarget, String> {
    let BlockKind::Target { target, assert } = &rule.block.kind else { return Err("not a `target:` rule".into()) };
    let paths = match target {
        serde_json::Value::String(s) => vec![Path::parse(s)?],
        serde_json::Value::Array(a) if !a.is_empty() => {
            a.iter().map(|p| p.as_str().ok_or("`target:` is a path or a list of paths".to_owned()).and_then(Path::parse)).collect::<Result<_, _>>()?
        }
        _ => return Err("`target:` is a path or a list of paths, like `scripts` or `jobs.*.steps[*]`".into()),
    };
    Ok(CompiledTarget { paths, assert: Pred::parse(assert, tables)? })
}

impl CompiledTarget {
    /// Every place the assertion fails in a file.
    pub fn check(&self, tree: &DataTree) -> Vec<TargetFinding> {
        let mut out = Vec::new();
        for &doc in &tree.documents {
            let mut seen: Vec<Vec<Step>> = Vec::new();
            for path in &self.paths {
                for slot in resolve(tree, doc, &[], path) {
                    if seen.contains(&slot.steps) {
                        continue;
                    }
                    seen.push(slot.steps.clone());
                    let mut errors = Vec::new();
                    if self.assert.holds(tree, doc, &slot, &mut errors) {
                        continue;
                    }
                    if errors.is_empty() {
                        let (start, end) = slot_span(tree, &slot, doc);
                        out.push(TargetFinding { document: doc, slot, start, end, error: None });
                    } else {
                        for e in errors {
                            out.push(TargetFinding { document: doc, slot: slot.clone(), start: e.start, end: e.end, error: Some(e.message) });
                        }
                    }
                }
            }
        }
        out
    }
}

/// Where to report a slot: the key of the member it is, or its value; for a
/// missing key, where its deepest existing parent is.
pub fn slot_span(tree: &DataTree, slot: &Slot, doc: NodeId) -> (usize, usize) {
    let (node, steps): (NodeId, &[Step]) = match slot.node {
        Some(n) => (n, &slot.steps[..]),
        None => (slot.anchor, &slot.steps[..slot.steps.len() - slot.missing.len()]),
    };
    node_span(tree, doc, node, steps)
}

/// A value's report span: its member key, or the first line of its text.
pub fn node_span(tree: &DataTree, doc: NodeId, node: NodeId, steps: &[Step]) -> (usize, usize) {
    if let (Some(Step::Key(k)), Some(parent)) = (steps.last(), steps.len().checked_sub(1).and_then(|n| lookup(tree, doc, &steps[..n])))
        && let Some(ks) = tree.member(parent, k).and_then(|m| m.key_span)
    {
        return ks;
    }
    let (s, e) = tree.node(node).span;
    match tree.node(node).value {
        Value::Object(_) | Value::Array(_) if e > s => (s, s + 1),
        _ => (s, e),
    }
}

/// A message or template with `$PATH`, `$KEY`, `$VALUE` (the value as
/// written) and `$ERROR` (a schema's reason) filled in.
pub fn interpolate(template: &str, tree: &DataTree, slot: &Slot, source: &str, error: Option<&str>) -> String {
    let value = slot.node.map(|n| {
        let (s, e) = tree.node(n).span;
        source.get(s..e).unwrap_or_default()
    });
    template
        .replace("$ERROR", error.unwrap_or_default())
        .replace("$PATH", &slot.display())
        .replace("$KEY", slot.key().unwrap_or_default())
        .replace("$VALUE", value.unwrap_or_default())
}
