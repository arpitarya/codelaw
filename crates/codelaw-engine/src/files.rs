//! What a native rule sees of a file beyond its text (SR-ARCHITECTURE
//! decision 15b): the file's path, its bytes as they are on disk, and a
//! read-only view of the files around it. An embedded region sees its host
//! file.
//!
//! Every question a rule asks about another file is recorded with a hash of
//! its answer. A cached result is reused only while every answer still
//! holds (SR-CACHE §1), so a link checked against a missing file is checked
//! again once the file exists.

use codelaw_cache::{Key, hash_bytes};
use codelaw_core::Language;
use codelaw_native::FileContext;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

/// Where the files around a file come from.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Tree<'a> {
    /// The files on disk under the project root.
    Disk,
    /// An in-memory set: `check_files` and the rule tester. Nothing on disk
    /// is read.
    Memory(&'a [(String, Language, String)]),
}

/// One question a rule asked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Question {
    Exists(String),
    IsDir(String),
    Read(String),
    List(String),
}

/// A question and the hash of its answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Asked {
    pub question: Question,
    pub answer: [u8; 32],
}

/// A file as a native rule sees it.
pub(crate) struct ProjectFile<'a> {
    path: &'a str,
    bytes: Vec<u8>,
    root: &'a Path,
    tree: Tree<'a>,
    asked: Mutex<Vec<Asked>>,
}

impl<'a> ProjectFile<'a> {
    /// `text` is the file's text without a byte-order mark; `bom` says
    /// whether the file has one.
    pub fn new(path: &'a str, text: &str, bom: bool, root: &'a Path, tree: Tree<'a>) -> Self {
        let mut bytes = Vec::with_capacity(text.len() + 3);
        if bom {
            bytes.extend_from_slice("\u{feff}".as_bytes());
        }
        bytes.extend_from_slice(text.as_bytes());
        ProjectFile { path, bytes, root, tree, asked: Mutex::new(Vec::new()) }
    }

    /// Every question asked so far, in order, without repeats.
    pub fn asked(&self) -> Vec<Asked> {
        self.asked.lock().map(|a| a.clone()).unwrap_or_default()
    }

    fn record(&self, question: Question, answer: Key) {
        if let Ok(mut a) = self.asked.lock()
            && !a.iter().any(|x| x.question == question)
        {
            a.push(Asked { question, answer: answer.0 });
        }
    }
}

/// A project-relative path a rule may name: `/`-separated, with no empty,
/// `.` or `..` segment and nothing that could leave the root. `""` is the
/// root itself.
fn safe(path: &str) -> bool {
    path.is_empty() || (!path.contains('\\') && !path.contains(':') && path.split('/').all(|s| !s.is_empty() && s != "." && s != ".."))
}

fn hash_bool(b: bool) -> Key {
    hash_bytes(&[b as u8])
}

fn hash_text(t: &Option<String>) -> Key {
    match t {
        None => hash_bytes(b"\0none"),
        Some(s) => hash_bytes(format!("\x01{s}").as_bytes()),
    }
}

fn hash_list(l: &Option<Vec<String>>) -> Key {
    match l {
        None => hash_bytes(b"\0none"),
        Some(v) => hash_bytes(format!("\x01{}", v.join("\0")).as_bytes()),
    }
}

/// The answer to a question, from `tree`.
fn answer(root: &Path, tree: Tree<'_>, q: &Question) -> Key {
    match q {
        Question::Exists(p) => hash_bool(exists(root, tree, p)),
        Question::IsDir(p) => hash_bool(is_dir(root, tree, p)),
        Question::Read(p) => hash_text(&read(root, tree, p)),
        Question::List(p) => hash_list(&list(root, tree, p)),
    }
}

/// Whether every recorded answer still holds against the disk (a cached
/// result's check before it is reused).
pub(crate) fn still_holds(root: &Path, asked: &[Asked]) -> bool {
    asked.iter().all(|a| answer(root, Tree::Disk, &a.question).0 == a.answer)
}

fn exists(root: &Path, tree: Tree<'_>, path: &str) -> bool {
    if !safe(path) {
        return false;
    }
    match tree {
        Tree::Disk => root.join(path).exists(),
        Tree::Memory(files) => path.is_empty() || files.iter().any(|(p, _, _)| p == path || p.strip_prefix(path).is_some_and(|r| r.starts_with('/'))),
    }
}

fn is_dir(root: &Path, tree: Tree<'_>, path: &str) -> bool {
    if !safe(path) {
        return false;
    }
    match tree {
        Tree::Disk => root.join(path).is_dir(),
        Tree::Memory(files) => path.is_empty() || files.iter().any(|(p, _, _)| p.strip_prefix(path).is_some_and(|r| r.starts_with('/'))),
    }
}

fn read(root: &Path, tree: Tree<'_>, path: &str) -> Option<String> {
    if !safe(path) || path.is_empty() {
        return None;
    }
    match tree {
        Tree::Disk => {
            let p = root.join(path);
            if !p.is_file() {
                return None;
            }
            let bytes = std::fs::read(p).ok()?;
            let text = String::from_utf8(bytes).ok()?;
            Some(text.strip_prefix('\u{feff}').map(str::to_owned).unwrap_or(text))
        }
        Tree::Memory(files) => files.iter().find(|(p, _, _)| p == path).map(|(_, _, t)| t.clone()),
    }
}

fn list(root: &Path, tree: Tree<'_>, path: &str) -> Option<Vec<String>> {
    if !safe(path) || !is_dir(root, tree, path) {
        return None;
    }
    let mut names: Vec<String> = match tree {
        Tree::Disk => std::fs::read_dir(root.join(path)).ok()?.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect(),
        Tree::Memory(files) => {
            let prefix = if path.is_empty() { String::new() } else { format!("{path}/") };
            files.iter().filter_map(|(p, _, _)| p.strip_prefix(prefix.as_str())).map(|r| r.split('/').next().unwrap_or(r).to_owned()).collect()
        }
    };
    names.sort();
    names.dedup();
    Some(names)
}

impl FileContext for ProjectFile<'_> {
    fn path(&self) -> &str {
        self.path
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn exists(&self, path: &str) -> bool {
        let a = exists(self.root, self.tree, path);
        self.record(Question::Exists(path.to_owned()), hash_bool(a));
        a
    }
    fn is_dir(&self, path: &str) -> bool {
        let a = is_dir(self.root, self.tree, path);
        self.record(Question::IsDir(path.to_owned()), hash_bool(a));
        a
    }
    fn read(&self, path: &str) -> Option<String> {
        let a = read(self.root, self.tree, path);
        self.record(Question::Read(path.to_owned()), hash_text(&a));
        a
    }
    fn list(&self, path: &str) -> Option<Vec<String>> {
        let a = list(self.root, self.tree, path);
        self.record(Question::List(path.to_owned()), hash_list(&a));
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_tree_answers_and_records() {
        let files = vec![
            ("docs/a.md".to_owned(), Language::Markdown, "# A\n".to_owned()),
            ("docs/sub/b.md".to_owned(), Language::Markdown, "# B\n".to_owned()),
            ("README.md".to_owned(), Language::Markdown, "# R\n".to_owned()),
        ];
        let f = ProjectFile::new("docs/a.md", "# A\n", true, Path::new("/nowhere"), Tree::Memory(&files));
        assert_eq!(f.bytes(), "\u{feff}# A\n".as_bytes());
        assert!(f.exists("docs/sub/b.md"));
        assert!(f.exists("docs/sub"));
        assert!(f.is_dir("docs/sub"));
        assert!(!f.is_dir("docs/a.md"));
        assert!(!f.exists("docs/c.md"));
        assert!(!f.exists("../etc/passwd"));
        assert!(!f.exists("docs/./a.md"));
        assert_eq!(f.read("README.md").as_deref(), Some("# R\n"));
        assert_eq!(f.list("docs"), Some(vec!["a.md".to_owned(), "sub".to_owned()]));
        assert_eq!(f.list(""), Some(vec!["README.md".to_owned(), "docs".to_owned()]));
        assert_eq!(f.list("README.md"), None);
        f.exists("docs/sub/b.md");
        assert_eq!(f.asked().len(), 11, "a repeated question is recorded once");
    }

    #[test]
    fn disk_answers_are_checked_again() {
        let dir = std::env::temp_dir().join(format!("codelaw-files-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = ProjectFile::new("a.md", "", false, &dir, Tree::Disk);
        assert!(!f.exists("b.md"));
        let asked = f.asked();
        assert!(still_holds(&dir, &asked));
        std::fs::write(dir.join("b.md"), "x").unwrap();
        assert!(!still_holds(&dir, &asked), "the file now exists, so the cached answer is stale");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
