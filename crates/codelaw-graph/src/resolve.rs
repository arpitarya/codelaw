//! Module resolution without a type checker (SR-PROJECT-GRAPH decision 2),
//! over the project's own file set: no file is read here, so the result
//! is a pure function of the files given (L5) and the rule tester's
//! virtual repos resolve exactly as a disk checkout does.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// `a/b/../c` → `a/c`; `None` if it climbs out of the project.
pub fn normalize(path: &str) -> Option<String> {
    let mut out: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                out.pop()?;
            }
            p => out.push(p),
        }
    }
    Some(out.join("/"))
}

pub fn join(dir: &str, rel: &str) -> Option<String> {
    if dir.is_empty() { normalize(rel) } else { normalize(&format!("{dir}/{rel}")) }
}

pub fn dirname(path: &str) -> &str {
    path.rfind('/').map(|i| &path[..i]).unwrap_or("")
}

/// Strip `//` and `/* */` comments and trailing commas, for tsconfig.
pub fn jsonc(text: &str) -> Option<Value> {
    let mut out = String::with_capacity(text.len());
    let b = text.as_bytes();
    let mut i = 0;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            out.push(c as char);
            if c == b'\\' && i + 1 < b.len() {
                out.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_str = true;
        } else if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        } else if c == b'/' && b.get(i + 1) == Some(&b'*') {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
            continue;
        } else if c == b',' {
            let rest = text[i + 1..].trim_start();
            if rest.starts_with('}') || rest.starts_with(']') {
                i += 1;
                continue;
            }
        }
        out.push(c as char);
        i += 1;
    }
    // Bytes were pushed one by one; rebuild as UTF-8.
    let bytes: Vec<u8> = out.chars().map(|c| c as u32 as u8).collect();
    serde_json::from_slice(&bytes).ok()
}

/// A `package.json`.
#[derive(Clone, Debug, Default)]
pub struct PackageJson {
    pub dir: String,
    pub name: Option<String>,
    pub private: bool,
    pub exports: Option<Value>,
    /// `main`, `module`, `types`, `typings`, `source`, `browser`, `style`.
    pub fields: Vec<String>,
    pub bin: Vec<String>,
}

impl PackageJson {
    pub fn parse(dir: &str, text: &str) -> Option<PackageJson> {
        let v: Value = serde_json::from_str(text).ok()?;
        let mut p = PackageJson {
            dir: dir.to_owned(),
            name: v.get("name").and_then(Value::as_str).map(str::to_owned),
            private: v.get("private").and_then(Value::as_bool).unwrap_or(false),
            exports: v.get("exports").cloned(),
            ..Default::default()
        };
        for k in ["main", "module", "types", "typings", "source", "browser", "style", "svelte"] {
            if let Some(s) = v.get(k).and_then(Value::as_str) {
                p.fields.push(s.to_owned());
            }
        }
        match v.get("bin") {
            Some(Value::String(s)) => p.bin.push(s.clone()),
            Some(Value::Object(m)) => p.bin.extend(m.values().filter_map(Value::as_str).map(str::to_owned)),
            _ => {}
        }
        Some(p)
    }
}

/// A `tsconfig.json` / `jsconfig.json`, its `extends` chain applied.
#[derive(Clone, Debug, Default)]
pub struct TsConfig {
    pub dir: String,
    pub base_url: Option<String>,
    /// Pattern → targets, each relative to the project root.
    pub paths: Vec<(String, Vec<String>)>,
}

/// A `pyproject.toml`.
#[derive(Clone, Debug, Default)]
pub struct PyProject {
    pub dir: String,
    /// `[project]` is present: a distribution.
    pub distribution: bool,
    /// `pkg.mod:func` from `[project.scripts]`, `gui-scripts`, `entry-points`.
    pub entry_refs: Vec<String>,
}

impl PyProject {
    pub fn parse(dir: &str, text: &str) -> Option<PyProject> {
        let v: toml::Table = toml::from_str(text).ok()?;
        let mut p = PyProject { dir: dir.to_owned(), ..Default::default() };
        let mut refs = Vec::new();
        if let Some(project) = v.get("project").and_then(|x| x.as_table()) {
            p.distribution = true;
            for k in ["scripts", "gui-scripts"] {
                if let Some(t) = project.get(k).and_then(|x| x.as_table()) {
                    refs.extend(t.values().filter_map(|x| x.as_str()).map(str::to_owned));
                }
            }
            if let Some(groups) = project.get("entry-points").and_then(|x| x.as_table()) {
                for g in groups.values().filter_map(|x| x.as_table()) {
                    refs.extend(g.values().filter_map(|x| x.as_str()).map(str::to_owned));
                }
            }
        }
        // Poetry's scripts table.
        if let Some(t) = v.get("tool").and_then(|t| t.get("poetry")).and_then(|t| t.get("scripts")).and_then(|t| t.as_table()) {
            refs.extend(t.values().filter_map(|x| x.as_str()).map(str::to_owned));
        }
        p.entry_refs = refs;
        Some(p)
    }
}

/// What an import names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    /// A file of the project.
    File(String),
    /// A Python namespace package (a folder without `__init__.py`).
    Namespace(String),
    /// A workspace package whose entry could not be mapped to a source file:
    /// every module in it counts as used (the conservative choice).
    Package(String),
}

const JS_EXT: &[&str] = &[".ts", ".tsx", ".d.ts", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts"];
const BUILD_DIRS: &[&str] = &["dist", "lib", "build", "out", "esm", "cjs", "es", "types"];

pub struct Resolver {
    pub files: BTreeSet<String>,
    pub packages: Vec<PackageJson>,
    pub tsconfigs: Vec<TsConfig>,
    pub pyprojects: Vec<PyProject>,
    /// Python source roots, longest first.
    pub py_roots: Vec<String>,
    dirs: BTreeSet<String>,
    by_name: BTreeMap<String, usize>,
}

impl Resolver {
    pub fn new(files: BTreeSet<String>, packages: Vec<PackageJson>, raw_tsconfigs: Vec<(String, Value)>, pyprojects: Vec<PyProject>) -> Resolver {
        let mut dirs = BTreeSet::new();
        for f in &files {
            let mut d = dirname(f);
            while !d.is_empty() && dirs.insert(d.to_owned()) {
                d = dirname(d);
            }
        }
        let by_name = packages.iter().enumerate().filter_map(|(i, p)| p.name.clone().map(|n| (n, i))).collect();
        let tsconfigs = resolve_tsconfigs(&raw_tsconfigs);
        let mut roots: BTreeSet<String> = BTreeSet::new();
        roots.insert(String::new());
        if dirs.contains("src") {
            roots.insert("src".into());
        }
        for p in &pyprojects {
            roots.insert(p.dir.clone());
            let src = join(&p.dir, "src").unwrap_or_default();
            if dirs.contains(&src) {
                roots.insert(src);
            }
        }
        for f in &files {
            let name = f.rsplit('/').next().unwrap_or_default();
            if name == "setup.py" || name == "setup.cfg" {
                roots.insert(dirname(f).to_owned());
            }
        }
        let mut py_roots: Vec<String> = roots.into_iter().collect();
        py_roots.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
        Resolver { files, packages, tsconfigs, pyprojects, py_roots, dirs, by_name }
    }

    pub fn exists(&self, f: &str) -> bool {
        self.files.contains(f)
    }

    /// The package whose folder holds `file` (the nearest `package.json`).
    pub fn package_of(&self, file: &str) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (i, p) in self.packages.iter().enumerate() {
            let inside = p.dir.is_empty() || file.starts_with(&format!("{}/", p.dir));
            if inside && best.is_none_or(|b| self.packages[b].dir.len() < p.dir.len()) {
                best = Some(i);
            }
        }
        best
    }

    /// Extension and index probing.
    pub fn probe(&self, base: &str) -> Option<String> {
        if self.exists(base) {
            return Some(base.to_owned());
        }
        for (js, ts) in [(".js", &[".ts", ".tsx"][..]), (".jsx", &[".tsx"][..]), (".mjs", &[".mts"][..]), (".cjs", &[".cts"][..])] {
            if let Some(stem) = base.strip_suffix(js) {
                for t in ts {
                    let c = format!("{stem}{t}");
                    if self.exists(&c) {
                        return Some(c);
                    }
                }
            }
        }
        for e in JS_EXT {
            let c = format!("{base}{e}");
            if self.exists(&c) {
                return Some(c);
            }
        }
        if self.dirs.contains(base) || base.is_empty() {
            for e in JS_EXT {
                let c = if base.is_empty() { format!("index{e}") } else { format!("{base}/index{e}") };
                if self.exists(&c) {
                    return Some(c);
                }
            }
        }
        None
    }

    /// A build-output path mapped back to its source (`dist/x.js` →
    /// `src/x.ts`): workspace packages resolve to the local source.
    fn probe_source(&self, pkg_dir: &str, rel: &str) -> Option<String> {
        let rel = rel.trim_start_matches("./");
        if let Some(f) = join(pkg_dir, rel).and_then(|p| self.probe(&p)) {
            return Some(f);
        }
        let stripped = rel.strip_suffix(".d.ts").or_else(|| rel.rsplit_once('.').map(|(a, _)| a)).unwrap_or(rel);
        let parts: Vec<&str> = stripped.split('/').collect();
        if let Some(first) = parts.first()
            && BUILD_DIRS.contains(first)
        {
            let rest = parts[1..].join("/");
            for src in ["src", "lib", ""] {
                let cand = if src.is_empty() { rest.clone() } else if rest.is_empty() { src.to_owned() } else { format!("{src}/{rest}") };
                if let Some(f) = join(pkg_dir, &cand).and_then(|p| self.probe(&p)) {
                    return Some(f);
                }
            }
        }
        None
    }

    /// Every entry file of a package: `exports`, `main` and friends, `bin`;
    /// `index.*` when it names none.
    pub fn package_entries(&self, pkg: usize) -> (Vec<String>, bool) {
        let p = &self.packages[pkg];
        let mut targets: Vec<String> = Vec::new();
        if let Some(e) = &p.exports {
            collect_export_targets(e, &mut targets);
        }
        targets.extend(p.fields.iter().cloned());
        targets.extend(p.bin.iter().cloned());
        let mut out = Vec::new();
        let mut unmapped = false;
        for t in targets {
            if t.contains('*') {
                // A subpath pattern: every file under its fixed prefix.
                let prefix = t.split('*').next().unwrap_or_default().trim_start_matches("./");
                let Some(base) = join(&p.dir, prefix) else { continue };
                let hits: Vec<String> = self.files.iter().filter(|f| f.starts_with(&base)).cloned().collect();
                if hits.is_empty() {
                    unmapped = true;
                }
                out.extend(hits);
                continue;
            }
            match self.probe_source(&p.dir, &t) {
                Some(f) => out.push(f),
                None => unmapped |= !t.ends_with(".json"),
            }
        }
        if out.is_empty()
            && let Some(f) = self.probe(&p.dir)
        {
            out.push(f);
        }
        out.sort();
        out.dedup();
        (out, unmapped)
    }

    fn tsconfig_for(&self, file: &str) -> Option<&TsConfig> {
        let mut best: Option<&TsConfig> = None;
        for t in &self.tsconfigs {
            let inside = t.dir.is_empty() || file.starts_with(&format!("{}/", t.dir));
            if inside && best.is_none_or(|b| b.dir.len() < t.dir.len()) {
                best = Some(t);
            }
        }
        best
    }

    /// Resolve a JS/TS specifier imported by `from`. Empty: external.
    pub fn resolve_js(&self, from: &str, spec: &str) -> Vec<Target> {
        let spec = spec.split(['?', '#']).next().unwrap_or(spec);
        if spec.is_empty() || spec.starts_with("node:") || spec.contains("://") {
            return Vec::new();
        }
        if spec.starts_with("./") || spec.starts_with("../") || spec == "." || spec == ".." {
            return join(dirname(from), spec).and_then(|p| self.probe(&p)).map(Target::File).into_iter().collect();
        }
        if let Some(abs) = spec.strip_prefix('/') {
            return self.probe(abs).map(Target::File).into_iter().collect();
        }
        if let Some(ts) = self.tsconfig_for(from) {
            for (pattern, targets) in &ts.paths {
                if let Some(star) = match_pattern(pattern, spec) {
                    for t in targets {
                        let t = t.replace('*', &star);
                        if let Some(f) = self.probe(&t) {
                            return vec![Target::File(f)];
                        }
                    }
                }
            }
            if let Some(base) = &ts.base_url
                && let Some(f) = join(base, spec).and_then(|p| self.probe(&p))
            {
                return vec![Target::File(f)];
            }
        }
        // A workspace package, by name.
        let (name, sub) = split_package(spec);
        let Some(&pkg) = self.by_name.get(name) else { return Vec::new() };
        let p = &self.packages[pkg];
        let subpath = if sub.is_empty() { ".".to_owned() } else { format!("./{sub}") };
        let mut targets = Vec::new();
        if let Some(e) = &p.exports {
            exports_for(e, &subpath, &mut targets);
        } else if sub.is_empty() {
            targets.extend(p.fields.iter().cloned());
            if targets.is_empty() {
                targets.push("index".into());
            }
        } else {
            targets.push(sub.to_owned());
        }
        let mut out: Vec<Target> = targets.iter().filter_map(|t| self.probe_source(&p.dir, t)).map(Target::File).collect();
        out.sort();
        out.dedup();
        if out.is_empty() {
            out.push(Target::Package(p.dir.clone()));
        }
        out
    }

    /// A file's dotted Python module name under its source root.
    pub fn py_module_name(&self, file: &str) -> Option<String> {
        let stem = file.strip_suffix(".pyi").or_else(|| file.strip_suffix(".py"))?;
        for root in &self.py_roots {
            let rel = if root.is_empty() {
                stem
            } else if let Some(r) = stem.strip_prefix(&format!("{root}/")) {
                r
            } else {
                continue;
            };
            let rel = rel.strip_suffix("/__init__").unwrap_or(rel);
            if rel == "__init__" {
                continue;
            }
            return Some(rel.replace('/', "."));
        }
        None
    }

    fn py_lookup(&self, base: &str) -> Option<Target> {
        for cand in [format!("{base}.py"), format!("{base}/__init__.py"), format!("{base}.pyi"), format!("{base}/__init__.pyi")] {
            if self.exists(&cand) {
                return Some(Target::File(cand));
            }
        }
        if self.dirs.contains(base) {
            return Some(Target::Namespace(base.to_owned()));
        }
        None
    }

    /// Resolve a Python module imported by `from` (`a.b`, `.x`, `..`).
    pub fn resolve_py(&self, from: &str, module: &str) -> Option<Target> {
        let dots = module.chars().take_while(|c| *c == '.').count();
        let rest = &module[dots..];
        let rel_path = rest.replace('.', "/");
        if dots > 0 {
            let mut dir = dirname(from).to_owned();
            for _ in 1..dots {
                dir = dirname(&dir).to_owned();
            }
            if rel_path.is_empty() {
                return self.py_lookup(&dir).or(Some(Target::Namespace(dir)));
            }
            return join(&dir, &rel_path).and_then(|p| self.py_lookup(&p));
        }
        for root in &self.py_roots {
            if let Some(t) = join(root, &rel_path).and_then(|p| self.py_lookup(&p)) {
                return Some(t);
            }
        }
        // A script's own folder is on `sys.path`.
        join(dirname(from), &rel_path).and_then(|p| self.py_lookup(&p))
    }

    /// The submodule `name` of a Python package target.
    pub fn py_submodule(&self, pkg: &Target, name: &str) -> Option<Target> {
        let dir = match pkg {
            Target::File(f) if f.ends_with("/__init__.py") || f.ends_with("/__init__.pyi") || f == "__init__.py" => dirname(f).to_owned(),
            Target::Namespace(d) => d.clone(),
            _ => return None,
        };
        join(&dir, name).and_then(|p| self.py_lookup(&p))
    }
}

fn split_package(spec: &str) -> (&str, &str) {
    let mut idx = 0;
    let mut slashes = if spec.starts_with('@') { 2 } else { 1 };
    for (i, c) in spec.char_indices() {
        if c == '/' {
            slashes -= 1;
            if slashes == 0 {
                idx = i;
                break;
            }
        }
    }
    if idx == 0 { (spec, "") } else { (&spec[..idx], &spec[idx + 1..]) }
}

/// The `*` part of `spec` under a pattern like `@/*`.
fn match_pattern(pattern: &str, spec: &str) -> Option<String> {
    match pattern.split_once('*') {
        None => (pattern == spec).then(String::new),
        Some((pre, post)) => {
            if spec.len() >= pre.len() + post.len() && spec.starts_with(pre) && spec.ends_with(post) {
                Some(spec[pre.len()..spec.len() - post.len()].to_owned())
            } else {
                None
            }
        }
    }
}

/// Every string target in an `exports` value, all conditions.
fn collect_export_targets(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| collect_export_targets(x, out)),
        Value::Object(m) => m.values().for_each(|x| collect_export_targets(x, out)),
        _ => {}
    }
}

/// The targets of one subpath, every condition counted (conservative).
fn exports_for(v: &Value, subpath: &str, out: &mut Vec<String>) {
    match v {
        Value::String(_) | Value::Array(_) => {
            if subpath == "." {
                collect_export_targets(v, out);
            }
        }
        Value::Object(m) => {
            let subpaths = m.keys().any(|k| k.starts_with('.'));
            if !subpaths {
                if subpath == "." {
                    collect_export_targets(v, out);
                }
                return;
            }
            if let Some(x) = m.get(subpath) {
                collect_export_targets(x, out);
                return;
            }
            for (k, x) in m {
                if let Some(star) = match_pattern(k, subpath) {
                    let mut t = Vec::new();
                    collect_export_targets(x, &mut t);
                    out.extend(t.into_iter().map(|s| s.replace('*', &star)));
                }
            }
        }
        _ => {}
    }
}

fn resolve_tsconfigs(raw: &[(String, Value)]) -> Vec<TsConfig> {
    let by_path: BTreeMap<&str, &Value> = raw.iter().map(|(p, v)| (p.as_str(), v)).collect();
    let mut out = Vec::new();
    for (path, _) in raw {
        let dir = dirname(path).to_owned();
        let mut chain: Vec<(String, &Value)> = Vec::new();
        let mut cur = Some(path.clone());
        while let Some(p) = cur.take() {
            let Some(v) = by_path.get(p.as_str()) else { break };
            if chain.iter().any(|(q, _)| *q == p) || chain.len() > 8 {
                break;
            }
            chain.push((p.clone(), v));
            if let Some(ext) = v.get("extends").and_then(Value::as_str)
                && (ext.starts_with("./") || ext.starts_with("../"))
            {
                let target = join(dirname(&p), ext).unwrap_or_default();
                cur = Some(if target.ends_with(".json") { target } else { format!("{target}.json") });
            }
        }
        let mut ts = TsConfig { dir, ..Default::default() };
        // The base of the chain first, so the file's own settings win.
        for (p, v) in chain.iter().rev() {
            let Some(co) = v.get("compilerOptions") else { continue };
            let d = dirname(p);
            if let Some(b) = co.get("baseUrl").and_then(Value::as_str) {
                ts.base_url = join(d, b);
            }
            if let Some(Value::Object(paths)) = co.get("paths") {
                let base = ts.base_url.clone().unwrap_or_else(|| d.to_owned());
                ts.paths = paths
                    .iter()
                    .map(|(k, v)| {
                        let targets = v.as_array().map(|a| a.iter().filter_map(Value::as_str).filter_map(|t| join(&base, t)).collect()).unwrap_or_default();
                        (k.clone(), targets)
                    })
                    .collect();
            }
        }
        out.push(ts);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(files: &[&str]) -> Resolver {
        Resolver::new(files.iter().map(|s| s.to_string()).collect(), Vec::new(), Vec::new(), Vec::new())
    }

    #[test]
    fn relative_and_extension_probing() {
        let res = r(&["src/a.ts", "src/b/index.tsx", "src/c.js"]);
        assert_eq!(res.resolve_js("src/x.ts", "./a"), vec![Target::File("src/a.ts".into())]);
        assert_eq!(res.resolve_js("src/x.ts", "./a.js"), vec![Target::File("src/a.ts".into())]);
        assert_eq!(res.resolve_js("src/x.ts", "./b"), vec![Target::File("src/b/index.tsx".into())]);
        assert_eq!(res.resolve_js("src/x.ts", "../src/c"), vec![Target::File("src/c.js".into())]);
        assert!(res.resolve_js("src/x.ts", "react").is_empty());
    }

    #[test]
    fn workspace_packages_resolve_to_source() {
        let pkg = PackageJson::parse("packages/utils", r#"{"name":"@acme/utils","exports":{".":{"import":"./dist/index.js","types":"./dist/index.d.ts"}}}"#).unwrap();
        let res = Resolver::new(["packages/utils/src/index.ts".to_string()].into_iter().collect(), vec![pkg], Vec::new(), Vec::new());
        assert_eq!(res.resolve_js("packages/web/a.ts", "@acme/utils"), vec![Target::File("packages/utils/src/index.ts".into())]);
    }

    #[test]
    fn tsconfig_paths() {
        let ts = jsonc(r#"{ // c
          "compilerOptions": { "baseUrl": ".", "paths": { "@/*": ["src/*"], }, }, }"#)
        .unwrap();
        let res = Resolver::new(["src/lib/x.ts".to_string()].into_iter().collect(), Vec::new(), vec![("tsconfig.json".into(), ts)], Vec::new());
        assert_eq!(res.resolve_js("src/a.ts", "@/lib/x"), vec![Target::File("src/lib/x.ts".into())]);
    }

    #[test]
    fn python_modules() {
        let res = Resolver::new(
            ["src/acme/__init__.py", "src/acme/billing/legacy.py", "tests/test_x.py"].iter().map(|s| s.to_string()).collect(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(res.resolve_py("tests/test_x.py", "acme.billing.legacy"), Some(Target::File("src/acme/billing/legacy.py".into())));
        assert_eq!(res.resolve_py("src/acme/billing/legacy.py", ".."), Some(Target::File("src/acme/__init__.py".into())));
        assert_eq!(res.resolve_py("src/acme/__init__.py", ".billing"), Some(Target::Namespace("src/acme/billing".into())));
        assert_eq!(res.py_module_name("src/acme/billing/legacy.py").as_deref(), Some("acme.billing.legacy"));
        assert_eq!(res.py_module_name("src/acme/__init__.py").as_deref(), Some("acme"));
    }
}
