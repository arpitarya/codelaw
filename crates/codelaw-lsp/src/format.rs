//! `textDocument/formatting` (SR-EDITOR decision 1, SR-FORMATTER).
//!
//! The engine formats and verifies, exactly as `codelaw format` does
//! (SR-FORMATTER decision 4); the server turns the result into one
//! `TextEdit` and still writes nothing.

use codelaw_engine::{FileEntry, Project};

/// Whether `documentFormattingProvider` is advertised.
pub const AVAILABLE: bool = true;

/// The formatted text of `text`, or `None` when it cannot be formatted: it
/// does not parse, or the result failed verification. `format/failed` is
/// already among the document's diagnostics, so the editor shows why.
pub fn format_document(project: &Project, entry: &FileEntry, text: &str) -> Option<String> {
    project.format_source(&entry.rel, entry.language, text).ok()
}
