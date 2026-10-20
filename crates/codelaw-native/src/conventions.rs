//! Native rules behind `js/conventions`, the starter catalogue's house
//! conventions: import groups, type-only imports, and `<Component>Props`.

use crate::util::{named_children, string_value, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use tree_sitter::Node;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];
const TS: &[Language] = &[Language::Typescript, Language::Tsx];

/// Node.js's built-in modules, importable without `node:`.
const NODE_BUILTINS: &[&str] = &[
    "assert", "async_hooks", "buffer", "child_process", "cluster", "console", "constants", "crypto", "dgram", "diagnostics_channel", "dns",
    "domain", "events", "fs", "http", "http2", "https", "inspector", "module", "net", "os", "path", "perf_hooks", "process", "punycode",
    "querystring", "readline", "repl", "stream", "string_decoder", "sys", "timers", "tls", "trace_events", "tty", "url", "util", "v8", "vm",
    "wasi", "worker_threads", "zlib",
];

pub struct ImportGroups;

/// 0 builtin · 1 external · 2 internal · 3 relative.
fn group(source: &str, internal: &[String]) -> usize {
    let root = source.split('/').next().unwrap_or(source);
    if source.starts_with("node:") || NODE_BUILTINS.contains(&root) {
        0
    } else if source.starts_with('.') {
        3
    } else if internal.iter().any(|p| source.starts_with(p.as_str())) {
        2
    } else {
        1
    }
}

impl NativeRule for ImportGroups {
    fn id(&self) -> &'static str {
        "js/import-groups"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        let internal: Vec<String> = ctx
            .options
            .get("internal")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
            .unwrap_or_else(|| vec!["@/".into(), "~/".into()]);
        // The leading run of imports; a comment inside it leaves it alone,
        // since moving lines could detach the comment from what it describes.
        let stmts = named_children(node.get_inner_node());
        let imports: Vec<Node> = stmts.iter().copied().take_while(|s| s.kind() == "import_statement").collect();
        if imports.len() < 2 {
            return;
        }
        let (start, end) = (imports[0].start_byte(), imports.last().expect("two").end_byte());
        if src[start..end].contains("//") || src[start..end].contains("/*") {
            return;
        }
        let mut keyed: Vec<(usize, usize, &str)> = imports
            .iter()
            .enumerate()
            .map(|(i, imp)| {
                let source = imp.child_by_field_name("source").map(|s| string_value(s, src)).unwrap_or_default();
                (group(&source, &internal), i, text(*imp, src))
            })
            .collect();
        keyed.sort_by_key(|(g, i, _)| (*g, *i));
        let mut wanted = String::new();
        for (k, (g, _, t)) in keyed.iter().enumerate() {
            if k > 0 {
                wanted.push_str(if keyed[k - 1].0 != *g { "\n\n" } else { "\n" });
            }
            wanted.push_str(t);
        }
        if wanted != src[start..end] {
            out.push(NativeFinding {
                start: imports[0].start_byte(),
                end: imports[0].end_byte(),
                message: "Order imports builtin → external → internal → relative, with a blank line between groups".into(),
                edits: vec![TextEdit::new(start, end, wanted)],
            });
        }
    }
}

pub struct TypeOnlyImports;

impl NativeRule for TypeOnlyImports {
    fn id(&self) -> &'static str {
        "ts/type-only-imports"
    }
    fn languages(&self) -> &'static [Language] {
        TS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let root = node.get_inner_node();
        for i in model.imports.iter().filter(|i| !i.type_only) {
            let Some(local) = i.local else { continue };
            let s = model.symbol(local);
            if s.refs.is_empty() || s.exported || !s.refs.iter().all(|&r| model.references[r].type_only) {
                continue;
            }
            // Named specifiers get `type`; default and namespace imports would
            // need the whole statement rewritten, so they are reported only.
            // Without an alias a specifier spans exactly its identifier, so
            // climb from the smallest node to the specifier.
            let spec = root
                .descendant_for_byte_range(i.start, i.end)
                .and_then(|n| std::iter::successors(Some(n), |p| p.parent()).take_while(|p| p.start_byte() == i.start && p.end_byte() == i.end).find(|p| p.kind() == "import_specifier"));
            let Some(spec) = spec else {
                out.push(NativeFinding { start: i.start, end: i.end, message: format!("`{}` is used only as a type; import it with `import type`", s.name), edits: Vec::new() });
                continue;
            };
            out.push(NativeFinding {
                start: spec.start_byte(),
                end: spec.end_byte(),
                message: format!("`{}` is used only as a type; mark it `type`", s.name),
                edits: vec![TextEdit::insert(spec.start_byte(), "type ")],
            });
        }
    }
}

pub struct PropsTypeName;

impl NativeRule for PropsTypeName {
    fn id(&self) -> &'static str {
        "react/props-type-name"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Tsx]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["function_declaration", "variable_declarator"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        let n = node.get_inner_node();
        let (name, function) = match n.kind() {
            "function_declaration" => (n.child_by_field_name("name"), Some(n)),
            _ => (n.child_by_field_name("name"), n.child_by_field_name("value").filter(|v| matches!(v.kind(), "arrow_function" | "function_expression"))),
        };
        let (Some(name), Some(function)) = (name, function) else { return };
        let component = text(name, src);
        if !component.starts_with(|c: char| c.is_ascii_uppercase()) {
            return;
        }
        let Some(params) = function.child_by_field_name("parameters") else { return };
        let Some(first) = named_children(params).into_iter().next() else { return };
        let Some(ty) = first.child_by_field_name("type").and_then(|t| t.named_child(0)).filter(|t| t.kind() == "type_identifier") else { return };
        let wanted = format!("{component}Props");
        let actual = text(ty, src);
        if actual == wanted {
            return;
        }
        // Rename the type where this file declares it, and every use here.
        let Some(model) = ctx.semantic else { return };
        let Some(sym) = model.reference_at(ty.start_byte()).and_then(|r| r.symbol) else { return };
        let s = model.symbol(sym);
        if s.exported {
            out.push(NativeFinding { start: ty.start_byte(), end: ty.end_byte(), message: format!("Name the props type `{wanted}`"), edits: Vec::new() });
            return;
        }
        let mut edits: Vec<TextEdit> = s.decls.iter().map(|(a, b)| TextEdit::new(*a, *b, wanted.clone())).collect();
        edits.extend(s.refs.iter().map(|&r| TextEdit::new(model.references[r].start, model.references[r].end, wanted.clone())));
        edits.sort_by_key(|e| e.start);
        edits.dedup();
        out.push(NativeFinding { start: ty.start_byte(), end: ty.end_byte(), message: format!("Name the props type `{wanted}`, after its component"), edits });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_groups() {
        let src = "import x from './x';\nimport fs from 'node:fs';\nimport react from 'react';\nimport util from '@/util';\nimport path from 'path';\n\nrun();\n";
        let f = crate::run(&ImportGroups, Language::Javascript, src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].edits[0].text, "import fs from 'node:fs';\nimport path from 'path';\n\nimport react from 'react';\n\nimport util from '@/util';\n\nimport x from './x';");
        assert!(crate::run(&ImportGroups, Language::Javascript, "import fs from 'fs';\n\nimport x from './x';\n").is_empty());
    }

    #[test]
    fn type_only_and_props_name() {
        let src = "import { Config, load } from './c';\nimport { Other } from './o';\nlet c: Config = load();\nlet o = new Other();\n";
        let f = crate::run(&TypeOnlyImports, Language::Typescript, src);
        assert_eq!(f.iter().map(|x| &src[x.start..x.end]).collect::<Vec<_>>(), ["Config"]);
        let src = "type CardOptions = { title: string };\nexport function Card(props: CardOptions) {\n  return null;\n}\nfunction Badge(props: BadgeProps) { return null; }\n";
        let f = crate::run(&PropsTypeName, Language::Tsx, src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].edits.len(), 2);
    }
}
