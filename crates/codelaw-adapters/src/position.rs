//! Each tool counts positions its own way; codelaw counts bytes inside and
//! characters outside (SR-ARCHITECTURE decision 13). These turn a tool's
//! position into a byte offset of the file's text.

/// What a tool's column or offset counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// UTF-8 bytes (oxlint).
    Byte,
    /// Unicode code points (Ruff).
    CodePoint,
    /// UTF-16 code units: a JavaScript string index (stylelint,
    /// html-validate).
    Utf16,
}

/// The width of `c` in `unit`.
fn width(c: char, unit: Unit) -> usize {
    match unit {
        Unit::Byte => c.len_utf8(),
        Unit::CodePoint => 1,
        Unit::Utf16 => c.len_utf16(),
    }
}

/// The byte offset `n` units past byte `from`, stopping at the end of the
/// line when `in_line`; an offset inside a character moves to its end, and
/// one past the end of the text is the end.
fn advance(text: &str, from: usize, n: usize, unit: Unit, in_line: bool) -> usize {
    let mut counted = 0;
    for (i, c) in text[from..].char_indices() {
        if counted >= n || (in_line && c == '\n') {
            return from + i;
        }
        counted += width(c, unit);
    }
    text.len()
}

/// The byte offset of a whole-file offset in `unit`.
pub fn offset(text: &str, n: usize, unit: Unit) -> usize {
    advance(text, 0, n, unit, false)
}

/// The byte offset of a 1-based line and column in `unit`. A line past the
/// end is the end of the file; a column past the end of its line is the
/// line's end.
pub fn line_column(text: &str, line: u32, column: u32, unit: Unit) -> usize {
    let mut start = 0;
    for _ in 1..line.max(1) {
        match text[start..].find('\n') {
            Some(i) => start += i + 1,
            None => return text.len(),
        }
    }
    advance(text, start, column.saturating_sub(1) as usize, unit, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "a☕b\n𝄞x\n";

    #[test]
    fn offsets_in_each_unit() {
        // ☕ is 3 bytes, 1 code point, 1 UTF-16 unit; 𝄞 is 4, 1 and 2.
        assert_eq!(offset(TEXT, 4, Unit::Byte), 4);
        assert_eq!(offset(TEXT, 2, Unit::CodePoint), 4);
        assert_eq!(offset(TEXT, 2, Unit::Utf16), 4);
        let x = TEXT.find('x').unwrap();
        assert_eq!(offset(TEXT, 6, Unit::Utf16), x);
        assert_eq!(offset(TEXT, 5, Unit::CodePoint), x);
        assert_eq!(offset(TEXT, 99, Unit::Utf16), TEXT.len());
    }

    #[test]
    fn lines_and_columns_in_each_unit() {
        let x = TEXT.find('x').unwrap();
        assert_eq!(line_column(TEXT, 2, 2, Unit::CodePoint), x);
        assert_eq!(line_column(TEXT, 2, 3, Unit::Utf16), x);
        assert_eq!(line_column(TEXT, 1, 3, Unit::CodePoint), 4);
        // Past the end of the line: the line's end, not the next line.
        assert_eq!(line_column(TEXT, 1, 40, Unit::CodePoint), 5);
        assert_eq!(line_column(TEXT, 9, 1, Unit::CodePoint), TEXT.len());
    }
}
