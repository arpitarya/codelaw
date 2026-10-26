//! `js/use-instead` and `py/use-instead` (W-73): "use `apiFetch` instead of
//! `fetch`". One option, `replacements`, maps a dotted path to
//! `{ use, from, drop_in?, allow_in? }`; the rule is inert until a project
//! sets it. A key matches a call's callee or a member read, exactly or as a
//! prefix at a `.` boundary (`process.env` matches `process.env.API_URL`),
//! only where the path's root is a global or an import, never a local
//! shadow. A file matching `allow_in` is exempt (the wrapper itself calls
//! the thing it wraps). With `drop_in` the fix replaces the path with `use`
//! and adds the import when the file lacks it.

use crate::util::{named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use codelaw_semantic::SemanticModel;
use tree_sitter::Node;

pub struct UseInstead {
    pub id: &'static str,
    pub python: bool,
}

struct Replacement<'a> {
    key: &'a str,
    use_: &'a str,
    from: &'a str,
    drop_in: bool,
    allow_in: Vec<&'a str>,
}

fn replacements(options: &std::collections::BTreeMap<String, serde_json::Value>) -> Vec<Replacement<'_>> {
    let Some(map) = options.get("replacements").and_then(|v| v.as_object()) else { return Vec::new() };
    map.iter()
        .filter_map(|(key, v)| {
            let o = v.as_object()?;
            Some(Replacement {
                key,
                use_: o.get("use")?.as_str()?,
                from: o.get("from").and_then(|f| f.as_str()).unwrap_or_default(),
                drop_in: o.get("drop_in").and_then(|d| d.as_bool()).unwrap_or(false),
                allow_in: o.get("allow_in").and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|g| g.as_str()).collect()).unwrap_or_default(),
            })
        })
        .collect()
}

fn allowed_here(globs: &[&str], path: &str) -> bool {
    globs.iter().any(|g| globset::Glob::new(g).is_ok_and(|g| g.compile_matcher().is_match(path)))
}

/// The dotted chain that starts at `root` and climbs through member reads:
/// each segment's text and where it ends.
fn chain<'t>(root: Node<'t>, src: &str, python: bool) -> Vec<(String, usize)> {
    let member = if python { "attribute" } else { "member_expression" };
    let prop = if python { "attribute" } else { "property" };
    let mut out = vec![(text(root, src).to_owned(), root.end_byte())];
    let mut cur = root;
    while let Some(p) = cur.parent() {
        if p.kind() != member || p.child_by_field_name("object") != Some(cur) {
            break;
        }
        let Some(name) = p.child_by_field_name(prop) else { break };
        if !matches!(name.kind(), "property_identifier" | "identifier") {
            break;
        }
        out.push((text(name, src).to_owned(), name.end_byte()));
        cur = p;
    }
    out
}

/// Where a new import goes: after the last top-level import, else the top.
fn import_at(root: Node<'_>, src: &str, python: bool) -> usize {
    let kinds: &[&str] = if python { &["import_statement", "import_from_statement", "future_import_statement"] } else { &["import_statement"] };
    named_children(root)
        .into_iter()
        .rfind(|n| kinds.contains(&n.kind()))
        .map(|n| src[n.end_byte()..].find('\n').map(|i| n.end_byte() + i + 1).unwrap_or(src.len()))
        .unwrap_or(0)
}

fn has_import(model: &SemanticModel, from: &str, name: &str) -> bool {
    model.imports.iter().any(|i| i.source == from && i.local.is_some_and(|l| model.symbol(l).name == name))
}

impl NativeRule for UseInstead {
    fn id(&self) -> &'static str {
        self.id
    }
    fn languages(&self) -> &'static [Language] {
        if self.python { &[Language::Python] } else { &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx] }
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["identifier"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let rules = replacements(ctx.options);
        if rules.is_empty() {
            return;
        }
        let n = node.get_inner_node();
        let src = ctx.source;
        // Only a reference to a name: a global, or one an import bound.
        let Some(r) = model.reference_at(n.start_byte()) else { return };
        if r.write {
            return;
        }
        let imported = r.symbol.is_some_and(|s| model.import_of(s).is_some());
        if r.symbol.is_some() && !imported || (r.symbol.is_none() && model.has_star_import) {
            return;
        }
        // The head of a path, not its tail.
        if let Some(p) = n.parent() {
            let (member, prop) = if self.python { ("attribute", "attribute") } else { ("member_expression", "property") };
            if p.kind() == member && p.child_by_field_name(prop) == Some(n) {
                return;
            }
            if p.kind() == "keyword_argument" && p.child_by_field_name("name") == Some(n) {
                return;
            }
        }
        let segs = chain(n, src, self.python);
        // The longest key that is the path or a prefix of it.
        let hit = rules
            .iter()
            .filter_map(|rep| {
                let k: Vec<&str> = rep.key.split('.').collect();
                (k.len() <= segs.len() && k.iter().zip(&segs).all(|(a, (b, _))| a == b)).then_some((k.len(), rep))
            })
            .max_by_key(|(len, _)| *len);
        let Some((len, rep)) = hit else { return };
        if allowed_here(&rep.allow_in, ctx.file.path()) {
            return;
        }
        let (start, end) = (n.start_byte(), segs[len - 1].1);
        let root = n.parent().map(|_| {
            let mut c = n;
            while let Some(p) = c.parent() {
                c = p;
            }
            c
        });
        let use_root = rep.use_.split('.').next().unwrap_or_default();
        let mut edits = Vec::new();
        let mut how = String::new();
        if rep.drop_in {
            let clash = !has_import(model, rep.from, use_root)
                && model.scopes.first().is_some_and(|s| s.bindings.contains_key(use_root));
            if clash {
                how = format!(" (no automatic fix: `{use_root}` is already declared in this file)");
            } else {
                edits.push(TextEdit::new(start, end, rep.use_));
                if !rep.from.is_empty() && !has_import(model, rep.from, use_root) {
                    let line = if self.python {
                        format!("from {} import {use_root}\n", rep.from)
                    } else {
                        format!("import {{ {use_root} }} from \"{}\";\n", rep.from)
                    };
                    let at = root.map(|r| import_at(r, src, self.python)).unwrap_or(0);
                    edits.push(TextEdit::new(at, at, line));
                }
            }
        }
        let used = &src[start..end];
        let from = if rep.from.is_empty() { String::new() } else { format!(" from `{}`", rep.from) };
        out.push(NativeFinding {
            start,
            end,
            message: format!("`{used}` is not used here; use `{}`{from} instead{how}", rep.use_),
            edits,
        });
    }
}
