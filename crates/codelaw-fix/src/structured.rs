//! The fixes of a `target:` rule (SR-RULE-FORMAT decision 3c): `sort-keys`,
//! `ensure-key`, `replace` and `remove` act on the target, and a `when:`
//! step picks its `then:` or `else:` steps by an assertion.
//!
//! ```yaml
//! fix:
//!   - when: { path: { $.extends: { present: true } } }
//!     then: []
//!     else: [{ ensure-key: { key: strict, value: true } }]
//! ```
//!
//! Every edit is confined to the target's text: comments and key order
//! outside it stay byte for byte, and a sorted container's comments move
//! with the key they sit above. Each fix is verified before it is offered:
//! the file still parses, the assertion now holds at the target, and the
//! data outside what the fix set out to change is unchanged.

use crate::apply::apply_edits;
use codelaw_core::{Language, TextEdit};
use codelaw_match::data::{DataTree, NodeId, Style, Value};
use codelaw_match::target::{Path, Pred, Seg, Slot, SortSpec, Step, Tables, json_eq, lookup, resolve};
use serde_json::Value as J;

/// One step of a `target:` rule's fix.
#[derive(Clone, Debug)]
pub enum DataStep {
    /// Sort the target object's keys, or the target array's string items.
    SortKeys(SortSpec),
    /// Give the target (or `key`, a path below it) the value `value`:
    /// written in where missing, replaced where different.
    EnsureKey { key: Option<Path>, value: J },
    /// Replace the target's value with a template (`$VALUE`, `$KEY`,
    /// `$PATH`).
    Replace(String),
    /// Remove the target, with its key.
    Remove,
    When { cond: Pred, then: Vec<DataStep>, otherwise: Vec<DataStep> },
}

pub const DATA_PRIMITIVES: &[&str] = &["sort-keys", "ensure-key", "replace", "remove", "when"];

fn only_keys(o: &serde_json::Map<String, J>, keys: &[&str], what: &str) -> Result<(), String> {
    match o.keys().find(|k| !keys.contains(&k.as_str())) {
        Some(k) => Err(format!("`{what}` takes {}, not `{k}`", keys.iter().map(|k| format!("`{k}`")).collect::<Vec<_>>().join(", "))),
        None => Ok(()),
    }
}

fn steps(v: &J, tables: &Tables) -> Result<Vec<DataStep>, String> {
    match v {
        J::Array(items) => items.iter().map(|i| step(i, tables)).collect(),
        J::Null => Ok(Vec::new()),
        other => Ok(vec![step(other, tables)?]),
    }
}

fn step(v: &J, tables: &Tables) -> Result<DataStep, String> {
    if let J::Object(o) = v
        && o.contains_key("when")
    {
        only_keys(o, &["when", "then", "else"], "when")?;
        if !o.contains_key("then") && !o.contains_key("else") {
            return Err("`when:` needs `then:`, `else:` or both".into());
        }
        return Ok(DataStep::When {
            cond: Pred::parse(&o["when"], tables)?,
            then: o.get("then").map(|t| steps(t, tables)).transpose()?.unwrap_or_default(),
            otherwise: o.get("else").map(|t| steps(t, tables)).transpose()?.unwrap_or_default(),
        });
    }
    let (name, arg) = match v {
        J::String(s) => (s.as_str(), &J::Null),
        J::Object(o) if o.len() == 1 => {
            let (k, x) = o.iter().next().expect("one key");
            (k.as_str(), x)
        }
        _ => return Err("each fix step is one primitive: `- sort-keys`, `- ensure-key: { … }`".into()),
    };
    Ok(match name {
        "sort-keys" => DataStep::SortKeys(SortSpec::parse(arg, "sort-keys")?),
        "ensure-key" => {
            let J::Object(o) = arg else { return Err("`ensure-key` takes `{ key, value }`; `key` is optional".into()) };
            only_keys(o, &["key", "value"], "ensure-key")?;
            let key = match o.get("key") {
                Some(J::String(k)) => {
                    let p = Path::parse(k)?;
                    if p.absolute || p.segs.iter().any(|s| !matches!(s, Seg::Key(_))) {
                        return Err("`ensure-key.key` is a path of keys below the target, like `compilerOptions.strict`".into());
                    }
                    Some(p)
                }
                None => None,
                _ => return Err("`ensure-key.key` is a path of keys".into()),
            };
            DataStep::EnsureKey { key, value: o.get("value").cloned().ok_or("`ensure-key` needs `value`")? }
        }
        "replace" => DataStep::Replace(arg.as_str().ok_or("`replace` takes a template: `replace: \"$VALUE\"`")?.to_owned()),
        "remove" if arg.is_null() => DataStep::Remove,
        "remove" => return Err("`remove` takes nothing; it removes the target".into()),
        other => {
            return Err(format!(
                "`{other}` is not a fix primitive of a `target:` rule (one of {})",
                DATA_PRIMITIVES.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")
            ));
        }
    })
}

/// Parse a `target:` rule's `fix:`.
pub fn parse_data_fix(v: &J, tables: &Tables) -> Result<Vec<DataStep>, String> {
    if v.get("external").is_some() {
        return Err("a `target:` rule's fix is a list of `sort-keys`, `ensure-key`, `replace`, `remove` and `when` steps".into());
    }
    steps(v, tables)
}

/// What a fix can see of the file.
pub struct DataFixContext<'a> {
    pub source: &'a str,
    pub language: Language,
    pub tree: &'a DataTree,
    pub document: NodeId,
    /// The rule's assertion: a fix is offered only if it makes it hold.
    pub assert: &'a Pred,
}

/// What a step set out to change: the verifier checks nothing else did.
#[derive(Clone, Debug)]
enum Change {
    /// The order of a container's items.
    Order(Vec<Step>),
    /// The value at a path (written, replaced or removed).
    Value(Vec<Step>),
}

/// The edits of a `target:` rule's fix at one slot, verified.
pub fn data_edits(fix: &[DataStep], ctx: &DataFixContext<'_>, slot: &Slot) -> Result<Vec<TextEdit>, String> {
    let mut edits = Vec::new();
    let mut changes = Vec::new();
    run_steps(fix, ctx, slot, &mut edits, &mut changes)?;
    if edits.is_empty() {
        return Err("the fix has nothing to change here".into());
    }
    edits.sort_by_key(|a| (a.start, a.end));
    edits.dedup();
    verify(ctx, slot, &edits, &changes)?;
    Ok(edits)
}

fn run_steps(steps: &[DataStep], ctx: &DataFixContext<'_>, slot: &Slot, edits: &mut Vec<TextEdit>, changes: &mut Vec<Change>) -> Result<(), String> {
    for s in steps {
        match s {
            DataStep::When { cond, then, otherwise } => {
                let branch = if cond.holds(ctx.tree, ctx.document, slot, &mut Vec::new()) { then } else { otherwise };
                run_steps(branch, ctx, slot, edits, changes)?;
            }
            DataStep::SortKeys(spec) => {
                let node = slot.node.ok_or("`sort-keys`: the target is missing")?;
                edits.push(sort_keys(ctx, node, spec)?);
                changes.push(Change::Order(slot.steps.clone()));
            }
            DataStep::EnsureKey { key, value } => {
                let target = match key {
                    None => slot.clone(),
                    Some(p) => below(ctx, slot, p)?,
                };
                match target.node {
                    Some(n) => {
                        if json_eq(&ctx.tree.to_json(n), value) {
                            continue;
                        }
                        edits.push(replace_value(ctx, n, &render_inline(ctx.language, value)?)?);
                        changes.push(Change::Value(target.steps.clone()));
                    }
                    None => {
                        edits.push(insert(ctx, &target, value)?);
                        let kept = target.steps.len() - target.missing.len() + 1;
                        changes.push(Change::Value(target.steps[..kept].to_vec()));
                    }
                }
            }
            DataStep::Replace(template) => {
                let n = slot.node.ok_or("`replace`: the target is missing")?;
                let text = codelaw_match::target::interpolate(template, ctx.tree, slot, ctx.source, None);
                edits.push(replace_value(ctx, n, &text)?);
                changes.push(Change::Value(slot.steps.clone()));
            }
            DataStep::Remove => {
                if slot.node.is_none() {
                    continue;
                }
                edits.push(remove(ctx, slot)?);
                changes.push(Change::Value(slot.steps.clone()));
            }
        }
    }
    Ok(())
}

/// The slot of a key path below a target.
fn below(ctx: &DataFixContext<'_>, slot: &Slot, p: &Path) -> Result<Slot, String> {
    match slot.node {
        Some(n) => resolve(ctx.tree, n, &slot.steps, p).into_iter().next().ok_or_else(|| "`ensure-key`: the target is not an object".to_owned()),
        None => {
            let keys: Vec<String> = p.segs.iter().filter_map(|s| if let Seg::Key(k) = s { Some(k.clone()) } else { None }).collect();
            let mut s = slot.clone();
            s.steps.extend(keys.iter().cloned().map(Step::Key));
            s.missing.extend(keys);
            Ok(s)
        }
    }
}

// ---- verification ----

fn without(v: &mut J, steps: &[Step]) {
    let Some((last, init)) = steps.split_last() else {
        *v = J::Null;
        return;
    };
    let mut cur = v;
    for s in init {
        cur = match (s, cur) {
            (Step::Key(k), J::Object(o)) => match o.get_mut(k) {
                Some(x) => x,
                None => return,
            },
            (Step::Index(i), J::Array(a)) => match a.get_mut(*i) {
                Some(x) => x,
                None => return,
            },
            _ => return,
        };
    }
    match (last, cur) {
        (Step::Key(k), J::Object(o)) => {
            o.remove(k);
        }
        (Step::Index(i), J::Array(a)) if *i < a.len() => a[*i] = J::Null,
        _ => {}
    }
}

fn at<'v>(v: &'v J, steps: &[Step]) -> Option<&'v J> {
    let mut cur = v;
    for s in steps {
        cur = match (s, cur) {
            (Step::Key(k), J::Object(o)) => o.get(k)?,
            (Step::Index(i), J::Array(a)) => a.get(*i)?,
            _ => return None,
        };
    }
    Some(cur)
}

fn sorted_items(v: Option<&J>) -> Option<Vec<String>> {
    match v? {
        J::Array(a) => {
            let mut s: Vec<String> = a.iter().map(J::to_string).collect();
            s.sort();
            Some(s)
        }
        other => Some(vec![other.to_string()]),
    }
}

fn verify(ctx: &DataFixContext<'_>, slot: &Slot, edits: &[TextEdit], changes: &[Change]) -> Result<(), String> {
    let fixed = apply_edits(ctx.source, edits)?;
    let before_root = codelaw_lang::parse(ctx.language, ctx.source);
    let after_root = codelaw_lang::parse(ctx.language, &fixed);
    if codelaw_lang::syntax_error_count(&after_root) > codelaw_lang::syntax_error_count(&before_root) {
        return Err("the fix would not parse".into());
    }
    let after = codelaw_match::data::build(ctx.language, after_root.root().get_inner_node(), &fixed).ok_or("no data tree")?;
    let index = ctx.tree.documents.iter().position(|d| *d == ctx.document).unwrap_or(0);
    let doc = *after.documents.get(index).ok_or("the fix removes the document")?;
    let path = Path { absolute: true, segs: slot.steps.iter().map(|s| match s { Step::Key(k) => Seg::Key(k.clone()), Step::Index(i) => Seg::Index(*i) }).collect() };
    let new_slot = resolve(&after, doc, &[], &path).into_iter().next().ok_or("the fix moves the target")?;
    if !ctx.assert.holds(&after, doc, &new_slot, &mut Vec::new()) {
        return Err("the fix does not make the assertion hold".into());
    }
    let mut a = ctx.tree.to_json(ctx.document);
    let mut b = after.to_json(doc);
    for c in changes {
        match c {
            Change::Order(steps) => {
                if sorted_items(at(&a, steps)) != sorted_items(at(&b, steps)) && !at(&a, steps).zip(at(&b, steps)).is_some_and(|(x, y)| json_eq(x, y)) {
                    return Err("sorting would change the items".into());
                }
                without(&mut a, steps);
                without(&mut b, steps);
            }
            Change::Value(steps) => {
                without(&mut a, steps);
                without(&mut b, steps);
            }
        }
    }
    if !json_eq(&a, &b) {
        return Err("the fix would change values it does not target".into());
    }
    Ok(())
}

// ---- text helpers ----

fn line_start(src: &str, at: usize) -> usize {
    src[..at].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

/// The offset just past the line `at` is on, its newline included.
fn line_end(src: &str, at: usize) -> usize {
    src[at..].find('\n').map(|i| at + i + 1).unwrap_or(src.len())
}

fn indent_at(src: &str, at: usize) -> &str {
    let s = line_start(src, at);
    let line = &src[s..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

fn same_line(src: &str, a: usize, b: usize) -> bool {
    !src[a.min(b)..a.max(b)].contains('\n')
}

/// The file's indentation step: the smallest indentation it uses, or two
/// spaces.
fn indent_unit(src: &str) -> String {
    let mut best: Option<&str> = None;
    for line in src.lines() {
        let ws = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
        if ws.is_empty() || line.trim().is_empty() {
            continue;
        }
        if ws.starts_with('\t') {
            return "\t".into();
        }
        if best.is_none_or(|b| ws.len() < b.len()) {
            best = Some(ws);
        }
    }
    best.map(str::to_owned).unwrap_or_else(|| "  ".into())
}

// ---- rendering ----

fn json_key(k: &str) -> String {
    serde_json::to_string(k).unwrap_or_default()
}

fn yaml_plain_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ' ' | '@' | '+' | '^' | '~' | '=' | '<' | '>' | '*'))
        && s.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_' || c == '.' || c == '/')
        && !s.ends_with(' ')
        && !matches!(s.to_ascii_lowercase().as_str(), "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "y" | "n" | "~")
        && s.parse::<f64>().is_err()
}

fn yaml_scalar(s: &str) -> String {
    if yaml_plain_ok(s) { s.to_owned() } else { json_key(s) }
}

fn toml_key(k: &str) -> String {
    if !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { k.to_owned() } else { json_key(k) }
}

/// A value written on one line, in the file's language.
fn render_inline(language: Language, v: &J) -> Result<String, String> {
    Ok(match language {
        Language::Json => match v {
            J::Array(a) => format!("[{}]", a.iter().map(|x| render_inline(language, x)).collect::<Result<Vec<_>, _>>()?.join(", ")),
            J::Object(o) if o.is_empty() => "{}".into(),
            J::Object(o) => {
                let parts: Vec<String> = o.iter().map(|(k, x)| Ok(format!("{}: {}", json_key(k), render_inline(language, x)?))).collect::<Result<_, String>>()?;
                format!("{{ {} }}", parts.join(", "))
            }
            other => other.to_string(),
        },
        Language::Yaml => match v {
            J::String(s) => yaml_scalar(s),
            J::Array(a) => format!("[{}]", a.iter().map(|x| render_inline(language, x)).collect::<Result<Vec<_>, _>>()?.join(", ")),
            J::Object(o) => {
                let parts: Vec<String> = o.iter().map(|(k, x)| Ok(format!("{}: {}", yaml_scalar(k), render_inline(language, x)?))).collect::<Result<_, String>>()?;
                format!("{{{}}}", parts.join(", "))
            }
            other => other.to_string(),
        },
        Language::Toml => match v {
            J::Null => return Err("TOML has no null".into()),
            J::String(s) => json_key(s),
            J::Array(a) => format!("[{}]", a.iter().map(|x| render_inline(language, x)).collect::<Result<Vec<_>, _>>()?.join(", ")),
            J::Object(o) if o.is_empty() => "{}".into(),
            J::Object(o) => {
                let parts: Vec<String> = o.iter().map(|(k, x)| Ok(format!("{} = {}", toml_key(k), render_inline(language, x)?))).collect::<Result<_, String>>()?;
                format!("{{ {} }}", parts.join(", "))
            }
            other => other.to_string(),
        },
        other => return Err(format!("{other} is not a data language")),
    })
}

/// A JSON value over several lines, each after the first indented by
/// `indent`.
fn render_json_block(v: &J, indent: &str, unit: &str) -> String {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(unit.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    serde::Serialize::serialize(v, &mut ser).expect("JSON values serialize");
    let text = String::from_utf8(buf).expect("UTF-8");
    text.lines().enumerate().map(|(i, l)| if i == 0 { l.to_owned() } else { format!("{indent}{l}") }).collect::<Vec<_>>().join("\n")
}

/// `key: value` for a YAML block mapping at `indent`, as lines.
fn yaml_member(key: &str, v: &J, indent: &str, unit: &str) -> Result<String, String> {
    match v {
        J::Object(o) if !o.is_empty() => {
            let inner = format!("{indent}{unit}");
            let mut out = format!("{indent}{}:", yaml_scalar(key));
            for (k, x) in o {
                out.push('\n');
                out.push_str(&yaml_member(k, x, &inner, unit)?);
            }
            Ok(out)
        }
        _ => Ok(format!("{indent}{}: {}", yaml_scalar(key), render_inline(Language::Yaml, v)?)),
    }
}

fn nest(keys: &[String], value: &J) -> J {
    keys.iter().rev().fold(value.clone(), |acc, k| {
        let mut m = serde_json::Map::new();
        m.insert(k.clone(), acc);
        J::Object(m)
    })
}

// ---- the primitives ----

fn member_span(ctx: &DataFixContext<'_>, slot: &Slot) -> Option<(usize, usize)> {
    let (Some(Step::Key(k)), Some(n)) = (slot.steps.last(), slot.steps.len().checked_sub(1)) else { return None };
    let parent = lookup(ctx.tree, ctx.document, &slot.steps[..n])?;
    ctx.tree.member(parent, k)?.span
}

fn replace_value(ctx: &DataFixContext<'_>, node: NodeId, text: &str) -> Result<TextEdit, String> {
    let n = ctx.tree.node(node);
    if matches!(n.style, Style::TomlSection { .. } | Style::TomlImplicit) {
        return Err("a TOML table is not replaced in place".into());
    }
    let (s, e) = n.span;
    if s == e {
        // `key:` with no value: write one after the colon.
        return Ok(TextEdit::insert(s, format!(" {text}")));
    }
    Ok(TextEdit::new(s, e, text))
}

/// Write a missing key: below `slot.anchor`, the chain `slot.missing`.
fn insert(ctx: &DataFixContext<'_>, slot: &Slot, value: &J) -> Result<TextEdit, String> {
    let src = ctx.source;
    let unit = indent_unit(src);
    let anchor = ctx.tree.node(slot.anchor);
    let first = slot.missing.first().ok_or("nothing is missing")?;
    let rest = &slot.missing[1..];
    if !matches!(anchor.value, Value::Object(_)) {
        return Err("`ensure-key`: the parent is not an object".into());
    }
    let members = match &anchor.value {
        Value::Object(ms) => ms,
        _ => unreachable!(),
    };
    let (s, e) = anchor.span;
    match anchor.style {
        Style::JsonObject => {
            let v = nest(rest, value);
            let last = members.last().and_then(|m| m.span);
            match last {
                None => {
                    let inner = &src[s + 1..e - 1];
                    let close = indent_at(src, s).to_owned();
                    let mi = format!("{close}{unit}");
                    let text = format!("{}: {}", json_key(first), render_json_block(&v, &mi, &unit));
                    if inner.trim().is_empty() {
                        Ok(TextEdit::new(s + 1, e - 1, format!("\n{mi}{text}\n{close}")))
                    } else {
                        Ok(TextEdit::insert(s + 1, format!("\n{mi}{text}")))
                    }
                }
                Some((ms, me)) => {
                    let after = &src[me..e];
                    let trailing_comma = after.trim_start().starts_with(',');
                    if same_line(src, s, ms) {
                        let text = format!("{}: {}", json_key(first), render_inline(Language::Json, &v)?);
                        return Ok(TextEdit::insert(me, format!(", {text}")));
                    }
                    let mi = indent_at(src, ms).to_owned();
                    let text = format!("{}: {}", json_key(first), render_json_block(&v, &mi, &unit));
                    if trailing_comma {
                        let comma = me + after.find(',').expect("a comma");
                        Ok(TextEdit::insert(comma + 1, format!("\n{mi}{text},")))
                    } else {
                        Ok(TextEdit::insert(me, format!(",\n{mi}{text}")))
                    }
                }
            }
        }
        Style::YamlBlockMapping => {
            let v = nest(rest, value);
            let (ms, me) = members.last().and_then(|m| m.span).ok_or("`ensure-key`: the mapping's keys are not all written in it")?;
            let first_start = members.first().and_then(|m| m.span).map(|x| x.0).unwrap_or(ms);
            let col = first_start - line_start(src, first_start);
            let mi = if src[line_start(src, first_start)..first_start].trim().is_empty() { src[line_start(src, first_start)..first_start].to_owned() } else { " ".repeat(col) };
            let at = line_end(src, me);
            let lead = if at == src.len() && !src.ends_with('\n') { "\n" } else { "" };
            Ok(TextEdit::insert(at, format!("{lead}{}\n", yaml_member(first, &v, &mi, &unit)?)))
        }
        Style::YamlFlowMapping | Style::TomlInlineTable => {
            let v = nest(rest, value);
            let sep = if ctx.language == Language::Toml { " = " } else { ": " };
            let key = if ctx.language == Language::Toml { toml_key(first) } else { yaml_scalar(first) };
            let text = format!("{key}{sep}{}", render_inline(ctx.language, &v)?);
            match members.last().and_then(|m| m.span) {
                Some((_, me)) => Ok(TextEdit::insert(me, format!(", {text}"))),
                None if ctx.language == Language::Toml => Ok(TextEdit::new(s, e, format!("{{ {text} }}"))),
                None => Ok(TextEdit::new(s, e, format!("{{{text}}}"))),
            }
        }
        Style::TomlSection { header, end } => {
            let is_root = header.is_none();
            if is_root && (!rest.is_empty() || matches!(value, J::Object(o) if !o.is_empty() && o.values().all(|x| !x.is_object()))) {
                // A new table goes at the end of the file, as `[a.b]`.
                let (table_keys, pairs): (Vec<String>, J) = if rest.is_empty() {
                    (vec![first.clone()], value.clone())
                } else {
                    let mut keys = vec![first.clone()];
                    keys.extend(rest[..rest.len() - 1].iter().cloned());
                    let mut m = serde_json::Map::new();
                    m.insert(rest.last().expect("not empty").clone(), value.clone());
                    (keys, J::Object(m))
                };
                let J::Object(pairs) = pairs else { unreachable!() };
                let mut text = String::new();
                if !src.is_empty() {
                    if !src.ends_with('\n') {
                        text.push('\n');
                    }
                    if !src.ends_with("\n\n") {
                        text.push('\n');
                    }
                }
                text.push_str(&format!("[{}]\n", table_keys.iter().map(|k| toml_key(k)).collect::<Vec<_>>().join(".")));
                for (k, x) in &pairs {
                    text.push_str(&format!("{} = {}\n", toml_key(k), render_inline(Language::Toml, x)?));
                }
                return Ok(TextEdit::insert(src.len(), text));
            }
            let key: Vec<String> = std::iter::once(first).chain(rest.iter()).map(|k| toml_key(k)).collect();
            let line = format!("{} = {}\n", key.join("."), render_inline(Language::Toml, value)?);
            let at = if is_root && members.iter().all(|m| m.span.is_none()) { 0 } else { line_end(src, end) };
            let lead = if at == src.len() && !src.is_empty() && !src.ends_with('\n') { "\n" } else { "" };
            Ok(TextEdit::insert(at, format!("{lead}{line}")))
        }
        Style::TomlImplicit => Err("`ensure-key`: this TOML table is defined only by its sub-tables; add a `[table]` for it first".into()),
        _ => Err("`ensure-key`: this object is not written in one place (an alias or merged keys)".into()),
    }
}

/// Remove a member, or an item, and what separates it from its neighbours.
fn remove(ctx: &DataFixContext<'_>, slot: &Slot) -> Result<TextEdit, String> {
    let src = ctx.source;
    let node = slot.node.ok_or("nothing to remove")?;
    let parent_steps = &slot.steps[..slot.steps.len().saturating_sub(1)];
    let parent = lookup(ctx.tree, ctx.document, parent_steps).filter(|_| !slot.steps.is_empty()).ok_or("`remove`: the root is not removed")?;
    let (s, e) = match slot.steps.last() {
        Some(Step::Key(_)) => member_span(ctx, slot).ok_or("`remove`: this key is not written in one place")?,
        _ => ctx.tree.node(node).span,
    };
    let style = ctx.tree.node(parent).style;
    let line_based = matches!(style, Style::YamlBlockMapping | Style::YamlBlockSequence | Style::TomlSection { .. });
    if line_based {
        let ls = line_start(src, s);
        let prefix = src[ls..s].trim();
        if !(prefix.is_empty() || prefix == "-") {
            return Err("`remove`: the item shares its line".into());
        }
        return Ok(TextEdit::delete(ls, line_end(src, e)));
    }
    if !matches!(style, Style::JsonObject | Style::JsonArray | Style::YamlFlowMapping | Style::YamlFlowSequence | Style::TomlInlineTable | Style::TomlArray) {
        return Err("`remove`: this container is not written in one place".into());
    }
    let (mut s, mut e) = (s, e);
    let after = &src[e..];
    let ws = after.len() - after.trim_start().len();
    if after.trim_start().starts_with(',') {
        e += ws + 1;
        e += src[e..].bytes().take_while(|b| *b == b' ' || *b == b'\t').count();
    } else {
        let before = &src[..s];
        let trimmed = before.trim_end();
        if trimmed.ends_with(',') {
            s = trimmed.len() - 1;
        }
    }
    let ls = line_start(src, s);
    if src[ls..s].trim().is_empty() {
        let le = line_end(src, e);
        if src[e..le].trim().is_empty() {
            return Ok(TextEdit::delete(ls, le));
        }
    }
    Ok(TextEdit::delete(s, e))
}

/// `sort-keys`: one edit spanning the container's items, rewritten in
/// order; what lies between them stays where it was.
fn sort_keys(ctx: &DataFixContext<'_>, node: NodeId, spec: &SortSpec) -> Result<TextEdit, String> {
    let src = ctx.source;
    let n = ctx.tree.node(node);
    let names = SortSpec::names(ctx.tree, node).ok_or("`sort-keys` sorts an object, or an array of strings")?;
    let spans: Vec<(usize, usize)> = match &n.value {
        Value::Object(ms) => ms.iter().map(|m| m.span).collect::<Option<_>>().ok_or("`sort-keys`: some keys are not written in this table")?,
        Value::Array(items) => items.iter().map(|i| ctx.tree.node(*i).span).collect(),
        _ => unreachable!("names() said it is a container"),
    };
    let mut seen = std::collections::BTreeSet::new();
    if matches!(n.value, Value::Object(_)) && !names.iter().all(|k| seen.insert(k)) {
        return Err("`sort-keys`: a key is written twice".into());
    }
    let mut order: Vec<usize> = (0..names.len()).collect();
    order.sort_by(|a, b| spec.key(&names[*a]).cmp(&spec.key(&names[*b])));
    if order.iter().enumerate().all(|(i, o)| i == *o) {
        return Err("already sorted".into());
    }
    let line_mode = matches!(n.style, Style::YamlBlockMapping | Style::YamlBlockSequence | Style::TomlSection { .. });
    let inline_mode = matches!(n.style, Style::JsonObject | Style::JsonArray | Style::YamlFlowMapping | Style::YamlFlowSequence | Style::TomlInlineTable | Style::TomlArray);
    if line_mode {
        sort_lines(src, &spans, &order, n.style == Style::YamlBlockSequence)
    } else if inline_mode {
        sort_inline(ctx, &spans, &order)
    } else {
        Err("`sort-keys`: this container is not written in one place (an alias or merged keys)".into())
    }
}

/// Items one per line (YAML block collections, TOML tables): each moves
/// with its whole lines and the comment lines just above it at its own
/// indentation.
fn sort_lines(src: &str, spans: &[(usize, usize)], order: &[usize], sequence: bool) -> Result<TextEdit, String> {
    let mut chunks: Vec<(usize, usize)> = Vec::new();
    let mut prev_end = 0;
    for (i, &(s, e)) in spans.iter().enumerate() {
        let mut ls = line_start(src, s);
        let prefix = src[ls..s].trim();
        if !(prefix.is_empty() || (sequence && prefix == "-")) {
            return Err("`sort-keys`: an item shares its line with another".into());
        }
        let indent = &src[ls..ls + (src[ls..s].len() - src[ls..s].trim_start().len())];
        let le = line_end(src, e);
        // Comment lines directly above, at this indentation, come along.
        let floor = if i == 0 { 0 } else { prev_end };
        while ls > floor {
            let pl = line_start(src, ls - 1);
            let line = &src[pl..ls];
            let t = line.trim_start_matches([' ', '\t']);
            if t.starts_with('#') && line.len() - t.len() == indent.len() && pl >= floor {
                ls = pl;
            } else {
                break;
            }
        }
        if i > 0 && ls < prev_end {
            return Err("`sort-keys`: two items share a line".into());
        }
        chunks.push((ls, le));
        prev_end = le;
    }
    let start = chunks[0].0;
    let end = chunks.last().expect("items").1;
    let no_final_newline = !src[..end].ends_with('\n');
    let texts: Vec<String> = chunks.iter().map(|&(s, e)| {
        let t = &src[s..e];
        if t.ends_with('\n') { t.to_owned() } else { format!("{t}\n") }
    }).collect();
    let mut out = String::new();
    for (slot, &(cs, _)) in chunks.iter().enumerate() {
        if slot > 0 {
            out.push_str(&src[chunks[slot - 1].1..cs]);
        }
        out.push_str(&texts[order[slot]]);
    }
    if no_final_newline {
        out.pop();
    }
    Ok(TextEdit::new(start, end, out))
}

/// Items in brackets (JSON, YAML flow, TOML inline): the item texts trade
/// places; separators and line breaks stay. Comments between items would be
/// left behind, so they refuse the fix.
fn sort_inline(ctx: &DataFixContext<'_>, spans: &[(usize, usize)], order: &[usize]) -> Result<TextEdit, String> {
    let src = ctx.source;
    for w in spans.windows(2) {
        let gap = &src[w[0].1..w[1].0];
        if !gap.chars().all(|c| c == ',' || c.is_whitespace()) {
            return Err("`sort-keys`: comments between the items would be left behind".into());
        }
    }
    let start = spans[0].0;
    let end = spans.last().expect("items").1;
    let mut out = String::new();
    for (slot, &(s, _)) in spans.iter().enumerate() {
        if slot > 0 {
            out.push_str(&src[spans[slot - 1].1..s]);
        }
        let (a, b) = spans[order[slot]];
        out.push_str(&src[a..b]);
    }
    Ok(TextEdit::new(start, end, out))
}

#[cfg(test)]
#[path = "structured_tests.rs"]
mod tests;
