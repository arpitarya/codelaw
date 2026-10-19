//! `js/no-unused-vars` and `ts/no-unused-vars`: declarations nothing reads,
//! with ESLint's options — `vars`, `args` (`after-used` by default),
//! `caughtErrors`, `ignoreRestSiblings` and the ignore patterns. Parameter
//! order is what a pattern can't see: only the parameters after the last
//! used one are reported.

use crate::util::named_children;
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use codelaw_semantic::{ScopeKind, SymbolKind};
use std::collections::BTreeMap;

pub struct NoUnusedVars {
    pub id: &'static str,
    pub languages: &'static [Language],
}

fn opt_str<'a>(ctx: &'a NativeContext<'_>, key: &str, default: &'a str) -> &'a str {
    ctx.options.get(key).and_then(|v| v.as_str()).unwrap_or(default)
}

fn opt_regex(ctx: &NativeContext<'_>, key: &str) -> Option<regex::Regex> {
    ctx.options.get(key).and_then(|v| v.as_str()).and_then(|p| regex::Regex::new(p).ok())
}

/// Parameters of a signature without a body are not runtime bindings.
const SIGNATURES: &[&str] =
    &["function_type", "constructor_type", "method_signature", "call_signature", "construct_signature", "function_signature", "abstract_method_signature"];

impl NativeRule for NoUnusedVars {
    fn id(&self) -> &'static str {
        self.id
    }
    fn languages(&self) -> &'static [Language] {
        self.languages
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let root = node.get_inner_node();
        let vars = opt_str(ctx, "vars", "all");
        let args = opt_str(ctx, "args", "after-used");
        let caught = opt_str(ctx, "caughtErrors", "all");
        let rest_siblings = ctx.options.get("ignoreRestSiblings").and_then(|v| v.as_bool()).unwrap_or(false);
        let vars_ignore = opt_regex(ctx, "varsIgnorePattern");
        let args_ignore = opt_regex(ctx, "argsIgnorePattern");
        let caught_ignore = opt_regex(ctx, "caughtErrorsIgnorePattern");
        let array_ignore = opt_regex(ctx, "destructuredArrayIgnorePattern");
        let ignored = |re: &Option<regex::Regex>, name: &str| re.as_ref().is_some_and(|r| r.is_match(name));

        let mut params: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
        for (id, s) in model.symbols.iter().enumerate() {
            let Some(&(start, end)) = s.decls.first() else { continue };
            let Some(ident) = root.descendant_for_byte_range(start, end) else { continue };
            if ident.start_byte() != start {
                continue;
            }
            let parent = ident.parent();
            let parent_kind = parent.map(|p| p.kind()).unwrap_or_default();
            let ambient = {
                let mut a = ident.parent();
                let mut found = false;
                while let Some(n) = a {
                    if n.kind() == "ambient_declaration" {
                        found = true;
                        break;
                    }
                    a = n.parent();
                }
                found
            };
            if ambient {
                continue;
            }
            match s.kind {
                SymbolKind::Param => {
                    let scope = &model.scopes[s.scope];
                    let owner = root.descendant_for_byte_range(scope.start, scope.end).map(|n| n.kind()).unwrap_or_default();
                    if SIGNATURES.contains(&owner) {
                        continue;
                    }
                    params.entry(s.scope).or_default().push((start, id));
                    continue;
                }
                SymbolKind::TypeParam => continue,
                // A function or class expression's own name.
                SymbolKind::Function | SymbolKind::Class if matches!(parent_kind, "function_expression" | "function" | "generator_function" | "class") => continue,
                _ => {}
            }
            if !model.is_unused(id) {
                continue;
            }
            if s.kind == SymbolKind::CatchParam {
                if caught == "none" || ignored(&caught_ignore, &s.name) {
                    continue;
                }
            } else {
                if vars == "local" && model.scopes[s.scope].kind == ScopeKind::Module {
                    continue;
                }
                if ignored(&vars_ignore, &s.name) {
                    continue;
                }
                if parent_kind == "array_pattern" && ignored(&array_ignore, &s.name) {
                    continue;
                }
                if rest_siblings
                    && let Some(p) = parent
                    && p.kind() == "object_pattern"
                    && named_children(p).iter().any(|c| c.kind() == "rest_pattern")
                {
                    continue;
                }
            }
            let assigned = s.refs.iter().any(|&r| model.references[r].write)
                || parent.is_some_and(|p| p.kind() == "variable_declarator" && p.child_by_field_name("value").is_some());
            let what = if assigned { "is assigned a value but never used" } else { "is defined but never used" };
            out.push(NativeFinding { start, end, message: format!("`{}` {what}", s.name), edits: Vec::new() });
        }
        if args == "none" {
            return;
        }
        for (_, mut list) in params {
            list.sort();
            let last_used = list.iter().rposition(|(_, id)| !model.is_unused(*id));
            for (i, (start, id)) in list.iter().enumerate() {
                if !model.is_unused(*id) {
                    continue;
                }
                if args == "after-used" && last_used.is_some_and(|u| i < u) {
                    continue;
                }
                let s = model.symbol(*id);
                if ignored(&args_ignore, &s.name) {
                    continue;
                }
                out.push(NativeFinding { start: *start, end: s.decls[0].1, message: format!("`{}` is defined but never used", s.name), edits: Vec::new() });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(language: Language, src: &str) -> Vec<String> {
        let rule = NoUnusedVars { id: "js/no-unused-vars", languages: &[Language::Javascript, Language::Typescript] };
        crate::run(&rule, language, src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn vars_args_and_catch() {
        let src = "import { a, b } from 'm';\nconst c = 1, d = a;\nfunction f(x, y, z) { return y; }\nf();\nlet e; e = 2;\ntry {} catch (err) {}\nexport { d };\n";
        assert_eq!(found(Language::Javascript, src), [
            "`b` is defined but never used",
            "`c` is assigned a value but never used",
            "`e` is assigned a value but never used",
            "`err` is defined but never used",
            "`z` is defined but never used",
        ]);
    }

    #[test]
    fn expressions_and_signatures_are_left_alone() {
        let src = "export const g = function named() {};\nexport type F = (unusedParam: number) => void;\ninterface Unused {}\nexport class K { constructor(private s: string) {} m(a: number): void; m(a: any) { return a; } }\ndeclare const amb: number;\n";
        assert_eq!(found(Language::Typescript, src), ["`Unused` is defined but never used"]);
    }
}
