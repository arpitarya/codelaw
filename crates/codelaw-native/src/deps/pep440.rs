//! PEP 440 versions and specifiers, as far as `data/outdated-dependency`
//! needs them: parse, order, and decide whether a specifier allows a
//! version. Written here rather than taken from a crate so the engine gains
//! no dependency (L2).

use std::cmp::Ordering;

/// One PEP 440 version, normalised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pep440 {
    pub epoch: u64,
    pub release: Vec<u64>,
    /// `a`, `b` or `rc` (0, 1, 2) and its number.
    pub pre: Option<(u8, u64)>,
    pub post: Option<u64>,
    pub dev: Option<u64>,
    pub local: Option<String>,
}

fn take_num(s: &str) -> (Option<u64>, &str) {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if end == 0 {
        return (None, s);
    }
    (s[..end].parse().ok(), &s[end..])
}

fn sep(s: &str) -> &str {
    s.strip_prefix(['.', '-', '_']).unwrap_or(s)
}

impl Pep440 {
    pub fn parse(text: &str) -> Option<Pep440> {
        let lower = text.trim().to_ascii_lowercase();
        let mut s = lower.strip_prefix('v').unwrap_or(&lower);
        let (local, rest) = match s.split_once('+') {
            Some((a, b)) if !b.is_empty() => (Some(b.to_owned()), a),
            Some(_) => return None,
            None => (None, s),
        };
        s = rest;
        let mut epoch = 0;
        if let Some((e, r)) = s.split_once('!') {
            epoch = e.parse().ok()?;
            s = r;
        }
        let mut release = Vec::new();
        loop {
            let (n, r) = take_num(s);
            release.push(n?);
            s = r;
            match s.strip_prefix('.') {
                Some(r) if r.starts_with(|c: char| c.is_ascii_digit()) => s = r,
                _ => break,
            }
        }
        let mut pre = None;
        let t = sep(s);
        for (word, kind) in [("alpha", 0), ("beta", 1), ("preview", 2), ("pre", 2), ("rc", 2), ("a", 0), ("b", 1), ("c", 2)] {
            if let Some(r) = t.strip_prefix(word) {
                let (n, r) = take_num(sep(r));
                pre = Some((kind, n.unwrap_or(0)));
                s = r;
                break;
            }
        }
        let mut post = None;
        if let Some(r) = s.strip_prefix('-')
            && r.starts_with(|c: char| c.is_ascii_digit())
        {
            let (n, r) = take_num(r);
            post = n;
            s = r;
        } else {
            let t = sep(s);
            for word in ["post", "rev", "r"] {
                if let Some(r) = t.strip_prefix(word) {
                    let (n, r) = take_num(sep(r));
                    post = Some(n.unwrap_or(0));
                    s = r;
                    break;
                }
            }
        }
        let mut dev = None;
        let t = sep(s);
        if let Some(r) = t.strip_prefix("dev") {
            let (n, r) = take_num(sep(r));
            dev = Some(n.unwrap_or(0));
            s = r;
        }
        if !s.is_empty() {
            return None;
        }
        Some(Pep440 { epoch, release, pre, post, dev, local })
    }

    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some() || self.dev.is_some()
    }

    /// The release component `i`, zero past the written ones.
    pub fn part(&self, i: usize) -> u64 {
        self.release.get(i).copied().unwrap_or(0)
    }

    fn release_cmp(&self, other: &Pep440) -> Ordering {
        let n = self.release.len().max(other.release.len());
        (0..n).map(|i| self.part(i).cmp(&other.part(i))).find(|o| o.is_ne()).unwrap_or(Ordering::Equal)
    }

    /// The sort key after the release, as `packaging` builds it.
    fn suffix_key(&self) -> (i128, i128, i128) {
        const LOW: i128 = -1;
        const HIGH: i128 = i128::MAX;
        let pre = match (self.pre, self.post, self.dev) {
            (None, None, Some(_)) => LOW,
            (None, _, _) => HIGH,
            (Some((k, n)), _, _) => (k as i128) * (1 << 64) + n as i128,
        };
        let post = self.post.map(|n| n as i128).unwrap_or(LOW);
        let dev = self.dev.map(|n| n as i128).unwrap_or(HIGH);
        (pre, post, dev)
    }

    /// The version without its local label.
    pub fn public(&self) -> Pep440 {
        Pep440 { local: None, ..self.clone() }
    }
}

impl PartialOrd for Pep440 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pep440 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.epoch
            .cmp(&other.epoch)
            .then_with(|| self.release_cmp(other))
            .then_with(|| self.suffix_key().cmp(&other.suffix_key()))
            .then_with(|| self.local.cmp(&other.local))
    }
}

impl std::fmt::Display for Pep440 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.epoch != 0 {
            write!(f, "{}!", self.epoch)?;
        }
        write!(f, "{}", self.release.iter().map(u64::to_string).collect::<Vec<_>>().join("."))?;
        if let Some((k, n)) = self.pre {
            write!(f, "{}{n}", ["a", "b", "rc"][k as usize])?;
        }
        if let Some(n) = self.post {
            write!(f, ".post{n}")?;
        }
        if let Some(n) = self.dev {
            write!(f, ".dev{n}")?;
        }
        if let Some(l) = &self.local {
            write!(f, "+{l}")?;
        }
        Ok(())
    }
}

/// One clause of a specifier: `>=2.31`, `~=4.2`, `==1.*`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub op: String,
    /// The version as written, without a trailing `.*`.
    pub version: String,
    pub wildcard: bool,
    /// Byte offsets of `version` in the requirement string.
    pub start: usize,
    pub end: usize,
}

impl Clause {
    pub fn allows(&self, v: &Pep440) -> bool {
        let Some(base) = Pep440::parse(&self.version) else { return true };
        let v = v.public();
        let prefix = |v: &Pep440| v.epoch == base.epoch && (0..base.release.len()).all(|i| v.part(i) == base.part(i));
        match self.op.as_str() {
            "==" if self.wildcard => prefix(&v),
            "!=" if self.wildcard => !prefix(&v),
            "==" | "===" => v == base,
            "!=" => v != base,
            ">=" => v >= base,
            "<=" => v <= base,
            ">" => v > base,
            "<" => v < base,
            "~=" => {
                if base.release.len() < 2 {
                    return v >= base;
                }
                let stem = Pep440 { release: base.release[..base.release.len() - 1].to_vec(), pre: None, post: None, dev: None, local: None, epoch: base.epoch };
                let in_stem = (0..stem.release.len()).all(|i| v.part(i) == stem.part(i)) && v.epoch == base.epoch;
                v >= base && in_stem
            }
            _ => true,
        }
    }
}

/// A PEP 508 requirement as far as the rule reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    /// The name as written and normalised (PEP 503).
    pub name: String,
    pub normalized: String,
    pub clauses: Vec<Clause>,
}

/// A distribution name normalised (PEP 503): lower case, runs of `-_.` one `-`.
pub fn normalize(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if matches!(c, '-' | '_' | '.') {
            if !out.ends_with('-') {
                out.push('-');
            }
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

const OPS: [&str; 8] = ["===", "~=", "==", "!=", "<=", ">=", "<", ">"];

/// Parse `name[extras] (specifiers) ; marker`. A direct reference
/// (`name @ url`) has no clauses the rule can read, so it is `None`.
pub fn requirement(req: &str) -> Option<Requirement> {
    let lead = req.len() - req.trim_start().len();
    let body = &req[lead..];
    let name_len = body.find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))).unwrap_or(body.len());
    if name_len == 0 {
        return None;
    }
    let name = &body[..name_len];
    let mut at = lead + name_len;
    let rest = &req[at..];
    let skipped = rest.len() - rest.trim_start().len();
    at += skipped;
    if req[at..].starts_with('[') {
        at += req[at..].find(']')? + 1;
    }
    let end = req[at..].find(';').map(|i| at + i).unwrap_or(req.len());
    if req[at..end].trim_start().starts_with('@') {
        return None;
    }
    let mut clauses = Vec::new();
    let mut pos = at;
    for piece in req[at..end].split(',') {
        let piece_start = pos;
        pos += piece.len() + 1;
        let p = piece.trim_start().trim_start_matches('(').trim_start();
        let offset = piece_start + (piece.len() - p.len());
        if p.trim().is_empty() || p.trim() == ")" {
            continue;
        }
        let op = OPS.iter().find(|o| p.starts_with(**o))?;
        let after = &p[op.len()..];
        let vlead = after.len() - after.trim_start().len();
        let v = after.trim_start();
        let vlen = v.find(|c: char| c.is_whitespace() || c == ')').unwrap_or(v.len());
        let mut version = &v[..vlen];
        let wildcard = version.ends_with(".*");
        if wildcard {
            version = &version[..version.len() - 2];
        }
        let start = offset + op.len() + vlead;
        clauses.push(Clause { op: (*op).to_owned(), version: version.to_owned(), wildcard, start, end: start + version.len() });
    }
    Some(Requirement { name: name.to_owned(), normalized: normalize(name), clauses })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Pep440 {
        Pep440::parse(s).unwrap_or_else(|| panic!("{s} parses"))
    }

    #[test]
    fn versions_order_as_pep_440_says() {
        let ordered = ["1.0.dev0", "1.0a1", "1.0a2.dev1", "1.0a2", "1.0b1", "1.0rc1", "1.0", "1.0.post1.dev0", "1.0.post1", "1.1", "2!0.1"];
        for w in ordered.windows(2) {
            assert!(v(w[0]) < v(w[1]), "{} < {}", w[0], w[1]);
        }
        assert_eq!(v("1.0").cmp(&v("1.0.0")), Ordering::Equal);
        assert_eq!(v("1.0-1"), v("1.0.post1"));
        assert_eq!(v("1.0ALPHA1"), v("1.0a1"));
        assert!(v("1.0rc1").is_prerelease() && !v("1.0.post1").is_prerelease());
        assert!(Pep440::parse("not a version").is_none());
    }

    #[test]
    fn requirements_and_clauses() {
        let r = requirement("Django[bcrypt] >=4.2, <5 ; python_version >= '3.10'").unwrap();
        assert_eq!(r.normalized, "django");
        assert_eq!(r.clauses.iter().map(|c| (c.op.as_str(), c.version.as_str())).collect::<Vec<_>>(), [(">=", "4.2"), ("<", "5")]);
        let req = "requests~=2.31";
        let c = &requirement(req).unwrap().clauses[0];
        assert_eq!(&req[c.start..c.end], "2.31");
        assert!(c.allows(&v("2.32.3")) && !c.allows(&v("3.0")) && !c.allows(&v("2.30")));
        let w = &requirement("a==1.*").unwrap().clauses[0];
        assert!(w.allows(&v("1.9")) && !w.allows(&v("2.0")));
        assert!(requirement("pkg @ https://x/y.whl").is_none());
        assert_eq!(requirement("black").unwrap().clauses, []);
    }
}
