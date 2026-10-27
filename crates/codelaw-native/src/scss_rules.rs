//! The `scss/*` native rules (W-62), ported from stylelint-scss (MIT). They
//! read the same statement reader as the `css/*` rules, in its SCSS mode
//! (`css_syntax::parse_scss`): `//` comments are blanked, and the Sass
//! at-rules (`@mixin`, `@include`, `@use`, `@if` …) are `Other` statements
//! the rules here look at.
//!
//! Every rule reads the *view* — the source with `//` comments blanked, the
//! same length — and edits its offsets, which are the source's.

use crate::css_syntax::{self, Kind, Sheet, finding, fixed, opt_list, opt_str, scss_line_comments, skip_comment, skip_interpolation, skip_string, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use std::collections::{BTreeMap, BTreeSet};

/// Declare an SCSS native rule: it runs once per stylesheet, on its root.
macro_rules! scss_rule {
    ($(#[$m:meta])* $name:ident, $id:literal, |$sheet:ident, $src:ident, $ctx:ident, $out:ident| $body:block) => {
        $(#[$m])*
        pub struct $name;
        impl NativeRule for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn languages(&self) -> &'static [Language] {
                &[Language::Scss]
            }
            fn kinds(&self) -> &'static [&'static str] {
                // `ERROR` is the root when tree-sitter-scss cannot recover: the
                // rules read the text, so they run once, on the root.
                &["stylesheet", "ERROR"]
            }
            /// `[css] modules` names the CSS Modules, which the SCSS rules read as modules.
            fn config_section(&self) -> Option<&'static str> {
                Some("css")
            }
            #[allow(unused_variables)]
            fn check(&self, node: SyntaxNode<'_>, $ctx: &NativeContext<'_>, $out: &mut Vec<NativeFinding>) {
                if node.parent().is_some() {
                    return;
                }
                let view = css_syntax::scss_view($ctx.source);
                let $src: &str = &view;
                let sheet_rc = css_syntax::sheet_for($ctx.source, true, css_syntax::is_module($ctx));
                let $sheet: &Sheet = &sheet_rc;
                $body
            }
        }
    };
}

/// The lowercased name of a Sass at-rule statement (`@mixin` → `mixin`).
fn sass_name(sheet: &Sheet, i: usize, src: &str) -> Option<String> {
    let n = &sheet.nodes[i];
    (n.kind == Kind::Other && src[n.start..].starts_with('@')).then(|| text(src, n.name).to_ascii_lowercase())
}

/// The Sass statements named `name`, in source order.
fn sass_statements<'a>(sheet: &'a Sheet, src: &'a str, names: &'a [&'a str]) -> impl Iterator<Item = (usize, String)> + 'a {
    sheet.walk().into_iter().filter_map(move |i| sass_name(sheet, i, src).filter(|n| names.contains(&n.as_str())).map(|n| (i, n)))
}

/// `src` with strings, comments and unquoted `url(…)` contents blanked
/// (same length): what the operator and function scans read.
fn masked(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = b.to_vec();
    let blank = |out: &mut Vec<u8>, from: usize, to: usize| {
        for x in &mut out[from..to] {
            if *x != b'\n' {
                *x = b' ';
            }
        }
    };
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => {
                let e = skip_string(b, i);
                blank(&mut out, i + 1, e.saturating_sub(1).max(i + 1));
                i = e;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let e = skip_comment(src, i);
                blank(&mut out, i, e);
                i = e;
            }
            b'u' | b'U' if src[i..].len() >= 4 && src[i..i + 4].eq_ignore_ascii_case("url(") && (i == 0 || !is_word(b[i - 1])) => {
                if src[i + 4..].trim_start().starts_with(['"', '\'']) {
                    i += 4;
                } else {
                    let e = src[i..].find(')').map_or(b.len(), |o| i + o);
                    blank(&mut out, i + 4, e);
                    i = e;
                }
            }
            _ => i += 1,
        }
    }
    String::from_utf8(out).expect("blanking whole ASCII-delimited runs keeps UTF-8")
}

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b >= 0x80
}

/// The ranges of a Sass statement's value text where an expression may
/// appear: declaration values, and the parameters of the control-flow and
/// `@return` statements.
fn expressions(sheet: &Sheet, src: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        match n.kind {
            Kind::Decl if !text(src, n.name).starts_with("--") => out.push(n.params),
            Kind::Other => {
                if let Some(name) = sass_name(sheet, i, src)
                    && matches!(name.as_str(), "return" | "if" | "else" | "while" | "each" | "for" | "debug" | "warn" | "error")
                {
                    out.push(n.params);
                }
            }
            _ => {}
        }
    }
    out
}

/// Where a new `@use "sass:…";` goes: after the leading `@charset`,
/// `@use` and `@forward` statements and comments, or at the start.
fn use_position(sheet: &Sheet, src: &str) -> (usize, bool) {
    let mut at = 0;
    let mut had_use = false;
    for &i in &sheet.top {
        let n = &sheet.nodes[i];
        let name = if n.kind == Kind::Comment {
            String::new()
        } else if n.kind == Kind::AtRule || n.kind == Kind::Other {
            text(src, n.name).to_ascii_lowercase()
        } else {
            break;
        };
        match name.as_str() {
            "" | "charset" | "use" | "forward" => {
                had_use |= name == "use" || name == "forward";
                at = src[n.end..].find('\n').map_or(src.len(), |o| n.end + o + 1);
                // A semicolon after the statement is part of it.
                if let Some(s) = n.semicolon {
                    at = src[s..].find('\n').map_or(src.len(), |o| s + o + 1);
                }
            }
            _ => break,
        }
    }
    (at, had_use)
}

/// The edit that adds `@use "sass:<module>";` — none when the file already
/// loads it (any `as` form counts).
fn use_edit(sheet: &Sheet, src: &str, module: &str) -> Option<TextEdit> {
    let want = format!("sass:{module}");
    for (i, _) in sass_statements(sheet, src, &["use"]) {
        let p = text(src, sheet.nodes[i].params);
        if p.trim_start_matches(['"', '\'']).starts_with(&want) {
            return None;
        }
    }
    let (at, had_use) = use_position(sheet, src);
    let mut s = format!("@use \"sass:{module}\";\n");
    if !had_use && at < src.len() && !src[at..].starts_with('\n') {
        s.push('\n');
    }
    Some(TextEdit::insert(at, s))
}

fn pattern_option(ctx: &NativeContext<'_>) -> regex::Regex {
    let p = opt_str(ctx, "pattern", KEBAB);
    regex::Regex::new(p).unwrap_or_else(|_| regex::Regex::new(KEBAB).expect("the default pattern is valid"))
}

const KEBAB: &str = "^[a-z][a-z0-9]*(-[a-z0-9]+)*$";

// ---------------------------------------------------------------------

scss_rule!(NoImport, "scss/no-import", |sheet, src, ctx, out| {
    for (i, _) in sass_statements(sheet, src, &["import"]) {
        let n = &sheet.nodes[i];
        // A nested `@import` has no `@use` equivalent.
        if n.parent.is_some() {
            continue;
        }
        let params = text(src, n.params);
        let parts = split_top(params);
        if parts.iter().all(|p| plain_css_import(p)) {
            continue;
        }
        let mut edits = Vec::new();
        if parts.len() == 1 && parts[0].trim().starts_with(['"', '\'']) && parts[0].trim().ends_with(['"', '\'']) {
            edits.push(TextEdit::new(n.start, n.name.1, "@use"));
        }
        out.push(fixed(n.start, n.name.1, "`@import` is deprecated in Dart Sass and removed in 3.0; use `@use` or `@forward`", edits));
    }
});

/// An import Sass leaves as a plain CSS `@import`: a URL, a `.css` file or
/// one with a media query.
fn plain_css_import(part: &str) -> bool {
    let p = part.trim();
    let lower = p.to_ascii_lowercase();
    if lower.starts_with("url(") || lower.contains("://") || lower.trim_matches(['"', '\'']).starts_with("//") {
        return true;
    }
    if let Some(q) = p.chars().next().filter(|c| matches!(c, '"' | '\'')) {
        let end = p[1..].find(q).map_or(p.len(), |o| o + 2);
        return lower[..end].trim_matches(['"', '\'']).ends_with(".css") || !p[end..].trim().is_empty();
    }
    false
}

/// Split at top-level commas.
fn split_top(s: &str) -> Vec<&str> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let (mut depth, mut from, mut i) = (0usize, 0, 0);
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_string(b, i);
                continue;
            }
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                out.push(&s[from..i]);
                from = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&s[from..]);
    out
}

scss_rule!(LoadNoPartialLeadingUnderscore, "scss/load-no-partial-leading-underscore", |sheet, src, ctx, out| {
    for (i, name) in sass_statements(sheet, src, &["use", "forward", "import"]) {
        let n = &sheet.nodes[i];
        let params = text(src, n.params);
        let base = n.params.0;
        let b = params.as_bytes();
        let mut j = 0;
        while j < b.len() {
            if b[j] == b'"' || b[j] == b'\'' {
                let e = skip_string(b, j);
                let inner_start = j + 1;
                let inner_end = e.saturating_sub(1).max(inner_start);
                let url = &params[inner_start..inner_end];
                let base_at = url.rfind('/').map_or(0, |o| o + 1);
                if url.as_bytes().get(base_at) == Some(&b'_') && !url.starts_with("sass:") && !url.contains("://") && !url.ends_with(".css") {
                    let at = base + inner_start + base_at;
                    out.push(fixed(at, at + 1, format!("Remove the leading underscore from the partial `{url}`"), vec![TextEdit::delete(at, at + 1)]));
                }
                j = e;
                if name != "import" {
                    break;
                }
                continue;
            }
            j += 1;
        }
    }
});

scss_rule!(AtIfNoNull, "scss/at-if-no-null", |sheet, src, ctx, out| {
    let re = regex::Regex::new(r"(?P<a>\$?[\w.\-]+)\s*(?P<op>==|!=)\s*null\b").expect("valid");
    for (i, name) in sass_statements(sheet, src, &["if", "else"]) {
        let n = &sheet.nodes[i];
        let params = text(src, n.params);
        let (offset, expr) = if name == "else" {
            match params.strip_prefix("if") {
                Some(rest) if rest.starts_with(|c: char| c.is_whitespace() || c == '(') => (params.len() - rest.len(), rest),
                _ => continue,
            }
        } else {
            (0, params)
        };
        let Some(m) = re.captures(expr) else { continue };
        let whole = m.get(0).expect("group 0");
        let (op, a) = (&m["op"], &m["a"]);
        let (s, e) = (n.params.0 + offset + whole.start(), n.params.0 + offset + whole.end());
        // `!= null` becomes the value, `== null` becomes `not` the value; a
        // comparison that is only part of a condition is reported, not fixed.
        let only = expr.trim() == whole.as_str();
        let edits = match (only, op) {
            (true, "!=") => vec![TextEdit::new(s, e, a.to_owned())],
            (true, _) => vec![TextEdit::new(s, e, format!("not {a}"))],
            _ => Vec::new(),
        };
        out.push(fixed(s, e, format!("Don't compare to `null` with `{op}`; test the value itself"), edits));
    }
});

scss_rule!(AtExtendNoMissingPlaceholder, "scss/at-extend-no-missing-placeholder", |sheet, src, ctx, out| {
    for (i, _) in sass_statements(sheet, src, &["extend"]) {
        let n = &sheet.nodes[i];
        let params = text(src, n.params);
        let params = params.strip_suffix("!optional").unwrap_or(params);
        let mut at = n.params.0;
        for part in params.split(',') {
            let t = part.trim();
            if !t.is_empty() && !t.starts_with('%') && !t.contains("#{") {
                let s = at + (part.len() - part.trim_start().len());
                out.push(finding(s, s + t.len(), format!("`@extend {t}` should extend a `%placeholder`")));
            }
            at += part.len() + 1;
        }
    }
});

scss_rule!(DoubleSlashCommentEmpty, "scss/double-slash-comment-empty", |sheet, src, ctx, out| {
    for (s, e) in scss_line_comments(ctx.source) {
        if ctx.source[s + 2..e].trim().is_empty() {
            let line_start = ctx.source[..s].rfind('\n').map_or(0, |o| o + 1);
            let edit = if ctx.source[line_start..s].trim().is_empty() {
                let end = if ctx.source[e..].starts_with('\n') { e + 1 } else { e };
                TextEdit::delete(line_start, end)
            } else {
                let from = ctx.source[..s].trim_end_matches([' ', '\t']).len();
                TextEdit::delete(from, e)
            };
            out.push(fixed(s, e, "Empty `//` comment", vec![edit]));
        }
    }
});

scss_rule!(NoDuplicateDollarVariables, "scss/no-duplicate-dollar-variables", |sheet, src, ctx, out| {
    let mut seen: BTreeMap<(Option<usize>, String), usize> = BTreeMap::new();
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        let name = text(src, n.name);
        if n.kind != Kind::Decl || !name.starts_with('$') || name.contains('.') {
            continue;
        }
        let is_default = n.important.is_some_and(|r| text(src, r).trim_start_matches('!').trim() == "default");
        match seen.get(&(n.parent, name.to_owned())) {
            Some(&first) if !is_default => {
                let line = css_syntax::line(src, sheet.nodes[first].start);
                out.push(finding(n.start, n.name.1, format!("`{name}` is already declared in this scope, on line {line}")));
            }
            Some(_) => {}
            None => {
                seen.insert((n.parent, name.to_owned()), i);
            }
        }
    }
});

scss_rule!(DollarVariablePattern, "scss/dollar-variable-pattern", |sheet, src, ctx, out| {
    let re = pattern_option(ctx);
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        let name = text(src, n.name);
        if n.kind == Kind::Decl && name.starts_with('$') && !name.contains('.') && !name.contains("#{") && !re.is_match(&name[1..]) {
            out.push(finding(n.start, n.name.1, format!("`{name}` should match `{}`", re.as_str())));
        }
    }
});

scss_rule!(AtMixinPattern, "scss/at-mixin-pattern", |sheet, src, ctx, out| {
    let re = pattern_option(ctx);
    for (i, _) in sass_statements(sheet, src, &["mixin"]) {
        let n = &sheet.nodes[i];
        let params = text(src, n.params);
        let end = params.find(|c: char| c == '(' || c.is_whitespace()).unwrap_or(params.len());
        let name = &params[..end];
        if !name.is_empty() && !name.contains("#{") && !re.is_match(name) {
            out.push(finding(n.params.0, n.params.0 + end, format!("The mixin `{name}` should match `{}`", re.as_str())));
        }
    }
});

scss_rule!(PercentPlaceholderPattern, "scss/percent-placeholder-pattern", |sheet, src, ctx, out| {
    let re = pattern_option(ctx);
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        if n.kind != Kind::Rule {
            continue;
        }
        let sel = text(src, n.name);
        let mut from = 0;
        while let Some(o) = sel[from..].find('%') {
            let at = from + o;
            let end = at + 1 + sel[at + 1..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')).unwrap_or(sel.len() - at - 1);
            let name = &sel[at + 1..end];
            if !name.is_empty() && !sel[..at].ends_with("#{") && !re.is_match(name) {
                out.push(finding(n.name.0 + at, n.name.0 + end, format!("The placeholder `%{name}` should match `{}`", re.as_str())));
            }
            from = end.max(at + 1);
        }
    }
});

scss_rule!(NoDuplicateMixins, "scss/no-duplicate-mixins", |sheet, src, ctx, out| {
    let mut seen: BTreeMap<(Option<usize>, String), usize> = BTreeMap::new();
    for (i, _) in sass_statements(sheet, src, &["mixin"]) {
        let n = &sheet.nodes[i];
        let params = text(src, n.params);
        let end = params.find(|c: char| c == '(' || c.is_whitespace()).unwrap_or(params.len());
        let name = params[..end].to_owned();
        match seen.get(&(n.parent, name.clone())) {
            Some(&first) => {
                let line = css_syntax::line(src, sheet.nodes[first].start);
                out.push(finding(n.params.0, n.params.0 + end, format!("The mixin `{name}` is already defined, on line {line}")));
            }
            None => {
                seen.insert((n.parent, name), i);
            }
        }
    }
});

scss_rule!(AtRuleNoUnknown, "scss/at-rule-no-unknown", |sheet, src, ctx, out| {
    let known = css_syntax::set(ctx.data, "scss/at-rules", "known");
    if known.is_empty() {
        return;
    }
    let ignore = opt_list(ctx, "ignoreAtRules");
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        // Sass's own at-rules are `Other` statements: always known.
        if n.kind != Kind::AtRule {
            continue;
        }
        let raw = text(src, n.name);
        if raw.is_empty() || raw.eq_ignore_ascii_case("charset") || known.contains(&sheet.at_name(i, src)) || css_syntax::matches_any(&ignore, raw) {
            continue;
        }
        out.push(finding(n.start, n.name.1, format!("Unknown at-rule `@{raw}`")));
    }
});

scss_rule!(MaxNestingDepth, "scss/max-nesting-depth", |sheet, src, ctx, out| {
    let max = ctx.options.get("max").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        if n.kind != Kind::Rule {
            continue;
        }
        let depth = sheet.ancestors(i).filter(|&a| sheet.nodes[a].kind == Kind::Rule).count();
        if depth > max {
            out.push(finding(n.start, n.name.1, format!("Nesting depth {depth} is deeper than the {max} allowed")));
        }
    }
});

// ---- expressions ----

/// The Sass global functions that moved into a module: global name, module,
/// the name inside it, and whether the move is a plain rename a fix can make.
/// The functions that are also plain CSS (`min`, `max`, `round`, `abs`,
/// `grayscale`, `invert`, `saturate`, `opacity`, `alpha` …) are left out: a
/// call to one may be CSS.
const MOVED: &[(&str, &str, &str, bool)] = &[
    ("map-get", "map", "get", true),
    ("map-merge", "map", "merge", true),
    ("map-remove", "map", "remove", true),
    ("map-keys", "map", "keys", true),
    ("map-values", "map", "values", true),
    ("map-has-key", "map", "has-key", true),
    ("append", "list", "append", true),
    ("index", "list", "index", true),
    ("is-bracketed", "list", "is-bracketed", true),
    ("join", "list", "join", true),
    ("length", "list", "length", true),
    ("list-separator", "list", "separator", true),
    ("nth", "list", "nth", true),
    ("set-nth", "list", "set-nth", true),
    ("zip", "list", "zip", true),
    ("percentage", "math", "percentage", true),
    ("random", "math", "random", true),
    ("unit", "math", "unit", true),
    ("unitless", "math", "is-unitless", true),
    ("comparable", "math", "compatible", true),
    ("quote", "string", "quote", true),
    ("unquote", "string", "unquote", true),
    ("str-index", "string", "index", true),
    ("str-insert", "string", "insert", true),
    ("str-length", "string", "length", true),
    ("str-slice", "string", "slice", true),
    ("to-upper-case", "string", "to-upper-case", true),
    ("to-lower-case", "string", "to-lower-case", true),
    ("unique-id", "string", "unique-id", true),
    ("selector-nest", "selector", "nest", true),
    ("selector-append", "selector", "append", true),
    ("selector-extend", "selector", "extend", true),
    ("selector-replace", "selector", "replace", true),
    ("selector-unify", "selector", "unify", true),
    ("is-superselector", "selector", "is-superselector", true),
    ("simple-selectors", "selector", "simple-selectors", true),
    ("selector-parse", "selector", "parse", true),
    ("feature-exists", "meta", "feature-exists", true),
    ("variable-exists", "meta", "variable-exists", true),
    ("global-variable-exists", "meta", "global-variable-exists", true),
    ("function-exists", "meta", "function-exists", true),
    ("mixin-exists", "meta", "mixin-exists", true),
    ("inspect", "meta", "inspect", true),
    ("type-of", "meta", "type-of", true),
    ("call", "meta", "call", true),
    ("get-function", "meta", "get-function", true),
    ("content-exists", "meta", "content-exists", true),
    ("keywords", "meta", "keywords", true),
    ("adjust-color", "color", "adjust", true),
    ("scale-color", "color", "scale", true),
    ("change-color", "color", "change", true),
    ("mix", "color", "mix", true),
    ("complement", "color", "complement", true),
    ("ie-hex-str", "color", "ie-hex-str", true),
    ("darken", "color", "adjust", false),
    ("lighten", "color", "adjust", false),
    ("desaturate", "color", "adjust", false),
    ("opacify", "color", "adjust", false),
    ("fade-in", "color", "adjust", false),
    ("transparentize", "color", "adjust", false),
    ("fade-out", "color", "adjust", false),
    ("adjust-hue", "color", "adjust", false),
];

/// Every `name(` call in masked expression text: (name start, name end).
fn calls(m: &str, from: usize, to: usize) -> Vec<(usize, usize)> {
    let b = m.as_bytes();
    let mut out = Vec::new();
    let mut i = from;
    while i < to {
        if b[i] == b'#' && b.get(i + 1) == Some(&b'{') {
            // The inside of an interpolation is still an expression.
            i += 2;
            continue;
        }
        if is_word(b[i]) && !b[i].is_ascii_digit() && (i == from || !(is_word(b[i - 1]) || matches!(b[i - 1], b'$' | b'.' | b'@' | b'%'))) {
            let mut j = i;
            while j < to && is_word(b[j]) {
                j += 1;
            }
            if b.get(j) == Some(&b'(') {
                out.push((i, j));
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

fn defined_functions(sheet: &Sheet, src: &str) -> BTreeSet<String> {
    sass_statements(sheet, src, &["function"])
        .map(|(i, _)| {
            let p = text(src, sheet.nodes[i].params);
            p[..p.find(|c: char| c == '(' || c.is_whitespace()).unwrap_or(p.len())].to_ascii_lowercase()
        })
        .collect()
}

scss_rule!(NoGlobalFunctionNames, "scss/no-global-function-names", |sheet, src, ctx, out| {
    let m = masked(src);
    let own = defined_functions(sheet, src);
    let mut first_for: BTreeSet<&str> = BTreeSet::new();
    for (from, to) in expressions_and_arguments(sheet, src) {
        for (s, e) in calls(&m, from, to) {
            let name = &src[s..e];
            let lower = name.to_ascii_lowercase();
            if own.contains(&lower) {
                continue;
            }
            let Some(&(_, module, inner, rename)) = MOVED.iter().find(|(g, ..)| *g == lower) else { continue };
            let mut edits = Vec::new();
            if rename {
                edits.push(TextEdit::new(s, e, format!("{module}.{inner}")));
                if let Some(u) = use_edit(sheet, src, module)
                    && first_for.insert(module)
                {
                    edits.push(u);
                }
            }
            let hint = if rename { format!("`{module}.{inner}` with `@use \"sass:{module}\"`") } else { "`color.adjust` or `color.scale` with `@use \"sass:color\"`".to_owned() };
            out.push(fixed(s, e, format!("The global function `{name}` is deprecated in Dart Sass; use {hint}"), edits));
        }
    }
});

scss_rule!(NoSlashDivision, "scss/no-slash-division", |sheet, src, ctx, out| {
    let m = masked(src);
    let b = m.as_bytes();
    let mut math_needed = BTreeSet::new();
    for (from, to) in expressions(sheet, src) {
        // Parentheses that are not a function call group an expression;
        // calc-like functions and `url()` read `/` as CSS.
        let mut stack: Vec<bool> = Vec::new(); // true: a Sass expression group
        let mut i = from;
        while i < to {
            match b[i] {
                b'#' if b.get(i + 1) == Some(&b'{') => {
                    i = skip_interpolation(b, i).min(to);
                    continue;
                }
                b'(' => {
                    let mut k = i;
                    while k > from && (is_word(b[k - 1]) || b[k - 1] == b'.') {
                        k -= 1;
                    }
                    let callee = &m[k..i];
                    let parent_ok = stack.iter().all(|g| *g);
                    stack.push(parent_ok && (callee.is_empty() || is_sass_call(callee)));
                }
                b')' => {
                    stack.pop();
                }
                b'/' if i > from && b.get(i + 1) != Some(&b'/') && b[i - 1] != b'/' => {
                    let in_expression = stack.last().copied().unwrap_or(false);
                    let (ls, le) = operand_before(&m, from, i);
                    let (rs, re_) = operand_after(&m, i + 1, to);
                    let variable = m[ls..le].contains('$') || m[rs..re_].contains('$');
                    let numeric = |t: &str| t.bytes().next().is_some_and(|c| c.is_ascii_digit() || c == b'.' || c == b'$' || c == b'(' || c == b'-' || c == b'+');
                    let math_context = in_expression && numeric(&m[ls..le]) && numeric(&m[rs..re_]);
                    if (variable && stack.iter().all(|g| *g)) || math_context {
                        let simple = ls < le && rs < re_ && !m[..ls].trim_end().ends_with(['/', '*']) && !m[re_..].trim_start().starts_with(['/', '*']);
                        let mut edits = Vec::new();
                        if simple && (variable || math_context) {
                            edits.push(TextEdit::new(ls, re_, format!("math.div({}, {})", &src[ls..le], &src[rs..re_])));
                            if let Some(u) = use_edit(sheet, src, "math")
                                && math_needed.insert(())
                            {
                                edits.push(u);
                            }
                        }
                        out.push(fixed(i, i + 1, "`/` as division is deprecated in Dart Sass; use `math.div()` with `@use \"sass:math\"`", edits));
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
});

/// Whether `callee` is a call that evaluates its arguments as Sass.
fn is_sass_call(callee: &str) -> bool {
    let c = callee.to_ascii_lowercase();
    !matches!(c.as_str(), "calc" | "clamp" | "min" | "max" | "url" | "var" | "env" | "attr" | "round" | "mod" | "rem" | "abs" | "sign" | "hypot" | "pow" | "sqrt" | "log" | "exp" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" | "image-set" | "format" | "local")
        && !c.ends_with("calc")
}

fn operand_before(m: &str, from: usize, at: usize) -> (usize, usize) {
    let b = m.as_bytes();
    let mut e = at;
    while e > from && b[e - 1].is_ascii_whitespace() {
        e -= 1;
    }
    let mut s = e;
    let mut depth = 0usize;
    while s > from {
        let c = b[s - 1];
        if c == b')' {
            depth += 1;
        } else if c == b'(' {
            if depth == 0 {
                break;
            }
            depth -= 1;
        } else if depth == 0 && !(is_word(c) || matches!(c, b'$' | b'.' | b'%')) {
            break;
        }
        s -= 1;
    }
    (s, e)
}

fn operand_after(m: &str, at: usize, to: usize) -> (usize, usize) {
    let b = m.as_bytes();
    let mut s = at;
    while s < to && b[s].is_ascii_whitespace() {
        s += 1;
    }
    let mut e = s;
    let mut depth = 0usize;
    if e < to && b[e] == b'-' {
        e += 1;
    }
    while e < to {
        let c = b[e];
        if c == b'(' {
            depth += 1;
        } else if c == b')' {
            if depth == 0 {
                break;
            }
            depth -= 1;
        } else if depth == 0 && !(is_word(c) || matches!(c, b'$' | b'.' | b'%')) {
            break;
        }
        e += 1;
    }
    (s, e)
}

scss_rule!(FunctionNoUnknown, "scss/function-no-unknown", |sheet, src, ctx, out| {
    // A file that loads names it cannot see (`@import`, `@use … as *`) may
    // call functions defined there.
    let opaque = sass_statements(sheet, src, &["import", "use", "forward"]).any(|(i, name)| {
        let p = text(src, sheet.nodes[i].params);
        name == "import" || p.trim_end().ends_with("as *")
    });
    if opaque {
        return;
    }
    let mut known = css_syntax::set(ctx.data, "scss/functions", "css");
    known.extend(css_syntax::set(ctx.data, "scss/functions", "sass"));
    if known.is_empty() {
        return;
    }
    known.extend(defined_functions(sheet, src));
    let ignore = opt_list(ctx, "ignoreFunctions");
    let m = masked(src);
    for (from, to) in expressions_and_arguments(sheet, src) {
        for (s, e) in calls(&m, from, to) {
            let name = &src[s..e];
            let lower = name.to_ascii_lowercase();
            let bare = css_syntax::unprefixed(&lower);
            if name.contains('.') || known.contains(&lower) || known.contains(bare) || css_syntax::matches_any(&ignore, name) {
                continue;
            }
            // `ns.fn(` — the scan stops at the dot, so look behind.
            if s > 0 && src.as_bytes()[s - 1] == b'.' {
                continue;
            }
            out.push(finding(s, e, format!("Unknown function `{name}()`")));
        }
    }
});

/// Expression ranges, plus the arguments of `@include` and the defaults of
/// `@mixin` and `@function` parameters (after the name).
fn expressions_and_arguments(sheet: &Sheet, src: &str) -> Vec<(usize, usize)> {
    let mut out = expressions(sheet, src);
    for (i, _) in sass_statements(sheet, src, &["include", "mixin"]) {
        let n = &sheet.nodes[i];
        let p = text(src, n.params);
        let skip = p.find('(').unwrap_or(p.len());
        out.push((n.params.0 + skip, n.params.1));
    }
    out
}
