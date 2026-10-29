//! Feature-flag expiry (SR-PROJECT-GRAPH decision 11, W-130): which calls
//! read a flag — the accessors `[flags]` names, the presets for six SDKs
//! and every wrapper whose parameter flows into one, found through the
//! graph — the flag each read names, and what the registry
//! (`.codelaw/flags.toml`) and the inline `codelaw-flag` markers say about
//! it. A pure function of the files, the config and the as-of date (L5):
//! no flag service is ever asked.

mod extract;
mod patterns;
mod verdict;

pub use extract::{Arg, Call, Collapse, FlagFacts, Func, MARKER, Marker, extract_flags};
pub use patterns::{ArgPat, Pattern, PRESETS, preset};
pub use verdict::{FlagCheck, FlagFinding, FlagOptions, FlagReport, REGISTRY_REL, check};

use crate::facts::{FactLang, ImportKind, Span};
use crate::link::Graph;
use crate::resolve::Target;
use std::collections::{BTreeMap, BTreeSet};

/// Bumped whenever extraction changes: part of the flag facts' cache key.
pub const FLAGS_VERSION: &str = "flag-facts-1";

/// One accessor in force: a pattern from config or a preset, or a wrapper
/// found through the graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accessor {
    /// `client.variation($NAME, $$$)`, `isOn($NAME)`, `$_.isOn($NAME)`.
    pub pattern: String,
    /// `preset launchdarkly`, `config`, or `wrapper of <accessor>`.
    pub origin: String,
    /// A wrapper's declaration: file and line.
    pub at: Option<(String, u32)>,
}

/// One read of a flag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagRead {
    pub file: String,
    /// The call.
    pub span: Span,
    /// The flag's name, or why it cannot be known here.
    pub name: Result<String, String>,
    pub collapse: Option<Collapse>,
}

/// A wrapper: a named function (file, index in its `funcs`) and the
/// parameter that carries the flag's name.
type WrapperId = (String, u32);

#[derive(Clone, Debug)]
struct Wrapper {
    param: u32,
    via: String,
}

/// What one name of a file is bound to: a top-level name of another file
/// (or of itself), or a whole module.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Bound {
    Name(String, String),
    Module(String),
}

/// The files a file's imports reach, by local name.
fn bindings(graph: &Graph, file: &str) -> BTreeMap<String, Vec<Bound>> {
    let mut out: BTreeMap<String, Vec<Bound>> = BTreeMap::new();
    let Some(m) = graph.modules.get(file) else { return out };
    let python = m.facts.lang == Some(FactLang::Python);
    for imp in &m.facts.imports {
        let Some(local) = imp.local.clone() else { continue };
        let targets: Vec<String> = if python {
            graph.resolver.resolve_py(file, &imp.source).into_iter().collect::<Vec<Target>>()
        } else {
            graph.resolver.resolve_js(file, &imp.source)
        }
        .into_iter()
        .filter_map(|t| match t {
            Target::File(f) => Some(f),
            _ => None,
        })
        .collect();
        for t in targets {
            let bound = match &imp.kind {
                ImportKind::Named(e) => {
                    // Python: `from pkg import mod` names a submodule.
                    let sub = python.then(|| graph.resolver.py_submodule(&Target::File(t.clone()), e)).flatten();
                    match sub {
                        Some(Target::File(s)) if !graph.modules.get(&t).is_some_and(|m| m.facts.decls.iter().any(|d| &d.name == e)) => Bound::Module(s),
                        _ => export_local(graph, &t, e, 0).map(|(f, n)| Bound::Name(f, n)).unwrap_or(Bound::Name(t.clone(), e.clone())),
                    }
                }
                ImportKind::Default => export_local(graph, &t, "default", 0).map(|(f, n)| Bound::Name(f, n)).unwrap_or(Bound::Module(t.clone())),
                ImportKind::Namespace | ImportKind::Module => Bound::Module(t.clone()),
                _ => continue,
            };
            out.entry(local.clone()).or_default().push(bound);
        }
    }
    out
}

/// The top-level name an export of `file` stands for, through re-exports.
fn export_local(graph: &Graph, file: &str, name: &str, depth: usize) -> Option<(String, String)> {
    let m = graph.modules.get(file)?;
    if m.facts.lang == Some(FactLang::Python) || depth > 6 {
        return Some((file.to_owned(), name.to_owned()));
    }
    for e in &m.facts.exports {
        if e.name == name {
            if let Some(src) = &e.source {
                let imported = e.imported.clone().unwrap_or_else(|| name.to_owned());
                for t in graph.resolver.resolve_js(file, src) {
                    if let Target::File(f) = t {
                        return export_local(graph, &f, &imported, depth + 1);
                    }
                }
                return None;
            }
            return Some((file.to_owned(), e.local.clone().unwrap_or_else(|| name.to_owned())));
        }
    }
    for e in m.facts.exports.iter().filter(|e| e.name == "*") {
        let Some(src) = &e.source else { continue };
        for t in graph.resolver.resolve_js(file, src) {
            if let Target::File(f) = t
                && let Some(found) = export_local(graph, &f, name, depth + 1)
                && graph.modules.get(&found.0).is_some_and(|m| m.facts.decls.iter().any(|d| d.name == found.1))
            {
                return Some(found);
            }
        }
    }
    None
}

/// The flag analysis of a project: the accessors in force and every read.
pub struct Analysis {
    pub accessors: Vec<Accessor>,
    pub reads: Vec<FlagRead>,
}

/// Find every flag read: the patterns, then the wrappers over them to a
/// fixpoint (a wrapper of a wrapper is one too), then each read's name.
/// `files` holds every JS, TS and Python file's facts, by path.
pub fn analyze(graph: &Graph, files: &BTreeMap<String, FlagFacts>, patterns: &[Pattern], origins: &[String]) -> Analysis {
    let binds: BTreeMap<&str, BTreeMap<String, Vec<Bound>>> = files.keys().map(|f| (f.as_str(), bindings(graph, f))).collect();
    // The files each file imports, for the method wrappers it may call.
    let reach: BTreeMap<&str, BTreeSet<String>> = binds
        .iter()
        .map(|(f, b)| {
            let mut s: BTreeSet<String> = b.values().flatten().map(|x| match x { Bound::Name(f, _) | Bound::Module(f) => f.clone() }).collect();
            s.insert((*f).to_owned());
            (*f, s)
        })
        .collect();
    let top_index = |file: &str, name: &str| -> Option<u32> {
        files.get(file)?.funcs.iter().position(|f| f.top && f.name == name).map(|i| i as u32)
    };
    // What a call's callee names: a top-level function of the project.
    let callee_fn = |file: &str, callee: &[String]| -> Option<WrapperId> {
        match callee {
            [one] => top_index(file, one).map(|i| (file.to_owned(), i)).or_else(|| {
                binds.get(file)?.get(one)?.iter().find_map(|b| match b {
                    Bound::Name(f, n) => top_index(f, n).map(|i| (f.clone(), i)),
                    Bound::Module(_) => None,
                })
            }),
            [.., _] => {
                // `ns.fn` / `pkg.mod.fn`: a whole-module binding.
                let (last, head) = callee.split_last()?;
                let joined = head.join(".");
                binds.get(file)?.iter().find_map(|(local, bs)| {
                    (local == &joined || local == &head[0] && head.len() == 1)
                        .then(|| bs.iter().find_map(|b| match b {
                            Bound::Module(f) => top_index(f, last).map(|i| (f.clone(), i)),
                            Bound::Name(..) => None,
                        }))
                        .flatten()
                })
            }
            _ => None,
        }
    };
    let mut wrappers: BTreeMap<WrapperId, Wrapper> = BTreeMap::new();
    // The `$NAME` argument of a call, and what it matched.
    let matched = |file: &str, call: &Call, wrappers: &BTreeMap<WrapperId, Wrapper>| -> Option<(Arg, String)> {
        let facts = &files[file];
        for p in patterns {
            if (facts.python && !p.py) || (!facts.python && !p.js) {
                continue;
            }
            if let Some(a) = p.name_arg(call) {
                return Some((a.clone(), p.text.clone()));
            }
        }
        let id = callee_fn(file, &call.callee).filter(|id| wrappers.contains_key(id)).or_else(|| {
            // A method wrapper, called on anything, from a file that reaches
            // the file declaring it.
            let last = call.callee.last()?;
            if call.callee.len() < 2 {
                return None;
            }
            reach.get(file)?.iter().find_map(|f| {
                let ff = files.get(f)?;
                ff.funcs.iter().enumerate().find_map(|(i, fun)| (fun.method && &fun.name == last && wrappers.contains_key(&(f.clone(), i as u32))).then(|| (f.clone(), i as u32)))
            })
        })?;
        let w = &wrappers[&id];
        let fun = &files[&id.0].funcs[id.1 as usize];
        let pname = fun.params.get(w.param as usize)?;
        let positional: Vec<&Arg> = call.args.iter().filter(|a| !matches!(a, Arg::Keyword(..))).collect();
        let arg = positional.iter().take(w.param as usize + 1).position(|a| matches!(a, Arg::Spread(_))).map_or_else(
            || positional.get(w.param as usize).map(|a| (*a).clone()),
            |_| None,
        );
        let arg = arg.or_else(|| {
            call.args.iter().find_map(|a| match a {
                Arg::Keyword(k, v, _) if k == pname => Some((**v).clone()),
                _ => None,
            })
        })?;
        Some((arg, wrapper_pattern(fun, w.param)))
    };
    // The fixpoint: a function forwarding its parameter to a read is a
    // wrapper. Bounded by the number of functions.
    let total: usize = files.values().map(|f| f.funcs.len()).sum();
    for _ in 0..=total {
        let mut found: Vec<(WrapperId, Wrapper)> = Vec::new();
        for (file, facts) in files {
            for call in &facts.calls {
                let Some((Arg::Param { func, index, .. }, via)) = matched(file, call, &wrappers) else { continue };
                let id = (file.clone(), func);
                let fun = &facts.funcs[func as usize];
                if fun.name.is_empty() || !(fun.top || fun.method) || wrappers.contains_key(&id) || found.iter().any(|(i, _)| *i == id) {
                    continue;
                }
                found.push((id, Wrapper { param: index, via }));
            }
        }
        if found.is_empty() {
            break;
        }
        wrappers.extend(found);
    }
    let mut accessors: Vec<Accessor> = patterns.iter().zip(origins).map(|(p, o)| Accessor { pattern: p.text.clone(), origin: o.clone(), at: None }).collect();
    for ((file, i), w) in &wrappers {
        let fun = &files[file].funcs[*i as usize];
        accessors.push(Accessor { pattern: wrapper_pattern(fun, w.param), origin: format!("wrapper of `{}`", w.via), at: Some((file.clone(), fun.line)) });
    }
    let mut reads = Vec::new();
    for (file, facts) in files {
        for call in &facts.calls {
            let Some((arg, _)) = matched(file, call, &wrappers) else { continue };
            let name = match &arg {
                Arg::Str(s, _) => Ok(s.clone()),
                // Forwarded by a wrapper: its callers are the reads.
                Arg::Param { func, .. } if wrappers.contains_key(&(file.clone(), *func)) => continue,
                Arg::Param { .. } => Err("the name is a parameter of a function no caller resolves".to_owned()),
                Arg::Name(path, _) => resolve_const(files, &binds, file, path).ok_or_else(|| format!("`{}` is not a string constant codelaw can read", path.join("."))),
                _ => Err("the name is computed".to_owned()),
            };
            reads.push(FlagRead { file: file.clone(), span: call.span, name, collapse: call.collapse.clone() });
        }
    }
    Analysis { accessors, reads }
}

/// How a wrapper is called: `isOn($NAME)`, `$_.isOn($_, $NAME, $$$)`.
fn wrapper_pattern(fun: &Func, param: u32) -> String {
    let mut args: Vec<&str> = (0..=param).map(|i| if i == param { "$NAME" } else { "$_" }).collect();
    if fun.params.len() > param as usize + 1 {
        args.push("$$$");
    }
    let recv = if fun.method { "$_." } else { "" };
    format!("{recv}{}({})", fun.name, args.join(", "))
}

/// A constant's string: declared in the file, or imported from another.
fn resolve_const(files: &BTreeMap<String, FlagFacts>, binds: &BTreeMap<&str, BTreeMap<String, Vec<Bound>>>, file: &str, path: &[String]) -> Option<String> {
    let mut path = path.to_vec();
    let facts = files.get(file)?;
    if facts.python && path.len() > 1 && path.last().is_some_and(|l| l == "value") {
        path.pop();
    }
    let lookup = |f: &str, key: &str| files.get(f)?.consts.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
    if let Some(v) = lookup(file, &path.join(".")) {
        return Some(v);
    }
    let bound = binds.get(file)?.get(&path[0])?;
    bound.iter().find_map(|b| match b {
        Bound::Name(f, n) => {
            let mut key = vec![n.clone()];
            key.extend_from_slice(&path[1..]);
            lookup(f, &key.join("."))
        }
        Bound::Module(f) => (path.len() > 1).then(|| lookup(f, &path[1..].join("."))).flatten(),
    })
}

#[cfg(test)]
mod tests;
