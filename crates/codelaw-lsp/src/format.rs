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

/// `textDocument/rangeFormatting` (SR-FORMATTER decision 8): the whole
/// document is formatted and verified as `format_document` does, and only
/// the changed runs of lines that touch lines `first..=last` (0-based) come
/// back, as (start byte, end byte, new text). The engines format whole
/// files, so this is the selection's share of what `codelaw format` writes.
pub fn range_edits(text: &str, formatted: &str, first: usize, last: usize) -> Vec<(usize, usize, String)> {
    use similar::{DiffTag, TextDiff};
    let diff = TextDiff::from_lines(text, formatted);
    let mut starts = vec![0];
    starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
    let at = |line: usize| starts.get(line).copied().unwrap_or(text.len()).min(text.len());
    let new_lines = diff.new_slices();
    let mut out = Vec::new();
    for op in diff.ops() {
        if op.tag() == DiffTag::Equal {
            continue;
        }
        let (old, new) = (op.old_range(), op.new_range());
        // A run that maps line for line is split per line, so a selection
        // takes only its own lines of it.
        if old.len() == new.len() {
            for (o, n) in old.zip(new) {
                if (first..=last).contains(&o) {
                    out.push((at(o), at(o + 1), new_lines[n].to_owned()));
                }
            }
            continue;
        }
        // An insertion touches the line it goes before.
        let touches = if old.is_empty() { (first..=last + 1).contains(&old.start) } else { old.start <= last && first < old.end };
        if touches {
            out.push((at(old.start), at(old.end), new_lines[new].concat()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::range_edits;

    fn apply(text: &str, edits: &[(usize, usize, String)]) -> String {
        let mut s = text.to_owned();
        for (a, b, t) in edits.iter().rev() {
            s.replace_range(*a..*b, t);
        }
        s
    }

    #[test]
    fn only_the_lines_in_range_change() {
        let (text, formatted) = ("let a=1\nlet b=2\nlet c=3\n", "let a = 1;\nlet b = 2;\nlet c = 3;\n");
        assert_eq!(apply(text, &range_edits(text, formatted, 1, 1)), "let a=1\nlet b = 2;\nlet c=3\n");
        assert_eq!(apply(text, &range_edits(text, formatted, 0, 2)), formatted);
        assert!(range_edits(formatted, formatted, 0, 2).is_empty());
    }
}
