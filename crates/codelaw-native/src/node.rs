//! `node/no-deprecated-api`: Node.js APIs the docs deprecate, reached
//! through `import`, `require()` or a global. The table is the pack's
//! `node/deprecated-api` data: `{ "<module>": { "<member>": "<use instead>" } }`,
//! with `"*"` for a module deprecated as a whole and `"globals"` for
//! globals such as `Buffer()`.

use crate::util::{named_children, string_value, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use codelaw_semantic::Imported;
use serde_json::Value;
use tree_sitter::Node;

pub struct NoDeprecatedApi;

fn module_key(source: &str) -> &str {
    source.strip_prefix("node:").unwrap_or(source)
}

fn lookup<'d>(table: &'d serde_json::Map<String, Value>, module: &str, member: &str) -> Option<&'d str> {
    table.get(module_key(module)).and_then(|m| m.get(member)).and_then(|v| v.as_str())
}

/// The module a `require("<m>")` call loads.
fn required_module(n: Node<'_>, src: &str) -> Option<String> {
    if n.kind() != "call_expression" || n.child_by_field_name("function").is_none_or(|f| text(f, src) != "require") {
        return None;
    }
    let arg = named_children(n.child_by_field_name("arguments")?).into_iter().next()?;
    (arg.kind() == "string").then(|| string_value(arg, src))
}

impl NativeRule for NoDeprecatedApi {
    fn id(&self) -> &'static str {
        "node/no-deprecated-api"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let Some(Value::Object(table)) = ctx.data.get("node/deprecated-api").map(|v| v.as_ref()) else { return };
        let root = node.get_inner_node();
        let src = ctx.source;
        let mut report = |start: usize, end: usize, what: String, instead: &str| {
            out.push(NativeFinding { start, end, message: format!("`{what}` is deprecated; use {instead}"), edits: Vec::new() });
        };
        // Bindings of whole modules: `import fs from`, `import * as fs`, `const fs = require()`.
        let mut module_bindings: Vec<(usize, String)> = Vec::new();
        for i in &model.imports {
            if let Some(instead) = lookup(table, &i.source, "*") {
                report(i.start, i.end, module_key(&i.source).to_owned(), instead);
                continue;
            }
            match (&i.imported, i.local) {
                (Imported::Named(name), _) => {
                    if let Some(instead) = lookup(table, &i.source, name) {
                        report(i.start, i.end, format!("{}.{name}", module_key(&i.source)), instead);
                    }
                }
                (Imported::Default | Imported::Namespace, Some(local)) => module_bindings.push((local, i.source.clone())),
                _ => {}
            }
        }
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
            if n.kind() == "variable_declarator"
                && let Some(m) = n.child_by_field_name("value").and_then(|v| required_module(v, src))
            {
                let Some(name) = n.child_by_field_name("name") else { continue };
                if let Some(instead) = lookup(table, &m, "*") {
                    report(n.start_byte(), n.end_byte(), module_key(&m).to_owned(), instead);
                } else if name.kind() == "identifier" {
                    if let Some(id) = model.declared_at(name.start_byte()) {
                        module_bindings.push((id, m));
                    }
                } else if name.kind() == "object_pattern" {
                    for p in named_children(name) {
                        let key = match p.kind() {
                            "shorthand_property_identifier_pattern" => Some(text(p, src)),
                            "pair_pattern" => p.child_by_field_name("key").map(|k| text(k, src)),
                            _ => None,
                        };
                        if let Some(k) = key
                            && let Some(instead) = lookup(table, &m, k)
                        {
                            report(p.start_byte(), p.end_byte(), format!("{}.{k}", module_key(&m)), instead);
                        }
                    }
                }
            }
            // Globals: `new Buffer()`, `Buffer()`.
            if matches!(n.kind(), "new_expression" | "call_expression") {
                let callee = n.child_by_field_name("constructor").or_else(|| n.child_by_field_name("function"));
                if let Some(c) = callee.filter(|c| c.kind() == "identifier") {
                    let name = text(c, src);
                    let global = model.reference_at(c.start_byte()).is_some_and(|r| r.symbol.is_none());
                    if global && let Some(instead) = lookup(table, "globals", name) {
                        report(n.start_byte(), n.end_byte(), format!("{name}()"), instead);
                    }
                }
            }
        }
        // `fs.exists(...)` through a whole-module binding.
        for (sym, m) in module_bindings {
            for &r in &model.symbol(sym).refs {
                let reference = &model.references[r];
                let Some(id) = root.descendant_for_byte_range(reference.start, reference.end) else { continue };
                let Some(member) = id.parent().filter(|p| p.kind() == "member_expression" && p.child_by_field_name("object").is_some_and(|o| o.id() == id.id())) else { continue };
                let Some(prop) = member.child_by_field_name("property") else { continue };
                if let Some(instead) = lookup(table, &m, text(prop, src)) {
                    report(member.start_byte(), member.end_byte(), format!("{}.{}", module_key(&m), text(prop, src)), instead);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[test]
    fn deprecated_apis() {
        let src = "import fs from 'node:fs';\nimport { isArray } from 'util';\nconst url = require('url');\nconst { createCipher } = require('crypto');\nconst d = require('domain');\nfs.exists(p, cb);\nfs.readFile(p, cb);\nurl.parse(s);\nnew Buffer(4);\nconst ok = Buffer.from('a');\n";
        let root = codelaw_lang::parse(Language::Javascript, src);
        let model = codelaw_semantic::build(Language::Javascript, &root, src);
        let mut data = BTreeMap::new();
        data.insert(
            "node/deprecated-api".to_owned(),
            Arc::new(serde_json::json!({ "fs": { "exists": "fs.stat" }, "util": { "isArray": "Array.isArray" }, "url": { "parse": "new URL()" }, "crypto": { "createCipher": "crypto.createCipheriv" }, "domain": { "*": "async_hooks" }, "globals": { "Buffer": "Buffer.from" } })),
        );
        let options = BTreeMap::new();
        let ctx = NativeContext { source: src, language: Language::Javascript, semantic: model.as_ref(), options: &options, data: &data, file: &crate::DetachedFile::default() };
        let mut out = Vec::new();
        NoDeprecatedApi.check(root.root(), &ctx, &mut out);
        let mut msgs: Vec<String> = out.into_iter().map(|f| f.message).collect();
        msgs.sort();
        assert_eq!(msgs, [
            "`Buffer()` is deprecated; use Buffer.from",
            "`crypto.createCipher` is deprecated; use crypto.createCipheriv",
            "`domain` is deprecated; use async_hooks",
            "`fs.exists` is deprecated; use fs.stat",
            "`url.parse` is deprecated; use new URL()",
            "`util.isArray` is deprecated; use Array.isArray",
        ]);
    }
}
