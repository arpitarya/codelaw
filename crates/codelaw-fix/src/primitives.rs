//! The declarative edit primitives of a rule's `fix:` (DESIGN §3.3,
//! SR-RULE-FORMAT decision 3). Each turns a match into LSP-style text
//! edits; only the edited ranges change (L4 decision 6).

use codelaw_core::{Language, TextEdit};
use codelaw_lang::Root;
use codelaw_match::{Match, interpolate};
use codelaw_semantic::SemanticModel;
use serde_json::Value;

/// What a primitive acts on: the matched node, or one capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Match,
    Capture(String),
}

/// Where `move-to` and `collect-into-block` put text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    EndOfFile,
    StartOfFile,
    AfterImports,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Primitive {
    Replace { target: Target, with: String },
    Remove { target: Target },
    InsertBefore { target: Target, text: String },
    InsertAfter { target: Target, text: String },
    StripExportKeyword { target: Target },
    /// Gathers the names every match declares into one block. `$$$NAMES`
    /// in `block` is the value names; `$$$TYPE_NAMES` in `type_block` the
    /// type-only names (interfaces and type aliases).
    CollectIntoBlock { target: Target, block: String, type_block: Option<String>, position: Place },
    MoveTo { target: Target, position: Place },
    /// Rename a symbol and every reference to it in this file. Refused for
    /// an exported symbol: importers elsewhere would break.
    RenameSymbolLocal { target: Target, to: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixSpec {
    Steps(Vec<Primitive>),
    /// `{ external: "<command>" }` (SR-ARCHITECTURE decision 18).
    External(String),
}

impl FixSpec {
    /// A fix with `collect-into-block` is computed once over every match in
    /// the file; each diagnostic then carries the whole edit set.
    pub fn is_group(&self) -> bool {
        matches!(self, FixSpec::Steps(s) if s.iter().any(|p| matches!(p, Primitive::CollectIntoBlock { .. })))
    }
}

const PRIMITIVES: &[&str] =
    &["replace", "remove", "insert-before", "insert-after", "strip-export-keyword", "collect-into-block", "move-to", "rename-symbol-local"];

fn target(v: &Value) -> Result<Target, String> {
    match v.as_str() {
        Some("match") | Some("$$MATCH") => Ok(Target::Match),
        Some(s) if s.starts_with('$') && s.len() > 1 => Ok(Target::Capture(s.trim_start_matches('$').to_owned())),
        _ => Err(format!("a fix target is `match` or a capture like `$DECL`, not `{v}`")),
    }
}

fn place(v: &Value) -> Result<Place, String> {
    match v.as_str() {
        Some("end-of-file") => Ok(Place::EndOfFile),
        Some("start-of-file") => Ok(Place::StartOfFile),
        Some("after-imports") => Ok(Place::AfterImports),
        _ => Err(format!("a position is `end-of-file`, `start-of-file` or `after-imports`, not `{v}`")),
    }
}

fn field<'a>(o: &'a serde_json::Map<String, Value>, key: &str, primitive: &str) -> Result<&'a Value, String> {
    o.get(key).ok_or_else(|| format!("`{primitive}` needs `{key}`"))
}

fn string(v: &Value, what: &str) -> Result<String, String> {
    v.as_str().map(str::to_owned).ok_or_else(|| format!("`{what}` is a string"))
}

fn only_keys(o: &serde_json::Map<String, Value>, keys: &[&str], primitive: &str) -> Result<(), String> {
    for k in o.keys() {
        if !keys.contains(&k.as_str()) {
            return Err(format!("`{primitive}` takes {}, not `{k}`", keys.iter().map(|k| format!("`{k}`")).collect::<Vec<_>>().join(", ")));
        }
    }
    Ok(())
}

fn primitive(item: &Value) -> Result<Primitive, String> {
    let (name, arg) = match item {
        Value::String(s) => (s.as_str(), &Value::Null),
        Value::Object(o) if o.len() == 1 => {
            let (k, v) = o.iter().next().expect("one key");
            (k.as_str(), v)
        }
        _ => return Err("each fix step is one primitive: `- remove: $X`".into()),
    };
    let target_or_match = |v: &Value| if v.is_null() { Ok(Target::Match) } else { target(v) };
    Ok(match name {
        "replace" => match arg {
            Value::String(s) => Primitive::Replace { target: Target::Match, with: s.clone() },
            Value::Object(o) => {
                only_keys(o, &["target", "with"], name)?;
                Primitive::Replace { target: target(field(o, "target", name)?)?, with: string(field(o, "with", name)?, "with")? }
            }
            _ => return Err("`replace` takes a template, or `{ target, with }`".into()),
        },
        "remove" => Primitive::Remove { target: target_or_match(arg)? },
        "strip-export-keyword" => Primitive::StripExportKeyword { target: target_or_match(arg)? },
        "insert-before" | "insert-after" => {
            let (t, text) = match arg {
                Value::String(s) => (Target::Match, s.clone()),
                Value::Object(o) => {
                    only_keys(o, &["target", "text"], name)?;
                    (target(field(o, "target", name)?)?, string(field(o, "text", name)?, "text")?)
                }
                _ => return Err(format!("`{name}` takes text, or `{{ target, text }}`")),
            };
            if name == "insert-before" { Primitive::InsertBefore { target: t, text } } else { Primitive::InsertAfter { target: t, text } }
        }
        "collect-into-block" => {
            let Value::Object(o) = arg else { return Err("`collect-into-block` takes `{ block, type_block, position }`".into()) };
            only_keys(o, &["target", "block", "type_block", "position"], name)?;
            Primitive::CollectIntoBlock {
                target: o.get("target").map(target).transpose()?.unwrap_or(Target::Match),
                block: string(field(o, "block", name)?, "block")?,
                type_block: o.get("type_block").map(|v| string(v, "type_block")).transpose()?,
                position: o.get("position").map(place).transpose()?.unwrap_or(Place::EndOfFile),
            }
        }
        "move-to" => match arg {
            Value::String(_) => Primitive::MoveTo { target: Target::Match, position: place(arg)? },
            Value::Object(o) => {
                only_keys(o, &["target", "position"], name)?;
                Primitive::MoveTo { target: target(field(o, "target", name)?)?, position: place(field(o, "position", name)?)? }
            }
            _ => return Err("`move-to` takes a position, or `{ target, position }`".into()),
        },
        "rename-symbol-local" => {
            let Value::Object(o) = arg else { return Err("`rename-symbol-local` takes `{ target, to }`".into()) };
            only_keys(o, &["target", "to"], name)?;
            Primitive::RenameSymbolLocal { target: target(field(o, "target", name)?)?, to: string(field(o, "to", name)?, "to")? }
        }
        other => {
            let near = PRIMITIVES
                .iter()
                .find(|p| p.starts_with(&other[..other.len().min(4)]))
                .map(|p| format!("; did you mean `{p}`?"))
                .unwrap_or_default();
            return Err(format!("unknown fix primitive `{other}`{near} (one of {})", PRIMITIVES.join(", ")));
        }
    })
}

/// Parse a rule's `fix:` value.
pub fn parse_fix(v: &Value) -> Result<FixSpec, String> {
    match v {
        Value::String(s) => Ok(FixSpec::Steps(vec![Primitive::Replace { target: Target::Match, with: s.clone() }])),
        Value::Object(o) if o.contains_key("external") => {
            only_keys(o, &["external"], "external")?;
            Ok(FixSpec::External(string(&o["external"], "external")?))
        }
        Value::Array(items) => {
            let steps = items.iter().map(primitive).collect::<Result<Vec<_>, _>>()?;
            let groups = steps.iter().filter(|p| matches!(p, Primitive::CollectIntoBlock { .. })).count();
            if groups > 1 {
                return Err("a fix has at most one `collect-into-block`".into());
            }
            Ok(FixSpec::Steps(steps))
        }
        Value::Object(_) => Ok(FixSpec::Steps(vec![primitive(v)?])),
        _ => Err("`fix:` is a template, a list of primitives, or `{ external: \"<command>\" }`".into()),
    }
}

/// What a primitive can see of the file.
pub struct FixContext<'a> {
    pub source: &'a str,
    pub root: &'a Root,
    pub language: Language,
    pub semantic: Option<&'a SemanticModel>,
}

fn range(t: &Target, m: &Match) -> Result<(usize, usize), String> {
    match t {
        Target::Match => Ok((m.start, m.end)),
        Target::Capture(name) => m
            .single
            .get(name)
            .map(|c| (c.start, c.end))
            .or_else(|| m.multi_span(name))
            .ok_or_else(|| format!("the fix names `${name}`, which this match did not capture")),
    }
}

/// The outermost node spanning exactly `start..end`.
fn node_at(root: &Root, start: usize, end: usize) -> Option<tree_sitter::Node<'_>> {
    let mut n = root.root().get_inner_node().descendant_for_byte_range(start, end)?;
    while let Some(p) = n.parent() {
        if p.start_byte() == start && p.end_byte() == end {
            n = p;
        } else {
            break;
        }
    }
    (n.start_byte() == start && n.end_byte() == end).then_some(n)
}

/// Widen a removal so the file stays tidy: a separating comma goes with
/// the item, and a node alone on its lines takes the lines with it.
fn removal(src: &str, node: Option<tree_sitter::Node<'_>>, mut start: usize, mut end: usize) -> (usize, usize) {
    if let Some(n) = node {
        if let Some(next) = n.next_sibling().filter(|s| s.kind() == ",") {
            end = next.end_byte();
            end += src[end..].bytes().take_while(|b| *b == b' ' || *b == b'\t').count();
        } else if let Some(prev) = n.prev_sibling().filter(|s| s.kind() == ",") {
            start = prev.start_byte();
        }
    }
    let line_start = src[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    if src[line_start..start].trim().is_empty() {
        let rest = &src[end..];
        let line_end = rest.find('\n').map(|i| i + 1).unwrap_or(rest.len());
        if rest[..line_end].trim().is_empty() {
            return (line_start, end + line_end);
        }
    }
    (start, end)
}

fn place_offset(ctx: &FixContext<'_>, place: Place) -> usize {
    match place {
        Place::StartOfFile => 0,
        Place::EndOfFile => ctx.source.len(),
        Place::AfterImports => {
            let root = ctx.root.root().get_inner_node();
            let mut c = root.walk();
            let last = root
                .named_children(&mut c)
                .filter(|n| matches!(n.kind(), "import_statement" | "import_from_statement" | "future_import_statement"))
                .last();
            match last {
                Some(n) => {
                    let end = n.end_byte();
                    ctx.source[end..].find('\n').map(|i| end + i + 1).unwrap_or(ctx.source.len())
                }
                None => 0,
            }
        }
    }
}

/// Text inserted at a place, on its own line.
fn line_at(ctx: &FixContext<'_>, offset: usize, text: &str) -> String {
    let needs_newline_before = offset > 0 && !ctx.source[..offset].ends_with('\n');
    format!("{}{}\n", if needs_newline_before { "\n" } else { "" }, text.trim_end_matches('\n'))
}

/// The export statement for a target that is it, or is its declaration.
fn export_statement<'t>(node: tree_sitter::Node<'t>) -> Option<tree_sitter::Node<'t>> {
    if node.kind() == "export_statement" { Some(node) } else { node.parent().filter(|p| p.kind() == "export_statement") }
}

fn declaration_of<'t>(node: tree_sitter::Node<'t>) -> tree_sitter::Node<'t> {
    export_statement(node).and_then(|s| s.child_by_field_name("declaration")).unwrap_or(node)
}

fn step_edits(p: &Primitive, m: &Match, ctx: &FixContext<'_>, out: &mut Vec<TextEdit>) -> Result<(), String> {
    match p {
        Primitive::Replace { target, with } => {
            let (s, e) = range(target, m)?;
            out.push(TextEdit::new(s, e, interpolate(with, m, ctx.source)));
        }
        Primitive::Remove { target } => {
            let (s, e) = range(target, m)?;
            let (s, e) = removal(ctx.source, node_at(ctx.root, s, e), s, e);
            out.push(TextEdit::delete(s, e));
        }
        Primitive::InsertBefore { target, text } => {
            let (s, _) = range(target, m)?;
            out.push(TextEdit::insert(s, interpolate(text, m, ctx.source)));
        }
        Primitive::InsertAfter { target, text } => {
            let (_, e) = range(target, m)?;
            out.push(TextEdit::insert(e, interpolate(text, m, ctx.source)));
        }
        Primitive::StripExportKeyword { target } => {
            let (s, e) = range(target, m)?;
            let node = node_at(ctx.root, s, e).ok_or("`strip-export-keyword`: no node at the target")?;
            let stmt = export_statement(node).ok_or("`strip-export-keyword` needs an export statement or its declaration")?;
            let decl = stmt.child_by_field_name("declaration").ok_or("`strip-export-keyword`: the export has no declaration")?;
            let mut c = stmt.walk();
            let keyword = stmt.children(&mut c).find(|k| k.kind() == "export").ok_or("no `export` keyword")?;
            out.push(TextEdit::delete(keyword.start_byte(), decl.start_byte()));
        }
        Primitive::MoveTo { target, position } => {
            let (s, e) = range(target, m)?;
            let text = ctx.source[s..e].to_owned();
            let (rs, re) = removal(ctx.source, node_at(ctx.root, s, e), s, e);
            let at = place_offset(ctx, *position);
            if at > rs && at < re {
                return Err("`move-to`: the target already contains the destination".into());
            }
            out.push(TextEdit::delete(rs, re));
            out.push(TextEdit::insert(at, line_at(ctx, at, &text)));
        }
        Primitive::RenameSymbolLocal { target, to } => {
            let (s, _) = range(target, m)?;
            let model = ctx.semantic.ok_or("`rename-symbol-local` needs a language with a semantic model")?;
            let id = model.symbol_at(s).ok_or("`rename-symbol-local`: the target names no symbol")?;
            let symbol = model.symbol(id);
            if symbol.exported {
                return Err(format!("`{}` is exported; renaming it would break importers", symbol.name));
            }
            let new = interpolate(to, m, ctx.source);
            let mut ranges: Vec<(usize, usize)> = symbol.decls.clone();
            ranges.extend(symbol.refs.iter().map(|&r| (model.references[r].start, model.references[r].end)));
            ranges.sort();
            ranges.dedup();
            out.extend(ranges.into_iter().map(|(s, e)| TextEdit::new(s, e, new.clone())));
        }
        Primitive::CollectIntoBlock { .. } => {}
    }
    Ok(())
}

/// Names a declaration introduces, split into values and type-only names.
fn declared(node: tree_sitter::Node<'_>, src: &str) -> (Vec<String>, Vec<String>) {
    let decl = declaration_of(node);
    let names: Vec<String> = codelaw_semantic::declared_names(decl).into_iter().map(|n| src[n.byte_range()].to_owned()).collect();
    if matches!(decl.kind(), "interface_declaration" | "type_alias_declaration") { (Vec::new(), names) } else { (names, Vec::new()) }
}

fn sort_edits(mut edits: Vec<TextEdit>) -> Vec<TextEdit> {
    edits.sort_by(|a, b| (a.start, a.end).cmp(&(b.start, b.end)));
    edits.dedup();
    edits
}

/// The edits for one match.
pub fn edits_for(spec: &FixSpec, m: &Match, ctx: &FixContext<'_>) -> Result<Vec<TextEdit>, String> {
    let FixSpec::Steps(steps) = spec else { return Err("an external fixer runs through `run_external`".into()) };
    let mut out = Vec::new();
    for p in steps {
        step_edits(p, m, ctx, &mut out)?;
    }
    Ok(sort_edits(out))
}

/// The edits for every match of a group fix together.
pub fn group_edits(spec: &FixSpec, matches: &[Match], ctx: &FixContext<'_>) -> Result<Vec<TextEdit>, String> {
    let FixSpec::Steps(steps) = spec else { return Err("not a group fix".into()) };
    let mut out = Vec::new();
    let mut values = Vec::new();
    let mut types = Vec::new();
    let mut collect = None;
    for m in matches {
        for p in steps {
            match p {
                Primitive::CollectIntoBlock { target, block, type_block, position } => {
                    let (s, e) = range(target, m)?;
                    let node = node_at(ctx.root, s, e).ok_or("`collect-into-block`: no node at the target")?;
                    let (v, t) = declared(node, ctx.source);
                    values.extend(v);
                    types.extend(t);
                    collect = Some((block, type_block, *position));
                }
                other => step_edits(other, m, ctx, &mut out)?,
            }
        }
    }
    if let Some((block, type_block, position)) = collect {
        let at = place_offset(ctx, position);
        let mut lines = Vec::new();
        if !values.is_empty() {
            lines.push(block.replace("$$$NAMES", &values.join(", ")));
        }
        if !types.is_empty() {
            match type_block {
                Some(tb) => lines.push(tb.replace("$$$TYPE_NAMES", &types.join(", "))),
                None => return Err("`collect-into-block` found type-only names but has no `type_block`".into()),
            }
        }
        if !lines.is_empty() {
            out.push(TextEdit::insert(at, line_at(ctx, at, &lines.join("\n"))));
        }
    }
    Ok(sort_edits(out))
}
