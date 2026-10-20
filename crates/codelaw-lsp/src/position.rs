//! Byte offsets ↔ LSP positions (SR-ARCHITECTURE decision 13).
//!
//! The engine counts bytes. LSP counts 0-based lines and, inside a line,
//! UTF-16 code units (the default `positionEncoding`). This module is the
//! only place that conversion happens; nothing else in the server sees a
//! UTF-16 number.

use lsp_types::{Position, Range};

/// The line starts of one text, for converting in both directions.
pub struct Utf16Index<'a> {
    text: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> Utf16Index<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Utf16Index { text, line_starts }
    }

    /// The LSP position of a byte offset. An offset past the end is the end;
    /// an offset inside a multi-byte character is that character's start.
    pub fn position(&self, offset: usize) -> Position {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        let line = self.line_starts.partition_point(|&s| s <= offset) - 1;
        let start = self.line_starts[line];
        let character: usize = self.text[start..offset].chars().map(char::len_utf16).sum();
        Position { line: line as u32, character: character as u32 }
    }

    pub fn range(&self, start: usize, end: usize) -> Range {
        Range { start: self.position(start), end: self.position(end) }
    }

    /// The byte offset of an LSP position. A line past the end is the end of
    /// the text; a character past the end of its line is the line's end
    /// (before its `\r\n` or `\n`), as the protocol asks; a character
    /// between the two halves of a surrogate pair is the pair's start.
    pub fn offset(&self, pos: Position) -> usize {
        let Some(&start) = self.line_starts.get(pos.line as usize) else { return self.text.len() };
        let end = self.line_starts.get(pos.line as usize + 1).copied().unwrap_or(self.text.len());
        let line = &self.text[start..end];
        let content = line.strip_suffix('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).unwrap_or(line);
        let mut units = 0u32;
        for (i, c) in content.char_indices() {
            let next = units + c.len_utf16() as u32;
            if next > pos.character {
                return start + i;
            }
            units = next;
        }
        start + content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    /// `é` is 2 bytes and 1 unit, `中` 3 bytes and 1 unit, `𝄞` and `😀`
    /// 4 bytes and 2 units (a surrogate pair).
    const TEXT: &str = "aé𝄞b\r\n中😀x\n\n𝄞";

    #[test]
    fn utf16_columns_on_multi_byte_and_astral_characters() {
        let idx = Utf16Index::new(TEXT);
        assert_eq!(idx.position(0), pos(0, 0));
        assert_eq!(idx.position(TEXT.find('é').unwrap()), pos(0, 1));
        assert_eq!(idx.position(TEXT.find('𝄞').unwrap()), pos(0, 2));
        assert_eq!(idx.position(TEXT.find('b').unwrap()), pos(0, 4));
        assert_eq!(idx.position(TEXT.find('\r').unwrap()), pos(0, 5));
        assert_eq!(idx.position(TEXT.find('中').unwrap()), pos(1, 0));
        assert_eq!(idx.position(TEXT.find('😀').unwrap()), pos(1, 1));
        assert_eq!(idx.position(TEXT.find('x').unwrap()), pos(1, 3));
        assert_eq!(idx.position(TEXT.rfind('𝄞').unwrap()), pos(3, 0));
        assert_eq!(idx.position(TEXT.len()), pos(3, 2));
        // Inside a character: its start.
        assert_eq!(idx.position(TEXT.find('𝄞').unwrap() + 2), pos(0, 2));
        assert_eq!(idx.position(usize::MAX), pos(3, 2));
    }

    #[test]
    fn every_character_boundary_round_trips() {
        let idx = Utf16Index::new(TEXT);
        for offset in (0..=TEXT.len()).filter(|&o| TEXT.is_char_boundary(o)) {
            // Between the `\r` and `\n` of a line ending is not a position
            // LSP can name; it maps to the line's end.
            if TEXT[..offset].ends_with('\r') {
                assert_eq!(idx.offset(idx.position(offset)), offset - 1);
                continue;
            }
            assert_eq!(idx.offset(idx.position(offset)), offset, "offset {offset}");
        }
    }

    #[test]
    fn positions_out_of_range_clamp() {
        let idx = Utf16Index::new(TEXT);
        // Past the end of line 0: before its `\r\n`.
        assert_eq!(idx.offset(pos(0, 99)), TEXT.find('\r').unwrap());
        // Between the halves of 😀: its start.
        assert_eq!(idx.offset(pos(1, 2)), TEXT.find('😀').unwrap());
        assert_eq!(idx.offset(pos(2, 5)), TEXT.rfind('\n').unwrap());
        assert_eq!(idx.offset(pos(42, 0)), TEXT.len());
    }

    #[test]
    fn utf16_matches_the_client_side_count() {
        // What a UTF-16 client (VS Code) computes: code units before the
        // character on its line.
        let idx = Utf16Index::new(TEXT);
        for (i, line) in TEXT.split('\n').enumerate() {
            let start = idx.line_starts[i];
            for (b, c) in line.char_indices() {
                let units = line[..b].encode_utf16().count() as u32;
                assert_eq!(idx.position(start + b), pos(i as u32, units), "{c:?}");
            }
        }
    }
}
