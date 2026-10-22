//! `graph:` queries (SR-PROJECT-GRAPH decision 6), their findings and fixes,
//! and the two debug commands: `graph refs` and `graph why-unused`.

use crate::facts::{Decl, DeclKind, FactLang, Span, SpecForm};
use crate::link::{Graph, Module, Use};
use crate::resolve::{Target, is_node_builtin, package_name};
use codelaw_core::{Language, TextEdit};
use serde_json::Value;
use std::collections::BTreeSet;

/// A query a `graph:` block names.
#[derive(Clone, Debug, PartialEq)]
pub enum Query {
    /// `symbol: { exported: true, kind: [...] }` with `unless:` and `when:`.
    UnusedExports { kinds: Vec<DeclKind>, tags: Vec<String>, entry: bool },
    /// `class: { defined_in: css }`.
    UnusedCssClasses,
    /// `deprecated: registry` (SR-BASELINE decision 6).
    DeprecatedUsage,
    /// `import: { form: [...], resolves | declared | published: false }`:
    /// the `node/` import rules.
    Imports { forms: Vec<SpecForm>, check: ImportCheck },
}

/// What a `graph: import:` query reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportCheck {
    /// `resolves: false`: a path of the project that names no file.
    Missing,
    /// `declared: false`: a package the nearest `package.json` doesn't declare.
    Undeclared,
    /// `published: false`: a file `npm publish` packs imports one it
    /// doesn't, or a package that is only a devDependency.
    Unpublished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    StripExportKeyword,
    RemoveDeclaration,
    RemoveRule,
}

/// A `fix:` of `when`/`else` branches over `local_references`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphFix {
    /// Applied when the symbol is still used in its own file.
    pub when_local: Vec<Primitive>,
    pub otherwise: Vec<Primitive>,
}

/// A compiled `graph:` rule.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphRule {
    pub query: Query,
    pub fix: GraphFix,
    /// `certainty_from: module.dynamic`.
    pub certainty_from_dynamic: bool,
}

fn strings(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_owned).collect(),
        _ => Vec::new(),
    }
}

fn primitives(v: Option<&Value>) -> Result<Vec<Primitive>, String> {
    strings(v)
        .iter()
        .map(|p| match p.as_str() {
            "strip-export-keyword" => Ok(Primitive::StripExportKeyword),
            "remove-declaration" => Ok(Primitive::RemoveDeclaration),
            "remove-rule" => Ok(Primitive::RemoveRule),
            other => Err(format!("unknown graph fix primitive `{other}` (strip-export-keyword, remove-declaration, remove-rule)")),
        })
        .collect()
}

/// Compile a rule's `graph:` block, its `fix:` and `certainty_from:`.
pub fn compile(graph: &Value, fix: Option<&Value>) -> Result<GraphRule, String> {
    let Value::Object(g) = graph else { return Err("`graph:` must be a mapping".into()) };
    let certainty_from_dynamic = match g.get("certainty_from") {
        None => false,
        Some(Value::String(s)) if s == "module.dynamic" => true,
        Some(other) => return Err(format!("`certainty_from: {other}` must be `module.dynamic`")),
    };
    for k in g.keys() {
        if !["symbol", "unless", "when", "class", "deprecated", "import", "certainty_from"].contains(&k.as_str()) {
            return Err(format!("unknown `graph:` key `{k}`"));
        }
    }
    let query = if let Some(sym) = g.get("symbol") {
        if sym.get("exported").and_then(Value::as_bool) != Some(true) {
            return Err("`graph: symbol:` queries need `exported: true`".into());
        }
        let mut kinds = Vec::new();
        for k in strings(sym.get("kind")) {
            kinds.push(DeclKind::parse(&k).ok_or_else(|| format!("unknown symbol kind `{k}`"))?);
        }
        if kinds.is_empty() {
            kinds = vec![DeclKind::Function, DeclKind::Class, DeclKind::Variable, DeclKind::Type, DeclKind::Interface, DeclKind::Enum];
        }
        let mut tags = Vec::new();
        let mut entry = false;
        if let Some(Value::Array(unless)) = g.get("unless") {
            for u in unless {
                if let Some(r) = u.get("reachable_from") {
                    if r.as_str() != Some("entry") {
                        return Err("`reachable_from:` takes `entry`".into());
                    }
                    entry = true;
                } else if u.get("tagged").is_some() {
                    tags.extend(strings(u.get("tagged")));
                } else {
                    return Err(format!("unknown `unless:` clause {u}"));
                }
            }
        }
        match g.get("when").and_then(|w| w.get("external_references")) {
            Some(v) if v.as_u64() == Some(0) || v.as_str() == Some("0") => {}
            _ => return Err("`graph: symbol:` queries need `when: { external_references: 0 }`".into()),
        }
        Query::UnusedExports { kinds, tags, entry }
    } else if let Some(class) = g.get("class") {
        if class.get("defined_in").and_then(Value::as_str) != Some("css") {
            return Err("`graph: class:` queries need `defined_in: css`".into());
        }
        Query::UnusedCssClasses
    } else if g.get("deprecated").is_some() {
        Query::DeprecatedUsage
    } else if let Some(imp) = g.get("import") {
        let Value::Object(o) = imp else { return Err("`graph: import:` must be a mapping".into()) };
        let mut forms = Vec::new();
        for f in strings(o.get("form")) {
            forms.push(SpecForm::parse(&f).ok_or_else(|| format!("unknown import form `{f}` (import, export, dynamic, require)"))?);
        }
        if forms.is_empty() {
            forms = vec![SpecForm::Import, SpecForm::Export, SpecForm::Dynamic, SpecForm::Require];
        }
        let mut checks = Vec::new();
        for (k, v) in o {
            let check = match k.as_str() {
                "form" => continue,
                "resolves" => ImportCheck::Missing,
                "declared" => ImportCheck::Undeclared,
                "published" => ImportCheck::Unpublished,
                other => return Err(format!("unknown `graph: import:` key `{other}`")),
            };
            if v.as_bool() != Some(false) {
                return Err(format!("`graph: import: {k}:` takes `false`"));
            }
            checks.push(check);
        }
        let [check] = checks[..] else {
            return Err("`graph: import:` takes exactly one of `resolves: false`, `declared: false`, `published: false`".into());
        };
        Query::Imports { forms, check }
    } else {
        return Err("`graph:` names no query (`symbol:`, `class:`, `import:` or `deprecated:`)".into());
    };
    let mut gf = GraphFix::default();
    match fix {
        None => {}
        Some(Value::Array(branches)) => {
            for b in branches {
                if let Some(w) = b.get("when") {
                    let lr = w.get("local_references").map(|v| v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string()));
                    if lr.as_deref() != Some(">0") {
                        return Err("a graph fix's `when:` takes `{ local_references: \">0\" }`".into());
                    }
                    gf.when_local = primitives(b.get("do"))?;
                } else if b.get("else").is_some() || b.get("do").is_some() {
                    gf.otherwise = primitives(b.get("do"))?;
                } else {
                    return Err("a graph fix branch is `when:` + `do:` or `else:` + `do:`".into());
                }
            }
        }
        Some(Value::String(_)) | Some(Value::Object(_)) => {
            gf.otherwise = primitives(fix.and_then(|f| f.get("do")).or(fix))?;
            gf.when_local = gf.otherwise.clone();
        }
        Some(other) => return Err(format!("unsupported graph `fix:` {other}")),
    }
    Ok(GraphRule { query, fix: gf, certainty_from_dynamic })
}

/// A project-rule finding, before it becomes a diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphFinding {
    pub file: String,
    pub span: Span,
    /// Capture values for the message: `SYMBOL`, `FILE`, `KIND`, …
    pub captures: Vec<(String, String)>,
    pub edits: Vec<TextEdit>,
    /// The module is `dynamic`: report as `info`, `certainty: uncertain`.
    pub uncertain: Option<String>,
    /// A fixed message, for queries that build their own.
    pub message: Option<String>,
}

impl GraphFinding {
    /// `$NAME` interpolation of a rule's message.
    pub fn render(&self, template: &str) -> String {
        if let Some(m) = &self.message {
            return m.clone();
        }
        let mut caps = self.captures.clone();
        caps.sort_by_key(|b| std::cmp::Reverse(b.0.len()));
        let mut out = template.to_owned();
        for (k, v) in caps {
            out = out.replace(&format!("${k}"), &v);
        }
        out
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// One `unless` / `when` clause and how it came out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub clause: String,
    /// Whether this clause keeps the symbol (it is not reported).
    pub keeps: bool,
    pub detail: String,
}

/// The verdict on one exported declaration.
pub struct Verdict {
    pub clauses: Vec<Clause>,
    pub reported: bool,
}

fn export_names(m: &Module, d: &Decl) -> Vec<String> {
    m.facts.exports.iter().filter(|e| e.local.as_deref() == Some(d.name.as_str())).map(|e| e.name.clone()).collect()
}

/// Every clause of `graph/no-unused-exports` for one declaration.
pub fn evaluate(graph: &Graph, m: &Module, d: &Decl, kinds: &[DeclKind], tags: &[String]) -> Verdict {
    let mut clauses = Vec::new();
    let mut c = |clause: &str, keeps: bool, detail: String| clauses.push(Clause { clause: clause.into(), keeps, detail });
    let names = export_names(m, d);
    let python = m.facts.lang == Some(FactLang::Python);
    c(
        "symbol.exported",
        names.is_empty(),
        if names.is_empty() { "not exported".into() } else { format!("exported as {}", names.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")) },
    );
    c("symbol.kind", !kinds.contains(&d.kind), format!("`{}` (the rule takes {})", d.kind.name(), kinds.iter().map(|k| k.name()).collect::<Vec<_>>().join(", ")));
    let entry = m.all_used.clone().or_else(|| graph.in_opaque_package(&m.path).map(|p| format!("its workspace package `{}` has entries that could not be mapped to source files", if p.is_empty() { "." } else { &p })));
    c("unless reachable_from: entry", entry.is_some(), entry.unwrap_or_else(|| "no entry point reaches the whole module".into()));
    let tagged: Vec<&String> = d.tags.iter().filter(|t| tags.contains(t)).collect();
    c(
        "unless tagged",
        !tagged.is_empty(),
        if tagged.is_empty() { format!("no {} tag", tags.iter().map(|t| format!("`{t}`")).collect::<Vec<_>>().join(" / ")) } else { format!("tagged `{}`", tagged[0]) },
    );
    let reg = d.decorators.iter().find(|x| graph.config.is_registrar(x));
    c("unless decorated by a registrar", reg.is_some(), reg.map(|r| format!("`@{r}`")).unwrap_or_else(|| "no registrar decorator".into()));
    let kept = graph.is_kept_path(&m.path);
    c("unless [graph] keep", kept, if kept { "the file matches `[graph] keep`".into() } else { "not in `[graph] keep`".into() });
    let uses: Vec<&Use> = graph.uses_of(&m.path, &d.name).iter().filter(|u| u.file != m.path).collect();
    c(
        "when external_references: 0",
        !uses.is_empty(),
        if uses.is_empty() { "no other module uses it".into() } else { format!("{} use(s), first in {} ({})", uses.len(), uses[0].file, uses[0].how) },
    );
    let imported = !m.importers.is_empty() || m.entry.is_some();
    c(
        "the module is imported",
        !imported,
        if imported {
            match &m.entry {
                Some(e) if m.importers.is_empty() => format!("an entry point: {e}"),
                _ => format!("imported by {} module(s)", m.importers.len()),
            }
        } else {
            "no module imports this file; whether the file is used at all is `graph/no-unused-files`' question".into()
        },
    );
    if python {
        let implicit = !m.facts.python_all;
        let keeps = implicit && !d.local_uses.is_empty();
        c(
            "python: listed in `__all__`, or unused in its own module",
            keeps,
            if !implicit {
                "the module sets `__all__`".into()
            } else if keeps {
                "no `__all__`, so the name is public only by convention, and its own module uses it".into()
            } else {
                "no `__all__`, and its own module does not use it either".into()
            },
        );
    }
    let reported = clauses.iter().all(|c| !c.keeps);
    Verdict { clauses, reported }
}

fn languages_match(path: &str, languages: &[Language]) -> bool {
    Language::from_path(std::path::Path::new(path)).is_some_and(|l| languages.contains(&l))
}

/// Declarations another module re-exports by name (`export { x } from`):
/// deleting one would break that re-export.
fn reexported(graph: &Graph) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    for m in graph.modules.values() {
        for e in &m.facts.exports {
            if let (Some(src), Some(imp)) = (&e.source, &e.imported)
                && imp != "*"
            {
                for t in resolve_from(graph, &m.path, src) {
                    out.extend(find_decl(graph, &t, imp, 0));
                }
            }
        }
    }
    out
}

fn fix_edits(fix: &GraphFix, d: &Decl, python_all: bool) -> Vec<TextEdit> {
    let branch = if d.local_uses.is_empty() { &fix.otherwise } else { &fix.when_local };
    for p in branch {
        match p {
            Primitive::StripExportKeyword => {
                if let Some((a, b)) = d.export_kw {
                    return vec![TextEdit::delete(a, b)];
                }
            }
            Primitive::RemoveDeclaration => {
                if let Some((a, b)) = d.stmt
                    && d.local_uses.is_empty()
                    && !python_all
                    && !d.listed_export
                {
                    return vec![TextEdit::delete(a, b)];
                }
            }
            Primitive::RemoveRule => {}
        }
    }
    Vec::new()
}

/// Run a compiled rule over the graph, for files in `languages`.
pub fn run(rule: &GraphRule, graph: &Graph, languages: &[Language]) -> Vec<GraphFinding> {
    match &rule.query {
        Query::UnusedExports { kinds, tags, .. } => unused_exports(rule, graph, languages, kinds, tags),
        Query::UnusedCssClasses => unused_css(rule, graph, languages),
        // The engine runs `graph/no-deprecated-usage` itself, with the
        // registry and the baseline (SR-BASELINE; the `deprecated` module).
        Query::DeprecatedUsage => Vec::new(),
        Query::Imports { forms, check } => imports(graph, languages, forms, *check),
    }
}

/// Whether `node/no-missing-import` judges a path: no extension, or a
/// JS/TS one. Any other (`.svg`, `.css`, `.json`) may be a file the
/// graph doesn't list, or one a bundler makes.
fn judged(spec: &str) -> bool {
    let last = spec.rsplit('/').next().unwrap_or(spec);
    match last.rsplit_once('.') {
        None => true,
        Some((stem, ext)) => stem.is_empty() || matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts"),
    }
}

fn manifest_path(dir: &str) -> String {
    if dir.is_empty() { "package.json".into() } else { format!("{dir}/package.json") }
}

fn imports(graph: &Graph, languages: &[Language], forms: &[SpecForm], check: ImportCheck) -> Vec<GraphFinding> {
    let r = &graph.resolver;
    let mut out = Vec::new();
    for m in graph.modules.values() {
        if m.facts.lang != Some(FactLang::Js) || !languages_match(&m.path, languages) {
            continue;
        }
        let pkg = r.package_of(&m.path);
        for s in m.facts.specifiers.iter().filter(|s| forms.contains(&s.form)) {
            let spec = s.spec.split('?').next().unwrap_or(&s.spec);
            let local = r.is_local_spec(&m.path, spec);
            let mut captures = vec![("SOURCE".to_owned(), s.spec.clone()), ("FILE".to_owned(), file_name(&m.path).to_owned())];
            let hit = match check {
                ImportCheck::Missing => local && judged(spec) && r.resolve_js(&m.path, spec).is_empty(),
                ImportCheck::Undeclared => {
                    let Some(p) = pkg.map(|i| &r.packages[i]) else { continue };
                    let Some(name) = package_name(spec).filter(|_| !local && !is_node_builtin(spec)) else { continue };
                    // `baseUrl` makes a bare-looking path the project's own.
                    let own_file = r.workspace_package(name).is_none() && r.resolve_js(&m.path, spec).iter().any(|t| matches!(t, Target::File(_)));
                    let types = format!("@types/{}", name.trim_start_matches('@').replace('/', "__"));
                    let declared = p.deps.contains(name) || p.dev_deps.contains(name) || (s.type_only && (p.deps.contains(&types) || p.dev_deps.contains(&types)));
                    captures.push(("PACKAGE".into(), name.to_owned()));
                    captures.push(("MANIFEST".into(), manifest_path(&p.dir)));
                    !own_file && !declared && p.name.as_deref() != Some(name)
                }
                ImportCheck::Unpublished => {
                    let Some(i) = pkg else { continue };
                    let p = &r.packages[i];
                    if p.private || p.files.is_none() || !r.published(i, &m.path) {
                        continue;
                    }
                    captures.push(("MANIFEST".into(), manifest_path(&p.dir)));
                    if local {
                        r.resolve_js(&m.path, spec).iter().any(|t| matches!(t, Target::File(f) if r.package_of(f) == Some(i) && !r.published(i, f)))
                    } else if let Some(name) = package_name(spec).filter(|_| !s.type_only) {
                        captures.push(("PACKAGE".into(), name.to_owned()));
                        p.dev_deps.contains(name) && !p.deps.contains(name)
                    } else {
                        false
                    }
                }
            };
            if hit {
                out.push(GraphFinding { file: m.path.clone(), span: s.span, captures, edits: Vec::new(), uncertain: None, message: None });
            }
        }
    }
    out
}

fn unused_exports(rule: &GraphRule, graph: &Graph, languages: &[Language], kinds: &[DeclKind], tags: &[String]) -> Vec<GraphFinding> {
    let mut out = Vec::new();
    let reexported = reexported(graph);
    for m in graph.modules.values() {
        if !matches!(m.facts.lang, Some(FactLang::Js | FactLang::Python)) || !languages_match(&m.path, languages) {
            continue;
        }
        for d in &m.facts.decls {
            let v = evaluate(graph, m, d, kinds, tags);
            if !v.reported {
                continue;
            }
            let uncertain = if rule.certainty_from_dynamic { m.facts.dynamic.clone() } else { None };
            out.push(GraphFinding {
                file: m.path.clone(),
                span: d.span,
                captures: vec![("SYMBOL".into(), d.name.clone()), ("FILE".into(), file_name(&m.path).into()), ("KIND".into(), d.kind.name().into())],
                edits: if reexported.contains(&(m.path.clone(), d.name.clone())) { Vec::new() } else { fix_edits(&rule.fix, d, m.facts.python_all) },
                uncertain,
                message: None,
            });
        }
    }
    out
}

fn unused_css(rule: &GraphRule, graph: &Graph, languages: &[Language]) -> Vec<GraphFinding> {
    let mut tokens: BTreeSet<&str> = BTreeSet::new();
    let mut prefixes: Vec<&str> = Vec::new();
    let mut content = false;
    for m in graph.modules.values() {
        if m.facts.lang != Some(FactLang::Css) {
            content = true;
        }
        tokens.extend(m.facts.class_tokens.iter().map(String::as_str));
        prefixes.extend(m.facts.class_prefixes.iter().map(String::as_str));
        // CSS modules: `styles.name` reads.
        for imp in &m.facts.imports {
            if imp.source.ends_with(".css")
                && let Some(members) = &imp.members
            {
                tokens.extend(members.iter().map(|x| x.name.as_str()));
            }
        }
    }
    if !content {
        return Vec::new();
    }
    let removes = rule.fix.otherwise.contains(&Primitive::RemoveRule);
    let mut out = Vec::new();
    for m in graph.modules.values() {
        if m.facts.lang != Some(FactLang::Css) || !languages_match(&m.path, languages) || m.entry.is_some() || graph.is_kept_path(&m.path) {
            continue;
        }
        if graph.in_opaque_package(&m.path).is_some() && m.entry.is_some() {
            continue;
        }
        for c in &m.facts.css_classes {
            if tokens.contains(c.name.as_str()) || prefixes.iter().any(|p| c.name.starts_with(p)) {
                continue;
            }
            let edits = match (removes, c.rule) {
                (true, Some((a, b))) => vec![TextEdit::delete(a, b)],
                _ => Vec::new(),
            };
            out.push(GraphFinding {
                file: m.path.clone(),
                span: c.span,
                captures: vec![("CLASS".into(), c.name.clone()), ("FILE".into(), file_name(&m.path).into())],
                edits,
                uncertain: None,
                message: None,
            });
        }
    }
    out
}

/// A symbol named on the command line: `<package or path>#<export>` (JS/TS)
/// or `<module>:<name>` (Python).
pub fn locate(graph: &Graph, spec: &str) -> Result<(String, String), String> {
    let (target, name) = if let Some((m, n)) = spec.rsplit_once('#') {
        let t = if graph.modules.contains_key(m) {
            Target::File(m.to_owned())
        } else {
            graph
                .resolver
                .resolve_js("", m)
                .into_iter()
                .find(|t| matches!(t, Target::File(_)))
                .or_else(|| graph.resolver.probe(m).map(Target::File))
                .ok_or_else(|| format!("`{m}` is neither a file nor a workspace package of this project"))?
        };
        (t, n.to_owned())
    } else if let Some((m, n)) = spec.split_once(':') {
        let t = if graph.modules.contains_key(m) {
            Target::File(m.to_owned())
        } else {
            graph.resolver.resolve_py("__codelaw__.py", m).ok_or_else(|| format!("no Python module `{m}` in this project"))?
        };
        (t, n.split('.').next().unwrap_or(n).to_owned())
    } else {
        return Err(format!("`{spec}`: name a symbol as `<package or path>#<export>` or `<module>:<name>`"));
    };
    find_decl(graph, &target, &name, 0).ok_or_else(|| format!("`{spec}`: no top-level declaration `{name}` found"))
}

/// The declaration an exported name ends at, through re-exports.
pub fn find_decl(graph: &Graph, target: &Target, name: &str, depth: usize) -> Option<(String, String)> {
    let Target::File(path) = target else { return None };
    let m = graph.modules.get(path)?;
    if depth > 16 {
        return None;
    }
    if let Some(e) = m.facts.exports.iter().find(|e| e.name == name) {
        if let Some(l) = &e.local {
            return Some((path.clone(), l.clone()));
        }
        if let (Some(src), Some(imp)) = (&e.source, &e.imported) {
            for t in resolve_from(graph, path, src) {
                if let Some(x) = find_decl(graph, &t, imp, depth + 1) {
                    return Some(x);
                }
            }
        }
    }
    if m.facts.decls.iter().any(|d| d.name == name) {
        return Some((path.clone(), name.to_owned()));
    }
    for imp in &m.facts.imports {
        let follows = imp.local.as_deref() == Some(name) || (imp.kind == crate::facts::ImportKind::Star);
        if !follows {
            continue;
        }
        let inner = match &imp.kind {
            crate::facts::ImportKind::Named(n) => n.clone(),
            crate::facts::ImportKind::Default => "default".into(),
            _ => name.to_owned(),
        };
        for t in resolve_from(graph, path, &imp.source) {
            if let Some(x) = find_decl(graph, &t, &inner, depth + 1) {
                return Some(x);
            }
        }
    }
    for e in m.facts.exports.iter().filter(|e| e.name == "*") {
        if let Some(src) = &e.source {
            for t in resolve_from(graph, path, src) {
                if let Some(x) = find_decl(graph, &t, name, depth + 1) {
                    return Some(x);
                }
            }
        }
    }
    None
}

fn resolve_from(graph: &Graph, from: &str, spec: &str) -> Vec<Target> {
    match graph.modules.get(from).and_then(|m| m.facts.lang) {
        Some(FactLang::Python) => graph.resolver.resolve_py(from, spec).into_iter().collect(),
        _ => graph.resolver.resolve_js(from, spec),
    }
}

/// `codelaw graph refs`: every use of a symbol and what keeps its module.
pub fn refs(graph: &Graph, spec: &str) -> Result<String, String> {
    let (module, name) = locate(graph, spec)?;
    let m = &graph.modules[&module];
    let d = m.facts.decls.iter().find(|d| d.name == name).ok_or("not a declaration")?;
    let mut out = format!("{module}#{name}  ({} declared at byte {})\n", d.kind.name(), d.span.0);
    let uses = graph.uses_of(&module, &name);
    if uses.is_empty() {
        out.push_str("  no use from another module\n");
    }
    for u in uses {
        out.push_str(&format!("  {}  {}\n", u.file, u.how));
    }
    out.push_str(&format!("  {} use(s) in its own file\n", d.local_uses.len()));
    match (&m.entry, &m.all_used) {
        (Some(e), _) => out.push_str(&format!("  entry point: {e}\n")),
        (None, Some(a)) => out.push_str(&format!("  every export of the module counts as used: {a}\n")),
        _ => out.push_str("  no entry point reaches the whole module\n"),
    }
    Ok(out)
}

/// `codelaw graph why-unused`: each clause and why it held or failed.
pub fn why_unused(graph: &Graph, spec: &str, kinds: &[DeclKind], tags: &[String]) -> Result<String, String> {
    let (module, name) = locate(graph, spec)?;
    let m = &graph.modules[&module];
    let d = m.facts.decls.iter().find(|d| d.name == name).ok_or("not a declaration")?;
    let v = evaluate(graph, m, d, kinds, tags);
    let mut out = format!("{module}#{name}\n");
    for c in &v.clauses {
        out.push_str(&format!("  {} {:<62} {}\n", if c.keeps { "keeps " } else { "passes" }, c.clause, c.detail));
    }
    let verdict = if v.reported {
        match &m.facts.dynamic {
            Some(r) => format!("reported as info, uncertain: {r}"),
            None => "reported: exported and unused".into(),
        }
    } else {
        "not reported: a clause keeps it".into()
    };
    out.push_str(&format!("  → {verdict}\n"));
    Ok(out)
}
