use serde::{Deserialize, Serialize};
use std::path::Path;

/// A 1-based line and a 1-based column counted in characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// Maps byte offsets to output positions.
pub struct LineIndex<'a> {
    text: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex { text, line_starts }
    }

    /// The position of a byte offset; an offset inside a multi-byte
    /// character counts as that character.
    pub fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let line = self.line_starts.partition_point(|&s| s <= offset) - 1;
        let start = self.line_starts[line];
        let mut end = offset;
        while !self.text.is_char_boundary(end) {
            end -= 1;
        }
        let column = self.text[start..end].chars().count();
        Position { line: line as u32 + 1, column: column as u32 + 1 }
    }

    /// The byte offset of the first character of a 1-based line.
    pub fn line_start(&self, line: u32) -> Option<usize> {
        self.line_starts.get((line as usize).checked_sub(1)?).copied()
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

/// Forward slashes in every output and cache key (SR-ARCHITECTURE decision 7).
pub fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    s.strip_prefix("./").map(str::to_owned).unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_count_characters() {
        let text = "ab\nçé𝄞x\n";
        let idx = LineIndex::new(text);
        assert_eq!(idx.position(0), Position { line: 1, column: 1 });
        assert_eq!(idx.position(3), Position { line: 2, column: 1 });
        // ç (2 bytes) é (2 bytes) 𝄞 (4 bytes) then x
        let x = text.find('x').unwrap();
        assert_eq!(idx.position(x), Position { line: 2, column: 4 });
        assert_eq!(idx.position(text.len()), Position { line: 3, column: 1 });
    }

    #[test]
    fn paths_use_forward_slashes() {
        assert_eq!(normalize_path(Path::new("./src\\a.ts")), "src/a.ts");
    }
}
