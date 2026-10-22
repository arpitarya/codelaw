//! Incremental re-parse in warm processes (SR-ARCHITECTURE decision 23): an
//! editor's document keeps its syntax tree between edits, each edit
//! re-parses only what it touched, and the check of the edited text is the
//! check of a fresh parse of it (SR-EDITOR decision 3).

use crate::{FileEntry, Project};
use codelaw_core::{Diagnostic, Language};
use codelaw_lang::Root;
use std::collections::VecDeque;

/// One document's text and its host syntax tree. An open document keeps its
/// tree; a closed one's tree goes to a [`TreeCache`], which bounds a warm
/// process's trees by its open files plus the most recently touched closed
/// ones (decision 24).
#[derive(Clone)]
pub struct DocumentTree {
    language: Language,
    root: Root,
}

impl DocumentTree {
    /// Parse `text` (no byte-order mark) as `language`.
    pub fn new(language: Language, text: &str) -> DocumentTree {
        DocumentTree { language, root: codelaw_lang::parse(language, text) }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    /// The text the tree is of.
    pub fn text(&self) -> &str {
        codelaw_lang::text_of(&self.root)
    }

    /// Replace the bytes `start..end` with `text` and re-parse incrementally.
    /// A range that is out of bounds or splits a character is refused and
    /// changes nothing; the caller parses afresh.
    pub fn edit(&mut self, start: usize, end: usize, text: &str) -> Result<(), String> {
        let current = self.text();
        if start > end || end > current.len() || !current.is_char_boundary(start) || !current.is_char_boundary(end) {
            return Err(format!("edit {start}..{end} does not fit the text"));
        }
        codelaw_lang::reparse(&mut self.root, start, end, text)
    }
}

impl Project {
    /// [`Project::check_document`] on a document's kept tree: the same
    /// diagnostics, without parsing the host text again. A tree of another
    /// language is parsed afresh, so the answer never depends on it.
    pub fn check_document_tree(&self, entry: &FileEntry, tree: &DocumentTree) -> Vec<Diagnostic> {
        let host = (tree.language == entry.language).then_some(&tree.root);
        self.check_document_with(entry, tree.text(), host)
    }
}

/// The syntax trees of recently closed files, most recently touched last,
/// at most `capacity` of them (SR-ARCHITECTURE decision 24, `[lsp]
/// tree_cache`). A file reopened while its tree is here gets it back
/// instead of parsing again; the tree is only used when its text is the
/// reopened text, so what the cache holds never changes a verdict.
#[derive(Clone, Default)]
pub struct TreeCache {
    capacity: usize,
    trees: VecDeque<(String, DocumentTree)>,
}

impl TreeCache {
    pub fn new(capacity: usize) -> TreeCache {
        TreeCache { capacity, trees: VecDeque::new() }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Change the bound, dropping the least recently touched trees over it.
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
        self.bound();
    }

    pub fn len(&self) -> usize {
        self.trees.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.trees.iter().any(|(k, _)| k == key)
    }

    /// Keep `tree` as the most recently touched, replacing any tree already
    /// kept for `key` and dropping the least recently touched over the bound.
    pub fn put(&mut self, key: String, tree: DocumentTree) {
        self.trees.retain(|(k, _)| *k != key);
        self.trees.push_back((key, tree));
        self.bound();
    }

    /// Drop the least recently touched trees, at the front, over the bound.
    fn bound(&mut self) {
        while self.trees.len() > self.capacity {
            self.trees.pop_front();
        }
    }

    /// Take the tree kept for `key` out of the cache.
    pub fn take(&mut self, key: &str) -> Option<DocumentTree> {
        let i = self.trees.iter().position(|(k, _)| k == key)?;
        self.trees.remove(i).map(|(_, t)| t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(text: &str) -> DocumentTree {
        DocumentTree::new(Language::Python, text)
    }

    #[test]
    fn the_bound_drops_the_least_recently_touched() {
        let mut c = TreeCache::new(2);
        c.put("a".into(), tree("a = 1\n"));
        c.put("b".into(), tree("b = 1\n"));
        c.put("a".into(), tree("a = 2\n"));
        c.put("c".into(), tree("c = 1\n"));
        assert_eq!(c.len(), 2);
        assert!(!c.contains("b"), "b was the least recently touched");
        assert_eq!(c.take("a").unwrap().text(), "a = 2\n", "a re-put replaces its tree");
        assert!(c.take("a").is_none(), "taking removes it");
        c.set_capacity(0);
        assert!(c.is_empty());
        c.put("d".into(), tree("d = 1\n"));
        assert!(c.is_empty(), "a bound of 0 keeps no closed tree");
    }
}
