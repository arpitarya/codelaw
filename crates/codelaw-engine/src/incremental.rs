//! Incremental re-parse in warm processes (SR-ARCHITECTURE decision 23): an
//! editor's document keeps its syntax tree between edits, each edit
//! re-parses only what it touched, and the check of the edited text is the
//! check of a fresh parse of it (SR-EDITOR decision 3).

use crate::{FileEntry, Project};
use codelaw_core::{Diagnostic, Language};
use codelaw_lang::Root;

/// One open document's text and its host syntax tree. It is kept only while
/// the document is open, which bounds a warm process's trees by its open
/// files (decision 24).
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
