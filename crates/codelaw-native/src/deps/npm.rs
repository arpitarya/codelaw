//! npm versions and the one-comparator ranges `data/outdated-dependency`
//! reads: `^1.2.3`, `~1.2`, `>=1`, `1.2.3`, `=1.2.3`. A range with `||`,
//! a hyphen, a space, an `x` or a tag is not one the rule can bump, so it
//! is not read.

use semver::Version;

/// One simple npm range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    /// `^`, `~`, `>=`, `>`, `<=`, `<`, `=` or `""`.
    pub op: String,
    /// The release components as written (one to three).
    pub parts: Vec<u64>,
    /// Byte offsets of the version in the range text.
    pub start: usize,
    pub end: usize,
    /// The version has a prerelease or build suffix.
    pub suffixed: bool,
}

const OPS: [&str; 7] = [">=", "<=", "^", "~", ">", "<", "="];

impl Range {
    pub fn parse(text: &str) -> Option<Range> {
        let lead = text.len() - text.trim_start().len();
        let t = text.trim();
        if t.is_empty() {
            return None;
        }
        let op = OPS.iter().find(|o| t.starts_with(**o)).copied().unwrap_or("");
        let after = &t[op.len()..];
        let vlead = after.len() - after.trim_start().len();
        let mut v = after.trim_start();
        let mut start = lead + op.len() + vlead;
        if let Some(r) = v.strip_prefix('v') {
            v = r;
            start += 1;
        }
        if v.is_empty() || v.contains(char::is_whitespace) {
            return None;
        }
        let core_len = v.find(['-', '+']).unwrap_or(v.len());
        let mut parts = Vec::new();
        for p in v[..core_len].split('.') {
            parts.push(p.parse::<u64>().ok()?);
        }
        if parts.is_empty() || parts.len() > 3 {
            return None;
        }
        let suffixed = core_len < v.len();
        if suffixed && (parts.len() != 3 || Version::parse(v).is_err()) {
            return None;
        }
        Some(Range { op: op.to_owned(), parts, start, end: start + v.len(), suffixed })
    }

    fn padded(&self) -> Version {
        Version::new(self.parts[0], self.parts.get(1).copied().unwrap_or(0), self.parts.get(2).copied().unwrap_or(0))
    }

    /// The exclusive upper bound of a `^`, `~` or partial range.
    fn upper(&self, bump: usize) -> Version {
        let mut p = [self.parts[0], self.parts.get(1).copied().unwrap_or(0), self.parts.get(2).copied().unwrap_or(0)];
        p[bump] += 1;
        for x in p.iter_mut().skip(bump + 1) {
            *x = 0;
        }
        Version::new(p[0], p[1], p[2])
    }

    /// Whether the range allows `v` (a stable version: prereleases are
    /// never targets unless asked for, and then they are compared plainly).
    pub fn allows(&self, v: &Version) -> bool {
        let base = self.padded();
        let plain = Version::new(v.major, v.minor, v.patch);
        let within = |bump: usize| plain >= base && plain < self.upper(bump);
        match self.op.as_str() {
            "^" => {
                let first = self.parts.iter().position(|p| *p != 0).unwrap_or(self.parts.len() - 1);
                within(first)
            }
            "~" => within(if self.parts.len() == 1 { 0 } else { 1 }),
            "" | "=" if self.parts.len() == 3 => {
                if self.suffixed {
                    return false;
                }
                plain == base && v.pre.is_empty()
            }
            "" | "=" => within(self.parts.len() - 1),
            ">=" => plain >= base,
            ">" => plain > base,
            "<=" => plain <= base,
            "<" => plain < base,
            _ => false,
        }
    }

    /// The version the range starts from.
    pub fn base(&self) -> Version {
        self.padded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn ranges_allow_what_npm_allows() {
        let r = |s: &str| Range::parse(s).unwrap_or_else(|| panic!("{s} parses"));
        assert!(r("^18.3.1").allows(&v("18.9.0")) && !r("^18.3.1").allows(&v("19.0.0")));
        assert!(r("^0.2.3").allows(&v("0.2.9")) && !r("^0.2.3").allows(&v("0.3.0")));
        assert!(r("~1.2.3").allows(&v("1.2.9")) && !r("~1.2.3").allows(&v("1.3.0")));
        assert!(r("~1").allows(&v("1.9.0")) && !r("~1").allows(&v("2.0.0")));
        assert!(r("1.2").allows(&v("1.2.7")) && !r("1.2").allows(&v("1.3.0")));
        assert!(r("1.2.3").allows(&v("1.2.3")) && !r("1.2.3").allows(&v("1.2.4")));
        assert!(r(">=1").allows(&v("7.0.0")));
        let t = "^18.3.1";
        let x = r(t);
        assert_eq!(&t[x.start..x.end], "18.3.1");
        for bad in ["1.x", "^1 || ^2", ">=1 <2", "latest", "workspace:*", "file:../a", "*", ""] {
            assert!(Range::parse(bad).is_none(), "{bad}");
        }
    }
}
