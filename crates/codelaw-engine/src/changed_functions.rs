//! The functions a change set adds or edits (W-71 step 1): for each changed
//! JS, TS or Python file, its functions set against the same file at the base
//! revision — new, edited (by token stream) or untouched. The change set is an
//! explicit input (L5): `--changed-since`, `--staged` or the hook's files.

use crate::{EngineError, Project, git};
use codelaw_core::Language;
use codelaw_lang::{Change, Function};
use serde_json::{Value, json};
use std::path::Path;

/// One function and what the change did to it.
#[derive(Clone, Debug)]
pub struct ChangedFunction {
    pub function: Function,
    pub change: Change,
}

/// One changed file.
#[derive(Clone, Debug)]
pub struct ChangedFile {
    pub path: String,
    pub language: Language,
    /// The file has no copy at the base revision.
    pub new_file: bool,
    pub functions: Vec<ChangedFunction>,
}

/// The answer for the whole change set.
#[derive(Clone, Debug, Default)]
pub struct ChangedFunctions {
    /// The revision the base copies were read at; none when no change set was given.
    pub base: Option<String>,
    pub files: Vec<ChangedFile>,
    /// Why part of the change set could not be judged.
    pub warnings: Vec<String>,
}

impl ChangedFunctions {
    /// No change set was given, so nothing is judged.
    pub fn no_change_set(&self) -> bool {
        self.base.is_none()
    }

    /// The new and edited functions: what the test rules judge.
    pub fn judged(&self) -> impl Iterator<Item = (&ChangedFile, &ChangedFunction)> {
        self.files.iter().flat_map(|f| f.functions.iter().map(move |c| (f, c))).filter(|(_, c)| c.change != Change::Untouched)
    }

    pub fn to_json(&self) -> Value {
        json!({
            "base": self.base,
            "warnings": self.warnings,
            "files": self.files.iter().map(|f| json!({
                "path": f.path,
                "language": f.language.name(),
                "new_file": f.new_file,
                "functions": f.functions.iter().map(|c| json!({
                    "name": c.function.name,
                    "change": c.change.name(),
                    "first_line": c.function.first_line,
                    "last_line": c.function.last_line,
                    "public": c.function.public,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        match &self.base {
            None => out.push_str("new code: no change set; test rules not evaluated\n"),
            Some(b) => out.push_str(&format!("base: {b}\n")),
        }
        for w in &self.warnings {
            out.push_str(&format!("warning: {w}\n"));
        }
        for f in &self.files {
            out.push_str(&format!("{}{}\n", f.path, if f.new_file { " (new file)" } else { "" }));
            for c in &f.functions {
                out.push_str(&format!("  {:<9} {} (lines {}-{}{})\n", c.change.name(), c.function.name, c.function.first_line, c.function.last_line, if c.function.public { ", public" } else { "" }));
            }
        }
        out
    }
}

impl Project {
    /// Each changed file's functions as new, edited or untouched. Base copies:
    /// `--changed-since` reads the merge base's, `--staged` and the hook's
    /// files `HEAD`'s. Without a change set nothing is judged.
    pub fn changed_functions(&self) -> Result<ChangedFunctions, EngineError> {
        let (set, warnings) = self.touched();
        let mut out = ChangedFunctions { warnings: warnings.clone(), ..Default::default() };
        let opts = &self.options;
        if set.is_empty() && opts.changed_since.is_none() && !opts.staged && opts.touched_paths.is_empty() {
            return Ok(out);
        }
        let root = self.root();
        let base = match &opts.changed_since {
            Some(r) => match git::merge_base(root, r) {
                Ok(b) => b,
                Err(e) => {
                    out.warnings.push(format!("--changed-since {r}: {e}; no function is judged"));
                    return Ok(out);
                }
            },
            None => "HEAD".to_owned(),
        };
        out.base = Some(base.clone());
        for path in set {
            let rel = Path::new(path);
            let Some(language) = Language::from_path(rel).filter(|l| codelaw_lang::has_functions(*l)) else { continue };
            if path.ends_with(".d.ts") || path.ends_with(".d.mts") || path.ends_with(".d.cts") {
                continue;
            }
            let Ok(now) = std::fs::read_to_string(root.join(rel)) else { continue };
            let now = now.strip_prefix('\u{feff}').map(str::to_owned).unwrap_or(now);
            let before = match git::file_at(root, &base, path) {
                Ok(b) => b,
                Err(e) => {
                    out.warnings.push(format!("{path}: base copy at {base}: {e}; it is not judged"));
                    continue;
                }
            };
            let current = codelaw_lang::functions(language, &now);
            let old = before.as_deref().map(|b| codelaw_lang::functions(language, b));
            let changes = codelaw_lang::classify(&current, old.as_deref());
            out.files.push(ChangedFile {
                path: path.clone(),
                language,
                new_file: before.is_none(),
                functions: current.into_iter().zip(changes).map(|(function, change)| ChangedFunction { function, change }).collect(),
            });
        }
        Ok(out)
    }
}
