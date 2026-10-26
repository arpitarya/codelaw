//! Discovery (SR-ARCHITECTURE decision 7): `.gitignore`, `.ignore`, then
//! `include` and `exclude`; files are skipped by extension and ignore rules
//! before they are read.

use crate::{EngineError, Project};
use codelaw_config::{DEFAULT_EXCLUDES, Home, load_file};
use codelaw_core::{Language, Skipped, normalize_path};
use ignore::WalkBuilder;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One file a run checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub abs: PathBuf,
    /// Relative to the project root, forward slashes.
    pub rel: String,
    pub language: Language,
}

fn excluded_by_default(path: &Path, root: &Path) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let rel_s = normalize_path(rel);
    DEFAULT_EXCLUDES.iter().any(|ex| {
        if ex.contains('/') {
            rel_s == *ex || rel_s.starts_with(&format!("{ex}/"))
        } else {
            rel.components().any(|c| c.as_os_str() == *ex)
        }
    })
}

/// A path relative to the project root, forward slashes. A file outside
/// the root is reached with `../`, never printed absolute; only a path on
/// another drive (Windows) has no relative form.
pub(crate) fn rel_path(abs: &Path, root: &Path) -> String {
    if let Ok(rel) = abs.strip_prefix(root) {
        return normalize_path(rel);
    }
    let a: Vec<_> = abs.components().collect();
    let r: Vec<_> = root.components().collect();
    let common = a.iter().zip(&r).take_while(|(x, y)| x == y).count();
    if common == 0 {
        return normalize_path(abs);
    }
    let mut out = PathBuf::new();
    for _ in common..r.len() {
        out.push("..");
    }
    for c in &a[common..] {
        out.push(c);
    }
    normalize_path(&out)
}

/// The ignore files of the folders between the git repository's root and
/// the project root, which the walk (it starts at the project root) does
/// not see. Nothing above the repository is read, and with no repository
/// nothing above the project root (L5: no machine-dependent input).
fn ancestor_ignores(root: &Path) -> Vec<Gitignore> {
    let Some(repo) = root.ancestors().skip(1).find(|d| d.join(".git").exists()) else { return Vec::new() };
    let mut dirs: Vec<&Path> = root.ancestors().skip(1).take_while(|d| d.starts_with(repo)).collect();
    dirs.reverse();
    let mut out = Vec::new();
    for d in dirs {
        let mut b = GitignoreBuilder::new(d);
        if d == repo {
            b.add(d.join(".git/info/exclude"));
        }
        b.add(d.join(".gitignore"));
        b.add(d.join(".ignore"));
        if let Ok(g) = b.build()
            && !g.is_empty()
        {
            out.push(g);
        }
    }
    out
}

fn ignored_by(ignores: &[Gitignore], path: &Path, is_dir: bool) -> bool {
    // The deepest folder's ignore file decides, as git does.
    for g in ignores.iter().rev() {
        match g.matched_path_or_any_parents(path, is_dir) {
            ignore::Match::Ignore(_) => return true,
            ignore::Match::Whitelist(_) => return false,
            ignore::Match::None => {}
        }
    }
    false
}

/// The path a walk error is about.
fn error_path(e: &ignore::Error) -> Option<PathBuf> {
    match e {
        ignore::Error::WithPath { path, .. } => Some(path.clone()),
        ignore::Error::Loop { child, .. } => Some(child.clone()),
        ignore::Error::WithDepth { err, .. } | ignore::Error::WithLineNumber { err, .. } => error_path(err),
        ignore::Error::Partial(errs) => errs.iter().find_map(error_path),
        _ => None,
    }
}

/// The files to check, the files skipped before reading, and warnings.
pub(crate) type Discovered = (Vec<FileEntry>, Vec<Skipped>, Vec<String>);

/// Every folder from the project root down to `dir`, `dir` included.
fn folders_down_to(dir: &Path, root: &Path) -> Vec<PathBuf> {
    if !dir.starts_with(root) {
        return Vec::new();
    }
    dir.ancestors().take_while(|d| d.starts_with(root)).map(Path::to_path_buf).collect()
}

pub(crate) fn discover(project: &mut Project, paths: &[PathBuf]) -> Result<Discovered, EngineError> {
    discover_with(project, paths, true)
}

/// `filtered: false` keeps the files `[files]` and `languages` leave out:
/// the project graph reads them for their uses and resolves imports to
/// them, and reports nothing in them (SR-PROJECT-GRAPH decision 1).
pub(crate) fn discover_with(project: &mut Project, paths: &[PathBuf], filtered: bool) -> Result<Discovered, EngineError> {
    let root = project.config.root_dir.clone();
    let roots: Vec<PathBuf> = if paths.is_empty() {
        vec![root.clone()]
    } else {
        paths.iter().map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone())).collect()
    };
    let mut warnings = Vec::new();
    let mut skipped = Vec::new();
    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut explicit: BTreeSet<PathBuf> = BTreeSet::new();
    let mut config_dirs: BTreeSet<PathBuf> = BTreeSet::new();
    let mut inside: Vec<PathBuf> = Vec::new();
    let mut outside: Vec<PathBuf> = Vec::new();
    for r in &roots {
        if r.is_file() {
            explicit.insert(r.clone());
            candidates.push(r.clone());
            // A file named on the command line gets the nested configs
            // above it, as it would in a walk (SR-CONFIG decision 2).
            if let Some(dir) = r.parent() {
                config_dirs.extend(folders_down_to(dir, &root));
            }
        } else if r.is_dir() {
            config_dirs.extend(folders_down_to(r, &root));
            if r.starts_with(&root) { inside.push(r.clone()) } else { outside.push(r.clone()) }
        } else {
            return Err(EngineError::Usage(format!("no such file or folder: {}", normalize_path(r))));
        }
    }
    // Folders inside the project are walked from the project root, so the
    // ignore files between it and them apply and none above it is read
    // (decision 7); the walk enters only what leads to a requested folder.
    let mut walk_roots: Vec<PathBuf> = Vec::new();
    if !inside.is_empty() {
        walk_roots.push(root.clone());
    }
    walk_roots.extend(outside);
    if let Some((first, rest)) = walk_roots.split_first() {
        let mut b = WalkBuilder::new(first);
        for r in rest {
            b.add(r);
        }
        let root_for_filter = root.clone();
        let wanted = if inside.contains(&root) { Vec::new() } else { inside.clone() };
        let above = ancestor_ignores(&root);
        b.hidden(false)
            .git_ignore(true)
            .git_exclude(true)
            // A machine's global gitignore would make two machines disagree (L5).
            .git_global(false)
            .ignore(true)
            .parents(false)
            .require_git(false)
            .follow_links(project.config.follow_symlinks())
            .sort_by_file_path(|a, b| a.cmp(b))
            .filter_entry(move |e| {
                let p = e.path();
                if excluded_by_default(p, &root_for_filter) {
                    return false;
                }
                if !p.starts_with(&root_for_filter) {
                    return true;
                }
                (wanted.is_empty() || wanted.iter().any(|w| p.starts_with(w) || w.starts_with(p)))
                    && !ignored_by(&above, p, e.file_type().is_some_and(|t| t.is_dir()))
            });
        for result in b.build() {
            let entry = match result {
                Ok(e) => e,
                // An unreadable folder or a symlink loop is never silently
                // unchecked (decision 26).
                Err(e) => {
                    let file = error_path(&e).map(|p| rel_path(&p, &root)).unwrap_or_else(|| ".".into());
                    skipped.push(Skipped { file, reason: format!("not walked: {e}") });
                    continue;
                }
            };
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let path = entry.into_path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if [Home::CodelawToml, Home::Pyproject, Home::PackageJson].iter().any(|h| h.file_name() == name)
                && let Some(dir) = path.parent()
            {
                config_dirs.insert(dir.to_path_buf());
            }
            if Language::from_path(&path).is_some() {
                candidates.push(path);
            }
        }
    }

    // Nested configs below the root (SR-CONFIG decision 2).
    let mut separate: Vec<PathBuf> = Vec::new();
    let known: BTreeSet<PathBuf> = project.config.files.iter().map(|f| f.dir.clone()).collect();
    let mut dirs: Vec<PathBuf> = config_dirs.into_iter().filter(|d| !known.contains(d) && d.starts_with(&root)).collect();
    dirs.sort_by_key(|d| d.components().count());
    for dir in dirs {
        if separate.iter().any(|s| dir.starts_with(s)) {
            continue;
        }
        let mut w = Vec::new();
        let Some(file) = load_file(&dir, &mut w)? else { continue };
        warnings.extend(w);
        let path = file.path.clone();
        let names: Vec<String> = file.raw.rules.keys().chain(file.scope_rule_tables().into_iter().flat_map(|t| t.keys())).cloned().collect();
        if !project.config.add_nested(file)? {
            separate.push(dir);
            continue;
        }
        for id in names {
            if project.rules.get(&id).is_none() && codelaw_rules::engine_rule(&id).is_none() {
                return Err(EngineError::Config(codelaw_config::ConfigError::new(Some(path.clone()), format!("unknown rule `{id}`"))));
            }
        }
    }
    warnings.append(&mut project.config.warnings);

    let mut files = Vec::new();
    candidates.sort();
    candidates.dedup();
    // With `follow_symlinks`, one file can be reached by several paths; it
    // is checked and written once, under the first path in sort order.
    if project.config.follow_symlinks() {
        let mut seen = BTreeSet::new();
        candidates.retain(|p| seen.insert(std::fs::canonicalize(p).unwrap_or_else(|_| p.clone())));
    }
    for abs in candidates {
        if separate.iter().any(|s| abs.starts_with(s)) {
            continue;
        }
        let Some(language) = Language::from_path(&abs) else {
            if explicit.contains(&abs) {
                warnings.push(format!("{}: not a file codelaw checks (unknown extension)", rel_path(&abs, &root)));
            }
            continue;
        };
        if filtered && !reported(project, &abs, language) {
            continue;
        }
        files.push(FileEntry { rel: rel_path(&abs, &root), abs, language });
    }
    Ok((files, skipped, warnings))
}

/// Whether `[files]` and `languages` let a file be checked.
pub(crate) fn reported(project: &Project, abs: &Path, language: Language) -> bool {
    project.config.is_included(abs) && language_enabled(project, abs, language)
}

/// `languages` in config and `--lang` on the command line (L3 decision 4).
pub(crate) fn language_enabled(project: &Project, abs: &Path, language: Language) -> bool {
    project.config.languages_for(abs).is_none_or(|set| set.contains(&language))
        && project.options.languages.as_ref().is_none_or(|set| set.contains(&language))
}
