//! Discovery (SR-ARCHITECTURE decision 7): `.gitignore`, `.ignore`, then
//! `include` and `exclude`; files are skipped by extension and ignore rules
//! before they are read.

use crate::{EngineError, Project};
use codelaw_config::{DEFAULT_EXCLUDES, Home, load_file};
use codelaw_core::{Language, Skipped, normalize_path};
use ignore::WalkBuilder;
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

pub(crate) fn rel_path(abs: &Path, root: &Path) -> String {
    normalize_path(abs.strip_prefix(root).unwrap_or(abs))
}

/// The files to check, the files skipped before reading, and warnings.
pub(crate) type Discovered = (Vec<FileEntry>, Vec<Skipped>, Vec<String>);

pub(crate) fn discover(project: &mut Project, paths: &[PathBuf]) -> Result<Discovered, EngineError> {
    let root = project.config.root_dir.clone();
    let roots: Vec<PathBuf> = if paths.is_empty() {
        vec![root.clone()]
    } else {
        paths.iter().map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone())).collect()
    };
    let mut warnings = Vec::new();
    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut explicit: BTreeSet<PathBuf> = BTreeSet::new();
    let mut config_dirs: BTreeSet<PathBuf> = BTreeSet::new();
    let mut walk_roots = Vec::new();
    for r in &roots {
        if r.is_file() {
            explicit.insert(r.clone());
            candidates.push(r.clone());
        } else if r.is_dir() {
            walk_roots.push(r.clone());
        } else {
            return Err(EngineError::Usage(format!("no such file or folder: {}", normalize_path(r))));
        }
    }
    if let Some((first, rest)) = walk_roots.split_first() {
        let mut b = WalkBuilder::new(first);
        for r in rest {
            b.add(r);
        }
        let root_for_filter = root.clone();
        b.hidden(false)
            .git_ignore(true)
            .git_exclude(true)
            // A machine's global gitignore would make two machines disagree (L5).
            .git_global(false)
            .ignore(true)
            .parents(true)
            .require_git(false)
            .follow_links(project.config.follow_symlinks())
            .sort_by_file_path(|a, b| a.cmp(b))
            .filter_entry(move |e| !excluded_by_default(e.path(), &root_for_filter));
        for entry in b.build().flatten() {
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
        let names: Vec<String> = file.raw.rules.keys().chain(file.raw.overrides.iter().flat_map(|o| o.rules.keys())).cloned().collect();
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
    let skipped = Vec::new();
    candidates.sort();
    candidates.dedup();
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
        if !project.config.is_included(&abs) {
            continue;
        }
        if !language_enabled(project, &abs, language) {
            continue;
        }
        files.push(FileEntry { rel: rel_path(&abs, &root), abs, language });
    }
    Ok((files, skipped, warnings))
}

/// `languages` in config and `--lang` on the command line (L3 decision 4).
pub(crate) fn language_enabled(project: &Project, abs: &Path, language: Language) -> bool {
    project.config.languages_for(abs).is_none_or(|set| set.contains(&language))
        && project.options.languages.as_ref().is_none_or(|set| set.contains(&language))
}
