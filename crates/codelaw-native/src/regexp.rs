//! Rules that read inside regular expressions: `js/no-invalid-regexp`,
//! `js/no-misleading-character-class` and `js/no-useless-backreference`.
//! A small scanner of JavaScript's regex syntax: groups, alternatives,
//! character classes, escapes and quantifiers.

use crate::util::{named_children, string_value, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

#[derive(Clone, Debug)]
struct Group {
    number: Option<usize>,
    name: Option<String>,
    start: usize,
    end: Option<usize>,
    /// (disjunction id, alternative index) from the outside in.
    path: Vec<(usize, usize)>,
    negative_lookaround: bool,
}

#[derive(Clone, Debug)]
struct Backref {
    start: usize,
    end: usize,
    target: Result<usize, String>,
    path: Vec<(usize, usize)>,
    /// Indices into `groups` of the groups open around it.
    inside: Vec<usize>,
}

#[derive(Debug, Default)]
struct Scan {
    groups: Vec<Group>,
    backrefs: Vec<Backref>,
    /// Character classes: byte ranges of their content.
    classes: Vec<(usize, usize)>,
    error: Option<String>,
}

/// Scan a pattern. Errors are the first syntax problem found.
fn scan(p: &str, unicode: bool) -> Scan {
    let b = p.as_bytes();
    let mut s = Scan::default();
    let mut stack: Vec<usize> = Vec::new(); // open groups (indices into s.groups); usize::MAX for non-capturing
    let mut path: Vec<(usize, usize)> = vec![(0, 0)];
    let mut next_disj = 1;
    let mut capture = 0;
    let mut can_repeat = false;
    let mut i = 0;
    let mut neg = 0usize;
    let mut neg_stack: Vec<bool> = Vec::new();
    while i < b.len() {
        let c = b[i];
        match c {
            b'\\' => {
                let Some(&n) = b.get(i + 1) else {
                    s.error.get_or_insert("the pattern ends with a lone `\\`".into());
                    break;
                };
                if n.is_ascii_digit() && n != b'0' {
                    let len = b[i + 1..].iter().take_while(|d| d.is_ascii_digit()).count();
                    let num: usize = p[i + 1..i + 1 + len].parse().unwrap_or(0);
                    s.backrefs.push(Backref { start: i, end: i + 1 + len, target: Ok(num), path: path.clone(), inside: stack.iter().copied().filter(|g| *g != usize::MAX).collect() });
                    i += 1 + len;
                } else if n == b'k' && b.get(i + 2) == Some(&b'<') {
                    let close = p[i + 3..].find('>').map(|x| i + 3 + x);
                    let Some(close) = close else {
                        s.error.get_or_insert("`\\k<` has no closing `>`".into());
                        break;
                    };
                    s.backrefs.push(Backref { start: i, end: close + 1, target: Err(p[i + 3..close].to_owned()), path: path.clone(), inside: stack.iter().copied().filter(|g| *g != usize::MAX).collect() });
                    i = close + 1;
                } else {
                    if unicode && n.is_ascii_alphabetic() && !b"dDwWsSbBnrtfvcxupPk".contains(&n) {
                        s.error.get_or_insert(format!("`\\{}` is not a valid escape with the `u` flag", n as char));
                    }
                    i += 2;
                }
                can_repeat = true;
                continue;
            }
            b'[' => {
                let start = i + 1;
                let mut j = start;
                if b.get(j) == Some(&b'^') {
                    j += 1;
                }
                let mut esc = false;
                let mut closed = None;
                while j < b.len() {
                    if esc {
                        esc = false;
                    } else if b[j] == b'\\' {
                        esc = true;
                    } else if b[j] == b']' {
                        closed = Some(j);
                        break;
                    }
                    j += 1;
                }
                let Some(end) = closed else {
                    s.error.get_or_insert("a character class `[` is never closed".into());
                    break;
                };
                s.classes.push((start, end));
                i = end + 1;
                can_repeat = true;
                continue;
            }
            b'(' => {
                let rest = &p[i + 1..];
                let (capturing, name, skip, negative) = if let Some(r) = rest.strip_prefix("?<") {
                    if r.starts_with('=') || r.starts_with('!') {
                        (false, None, 3, r.starts_with('!'))
                    } else {
                        match r.find('>') {
                            Some(x) if x > 0 => (true, Some(r[..x].to_owned()), 2 + x + 1, false),
                            _ => {
                                s.error.get_or_insert("an invalid group name in `(?<…>`".into());
                                break;
                            }
                        }
                    }
                } else if rest.starts_with("?:") || rest.starts_with("?=") || rest.starts_with("?!") {
                    (false, None, 2, rest.starts_with("?!"))
                } else if rest.starts_with('?') {
                    s.error.get_or_insert("an invalid group: `(?` must be followed by `:`, `=`, `!` or `<`".into());
                    break;
                } else {
                    (true, None, 0, false)
                };
                neg_stack.push(negative);
                if negative {
                    neg += 1;
                }
                if capturing {
                    capture += 1;
                    s.groups.push(Group { number: Some(capture), name, start: i, end: None, path: path.clone(), negative_lookaround: neg > 0 });
                    stack.push(s.groups.len() - 1);
                } else {
                    stack.push(usize::MAX);
                }
                path.push((next_disj, 0));
                next_disj += 1;
                i += 1 + skip;
                can_repeat = false;
                continue;
            }
            b')' => {
                let Some(g) = stack.pop() else {
                    s.error.get_or_insert("an unmatched `)`".into());
                    break;
                };
                if g != usize::MAX {
                    s.groups[g].end = Some(i + 1);
                }
                if neg_stack.pop().unwrap_or(false) {
                    neg -= 1;
                }
                path.pop();
                can_repeat = true;
            }
            b'|' => {
                if let Some(last) = path.last_mut() {
                    last.1 += 1;
                }
                can_repeat = false;
            }
            b'*' | b'+' | b'?' => {
                if !can_repeat {
                    s.error.get_or_insert(format!("nothing to repeat before `{}`", c as char));
                    break;
                }
                // A lazy `?` after a quantifier.
                if b.get(i + 1) == Some(&b'?') {
                    i += 1;
                }
                can_repeat = false;
            }
            b'{' => {
                let close = p[i..].find('}').map(|x| i + x);
                let body = close.map(|e| &p[i + 1..e]);
                let quantifier = body.filter(|x| !x.is_empty() && x.split(',').count() <= 2 && x.split(',').next().is_some_and(|a| !a.is_empty() && a.bytes().all(|d| d.is_ascii_digit())) && x.split(',').nth(1).is_none_or(|a| a.bytes().all(|d| d.is_ascii_digit())));
                if let (Some(q), Some(e)) = (quantifier, close) {
                    if !can_repeat {
                        s.error.get_or_insert("nothing to repeat before `{`".into());
                        break;
                    }
                    let mut parts = q.split(',');
                    let lo: u64 = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
                    if let Some(hi) = parts.next().filter(|x| !x.is_empty()).and_then(|x| x.parse::<u64>().ok())
                        && hi < lo
                    {
                        s.error.get_or_insert(format!("the quantifier `{{{q}}}` has its numbers out of order"));
                        break;
                    }
                    i = e + 1;
                    if b.get(i) == Some(&b'?') {
                        i += 1;
                    }
                    can_repeat = false;
                    continue;
                } else if unicode {
                    s.error.get_or_insert("a lone `{` is not allowed with the `u` flag".into());
                    break;
                }
                can_repeat = true;
            }
            b'^' | b'$' => can_repeat = false,
            _ => can_repeat = true,
        }
        i += 1;
    }
    if s.error.is_none() && !stack.is_empty() {
        s.error = Some("a group `(` is never closed".into());
    }
    s
}

fn valid_flags(flags: &str) -> Result<(), String> {
    let mut seen = String::new();
    for f in flags.chars() {
        if !"dgimsuvy".contains(f) {
            return Err(format!("`{f}` is not a regular expression flag"));
        }
        if seen.contains(f) {
            return Err(format!("the flag `{f}` is given twice"));
        }
        seen.push(f);
    }
    if seen.contains('u') && seen.contains('v') {
        return Err("the `u` and `v` flags can't be combined".into());
    }
    Ok(())
}

/// A regex literal's pattern and flags, or a `RegExp(...)` call's string
/// arguments; `None` when either part isn't a plain literal.
fn regex_parts(n: tree_sitter::Node<'_>, src: &str) -> Option<(String, String, usize)> {
    match n.kind() {
        "regex" => {
            let pattern = n.child_by_field_name("pattern")?;
            let flags = n.child_by_field_name("flags").map(|f| text(f, src).to_owned()).unwrap_or_default();
            Some((text(pattern, src).to_owned(), flags, pattern.start_byte()))
        }
        "call_expression" | "new_expression" => {
            let callee = n.child_by_field_name("function").or_else(|| n.child_by_field_name("constructor"))?;
            if text(callee, src) != "RegExp" {
                return None;
            }
            let args = named_children(n.child_by_field_name("arguments")?);
            let pattern = args.first().filter(|a| a.kind() == "string")?;
            let flags = match args.get(1) {
                Some(f) if f.kind() == "string" => string_value(*f, src),
                Some(_) => return None,
                None => String::new(),
            };
            Some((string_value(*pattern, src).replace("\\\\", "\\"), flags, pattern.start_byte() + 1))
        }
        _ => None,
    }
}

pub struct NoInvalidRegexp;

impl NativeRule for NoInvalidRegexp {
    fn id(&self) -> &'static str {
        "js/no-invalid-regexp"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression", "new_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let Some((pattern, flags, _)) = regex_parts(n, ctx.source) else { return };
        let problem = valid_flags(&flags).err().or_else(|| scan(&pattern, flags.contains('u') || flags.contains('v')).error);
        if let Some(p) = problem {
            out.push(NativeFinding { start: n.start_byte(), end: n.end_byte(), message: format!("Invalid regular expression: {p}"), edits: Vec::new() });
        }
    }
}

pub struct NoUselessBackreference;

impl NativeRule for NoUselessBackreference {
    fn id(&self) -> &'static str {
        "js/no-useless-backreference"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["regex", "call_expression", "new_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let Some((pattern, flags, base)) = regex_parts(n, ctx.source) else { return };
        let s = scan(&pattern, flags.contains('u') || flags.contains('v'));
        if s.error.is_some() {
            return;
        }
        for r in &s.backrefs {
            let group = match &r.target {
                Ok(num) => s.groups.iter().position(|g| g.number == Some(*num)),
                Err(name) => s.groups.iter().position(|g| g.name.as_deref() == Some(name.as_str())),
            };
            let Some(gi) = group else { continue };
            let g = &s.groups[gi];
            let why = if r.inside.contains(&gi) {
                Some("from inside the group it refers to")
            } else if g.start > r.start {
                Some("before the group it refers to")
            } else if g.path.iter().zip(&r.path).any(|(a, b)| a.0 == b.0 && a.1 != b.1) {
                Some("in a different alternative from the group it refers to")
            } else if g.negative_lookaround && !r.inside.iter().any(|x| s.groups[*x].negative_lookaround) {
                Some("to a group inside a negative lookaround")
            } else {
                None
            };
            if let Some(why) = why {
                let shown = &pattern[r.start..r.end];
                let (start, end) = if n.kind() == "regex" { (base + r.start, base + r.end) } else { (n.start_byte(), n.end_byte()) };
                out.push(NativeFinding { start, end, message: format!("`{shown}` always matches the empty string: it is {why}"), edits: Vec::new() });
            }
        }
    }
}

pub struct NoMisleadingCharacterClass;

fn misleading(c: char) -> bool {
    let u = c as u32;
    matches!(u, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F | 0xFE0E | 0xFE0F | 0x200D | 0x1F3FB..=0x1F3FF)
}

impl NativeRule for NoMisleadingCharacterClass {
    fn id(&self) -> &'static str {
        "js/no-misleading-character-class"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["regex"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let Some((pattern, flags, base)) = regex_parts(n, ctx.source) else { return };
        let unicode = flags.contains('u') || flags.contains('v');
        let s = scan(&pattern, unicode);
        for (start, end) in s.classes {
            let body = &pattern[start..end];
            let chars: Vec<char> = body.chars().collect();
            let joined = chars.iter().enumerate().any(|(i, c)| i > 0 && misleading(*c));
            let regional = chars.windows(2).any(|w| (0x1F1E6..=0x1F1FF).contains(&(w[0] as u32)) && (0x1F1E6..=0x1F1FF).contains(&(w[1] as u32)));
            let surrogate = !unicode && chars.iter().any(|c| (*c as u32) > 0xFFFF);
            if joined || regional || surrogate {
                let why = if surrogate && !joined && !regional {
                    "an astral character is two code units without the `u` flag"
                } else {
                    "it holds a character made of several code points"
                };
                out.push(NativeFinding { start: base + start - 1, end: base + end + 1, message: format!("This character class doesn't match what it shows: {why}"), edits: Vec::new() });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Javascript, src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn invalid_regexps() {
        let src = r#"RegExp("["); new RegExp("a)"); RegExp("*a"); RegExp("a{3,1}"); RegExp("a", "gg"); RegExp("a", "uv"); RegExp("(?x)"); RegExp("ok(a|b)+", "gi"); new RegExp(dynamic);"#;
        assert_eq!(found(&NoInvalidRegexp, src), [
            "Invalid regular expression: a character class `[` is never closed",
            "Invalid regular expression: an unmatched `)`",
            "Invalid regular expression: nothing to repeat before `*`",
            "Invalid regular expression: the quantifier `{3,1}` has its numbers out of order",
            "Invalid regular expression: the flag `g` is given twice",
            "Invalid regular expression: the `u` and `v` flags can't be combined",
            "Invalid regular expression: an invalid group: `(?` must be followed by `:`, `=`, `!` or `<`",
        ]);
    }

    #[test]
    fn useless_backreferences() {
        let src = r"a = /\1(a)/; b = /(a\1)/; c = /(a)|\1b/; d = /(?!(a))\1/; e = /(a)\1/; f = /(?<n>a)\k<n>/; g = /\k<n>(?<n>a)/;";
        let msgs = found(&NoUselessBackreference, src);
        assert_eq!(msgs.len(), 5, "{msgs:?}");
        assert!(msgs[0].contains("before the group"));
        assert!(msgs[1].contains("inside the group"));
        assert!(msgs[2].contains("different alternative"));
        assert!(msgs[3].contains("negative lookaround"));
    }

    #[test]
    fn misleading_classes() {
        let src = "a = /[👍🏻]/u; b = /[🇯🇵]/u; c = /[❤️]/u; d = /[a-z]/; e = /[😀]/; f = /[😀]/u;";
        assert_eq!(found(&NoMisleadingCharacterClass, src).len(), 4);
    }
}
