//! Per-file facts (SR-PROJECT-GRAPH decision 5): what one file declares,
//! exports, imports and references, read once from its syntax tree and
//! semantic model. Facts depend on the file's text alone, so they are
//! cached by content; linking them is the graph's job.

use codelaw_core::Language;
use codelaw_semantic::{Imported, SemanticModel, SymbolKind};
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

/// A byte range.
pub type Span = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FactLang {
    Js,
    Python,
    Css,
    Html,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DeclKind {
    Function,
    Class,
    Variable,
    Type,
    Interface,
    Enum,
    Namespace,
}

impl DeclKind {
    pub fn name(self) -> &'static str {
        match self {
            DeclKind::Function => "function",
            DeclKind::Class => "class",
            DeclKind::Variable => "variable",
            DeclKind::Type => "type",
            DeclKind::Interface => "interface",
            DeclKind::Enum => "enum",
            DeclKind::Namespace => "namespace",
        }
    }

    pub fn parse(s: &str) -> Option<DeclKind> {
        Some(match s {
            "function" => DeclKind::Function,
            "class" => DeclKind::Class,
            "variable" => DeclKind::Variable,
            "type" => DeclKind::Type,
            "interface" => DeclKind::Interface,
            "enum" => DeclKind::Enum,
            "namespace" => DeclKind::Namespace,
            _ => return None,
        })
    }
}

/// A top-level declaration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decl {
    pub name: String,
    pub kind: DeclKind,
    /// The declaring identifier.
    pub span: Span,
    /// Whole lines to delete to remove it, when its statement declares
    /// nothing else.
    pub stmt: Option<Span>,
    /// `export ` before a declaration, to strip it.
    pub export_kw: Option<Span>,
    /// Exported by name in an `export { … }` list: deleting the
    /// declaration would leave that list naming nothing.
    pub listed_export: bool,
    /// Reads of it in its own file (exports of it excluded).
    pub local_uses: Vec<Span>,
    /// `public`, `keep` (JSDoc `@public` / `@keep`, `# codelaw: keep`).
    pub tags: Vec<String>,
    /// Decorator names, without `@` or arguments (`app.get`).
    pub decorators: Vec<String>,
    /// An in-code deprecation marker and its message.
    pub deprecated: Option<String>,
}

/// One exported name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportFact {
    /// `default`, `*` for `export * from`.
    pub name: String,
    /// The top-level declaration it exports.
    pub local: Option<String>,
    /// A re-export: the module it comes from…
    pub source: Option<String>,
    /// …and the name there (`*`: the whole module).
    pub imported: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    Named(String),
    Default,
    Namespace,
    /// Python `import a.b`.
    Module,
    /// Python `from m import *`.
    Star,
    SideEffect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportFact {
    pub source: String,
    pub kind: ImportKind,
    pub local: Option<String>,
    pub span: Span,
    /// Reads of the local binding.
    pub uses: Vec<Span>,
    /// `ns.x` member reads of the binding; `None` when the binding itself
    /// is passed around, so every member counts as used.
    pub members: Option<Vec<Member>>,
    pub type_only: bool,
}

/// How a JS/TS file names a module (the `node/` import rules).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpecForm {
    /// `import … from "x"`, `import "x"`, `import x = require("x")`.
    Import,
    /// `export … from "x"`.
    Export,
    /// `import("x")` with a literal.
    Dynamic,
    /// `require("x")` with a literal.
    Require,
}

impl SpecForm {
    pub fn parse(s: &str) -> Option<SpecForm> {
        Some(match s {
            "import" => SpecForm::Import,
            "export" => SpecForm::Export,
            "dynamic" => SpecForm::Dynamic,
            "require" => SpecForm::Require,
            _ => return None,
        })
    }
}

/// One module specifier written in a JS/TS file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Specifier {
    pub spec: String,
    /// The string literal.
    pub span: Span,
    pub form: SpecForm,
    /// `import type` / `export type`: erased at build time.
    pub type_only: bool,
}

/// `import("x")`, `require("x")`, `importlib.import_module("x")`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicImport {
    pub spec: String,
    /// `spec` is only the static prefix of a template.
    pub prefix: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CssClass {
    pub name: String,
    pub span: Span,
    /// The whole rule, when its only selector is this class.
    pub rule: Option<Span>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFacts {
    pub lang: Option<FactLang>,
    pub decls: Vec<Decl>,
    pub exports: Vec<ExportFact>,
    pub imports: Vec<ImportFact>,
    pub dynamic_imports: Vec<DynamicImport>,
    /// Why the module is `dynamic` (SR-PROJECT-GRAPH decision 4).
    pub dynamic: Option<String>,
    /// Python: the module sets `__all__`.
    pub python_all: bool,
    /// Python: `if __name__ == "__main__":`.
    pub is_main: bool,
    pub css_classes: Vec<CssClass>,
    /// Words that may name a CSS class (strings, attribute values).
    pub class_tokens: Vec<String>,
    /// Static prefixes of built class names (`btn-${x}`).
    pub class_prefixes: Vec<String>,
    /// HTML `<script src>` / `<link href>` targets.
    pub links: Vec<String>,
    /// JS/TS: every module specifier, in source order.
    pub specifiers: Vec<Specifier>,
}

/// The facts of one file, or `None` for a language the graph doesn't read.
pub fn extract(language: Language, text: &str) -> Option<FileFacts> {
    let lang = if language.is_js_family() {
        FactLang::Js
    } else {
        match language {
            Language::Python => FactLang::Python,
            Language::Css => FactLang::Css,
            Language::Html => FactLang::Html,
            _ => return None,
        }
    };
    let root = codelaw_lang::parse(language, text);
    let node = root.root().get_inner_node();
    let mut f = FileFacts { lang: Some(lang), ..Default::default() };
    match lang {
        FactLang::Js | FactLang::Python => {
            let model = codelaw_semantic::build(language, &root, text)?;
            Extractor { text, root: node, m: &model, f: &mut f, python: lang == FactLang::Python }.run();
        }
        FactLang::Css => css(node, text, &mut f),
        FactLang::Html => html(text, &mut f),
    }
    f.class_tokens.sort();
    f.class_tokens.dedup();
    f.class_prefixes.sort();
    f.class_prefixes.dedup();
    Some(f)
}

fn named_children<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c).collect()
}

fn children<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.children(&mut c).collect()
}

/// Extend a range to whole lines when only whitespace surrounds it.
fn full_lines(text: &str, s: usize, e: usize) -> Span {
    let ls = text[..s].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let start = if text[ls..s].trim().is_empty() { ls } else { s };
    let le = text[e..].find('\n').map(|i| e + i + 1).unwrap_or(text.len());
    let end = if text[e..le].trim().is_empty() { le } else { e };
    (start, end)
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at.min(text.len())].matches('\n').count()
}

fn push_tokens(f: &mut FileFacts, s: &str) {
    for t in s.split(|c: char| c.is_whitespace()) {
        let t = t.trim_matches(|c: char| matches!(c, '.' | ',' | ';'));
        if t.is_empty() || t.len() > 100 {
            continue;
        }
        if t.ends_with('-') || t.ends_with('_') {
            f.class_prefixes.push(t.to_owned());
        }
        f.class_tokens.push(t.to_owned());
    }
}

struct Extractor<'a> {
    text: &'a str,
    root: Node<'a>,
    m: &'a SemanticModel,
    f: &'a mut FileFacts,
    python: bool,
}

impl<'a> Extractor<'a> {
    fn t(&self, n: Node<'_>) -> &'a str {
        &self.text[n.byte_range()]
    }

    /// The top-level statement containing a byte offset.
    fn top_level(&self, at: usize) -> Option<Node<'a>> {
        let mut n = self.root.descendant_for_byte_range(at, at)?;
        while let Some(p) = n.parent() {
            if p.id() == self.root.id() {
                return Some(n);
            }
            n = p;
        }
        None
    }

    /// Export statements that only list names (`export { a }`): reads in
    /// them are the export itself, not a use.
    fn export_clauses(&self) -> Vec<Span> {
        named_children(self.root)
            .into_iter()
            .filter(|n| {
                n.kind() == "export_statement" && n.child_by_field_name("declaration").is_none() && n.child_by_field_name("value").is_none()
            })
            .map(|n| (n.start_byte(), n.end_byte()))
            .collect()
    }

    fn reads(&self, id: usize, skip: &[Span]) -> Vec<Span> {
        let s = &self.m.symbols[id];
        let mut out: Vec<Span> = s
            .refs
            .iter()
            .map(|&r| &self.m.references[r])
            .filter(|r| r.read || r.type_only)
            .filter(|r| !skip.iter().any(|(a, b)| *a <= r.start && r.end <= *b))
            .map(|r| (r.start, r.end))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    fn run(mut self) {
        let skip = if self.python { Vec::new() } else { self.export_clauses() };
        self.decls(&skip);
        self.exports();
        self.imports(&skip);
        self.walk(self.root);
        if self.python {
            for n in named_children(self.root) {
                match n.kind() {
                    "if_statement" => {
                        let c = n.child_by_field_name("condition").map(|c| self.t(c)).unwrap_or_default();
                        if c.contains("__name__") && c.contains("__main__") {
                            self.f.is_main = true;
                        }
                    }
                    "expression_statement" => {
                        if let Some(a) = n.named_child(0)
                            && matches!(a.kind(), "assignment" | "augmented_assignment")
                            && a.child_by_field_name("left").is_some_and(|l| self.t(l) == "__all__")
                        {
                            self.f.python_all = true;
                        }
                    }
                    "function_definition" if n.child_by_field_name("name").is_some_and(|x| self.t(x) == "__getattr__") => {
                        self.f.dynamic = Some("the module defines `__getattr__`".into());
                    }
                    _ => {}
                }
            }
        }
    }

    fn decls(&mut self, skip: &[Span]) {
        let module_syms: Vec<usize> = (0..self.m.symbols.len()).filter(|&i| self.m.symbols[i].scope == 0).collect();
        for &id in &module_syms {
            let s = &self.m.symbols[id];
            let kind = match s.kind {
                SymbolKind::Function => DeclKind::Function,
                SymbolKind::Class => DeclKind::Class,
                SymbolKind::Var | SymbolKind::Let | SymbolKind::Const | SymbolKind::Variable => DeclKind::Variable,
                SymbolKind::Interface => DeclKind::Interface,
                SymbolKind::TypeAlias => DeclKind::Type,
                SymbolKind::Enum => DeclKind::Enum,
                SymbolKind::Namespace => DeclKind::Namespace,
                _ => continue,
            };
            if self.python && s.name.starts_with("__") && s.name.ends_with("__") {
                continue;
            }
            let Some(&span) = s.decls.first() else { continue };
            let stmt_node = self.top_level(span.0);
            let mut decl = Decl {
                name: s.name.clone(),
                kind,
                span,
                stmt: None,
                export_kw: None,
                listed_export: self.m.exports.iter().any(|e| e.local == Some(id) && e.source.is_none() && !(e.start <= span.0 && span.1 <= e.end)),
                local_uses: self.reads(id, skip),
                tags: Vec::new(),
                decorators: Vec::new(),
                deprecated: None,
            };
            if let Some(stmt) = stmt_node {
                let (a, b) = (stmt.start_byte(), stmt.end_byte());
                // Removable only when it declares nothing else, once.
                let others = module_syms.iter().filter(|&&o| self.m.symbols[o].decls.iter().any(|d| a <= d.0 && d.1 <= b)).count();
                if others == 1 && s.decls.len() == 1 {
                    decl.stmt = Some(full_lines(self.text, a, b));
                }
                if stmt.kind() == "export_statement"
                    && let Some(d) = stmt.child_by_field_name("declaration")
                    && !children(stmt).iter().any(|c| c.kind() == "default")
                    && let Some(kw) = children(stmt).into_iter().find(|c| c.kind() == "export")
                {
                    decl.export_kw = Some((kw.start_byte(), d.start_byte()));
                }
                self.markers(stmt, &mut decl);
            }
            self.f.decls.push(decl);
        }
        self.f.decls.sort_by_key(|d| d.span);
    }

    /// Keep tags, decorators and deprecation markers of a statement.
    fn markers(&self, stmt: Node<'_>, decl: &mut Decl) {
        let stmt_line = line_of(self.text, stmt.start_byte());
        let mut comments = Vec::new();
        if let Some(p) = stmt.prev_named_sibling()
            && p.kind() == "comment"
            && line_of(self.text, p.end_byte()) + 1 >= stmt_line
        {
            comments.push(self.t(p));
        }
        if let Some(nx) = stmt.next_named_sibling()
            && nx.kind() == "comment"
            && line_of(self.text, nx.start_byte()) == line_of(self.text, stmt.end_byte())
        {
            comments.push(self.t(nx));
        }
        // A trailing comment on the statement's first line (`def f():  # …`).
        let ls = self.text[..stmt.start_byte()].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let le = self.text[stmt.start_byte()..].find('\n').map(|i| stmt.start_byte() + i).unwrap_or(self.text.len());
        let first_line = &self.text[ls..le];
        if let Some(i) = first_line.find(if self.python { '#' } else { '/' }) {
            comments.push(&first_line[i..]);
        }
        for c in comments {
            if c.contains("@public") {
                decl.tags.push("public".into());
            }
            if c.contains("@keep") || c.contains("codelaw: keep") {
                decl.tags.push("keep".into());
            }
            if let Some(i) = c.find("@deprecated") {
                let rest = &c[i + "@deprecated".len()..];
                let line = rest.lines().next().unwrap_or_default();
                decl.deprecated = Some(line.trim().trim_end_matches("*/").trim().to_owned());
            }
        }
        let mut holders = vec![stmt];
        holders.extend(stmt.child_by_field_name("declaration"));
        holders.extend(stmt.child_by_field_name("definition"));
        for h in holders {
            for d in children(h).into_iter().filter(|c| c.kind() == "decorator") {
                let Some(e) = d.named_child(0) else { continue };
                let (name, call) = if matches!(e.kind(), "call_expression" | "call") {
                    (e.child_by_field_name("function").map(|f| self.t(f)).unwrap_or_default(), Some(e))
                } else {
                    (self.t(e), None)
                };
                let name: String = name.split_whitespace().collect();
                if name.rsplit('.').next() == Some("deprecated") {
                    let msg = call
                        .and_then(|c| c.child_by_field_name("arguments"))
                        .and_then(|a| a.named_child(0))
                        .map(|s| self.t(s).trim_matches(|c| c == '"' || c == '\'').to_owned())
                        .unwrap_or_default();
                    decl.deprecated = Some(msg);
                }
                decl.decorators.push(name);
            }
        }
        decl.tags.sort();
        decl.tags.dedup();
    }

    fn imported_name(i: &Imported) -> String {
        match i {
            Imported::Named(x) => x.clone(),
            Imported::Default => "default".into(),
            _ => "*".into(),
        }
    }

    fn exports(&mut self) {
        let tops = named_children(self.root);
        for e in &self.m.exports {
            let mut fact = ExportFact { name: e.name.clone(), local: None, source: None, imported: None, span: (e.start, e.end) };
            if let Some(src) = &e.source {
                fact.source = Some(src.clone());
                fact.imported = Some(if e.name == "*" { "*".into() } else { self.reexported_name(&tops, e.start, &e.name) });
            } else if let Some(id) = e.local {
                let s = &self.m.symbols[id];
                if s.kind == SymbolKind::Import {
                    if let Some(imp) = self.m.import_of(id) {
                        fact.source = Some(imp.source.clone());
                        fact.imported = Some(Self::imported_name(&imp.imported));
                    }
                } else if s.scope == 0 {
                    fact.local = Some(s.name.clone());
                }
            }
            self.f.exports.push(fact);
        }
    }

    /// The source-side name of `export { x as name } from`.
    fn reexported_name(&self, tops: &[Node<'_>], start: usize, name: &str) -> String {
        let Some(stmt) = tops.iter().find(|n| n.start_byte() == start) else { return name.to_owned() };
        for c in named_children(*stmt) {
            match c.kind() {
                "export_clause" => {
                    for spec in named_children(c) {
                        let Some(n) = spec.child_by_field_name("name") else { continue };
                        let alias = spec.child_by_field_name("alias").unwrap_or(n);
                        if unquote(self.t(alias)) == name {
                            return unquote(self.t(n));
                        }
                    }
                }
                "namespace_export" => return "*".into(),
                _ => {}
            }
        }
        name.to_owned()
    }

    fn imports(&mut self, skip: &[Span]) {
        for i in &self.m.imports {
            let kind = match &i.imported {
                Imported::Named(x) => ImportKind::Named(x.clone()),
                Imported::Default => ImportKind::Default,
                Imported::Namespace => ImportKind::Namespace,
                Imported::Module => ImportKind::Module,
                Imported::Star => ImportKind::Star,
                Imported::SideEffect => ImportKind::SideEffect,
            };
            let local = i.local.map(|id| self.m.symbols[id].name.clone());
            let uses = i.local.map(|id| self.reads(id, skip)).unwrap_or_default();
            let expected: Vec<String> = match (&kind, &local) {
                (ImportKind::Module, Some(l)) if self.python => {
                    let parts: Vec<&str> = i.source.split('.').collect();
                    if parts.len() > 1 && parts[0] == l { parts[1..].iter().map(|s| s.to_string()).collect() } else { Vec::new() }
                }
                _ => Vec::new(),
            };
            let members = if local.is_some() { self.members(&uses, &expected) } else { Some(Vec::new()) };
            self.f.imports.push(ImportFact { source: i.source.clone(), kind, local, span: (i.start, i.end), uses, members, type_only: i.type_only });
        }
    }

    /// The member names read off a binding; `None` if it escapes.
    fn members(&self, uses: &[Span], expected: &[String]) -> Option<Vec<Member>> {
        let mut out = Vec::new();
        for &(s, e) in uses {
            let Some(node) = self.root.descendant_for_byte_range(s, e) else { return None };
            let mut chain: Vec<(String, Span)> = Vec::new();
            let mut cur = node;
            while let Some(p) = cur.parent() {
                let (obj, prop) = match p.kind() {
                    "member_expression" => ("object", "property"),
                    "attribute" => ("object", "attribute"),
                    "nested_type_identifier" => ("module", "name"),
                    "jsx_member_expression" => ("object", "property"),
                    "nested_identifier" => ("object", "property"),
                    _ => break,
                };
                let is_object = p.child_by_field_name(obj).is_some_and(|o| o.id() == cur.id())
                    || (p.kind() == "nested_type_identifier" && p.named_child(0).is_some_and(|o| o.id() == cur.id()));
                if !is_object {
                    break;
                }
                let Some(pn) = p.child_by_field_name(prop).or_else(|| p.named_child(p.named_child_count().saturating_sub(1) as u32)) else {
                    break;
                };
                chain.push((self.t(pn).to_owned(), (pn.start_byte(), pn.end_byte())));
                cur = p;
                if chain.len() > expected.len() {
                    break;
                }
            }
            let prefix_ok = chain.iter().zip(expected).all(|(c, x)| &c.0 == x);
            if !prefix_ok {
                // `a.other` through `import a.b`: another module's member.
                continue;
            }
            match chain.get(expected.len()) {
                Some((name, span)) => out.push(Member { name: name.clone(), span: *span }),
                None => return None,
            }
        }
        out.sort_by(|a, b| a.span.cmp(&b.span));
        Some(out)
    }

    fn string_literal(&self, n: Node<'_>) -> Option<String> {
        match n.kind() {
            "string" => {
                if named_children(n).iter().any(|c| c.kind() == "interpolation") {
                    return None;
                }
                Some(unquote(self.t(n)))
            }
            _ => None,
        }
    }

    /// A template's static prefix, if any part is dynamic.
    fn template_prefix(&self, n: Node<'_>) -> Option<String> {
        let parts = named_children(n);
        let dynamic = parts.iter().any(|c| matches!(c.kind(), "template_substitution" | "interpolation"));
        if !dynamic {
            return None;
        }
        let mut prefix = String::new();
        for c in parts {
            match c.kind() {
                "string_fragment" | "string_content" => prefix.push_str(self.t(c)),
                "string_start" => {}
                _ => break,
            }
        }
        Some(prefix)
    }

    fn walk(&mut self, root: Node<'a>) {
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            match n.kind() {
                "call_expression" | "call" => self.call(n),
                "import_statement" | "export_statement" if !self.python => self.statement_source(n),
                "string_fragment" | "string_content" => {
                    let t = self.t(n);
                    push_tokens(self.f, t);
                    // The fragment right before `${…}` is a class-name prefix.
                    if let Some(next) = n.next_sibling()
                        && matches!(next.kind(), "template_substitution" | "interpolation")
                        && !t.ends_with(char::is_whitespace)
                        && let Some(last) = t.split_whitespace().last()
                    {
                        self.f.class_prefixes.push(last.to_owned());
                    }
                }
                _ => {}
            }
            let mut c = n.walk();
            let kids: Vec<Node<'a>> = n.children(&mut c).collect();
            stack.extend(kids.into_iter().rev());
        }
    }

    /// The `from "x"` of an import or export statement, and the
    /// `require("x")` of `import x = require("x")`.
    fn statement_source(&mut self, n: Node<'_>) {
        let form = if n.kind() == "import_statement" { SpecForm::Import } else { SpecForm::Export };
        let type_only = children(n).iter().any(|c| c.kind() == "type");
        let mut sources: Vec<Node<'_>> = n.child_by_field_name("source").into_iter().collect();
        for c in named_children(n).into_iter().filter(|c| c.kind() == "import_require_clause") {
            sources.extend(c.child_by_field_name("source"));
        }
        for s in sources {
            if let Some(spec) = self.string_literal(s) {
                self.f.specifiers.push(Specifier { spec, span: (s.start_byte(), s.end_byte()), form, type_only });
            }
        }
    }

    fn call(&mut self, n: Node<'_>) {
        let Some(func) = n.child_by_field_name("function") else { return };
        let fname = self.t(func);
        let is_js_import = func.kind() == "import";
        let is_require = !self.python && fname == "require";
        let is_py_import = self.python && matches!(fname, "importlib.import_module" | "import_module" | "__import__");
        if !(is_js_import || is_require || is_py_import) {
            return;
        }
        let Some(arg) = n.child_by_field_name("arguments").and_then(|a| a.named_child(0)) else { return };
        let span = (n.start_byte(), n.end_byte());
        if let Some(lit) = self.string_literal(arg) {
            if !self.python {
                let form = if is_require { SpecForm::Require } else { SpecForm::Dynamic };
                self.f.specifiers.push(Specifier { spec: lit.clone(), span: (arg.start_byte(), arg.end_byte()), form, type_only: false });
            }
            self.f.dynamic_imports.push(DynamicImport { spec: lit, prefix: false, span });
            return;
        }
        let prefix = match arg.kind() {
            "template_string" | "string" => self.template_prefix(arg),
            "binary_expression" => arg.child_by_field_name("left").and_then(|l| self.string_literal(l)),
            _ => None,
        };
        match prefix {
            Some(p) if !p.is_empty() => self.f.dynamic_imports.push(DynamicImport { spec: p, prefix: true, span }),
            _ => {
                if self.f.dynamic.is_none() {
                    self.f.dynamic = Some(format!("`{}` with a computed name", fname.split_whitespace().collect::<String>()));
                }
            }
        }
    }
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    let s = s.trim_start_matches(|c: char| c.is_ascii_alphabetic() && c != '"' && c != '\'');
    let s = s.trim_start_matches("\"\"\"").trim_end_matches("\"\"\"").trim_start_matches("'''").trim_end_matches("'''");
    s.trim_matches(|c| c == '"' || c == '\'' || c == '`').to_owned()
}

fn css(root: Node<'_>, text: &str, f: &mut FileFacts) {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.kind() == "class_selector"
            && let Some(cn) = named_children(n).into_iter().rev().find(|c| c.kind() == "class_name")
        {
            let name: String = text[cn.byte_range()].chars().filter(|c| *c != '\\').collect();
            let mut rule = None;
            if let Some(sel) = n.parent()
                && sel.kind() == "selectors"
                && sel.named_child_count() == 1
                && let Some(rs) = sel.parent()
                && rs.kind() == "rule_set"
            {
                rule = Some(full_lines(text, rs.start_byte(), rs.end_byte()));
            }
            f.css_classes.push(CssClass { name, span: (cn.start_byte(), cn.end_byte()), rule });
        }
        if n.kind() == "at_rule" || n.kind() == "declaration" {
            // `@apply a b`, `composes: a b` name classes too.
            let t = &text[n.byte_range()];
            if t.starts_with("@apply") || t.starts_with("composes") {
                push_tokens(f, t.trim_start_matches("@apply").trim_start_matches("composes:"));
            }
        }
        let mut c = n.walk();
        let kids: Vec<Node<'_>> = n.children(&mut c).collect();
        stack.extend(kids.into_iter().rev());
    }
    f.css_classes.sort_by_key(|c| c.span);
}

/// HTML: every word may be a class (PurgeCSS's default extractor), and
/// `<script src>` / `<link href>` name files.
fn html(text: &str, f: &mut FileFacts) {
    for t in text.split(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '=' | '{' | '}' | '(' | ')' | ',' | ';' | '`')) {
        if !t.is_empty() && t.len() <= 100 {
            f.class_tokens.push(t.to_owned());
        }
    }
    for tag in ["<script", "<link"] {
        let mut rest = text;
        while let Some(i) = rest.find(tag) {
            rest = &rest[i + tag.len()..];
            let end = rest.find('>').unwrap_or(rest.len());
            let attrs = &rest[..end];
            for key in ["src=", "href="] {
                if let Some(k) = attrs.find(key) {
                    let v = attrs[k + key.len()..].trim_start();
                    let q = v.chars().next().unwrap_or(' ');
                    let v = if q == '"' || q == '\'' { v[1..].split(q).next().unwrap_or_default() } else { v.split_whitespace().next().unwrap_or_default() };
                    if !v.is_empty() && !v.contains("://") && !v.starts_with("//") {
                        f.links.push(v.to_owned());
                    }
                }
            }
        }
    }
}
