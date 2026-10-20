//! `file:` URIs ↔ paths (RFC 8089, percent-encoding per RFC 3986).

use std::path::{Path, PathBuf};

/// The path of a `file:` URI, or `None` for any other scheme
/// (`untitled:`, `git:` …), which codelaw does not check.
pub fn to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    // `file:///p` and `file://localhost/p`; a remote host is not a local file.
    let path = if rest.starts_with('/') { rest } else { rest.strip_prefix("localhost")? };
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let decoded = decode(path)?;
    // `/C:/x` is `C:/x` on Windows.
    let b = decoded.as_bytes();
    let decoded = if cfg!(windows) && b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b':' { decoded[1..].to_owned() } else { decoded };
    Some(PathBuf::from(decoded))
}

fn decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The `file:` URI of an absolute path.
pub fn from_path(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file://");
    if !s.starts_with('/') {
        out.push('/');
    }
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uris_round_trip() {
        let p = Path::new("/home/me/my project/é𝄞.ts");
        let uri = from_path(p);
        assert_eq!(uri, "file:///home/me/my%20project/%C3%A9%F0%9D%84%9E.ts");
        assert_eq!(to_path(&uri).as_deref(), Some(p));
        assert_eq!(to_path("file://localhost/a/b.js").as_deref(), Some(Path::new("/a/b.js")));
    }

    #[test]
    fn other_schemes_are_not_files() {
        assert_eq!(to_path("untitled:Untitled-1"), None);
        assert_eq!(to_path("file://server/share/a.js"), None);
        assert_eq!(to_path("file:///bad%zz"), None);
    }
}
