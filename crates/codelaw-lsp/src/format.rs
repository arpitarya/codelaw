//! `textDocument/formatting` (SR-EDITOR decision 1, SR-FORMATTER).
//!
//! The formatter engines (`codelaw-format`, W-07) are not in this build, so
//! the capability is not advertised. Wiring them is this one function and
//! the flag: return the formatted text and set `AVAILABLE`; the server turns
//! the result into one `TextEdit` and still writes nothing.

use codelaw_engine::{FileEntry, Project};

/// Whether `documentFormattingProvider` is advertised.
pub const AVAILABLE: bool = false;

/// The formatted text of `text`, or `None` when it cannot be formatted.
pub fn format_document(_project: &Project, _entry: &FileEntry, _text: &str) -> Option<String> {
    None
}
