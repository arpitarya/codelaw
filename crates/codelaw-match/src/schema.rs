//! `schema:` — validate a value against a JSON Schema (draft-07) shipped in
//! a pack's `data/` (SR-RULE-FORMAT decision 3e). Nothing is fetched: a
//! `$ref` to another file accepts any value (L5).
//!
//! What it reads: `$ref` (to `#…` JSON pointers), `type`, `enum`, `const`,
//! `properties`, `patternProperties`, `additionalProperties`, `required`,
//! `minProperties`, `maxProperties`, `propertyNames`, `dependencies`,
//! `items`, `additionalItems`, `minItems`, `maxItems`, `uniqueItems`,
//! `contains`, `minLength`, `maxLength`, `pattern`, `minimum`, `maximum`,
//! `exclusiveMinimum`, `exclusiveMaximum`, `multipleOf`, `allOf`, `anyOf`,
//! `oneOf`, `not`, `if`/`then`/`else`. `format` is an annotation, as
//! draft-07 allows. Two departures, both toward fewer false reports:
//! `oneOf` holds when *at least* one branch does, a `pattern` the `regex`
//! crate cannot compile (look-around, back-references) accepts, and a YAML
//! key with no value at all is not checked.

use crate::data::{DataTree, NodeId, Value};
use crate::target::{JType, Step, display_steps, json_eq, lookup, node_span};
use regex::Regex;
use serde_json::Value as J;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaError {
    pub start: usize,
    pub end: usize,
    pub message: String,
    /// How deep in the document the error is, for choosing among the
    /// branches of an `anyOf`.
    depth: usize,
    /// A wrong type at the value a branch was tried on: that branch was
    /// about another kind of value.
    wrong_type: bool,
}

#[derive(Debug)]
pub struct Schema {
    root: Arc<J>,
    patterns: HashMap<String, Option<Regex>>,
}

const MAX_DEPTH: usize = 64;

fn collect_patterns(v: &J, out: &mut HashMap<String, Option<Regex>>) {
    match v {
        J::Object(o) => {
            for (k, x) in o {
                match (k.as_str(), x) {
                    ("pattern", J::String(p)) => {
                        out.entry(p.clone()).or_insert_with(|| Regex::new(p).ok());
                    }
                    ("patternProperties", J::Object(pp)) => {
                        for (p, s) in pp {
                            out.entry(p.clone()).or_insert_with(|| Regex::new(p).ok());
                            collect_patterns(s, out);
                        }
                    }
                    ("enum" | "const", _) => {}
                    _ => collect_patterns(x, out),
                }
            }
        }
        J::Array(a) => a.iter().for_each(|x| collect_patterns(x, out)),
        _ => {}
    }
}

struct Ctx<'a> {
    tree: &'a DataTree,
    doc: NodeId,
}

impl Schema {
    pub fn new(root: Arc<J>) -> Schema {
        let mut patterns = HashMap::new();
        collect_patterns(&root, &mut patterns);
        Schema { root, patterns }
    }

    /// Validate the value `node`, found at `steps` from its document root.
    pub fn validate(&self, tree: &DataTree, node: NodeId, steps: &[Step], errors: &mut Vec<SchemaError>) {
        let doc = tree.documents.iter().copied().find(|d| lookup(tree, *d, steps) == Some(node)).unwrap_or(node);
        let ctx = Ctx { tree, doc };
        let mut path = steps.to_vec();
        let mut found = Vec::new();
        self.check(&ctx, &self.root, node, &mut path, &mut found, 0);
        found.sort_by(|a, b| (a.start, a.end, &a.message).cmp(&(b.start, b.end, &b.message)));
        found.dedup_by(|a, b| a.start == b.start && a.message == b.message);
        errors.extend(found);
    }

    fn resolve<'s>(&'s self, r: &str) -> Option<&'s J> {
        let pointer = r.strip_prefix('#')?;
        if pointer.is_empty() {
            return Some(&self.root);
        }
        let decoded = percent_decode(pointer);
        self.root.pointer(&decoded)
    }

    fn matches_pattern(&self, p: &str, s: &str) -> bool {
        match self.patterns.get(p) {
            Some(Some(re)) => re.is_match(s),
            _ => true,
        }
    }

    fn err(&self, ctx: &Ctx<'_>, node: NodeId, path: &[Step], message: String) -> SchemaError {
        let (start, end) = node_span(ctx.tree, ctx.doc, node, path);
        SchemaError { start, end, message, depth: path.len(), wrong_type: false }
    }

    fn valid(&self, ctx: &Ctx<'_>, schema: &J, node: NodeId, path: &mut Vec<Step>, depth: usize) -> Result<(), Vec<SchemaError>> {
        let mut e = Vec::new();
        self.check(ctx, schema, node, path, &mut e, depth);
        if e.is_empty() { Ok(()) } else { Err(e) }
    }

    fn check(&self, ctx: &Ctx<'_>, schema: &J, node: NodeId, path: &mut Vec<Step>, out: &mut Vec<SchemaError>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        let o = match schema {
            J::Bool(true) => return,
            J::Bool(false) => {
                out.push(self.err(ctx, node, path, format!("`{}` is not allowed", display_steps(path))));
                return;
            }
            J::Object(o) => o,
            _ => return,
        };
        // Draft-07: a `$ref` stands for the whole schema.
        if let Some(J::String(r)) = o.get("$ref") {
            if let Some(target) = self.resolve(r) {
                self.check(ctx, target, node, path, out, depth + 1);
            }
            return;
        }
        let tree = ctx.tree;
        let value = &tree.node(node).value;
        // A YAML key written with no value (`features:`) is read by most
        // tools as empty, whatever the schema's type; it is not checked.
        let (vs, ve) = tree.node(node).span;
        if tree.language == codelaw_core::Language::Yaml && vs == ve && *value == Value::Null {
            return;
        }
        let here = display_steps(path);
        let have = JType::of(value);

        if let Some(t) = o.get("type") {
            let wanted: Vec<JType> = match t {
                J::String(s) => JType::parse(s).into_iter().collect(),
                J::Array(a) => a.iter().filter_map(|x| x.as_str().and_then(JType::parse)).collect(),
                _ => Vec::new(),
            };
            if !wanted.is_empty() && !wanted.iter().any(|w| have.is(*w)) {
                let names: Vec<&str> = wanted.iter().map(|w| w.name()).collect();
                let mut e = self.err(ctx, node, path, format!("`{here}` should be {}, not {}", or_list(&names), have.name()));
                e.wrong_type = true;
                out.push(e);
                return;
            }
        }
        if let Some(J::Array(options)) = o.get("enum") {
            let j = tree.to_json(node);
            if !options.iter().any(|x| json_eq(&j, x)) {
                let shown: Vec<String> = options.iter().take(8).map(short).collect();
                let more = if options.len() > 8 { ", …" } else { "" };
                out.push(self.err(ctx, node, path, format!("`{here}` should be one of {}{more}", shown.join(", "))));
            }
        }
        if let Some(c) = o.get("const")
            && !json_eq(&tree.to_json(node), c)
        {
            out.push(self.err(ctx, node, path, format!("`{here}` should be {}", short(c))));
        }

        match value {
            Value::Object(members) => self.object(ctx, o, node, members, path, out, depth),
            Value::Array(items) => self.array(ctx, o, node, items, path, out, depth),
            Value::String(s) => {
                let len = s.chars().count() as f64;
                if let Some(min) = o.get("minLength").and_then(J::as_f64)
                    && len < min
                {
                    out.push(self.err(ctx, node, path, format!("`{here}` should be at least {min} characters long")));
                }
                if let Some(max) = o.get("maxLength").and_then(J::as_f64)
                    && len > max
                {
                    out.push(self.err(ctx, node, path, format!("`{here}` should be at most {max} characters long")));
                }
                if let Some(J::String(p)) = o.get("pattern")
                    && !self.matches_pattern(p, s)
                {
                    out.push(self.err(ctx, node, path, format!("`{here}` doesn't match the pattern `{p}`")));
                }
            }
            Value::Number(n) => {
                let v = n.value;
                let bound = |k: &str| o.get(k).and_then(J::as_f64);
                if bound("minimum").is_some_and(|m| v < m) || bound("exclusiveMinimum").is_some_and(|m| v <= m) {
                    out.push(self.err(ctx, node, path, format!("`{here}` is too small")));
                }
                if bound("maximum").is_some_and(|m| v > m) || bound("exclusiveMaximum").is_some_and(|m| v >= m) {
                    out.push(self.err(ctx, node, path, format!("`{here}` is too large")));
                }
                if let Some(m) = bound("multipleOf")
                    && m > 0.0
                    && ((v / m).round() * m - v).abs() > 1e-9
                {
                    out.push(self.err(ctx, node, path, format!("`{here}` should be a multiple of {m}")));
                }
            }
            _ => {}
        }

        if let Some(J::Array(all)) = o.get("allOf") {
            for s in all {
                self.check(ctx, s, node, path, out, depth + 1);
            }
        }
        for k in ["anyOf", "oneOf"] {
            if let Some(J::Array(branches)) = o.get(k) {
                self.any_of(ctx, branches, node, path, out, depth);
            }
        }
        if let Some(n) = o.get("not")
            && self.valid(ctx, n, node, path, depth + 1).is_ok()
        {
            out.push(self.err(ctx, node, path, format!("`{here}` is not allowed in this form")));
        }
        if let Some(cond) = o.get("if") {
            let branch = if self.valid(ctx, cond, node, path, depth + 1).is_ok() { o.get("then") } else { o.get("else") };
            if let Some(b) = branch {
                self.check(ctx, b, node, path, out, depth + 1);
            }
        }
    }

    /// `anyOf` and `oneOf`: valid when one branch is. Otherwise the errors
    /// of the branch that got furthest into the value, or one error naming
    /// the forms when none got past the value itself.
    fn any_of(&self, ctx: &Ctx<'_>, branches: &[J], node: NodeId, path: &mut Vec<Step>, out: &mut Vec<SchemaError>, depth: usize) {
        let mut best: Option<Vec<SchemaError>> = None;
        let mut best_depth = 0;
        for b in branches {
            match self.valid(ctx, b, node, path, depth + 1) {
                Ok(()) => return,
                Err(e) => {
                    let d = e.iter().map(|x| x.depth).max().unwrap_or(0);
                    let relevant = !e.iter().all(|x| x.wrong_type && x.depth == path.len());
                    if relevant && (best.is_none() || d > best_depth || (d == best_depth && best.as_ref().is_some_and(|x| e.len() < x.len()))) {
                        best_depth = d;
                        best = Some(e);
                    }
                }
            }
        }
        match best {
            Some(e) => out.extend(e),
            None => {
                let here = display_steps(path);
                let mut types: Vec<&str> = Vec::new();
                for b in branches {
                    let b = match b.get("$ref").and_then(J::as_str).and_then(|r| self.resolve(r)) {
                        Some(t) => t,
                        None => b,
                    };
                    let ts: Vec<&str> = match b.get("type") {
                        Some(J::String(s)) => vec![s.as_str()],
                        Some(J::Array(a)) => a.iter().filter_map(J::as_str).collect(),
                        _ => Vec::new(),
                    };
                    for t in ts {
                        if let Some(n) = JType::parse(t).map(JType::name)
                            && !types.contains(&n)
                        {
                            types.push(n);
                        }
                    }
                }
                let have = JType::of(&ctx.tree.node(node).value);
                let message = if !types.is_empty() && !types.contains(&have.name()) {
                    format!("`{here}` should be {}, not {}", or_list(&types), have.name())
                } else {
                    format!("`{here}` doesn't match any of the forms the schema allows")
                };
                let mut e = self.err(ctx, node, path, message);
                e.wrong_type = true;
                out.push(e);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn object(
        &self,
        ctx: &Ctx<'_>,
        o: &serde_json::Map<String, J>,
        node: NodeId,
        members: &[crate::data::Member],
        path: &mut Vec<Step>,
        out: &mut Vec<SchemaError>,
        depth: usize,
    ) {
        let here = display_steps(path);
        let props = o.get("properties").and_then(J::as_object);
        let pattern_props = o.get("patternProperties").and_then(J::as_object);
        let additional = o.get("additionalProperties");
        for m in members {
            path.push(Step::Key(m.key.clone()));
            let mut matched = false;
            if let Some(s) = props.and_then(|p| p.get(&m.key)) {
                matched = true;
                self.check(ctx, s, m.value, path, out, depth + 1);
            }
            if let Some(pp) = pattern_props {
                for (p, s) in pp {
                    match self.patterns.get(p) {
                        Some(Some(re)) => {
                            if re.is_match(&m.key) {
                                matched = true;
                                self.check(ctx, s, m.value, path, out, depth + 1);
                            }
                        }
                        // A pattern this engine can't read might match: the
                        // key is not reported as unknown.
                        _ => matched = true,
                    }
                }
            }
            if !matched {
                match additional {
                    Some(J::Bool(false)) => {
                        let near = props
                            .into_iter()
                            .flat_map(|p| p.keys())
                            .filter(|k| strsim::levenshtein(k, &m.key) <= 2.max(m.key.len() / 4))
                            .min_by_key(|k| strsim::levenshtein(k, &m.key));
                        let hint = near.map(|k| format!("; did you mean `{k}`?")).unwrap_or_default();
                        let (start, end) = m.key_span.unwrap_or(ctx.tree.node(m.value).span);
                        let message = if here == "$" { format!("Unknown key `{}`{hint}", m.key) } else { format!("`{here}` has an unknown key `{}`{hint}", m.key) };
                        out.push(SchemaError { start, end, message, depth: path.len(), wrong_type: false });
                    }
                    Some(s @ J::Object(_)) => self.check(ctx, s, m.value, path, out, depth + 1),
                    _ => {}
                }
            }
            if let Some(names) = o.get("propertyNames") {
                let (start, end) = m.key_span.unwrap_or(ctx.tree.node(m.value).span);
                if !self.key_ok(names, &m.key) {
                    out.push(SchemaError { start, end, message: format!("The key `{}` is not an allowed name here", m.key), depth: path.len(), wrong_type: false });
                }
            }
            path.pop();
        }
        let has = |k: &str| members.iter().any(|m| m.key == k);
        if let Some(J::Array(req)) = o.get("required") {
            for r in req.iter().filter_map(J::as_str) {
                if !has(r) {
                    let message = if here == "$" { format!("The required key `{r}` is missing") } else { format!("`{here}` is missing the required key `{r}`") };
                    out.push(self.err(ctx, node, path, message));
                }
            }
        }
        let n = members.len() as f64;
        if o.get("minProperties").and_then(J::as_f64).is_some_and(|m| n < m) {
            out.push(self.err(ctx, node, path, format!("`{here}` needs more keys")));
        }
        if o.get("maxProperties").and_then(J::as_f64).is_some_and(|m| n > m) {
            out.push(self.err(ctx, node, path, format!("`{here}` has too many keys")));
        }
        if let Some(J::Object(deps)) = o.get("dependencies") {
            for (k, d) in deps {
                if !has(k) {
                    continue;
                }
                match d {
                    J::Array(need) => {
                        for r in need.iter().filter_map(J::as_str) {
                            if !has(r) {
                                out.push(self.err(ctx, node, path, format!("`{here}` sets `{k}`, which needs `{r}` too")));
                            }
                        }
                    }
                    s => self.check(ctx, s, node, path, out, depth + 1),
                }
            }
        }
    }

    /// Whether a key satisfies `propertyNames`: only the string keywords
    /// apply to a name.
    fn key_ok(&self, s: &J, key: &str) -> bool {
        let Some(o) = s.as_object() else { return s != &J::Bool(false) };
        if let Some(J::String(r)) = o.get("$ref") {
            return self.resolve(r).is_none_or(|t| self.key_ok(t, key));
        }
        if let Some(J::String(p)) = o.get("pattern")
            && !self.matches_pattern(p, key)
        {
            return false;
        }
        if let Some(J::Array(e)) = o.get("enum")
            && !e.iter().any(|x| x.as_str() == Some(key))
        {
            return false;
        }
        for k in ["anyOf", "oneOf"] {
            if let Some(J::Array(bs)) = o.get(k)
                && !bs.iter().any(|b| self.key_ok(b, key))
            {
                return false;
            }
        }
        if let Some(J::Array(bs)) = o.get("allOf") {
            return bs.iter().all(|b| self.key_ok(b, key));
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn array(&self, ctx: &Ctx<'_>, o: &serde_json::Map<String, J>, node: NodeId, items: &[NodeId], path: &mut Vec<Step>, out: &mut Vec<SchemaError>, depth: usize) {
        let here = display_steps(path);
        match o.get("items") {
            Some(J::Array(tuple)) => {
                for (i, item) in items.iter().enumerate() {
                    path.push(Step::Index(i));
                    match tuple.get(i) {
                        Some(s) => self.check(ctx, s, *item, path, out, depth + 1),
                        None => {
                            if let Some(extra) = o.get("additionalItems") {
                                self.check(ctx, extra, *item, path, out, depth + 1);
                            }
                        }
                    }
                    path.pop();
                }
            }
            Some(s) => {
                for (i, item) in items.iter().enumerate() {
                    path.push(Step::Index(i));
                    self.check(ctx, s, *item, path, out, depth + 1);
                    path.pop();
                }
            }
            None => {}
        }
        let n = items.len() as f64;
        if o.get("minItems").and_then(J::as_f64).is_some_and(|m| n < m) {
            out.push(self.err(ctx, node, path, format!("`{here}` needs at least {} items", o["minItems"])));
        }
        if o.get("maxItems").and_then(J::as_f64).is_some_and(|m| n > m) {
            out.push(self.err(ctx, node, path, format!("`{here}` has more than {} items", o["maxItems"])));
        }
        if o.get("uniqueItems") == Some(&J::Bool(true)) {
            let values: Vec<J> = items.iter().map(|i| ctx.tree.to_json(*i)).collect();
            for (i, v) in values.iter().enumerate() {
                if values[..i].iter().any(|w| json_eq(v, w)) {
                    path.push(Step::Index(i));
                    out.push(self.err(ctx, items[i], path, format!("`{here}` lists {} twice", short(v))));
                    path.pop();
                }
            }
        }
        if let Some(c) = o.get("contains") {
            let any = items.iter().enumerate().any(|(i, item)| {
                path.push(Step::Index(i));
                let ok = self.valid(ctx, c, *item, path, depth + 1).is_ok();
                path.pop();
                ok
            });
            if !any {
                out.push(self.err(ctx, node, path, format!("`{here}` has no item of the kind the schema requires")));
            }
        }
    }
}

fn or_list(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [a] => (*a).to_owned(),
        [init @ .., last] => format!("{} or {last}", init.join(", ")),
    }
}

fn short(v: &J) -> String {
    let s = v.to_string();
    let s = if s.chars().count() > 40 { format!("{}…", s.chars().take(40).collect::<String>()) } else { s };
    format!("`{s}`")
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(h) = s.get(i + 1..i + 3).and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(h);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use codelaw_core::Language;

    fn errors(language: Language, schema: J, src: &str) -> Vec<String> {
        let root = codelaw_lang::parse(language, src);
        let tree = crate::data::build(language, root.root().get_inner_node(), src).unwrap();
        let s = Schema::new(Arc::new(schema));
        let mut e = Vec::new();
        s.validate(&tree, tree.documents[0], &[], &mut e);
        e.into_iter().map(|e| format!("{}..{} {}", e.start, e.end, e.message)).collect()
    }

    #[test]
    fn types_required_and_unknown_keys() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["name"],
            "properties": { "name": { "type": "string", "minLength": 1 }, "strict": { "type": "boolean" } },
            "additionalProperties": false
        });
        assert_eq!(
            errors(Language::Json, schema.clone(), "{\"name\": 1, \"stirct\": true}"),
            vec!["1..7 `name` should be a string, not an integer", "12..20 Unknown key `stirct`; did you mean `strict`?"]
        );
        assert_eq!(errors(Language::Yaml, schema.clone(), "strict: true\n"), vec!["0..1 The required key `name` is missing"]);
        assert!(errors(Language::Toml, schema, "name = \"x\"\nstrict = false\n").is_empty());
    }

    #[test]
    fn refs_any_of_and_if_then() {
        let schema = serde_json::json!({
            "definitions": { "dep": { "anyOf": [ { "type": "string" }, { "type": "object", "properties": { "v": { "type": "string" } }, "required": ["v"] } ] } },
            "properties": {
                "deps": { "type": "object", "additionalProperties": { "$ref": "#/definitions/dep" } },
                "kind": { "enum": ["a", "b"] },
                "ext": { "$ref": "https://example.com/other.json" }
            },
            "if": { "properties": { "kind": { "const": "b" } } },
            "then": { "required": ["deps"] }
        });
        assert!(errors(Language::Json, schema.clone(), "{\"deps\": {\"x\": \"1\", \"y\": {\"v\": \"2\"}}, \"ext\": 5}").is_empty());
        let e = errors(Language::Json, schema.clone(), "{\"deps\": {\"x\": 1, \"y\": {\"w\": 2}}}");
        assert_eq!(e, vec!["10..13 `deps.x` should be a string or an object, not an integer", "18..21 `deps.y` is missing the required key `v`"]);
        let e = errors(Language::Json, schema, "{\"kind\": \"b\"}");
        assert_eq!(e, vec!["0..1 The required key `deps` is missing"]);
    }

    #[test]
    fn patterns_and_unique_items() {
        let schema = serde_json::json!({
            "patternProperties": { "^x-": { "type": "string" }, "(?<=a)b": { "type": "number" } },
            "properties": { "tags": { "type": "array", "uniqueItems": true, "items": { "pattern": "^[a-z]+$" } } }
        });
        let e = errors(Language::Yaml, schema, "x-a: 1\nab: s\ntags: [a, b, a, C]\n");
        assert_eq!(e.len(), 3, "{e:?}");
        assert!(e[0].ends_with("`x-a` should be a string, not an integer"), "{e:?}");
        assert!(e.iter().any(|m| m.contains("lists `\"a\"` twice")), "{e:?}");
        assert!(e.iter().any(|m| m.contains("`tags[3]` doesn't match the pattern")), "{e:?}");
    }
}
