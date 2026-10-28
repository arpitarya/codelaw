//! The `css-modules/*` native rules that read one stylesheet (W-63): which
//! classes a CSS Module defines, whether their names reach JavaScript without
//! brackets, whether `composes` points at something, and `:global`.
//!
//! The two rules that join stylesheets with the JS and TS that import them,
//! `css-modules/no-unused-class` and `css-modules/no-undefined-class`, are
//! project-graph queries (`codelaw-graph`).
//!
//! A file is a module when it is named `*.module.css` or `*.module.scss`, or
//! matches `[css] modules`; the rules say nothing about any other stylesheet.

use crate::css_syntax::{Complex, Kind, SKind, Sheet, finding, opt_str, selector_list, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use std::collections::BTreeSet;

/// A class a module defines, and where.
struct Local {
    name: String,
    start: usize,
    end: usize,
}

/// The local classes of a stylesheet. `incomplete` when a name could not be
/// worked out (`#{}` interpolation, a Sass `&-suffix` on a selector that is not
/// one class).
fn local_classes(sheet: &Sheet, src: &str) -> (Vec<Local>, bool) {
    let mut out = Vec::new();
    let mut incomplete = false;
    // The one class each rule's selector names, for `&-suffix` in a rule nested in it.
    let mut single: std::collections::BTreeMap<usize, String> = std::collections::BTreeMap::new();
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        if n.kind != Kind::Rule || sheet.in_keyframes(i, src) {
            continue;
        }
        if text(src, n.name).contains("#{") {
            incomplete = true;
            continue;
        }
        let parent = sheet.ancestors(i).find(|&a| sheet.nodes[a].kind == Kind::Rule);
        let parent_class = parent.and_then(|p| single.get(&p).cloned());
        let list = sheet.selectors(i, src);
        let mut named = Vec::new();
        for c in list.iter() {
            classes_of(src, c, &mut out, &mut named, parent_class.clone(), &mut incomplete);
        }
        if list.len() == 1 && named.len() == 1 {
            single.insert(i, named.remove(0));
        }
    }
    (out, incomplete)
}

fn classes_of(src: &str, c: &Complex, out: &mut Vec<Local>, named: &mut Vec<String>, parent: Option<String>, incomplete: &mut bool) {
    let mut global = false;
    for compound in &c.compounds {
        for s in &compound.parts {
            match s.kind {
                SKind::Class if !global => {
                    let name: String = text(src, s.name).chars().filter(|c| *c != '\\').collect();
                    named.push(name.clone());
                    out.push(Local { name, start: s.name.0, end: s.name.1 });
                }
                SKind::Nesting => {
                    let raw = text(src, (s.start, s.end));
                    if raw.len() > 1 {
                        match &parent {
                            Some(p) => {
                                let name = format!("{p}{}", &raw[1..]);
                                named.push(name.clone());
                                out.push(Local { name, start: s.start, end: s.end });
                            }
                            None => *incomplete = true,
                        }
                    }
                }
                SKind::PseudoClass => {
                    let name = text(src, s.name).to_ascii_lowercase();
                    match (name.as_str(), s.args) {
                        ("global", None) => global = true,
                        ("local", None) => global = false,
                        ("local", Some((a, b))) | ("not" | "is" | "where" | "has", Some((a, b))) if !global => {
                            for inner in selector_list(src, a, b) {
                                classes_of(src, &inner, out, &mut Vec::new(), None, incomplete);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

macro_rules! module_rule {
    ($(#[$m:meta])* $name:ident, $id:literal, |$sheet:ident, $src:ident, $ctx:ident, $out:ident| $body:block) => {
        $(#[$m])*
        pub struct $name;
        impl NativeRule for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn languages(&self) -> &'static [Language] {
                &[Language::Css, Language::Scss]
            }
            fn kinds(&self) -> &'static [&'static str] {
                &["stylesheet", "ERROR"]
            }
            /// `[css]`: `modules` names the module stylesheets, `modules_convention` how
            /// the bundler maps class names.
            fn config_section(&self) -> Option<&'static str> {
                Some("css")
            }
            #[allow(unused_variables)]
            fn check(&self, node: SyntaxNode<'_>, $ctx: &NativeContext<'_>, $out: &mut Vec<NativeFinding>) {
                if node.parent().is_some() || !crate::css_syntax::is_module($ctx) {
                    return;
                }
                let scss = $ctx.language == Language::Scss;
                let view = if scss { crate::css_syntax::scss_view($ctx.source) } else { $ctx.source.to_owned() };
                let $src: &str = &view;
                let sheet_rc = crate::css_syntax::sheet_for(Some(node.get_inner_node()), $ctx.source, scss, true);
                let $sheet: &Sheet = &sheet_rc;
                $body
            }
        }
    };
}

module_rule!(NoGlobal, "css-modules/no-global", |sheet, src, ctx, out| {
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        if n.kind != Kind::Rule {
            continue;
        }
        let sel = text(src, n.name);
        let mut from = 0;
        while let Some(o) = sel[from..].find(":global") {
            let at = from + o;
            let end = at + ":global".len();
            // `:globalfoo` is another name.
            if sel[end..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                from = end;
                continue;
            }
            out.push(finding(n.name.0 + at, n.name.0 + end, "`:global` leaves the module's scope; keep the selector local"));
            from = end;
        }
    }
});

/// A valid JavaScript identifier of the names `styles.x` can reach.
fn identifier(name: &str) -> bool {
    let mut it = name.chars();
    it.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$') && it.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

module_rule!(ClassNaming, "css-modules/class-naming", |sheet, src, ctx, out| {
    let style = opt_str(ctx, "style", "camelCase");
    if style == "any" {
        return;
    }
    let convention = opt_str(ctx, "modules_convention", "asIs");
    let (classes, _) = local_classes(sheet, src);
    let kebab = |n: &str| {
        let mut parts = n.split('-');
        parts.next().is_some_and(|p| p.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && p.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()))
            && parts.all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()))
    };
    let mut seen = BTreeSet::new();
    for c in classes {
        if !seen.insert((c.start, c.end)) {
            continue;
        }
        let ok = match style {
            "kebab" => kebab(&c.name),
            // `camelCase`: reachable as `styles.x`, with the bundler's convention counted.
            _ => identifier(&c.name) || (convention != "asIs" && js_names(&c.name, convention).iter().any(|n| identifier(n))),
        };
        if !ok {
            let why = if style == "kebab" {
                "it is not kebab-case".to_owned()
            } else {
                format!("`styles.{}` is not valid JavaScript, so it needs brackets", c.name)
            };
            out.push(finding(c.start, c.end, format!("The class `{}`: {why}", c.name)));
        }
    }
});

/// The names JavaScript reaches a class by, under css-loader's `localsConvention`.
fn js_names(class: &str, convention: &str) -> Vec<String> {
    let camel = |dashes_only: bool| -> String {
        let mut out = String::new();
        let mut up = false;
        for c in class.chars() {
            if c == '-' || (!dashes_only && c == '_') {
                up = true;
            } else if up {
                out.extend(c.to_uppercase());
                up = false;
            } else {
                out.push(c);
            }
        }
        out
    };
    match convention {
        "camelCase" => vec![class.to_owned(), camel(false)],
        "camelCaseOnly" => vec![camel(false)],
        "dashes" => vec![class.to_owned(), camel(true)],
        "dashesOnly" => vec![camel(true)],
        _ => vec![class.to_owned()],
    }
}

module_rule!(ComposesValid, "css-modules/composes-valid", |sheet, src, ctx, out| {
    let (here, here_incomplete) = local_classes(sheet, src);
    let here: BTreeSet<&str> = here.iter().map(|c| c.name.as_str()).collect();
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        if n.kind != Kind::Other || !text(src, n.name).eq_ignore_ascii_case("composes") || src[n.start..].starts_with('@') {
            continue;
        }
        // Where it stands.
        let stands = match n.parent {
            None => Some("it is outside a rule".to_owned()),
            Some(p) if sheet.nodes[p].kind != Kind::Rule => Some("it is inside an at-rule".to_owned()),
            Some(p) if sheet.nodes[p].parent.is_some() => Some("it is inside a nested rule".to_owned()),
            Some(p) => {
                let list = sheet.selectors(p, src);
                let single = list.len() == 1 && list[0].compounds.len() == 1 && list[0].compounds[0].parts.len() == 1 && list[0].compounds[0].parts[0].kind == SKind::Class;
                (!single).then(|| "its selector is not a single class".to_owned())
            }
        };
        if let Some(why) = stands {
            out.push(finding(n.start, n.end, format!("`composes` works only in a rule whose selector is one top-level class: {why}")));
            continue;
        }
        // What it names.
        let value = text(src, n.params);
        let (names, from) = match value.rfind(" from ") {
            Some(at) => (&value[..at], Some(value[at + 6..].trim())),
            None => (value, None),
        };
        let names: Vec<&str> = names.split_whitespace().collect();
        match from {
            None => {
                if !here_incomplete {
                    for name in names.iter().filter(|nm| !here.contains(*nm)) {
                        out.push(finding(n.params.0, n.params.1, format!("`composes: {name}`: this file defines no `.{name}`")));
                    }
                }
            }
            Some("global") => {}
            Some(target) => {
                let link = target.trim_matches(['"', '\'']);
                // A package or alias path is outside what the project holds.
                if !(link.starts_with("./") || link.starts_with("../") || link.starts_with('/')) {
                    continue;
                }
                let Some(path) = crate::resolve_path(ctx.file.path(), link) else {
                    out.push(finding(n.params.0, n.params.1, format!("`composes … from \"{link}\"` leaves the project")));
                    continue;
                };
                let Some(other) = ctx.file.read(&path) else {
                    out.push(finding(n.params.0, n.params.1, format!("`composes … from \"{link}\"`: no file `{path}`")));
                    continue;
                };
                let other_scss = path.to_ascii_lowercase().ends_with(".scss");
                let other_view = if other_scss { crate::css_syntax::scss_view(&other) } else { other.clone() };
                let other_tree = codelaw_lang::parse(if other_scss { Language::Scss } else { Language::Css }, &other);
                let other_sheet = crate::css_tree::sheet(other_tree.root().get_inner_node(), &other_view, other_scss, true)
                    .unwrap_or_else(|| crate::css_syntax::parse_with(&other, other_scss, true));
                let (classes, incomplete) = local_classes(&other_sheet, &other_view);
                if incomplete {
                    continue;
                }
                let defined: BTreeSet<&str> = classes.iter().map(|c| c.name.as_str()).collect();
                for name in names.iter().filter(|nm| !defined.contains(*nm)) {
                    out.push(finding(n.params.0, n.params.1, format!("`composes: {name} from \"{link}\"`: `{path}` defines no `.{name}`")));
                }
            }
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DetachedFile, NativeContext};
    use std::collections::BTreeMap;

    fn run(rule: &dyn NativeRule, path: &str, src: &str, options: &[(&str, serde_json::Value)]) -> Vec<String> {
        let language = if path.ends_with("scss") { Language::Scss } else { Language::Css };
        let root = codelaw_lang::parse(language, src);
        let options: BTreeMap<String, serde_json::Value> = options.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
        let data = BTreeMap::new();
        let file = DetachedFile::new(path, src);
        let ctx = NativeContext { source: src, language, semantic: None, options: &options, data: &data, file: &file };
        let mut out = Vec::new();
        for node in root.root().dfs() {
            if rule.kinds().contains(&node.kind().as_ref()) {
                rule.check(node, &ctx, &mut out);
            }
        }
        out.into_iter().map(|f| format!("{}: {}", &src[f.start..f.end], f.message)).collect()
    }

    #[test]
    fn only_module_files_are_judged() {
        assert!(run(&NoGlobal, "a.css", ":global(.x) { a: b }", &[]).is_empty());
        assert_eq!(run(&NoGlobal, "a.module.css", ":global(.x) .y { a: b }\n.z :global .w { }", &[]).len(), 2);
        let globs = serde_json::json!(["src/**/*.styles.css"]);
        assert_eq!(run(&NoGlobal, "src/a/b.styles.css", ":global .x { }", &[("modules", globs)]).len(), 1);
    }

    #[test]
    fn class_naming() {
        let src = ".btn-primary { a: b }\n.camelOk { }\n:global(.not-checked) { }\n.snake_ok { }\n";
        let got = run(&ClassNaming, "a.module.css", src, &[]);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got[0].starts_with("btn-primary:"));
        // The bundler's convention makes `styles.btnPrimary` reachable.
        assert!(run(&ClassNaming, "a.module.css", src, &[("modules_convention", serde_json::json!("camelCase"))]).is_empty());
        let kebab = run(&ClassNaming, "a.module.css", src, &[("style", serde_json::json!("kebab"))]);
        assert_eq!(kebab.len(), 2, "{kebab:?}");
        assert!(run(&ClassNaming, "a.module.css", src, &[("style", serde_json::json!("any"))]).is_empty());
    }

    #[test]
    fn composes_in_the_wrong_place_and_missing_targets() {
        let src = ".a { composes: b; }\n.b { }\n.c, .d { composes: a; }\n.e { .f { composes: a; } }\n.g { composes: missing; }\n.h { composes: x from \"./nope.module.css\"; composes: y from global; composes: z from \"lib/z.css\"; }\n";
        let got = run(&ComposesValid, "src/a.module.css", src, &[]);
        assert_eq!(got.len(), 4, "{got:?}");
        assert!(got.iter().any(|g| g.contains("not a single class")));
        assert!(got.iter().any(|g| g.contains("nested rule")));
        assert!(got.iter().any(|g| g.contains("defines no `.missing`")));
        assert!(got.iter().any(|g| g.contains("no file `src/nope.module.css`")));
    }
}
