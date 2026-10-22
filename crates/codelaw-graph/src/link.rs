//! Linking (SR-PROJECT-GRAPH decisions 1–4): resolve every import to the
//! module it names and record, for every top-level declaration, each use
//! from another module — through re-exports, barrels, namespace members
//! and Python packages — plus the entry points that keep a whole module.

use crate::config::GraphConfig;
use crate::facts::{FactLang, FileFacts, ImportKind, Span};
use crate::resolve::{PackageJson, PyProject, Resolver, Target, dirname, join};
use globset::GlobSet;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// One file given to the graph.
pub struct GraphInput {
    pub path: String,
    pub facts: Option<FileFacts>,
}

/// How a declaration or module is used.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Use {
    /// The file the use is in.
    pub file: String,
    pub span: Span,
    /// `import`, `namespace member`, `re-export`, …
    pub how: String,
    /// The reads of the imported binding at that site.
    pub refs: Vec<Span>,
}

pub struct Module {
    pub path: String,
    pub facts: FileFacts,
    /// The files that import it.
    pub importers: BTreeSet<String>,
    /// Why every export of the module counts as used.
    pub all_used: Option<String>,
    /// Why the module is an entry point.
    pub entry: Option<String>,
}

pub struct Graph {
    pub modules: BTreeMap<String, Module>,
    pub resolver: Resolver,
    pub config: GraphConfig,
    /// (module, declaration name) → uses from other modules.
    pub uses: BTreeMap<(String, String), Vec<Use>>,
    /// The packages whose entries could not be mapped to source.
    pub opaque_packages: BTreeSet<String>,
    keep: GlobSet,
}

/// A manifest the graph reads: `package.json`, `tsconfig.json`,
/// `jsconfig.json`, `pyproject.toml`.
pub struct Manifest {
    pub path: String,
    pub text: String,
}

pub fn is_manifest(name: &str) -> bool {
    matches!(name, "package.json" | "tsconfig.json" | "jsconfig.json" | "pyproject.toml" | "setup.py" | "setup.cfg")
}

impl Graph {
    pub fn build(inputs: Vec<GraphInput>, manifests: &[Manifest], config: GraphConfig) -> Graph {
        let mut files: BTreeSet<String> = inputs.iter().map(|i| i.path.clone()).collect();
        let mut packages = Vec::new();
        let mut tsconfigs: Vec<(String, Value)> = Vec::new();
        let mut pyprojects = Vec::new();
        for m in manifests {
            files.insert(m.path.clone());
            let dir = dirname(&m.path);
            match m.path.rsplit('/').next().unwrap_or_default() {
                "package.json" => packages.extend(PackageJson::parse(dir, &m.text)),
                "tsconfig.json" | "jsconfig.json" => tsconfigs.extend(crate::resolve::jsonc(&m.text).map(|v| (m.path.clone(), v))),
                "pyproject.toml" => pyprojects.extend(PyProject::parse(dir, &m.text)),
                _ => {}
            }
        }
        packages.sort_by(|a: &PackageJson, b| a.dir.cmp(&b.dir));
        let resolver = Resolver::new(files, packages, tsconfigs, pyprojects);
        let modules = inputs
            .into_iter()
            .filter_map(|i| {
                let facts = i.facts?;
                Some((i.path.clone(), Module { path: i.path, facts, importers: BTreeSet::new(), all_used: None, entry: None }))
            })
            .collect();
        let keep = config.keep_set();
        let mut g = Graph { modules, resolver, config, uses: BTreeMap::new(), opaque_packages: BTreeSet::new(), keep };
        g.link();
        g
    }

    pub fn is_kept_path(&self, path: &str) -> bool {
        self.keep.is_match(path)
    }

    fn lang(&self, path: &str) -> Option<FactLang> {
        self.modules.get(path).and_then(|m| m.facts.lang)
    }

    fn resolve(&self, from: &str, spec: &str) -> Vec<Target> {
        match self.lang(from) {
            Some(FactLang::Python) => self.resolver.resolve_py(from, spec).into_iter().collect(),
            _ => self.resolver.resolve_js(from, spec),
        }
    }

    fn add_use(&mut self, module: &str, name: &str, u: Use) {
        let v = self.uses.entry((module.to_owned(), name.to_owned())).or_default();
        if !v.contains(&u) {
            v.push(u);
        }
    }

    fn mark_imported(&mut self, target: &Target, by: &str) {
        match target {
            Target::File(f) => {
                if let Some(m) = self.modules.get_mut(f)
                    && f != by
                {
                    m.importers.insert(by.to_owned());
                }
            }
            Target::Package(dir) => {
                self.opaque_packages.insert(dir.clone());
            }
            Target::Namespace(_) => {}
        }
    }

    fn link(&mut self) {
        let paths: Vec<String> = self.modules.keys().cloned().collect();
        for path in &paths {
            let facts = self.modules[path].facts.clone();
            for imp in &facts.imports {
                let targets = self.resolve(path, &imp.source);
                for t in &targets {
                    self.mark_imported(t, path);
                    let site = |how: &str| Use { file: path.clone(), span: imp.span, how: how.to_owned(), refs: imp.uses.clone() };
                    match &imp.kind {
                        ImportKind::Named(n) => {
                            let mut seen = BTreeSet::new();
                            let found = self.use_name(t, n, &site("import"), &mut seen);
                            if !found || self.lang(path) == Some(FactLang::Python) {
                                // A Python name that is a submodule.
                                if let Some(sub) = self.resolver.py_submodule(t, n) {
                                    self.mark_imported(&sub, path);
                                    self.use_members(&sub, imp.members.as_deref(), &site("module import"), path);
                                }
                            }
                        }
                        ImportKind::Default => {
                            let mut seen = BTreeSet::new();
                            let found = self.use_name(t, "default", &site("import"), &mut seen);
                            if !found {
                                // CommonJS or a CSS module: `x.member` reads.
                                self.use_members(t, imp.members.as_deref(), &site("default import member"), path);
                            }
                        }
                        ImportKind::Namespace | ImportKind::Module => {
                            self.use_members(t, imp.members.as_deref(), &site("namespace member"), path);
                        }
                        ImportKind::Star => {
                            let mut seen = BTreeSet::new();
                            self.use_all(t, &format!("`from … import *` in {path}"), &mut seen);
                        }
                        ImportKind::SideEffect => {}
                    }
                }
            }
            for d in &facts.dynamic_imports {
                let reason = format!("dynamic import in {path}");
                if d.prefix {
                    let base = if self.lang(path) == Some(FactLang::Python) {
                        let mut out = Vec::new();
                        for root in self.resolver.py_roots.clone() {
                            out.extend(join(&root, &d.spec.replace('.', "/")));
                        }
                        out
                    } else {
                        join(dirname(path), &d.spec).into_iter().collect()
                    };
                    let hits: Vec<String> = self.modules.keys().filter(|m| base.iter().any(|b| m.starts_with(b.as_str()))).cloned().collect();
                    for h in hits {
                        let t = Target::File(h);
                        self.mark_imported(&t, path);
                        self.use_all(&t, &reason, &mut BTreeSet::new());
                    }
                } else {
                    for t in self.resolve(path, &d.spec) {
                        self.mark_imported(&t, path);
                        self.use_all(&t, &reason, &mut BTreeSet::new());
                    }
                }
            }
            for l in &facts.links {
                let spec = if l.starts_with('/') || l.starts_with('.') { l.clone() } else { format!("./{l}") };
                if let Some(f) = join(dirname(path), &spec).or_else(|| crate::resolve::normalize(l.trim_start_matches('/'))) {
                    let f = self.resolver.probe(&f).unwrap_or(f);
                    self.set_entry(&f, &format!("linked from {path}"));
                }
            }
        }
        self.entries();
    }

    fn set_entry(&mut self, path: &str, why: &str) {
        self.set_entry_keeping(path, why, true);
    }

    /// An entry is reachable from outside; `keep_all` also counts every
    /// export of it as used (a public package's surface, a framework file).
    fn set_entry_keeping(&mut self, path: &str, why: &str, keep_all: bool) {
        let Some(m) = self.modules.get_mut(path) else { return };
        if m.entry.is_none() {
            m.entry = Some(why.to_owned());
        }
        if keep_all {
            self.use_all(&Target::File(path.to_owned()), why, &mut BTreeSet::new());
        }
    }

    /// Entry points (decision 3).
    fn entries(&mut self) {
        for i in 0..self.resolver.packages.len() {
            let (files, unmapped) = self.resolver.package_entries(i);
            let p = &self.resolver.packages[i];
            let label = format!("{}/package.json", if p.dir.is_empty() { "." } else { &p.dir });
            let why = if p.private { format!("an entry of {label}") } else { format!("the public surface of {label}") };
            if unmapped && !p.private {
                self.opaque_packages.insert(p.dir.clone());
            }
            // A private package's entries are reachable, but each export
            // still needs an importer; a public one keeps its whole surface.
            let keep_all = !p.private;
            for f in files {
                self.set_entry_keeping(&f, &why, keep_all);
            }
        }
        for i in 0..self.resolver.pyprojects.len() {
            let p = self.resolver.pyprojects[i].clone();
            for r in &p.entry_refs {
                let (module, attr) = r.split_once(':').unwrap_or((r.as_str(), ""));
                let attr = attr.split('.').next().unwrap_or_default().split_whitespace().next().unwrap_or_default();
                let Some(t) = self.resolver.resolve_py(&format!("{}/x.py", p.dir), module.trim()) else { continue };
                let site = Use { file: format!("{}pyproject.toml", if p.dir.is_empty() { String::new() } else { format!("{}/", p.dir) }), span: (0, 0), how: "entry point".into(), refs: Vec::new() };
                if attr.is_empty() {
                    if let Target::File(f) = &t {
                        self.set_entry(f, "a `pyproject.toml` entry point");
                    }
                } else {
                    self.use_name(&t, attr, &site, &mut BTreeSet::new());
                }
            }
        }
        let entry_globs = self.config.entry_set();
        let paths: Vec<String> = self.modules.keys().cloned().collect();
        for path in paths {
            let m = &self.modules[&path];
            let why = if m.facts.is_main {
                Some("it runs as `__main__`".to_owned())
            } else if entry_globs.is_match(&path) {
                Some("`[graph] entry`".to_owned())
            } else {
                preset_entry(&path, m.facts.lang)
            };
            if let Some(w) = why {
                self.set_entry(&path, &w);
            }
        }
        // A Python distribution's top-level packages are its public surface.
        let dists: Vec<String> = self.resolver.pyprojects.iter().filter(|p| p.distribution).map(|p| p.dir.clone()).collect();
        if !dists.is_empty() {
            let inits: Vec<String> = self
                .modules
                .keys()
                .filter(|f| f.ends_with("__init__.py"))
                .filter(|f| self.resolver.py_module_name(f).is_some_and(|n| !n.contains('.')))
                .filter(|f| dists.iter().any(|d| d.is_empty() || f.starts_with(&format!("{d}/"))))
                .cloned()
                .collect();
            for f in inits {
                self.set_entry(&f, "the public surface of a `pyproject.toml` distribution");
            }
        }
    }

    /// Mark the declaration `name` of `target` used; follow re-exports.
    /// Whether anything of that name was found.
    pub(crate) fn use_name(&mut self, target: &Target, name: &str, site: &Use, seen: &mut BTreeSet<(String, String)>) -> bool {
        let Target::File(path) = target else { return false };
        if !seen.insert((path.clone(), name.to_owned())) {
            return true;
        }
        let Some(m) = self.modules.get(path) else { return false };
        let facts = m.facts.clone();
        let python = facts.lang == Some(FactLang::Python);
        let mut found = false;
        for e in facts.exports.iter().filter(|e| e.name == name) {
            if let Some(local) = &e.local {
                self.add_use(path, local, site.clone());
                found = true;
            } else if let (Some(src), Some(imported)) = (&e.source, &e.imported) {
                found = true;
                let via = Use { how: format!("{} (re-exported by {path})", site.how), ..site.clone() };
                for t in self.resolve(path, src) {
                    self.mark_imported(&t, path);
                    if imported == "*" {
                        self.use_all(&t, &format!("re-exported as a namespace by {path}"), &mut BTreeSet::new());
                    } else if !self.use_name(&t, imported, &via, seen) && python {
                        if let Some(sub) = self.resolver.py_submodule(&t, imported) {
                            self.use_all(&sub, &format!("re-exported by {path}"), &mut BTreeSet::new());
                        }
                    }
                }
            } else {
                found = true;
            }
        }
        if found {
            return true;
        }
        if python {
            // Any module-level binding is importable, imports included.
            if facts.decls.iter().any(|d| d.name == name) {
                self.add_use(path, name, site.clone());
                return true;
            }
            for imp in facts.imports.iter().filter(|i| i.local.as_deref() == Some(name)) {
                let via = Use { how: format!("{} (through {path})", site.how), ..site.clone() };
                if let Some(t) = self.resolver.resolve_py(path, &imp.source) {
                    self.mark_imported(&t, path);
                    match &imp.kind {
                        ImportKind::Named(n) => {
                            if !self.use_name(&t, n, &via, seen)
                                && let Some(sub) = self.resolver.py_submodule(&t, n)
                            {
                                self.use_all(&sub, &format!("re-exported by {path}"), &mut BTreeSet::new());
                            }
                        }
                        _ => self.use_all(&t, &format!("re-exported by {path}"), &mut BTreeSet::new()),
                    }
                }
                return true;
            }
            for imp in facts.imports.iter().filter(|i| i.kind == ImportKind::Star) {
                if let Some(t) = self.resolver.resolve_py(path, &imp.source)
                    && self.use_name(&t, name, site, seen)
                {
                    return true;
                }
            }
            return false;
        }
        // `export * from`: names pass through.
        for e in facts.exports.iter().filter(|e| e.name == "*") {
            if let Some(src) = &e.source {
                for t in self.resolve(path, src) {
                    self.mark_imported(&t, path);
                    if self.use_name(&t, name, site, seen) {
                        found = true;
                    }
                }
            }
        }
        found
    }

    /// Namespace / module members, or everything when the binding escapes.
    fn use_members(&mut self, target: &Target, members: Option<&[crate::facts::Member]>, site: &Use, from: &str) {
        match members {
            None => {
                self.use_all(target, &format!("its namespace is passed around in {from}"), &mut BTreeSet::new());
            }
            Some(list) => {
                for mem in list {
                    let u = Use { span: mem.span, refs: vec![mem.span], ..site.clone() };
                    let mut seen = BTreeSet::new();
                    if !self.use_name(target, &mem.name, &u, &mut seen)
                        && let Some(sub) = self.resolver.py_submodule(target, &mem.name)
                    {
                        self.mark_imported(&sub, from);
                        self.use_all(&sub, &format!("`{}` read in {from}", mem.name), &mut BTreeSet::new());
                    }
                }
            }
        }
    }

    /// Every export of a module counts as used.
    fn use_all(&mut self, target: &Target, why: &str, seen: &mut BTreeSet<String>) {
        let Target::File(path) = target else {
            if let Target::Package(dir) = target {
                self.opaque_packages.insert(dir.clone());
            }
            return;
        };
        if !seen.insert(path.clone()) {
            return;
        }
        let Some(m) = self.modules.get_mut(path) else { return };
        if m.all_used.is_none() {
            m.all_used = Some(why.to_owned());
        }
        let facts = m.facts.clone();
        for e in &facts.exports {
            if let (Some(src), Some(imported)) = (&e.source, &e.imported) {
                for t in self.resolve(path, src) {
                    self.mark_imported(&t, path);
                    if imported == "*" {
                        self.use_all(&t, &format!("re-exported by {path}"), seen);
                    } else {
                        let site = Use { file: path.clone(), span: e.span, how: format!("re-export ({why})"), refs: Vec::new() };
                        let found = self.use_name(&t, imported, &site, &mut BTreeSet::new());
                        if !found && let Some(sub) = self.resolver.py_submodule(&t, imported) {
                            self.use_all(&sub, &format!("re-exported by {path}"), seen);
                        }
                    }
                }
            }
        }
        if facts.lang == Some(FactLang::Python) {
            for imp in &facts.imports {
                if imp.kind == ImportKind::Star
                    && let Some(t) = self.resolver.resolve_py(path, &imp.source)
                {
                    self.use_all(&t, &format!("star-imported by {path}"), seen);
                }
            }
        }
    }

    /// Whether every module of a workspace package counts as used.
    pub fn in_opaque_package(&self, path: &str) -> Option<String> {
        self.opaque_packages.iter().find(|d| d.is_empty() || path.starts_with(&format!("{d}/"))).cloned()
    }

    /// Uses of a declaration from other modules.
    pub fn uses_of(&self, module: &str, name: &str) -> &[Use] {
        self.uses.get(&(module.to_owned(), name.to_owned())).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Framework presets (decision 3): files used from outside the repo.
fn preset_entry(path: &str, lang: Option<FactLang>) -> Option<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let stem = name.split('.').next().unwrap_or(name);
    let segs: Vec<&str> = path.split('/').collect();
    let in_dir = |d: &str| segs[..segs.len() - 1].contains(&d);
    match lang {
        Some(FactLang::Python) => {
            if name == "conftest.py" || name.starts_with("test_") || name.ends_with("_test.py") || name == "__main__.py" {
                return Some(format!("pytest / `__main__` preset (`{name}`)"));
            }
            if name == "manage.py" || name == "setup.py" || name == "noxfile.py" || name == "fabfile.py" || name == "tasks.py" {
                return Some(format!("tool preset (`{name}`)"));
            }
            if in_dir("migrations") {
                return Some("Django migrations preset".into());
            }
            None
        }
        Some(FactLang::Js) => {
            let parts: Vec<&str> = name.split('.').collect();
            if parts.len() >= 3 && matches!(parts[parts.len() - 2], "test" | "spec" | "stories" | "story" | "bench" | "e2e") {
                return Some(format!("test / story preset (`{name}`)"));
            }
            if parts.len() >= 3 && parts[parts.len() - 2] == "config" || name.starts_with('.') {
                return Some(format!("`*.config.*` preset (`{name}`)"));
            }
            if in_dir("__tests__") || in_dir("__mocks__") || in_dir(".storybook") {
                return Some("test / Storybook preset".into());
            }
            // Next.js routes.
            if in_dir("pages") || (in_dir("app") && matches!(stem, "page" | "layout" | "route" | "loading" | "error" | "not-found" | "template" | "default" | "head" | "middleware" | "global-error" | "opengraph-image" | "sitemap" | "robots")) {
                return Some("Next.js routes preset".into());
            }
            if matches!(stem, "middleware" | "instrumentation") && segs.len() <= 2 {
                return Some("Next.js preset".into());
            }
            None
        }
        _ => None,
    }
}
