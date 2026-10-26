//! An lcov tracefile, read by codelaw itself (W-71 step 4; L11: no
//! dependency). Only `SF:` (the source file), `DA:<line>,<hits>` (a line and
//! how often it ran) and `end_of_record` are read; `FN`/`FNDA` are not
//! trusted — a function's coverage is its `DA` lines inside the span codelaw
//! parses now, so Jest, Vitest, c8 and coverage.py are judged alike.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One report: each source file's instrumented lines and their hits.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lcov {
    /// Project-relative path → 1-based line → hits. A file named twice
    /// (two test runs merged) adds its hits.
    pub files: BTreeMap<String, BTreeMap<u32, u64>>,
    /// `SF:` paths outside the project root, or naming no file.
    pub unmapped: usize,
}

impl Lcov {
    /// Read the report at `path`; `root` is the project root. A relative
    /// `SF:` is read against the report's folder first, then the root (as
    /// Jest writes absolute paths and coverage.py paths relative to where it
    /// ran).
    pub fn read(path: &Path, root: &Path) -> Result<Lcov, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read the coverage report {}: {e}", path.display()))?;
        Ok(Lcov::parse(&text, path.parent().unwrap_or(Path::new(".")), root))
    }

    pub fn parse(text: &str, report_dir: &Path, root: &Path) -> Lcov {
        let root_c = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        let mut out = Lcov::default();
        let mut current: Option<String> = None;
        for raw in text.lines() {
            let line = raw.trim();
            if let Some(sf) = line.strip_prefix("SF:") {
                current = rel(sf.trim(), report_dir, root, &root_c);
                if current.is_none() {
                    out.unmapped += 1;
                }
            } else if let Some(da) = line.strip_prefix("DA:") {
                let Some(file) = &current else { continue };
                let mut parts = da.split(',');
                let (Some(l), Some(h)) = (parts.next(), parts.next()) else { continue };
                let (Ok(l), Ok(h)) = (l.trim().parse::<u32>(), h.trim().parse::<f64>()) else { continue };
                if l == 0 {
                    continue;
                }
                let hits = if h.is_finite() && h > 0.0 { h.ceil() as u64 } else { 0 };
                *out.files.entry(file.clone()).or_default().entry(l).or_default() += hits;
            } else if line == "end_of_record" {
                current = None;
            }
        }
        out
    }
}

/// A `SF:` path as project-relative, `/`-separated; `None` outside the root.
fn rel(sf: &str, report_dir: &Path, root: &Path, root_c: &Path) -> Option<String> {
    let sf = sf.replace('\\', "/");
    let p = PathBuf::from(&sf);
    let candidates: Vec<PathBuf> = if p.is_absolute() { vec![p] } else { vec![report_dir.join(&p), root.join(&p)] };
    for c in &candidates {
        let abs = std::fs::canonicalize(c).unwrap_or_else(|_| c.clone());
        for base in [root_c, root] {
            if let Ok(r) = abs.strip_prefix(base)
                && (abs.exists() || candidates.len() == 1)
            {
                return Some(codelaw_core::normalize_path(r));
            }
        }
    }
    // Relative and naming no file under either folder: take it as written,
    // against the root, so the file is still found stale, not silently lost.
    (!PathBuf::from(&sf).is_absolute()).then(|| sf.trim_start_matches("./").to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sf_and_da_and_ignores_the_rest() {
        let d = std::env::temp_dir().join(format!("codelaw-lcov-{}", std::process::id()));
        std::fs::create_dir_all(d.join("src")).unwrap();
        std::fs::write(d.join("src/a.ts"), "x\n").unwrap();
        std::fs::write(d.join("b.py"), "x\n").unwrap();
        let abs = d.join("src/a.ts");
        let text = format!(
            "TN:\nSF:{}\nFN:1,f\nFNDA:3,f\nDA:1,3\nDA:2,0\nDA:3,1.5\nLF:3\nLH:2\nend_of_record\nSF:b.py\nDA:1,0\nend_of_record\nSF:/elsewhere/c.js\nDA:1,1\nend_of_record\nSF:src/a.ts\nDA:2,4\nend_of_record\n",
            abs.display()
        );
        let l = Lcov::parse(&text, &d, &d);
        assert_eq!(l.files["src/a.ts"], BTreeMap::from([(1, 3), (2, 4), (3, 2)]));
        assert_eq!(l.files["b.py"], BTreeMap::from([(1, 0)]));
        assert_eq!(l.unmapped, 1);
        std::fs::remove_dir_all(&d).ok();
    }
}
