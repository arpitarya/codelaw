use super::*;

struct M {
    src: String,
    m: SemanticModel,
}

fn model(language: Language, src: &str) -> M {
    let root = codelaw_lang::parse(language, src);
    M { src: src.to_owned(), m: build(language, &root, src).unwrap() }
}

impl M {
    /// Byte offset of the `nth` (0-based) whole-word occurrence of `name`.
    fn at(&self, name: &str, nth: usize) -> usize {
        let is_word = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
        self.src
            .match_indices(name)
            .filter(|(i, _)| {
                let before = self.src[..*i].chars().next_back();
                let after = self.src[i + name.len()..].chars().next();
                !before.is_some_and(is_word) && !after.is_some_and(is_word)
            })
            .nth(nth)
            .unwrap_or_else(|| panic!("no occurrence {nth} of {name}"))
            .0
    }

    fn sym(&self, name: &str, nth: usize) -> SymbolId {
        self.m.symbol_at(self.at(name, nth)).unwrap_or_else(|| panic!("{name}#{nth} names no symbol"))
    }

    fn unused(&self, name: &str) -> bool {
        self.m.is_unused(self.sym(name, 0))
    }

    fn undeclared(&self, name: &str, nth: usize) -> bool {
        let r = self.m.reference_at(self.at(name, nth)).unwrap_or_else(|| panic!("{name}#{nth} is not a reference"));
        self.m.is_undeclared(r, &|_| false)
    }
}

#[test]
fn js_unused_and_used() {
    let t = model(
        Language::Javascript,
        "import { a, b as c } from 'm';\nimport d from 'd';\nconst x = 1, y = a;\nfunction f(p, q) { return q + y; }\nlet z = 0; z = 2;\nexport { f };\nclass K {}\nconst { r, s: [u] } = obj;\nconsole.log(u, d);\n",
    );
    assert!(!t.unused("a"));
    assert!(t.unused("c"), "an unused alias");
    assert!(!t.unused("d"));
    assert!(t.unused("x"));
    assert!(!t.unused("y"));
    assert!(!t.unused("f"), "exported");
    assert!(t.unused("p"));
    assert!(!t.unused("q"));
    assert!(t.unused("z"), "written, never read");
    assert!(t.unused("K"));
    assert!(t.unused("r"));
    assert!(!t.unused("u"));
}

#[test]
fn js_hoisting_and_block_scope() {
    let t = model(Language::Javascript, "g();\nfunction g() { return v; }\nvar v = 1;\n{ let w = 1; }\nw;\nfor (let i = 0; i < 3; i++) {}\ni;\n");
    assert!(!t.undeclared("g", 0));
    assert!(!t.undeclared("v", 0));
    assert!(t.undeclared("w", 1), "let is block-scoped");
    assert!(t.undeclared("i", 3), "the loop's let is the loop's");
    assert!(!t.m.is_reassigned(t.sym("v", 1)));
    assert!(t.m.is_reassigned(t.sym("i", 0)));
}

#[test]
fn js_undeclared_globals_and_properties() {
    let t = model(Language::Javascript, "const o = { k: 1, m() { return this.k; } };\nObject.keys(o);\nfoo.bar;\nlabel: for (;;) { break label; }\nundefinedThing;\n");
    assert!(!t.undeclared("Object", 0));
    assert!(t.undeclared("foo", 0));
    assert!(t.m.reference_at(t.at("bar", 0)).is_none(), "a property is not a reference");
    assert!(t.m.reference_at(t.at("k", 0)).is_none());
    assert!(t.m.reference_at(t.at("label", 1)).is_none());
    assert!(t.undeclared("undefinedThing", 0));
}

#[test]
fn js_shadowing_and_catch() {
    let t = model(Language::Javascript, "const e = 1;\ntry {} catch (e) { e; }\nfunction h(n) { const n2 = (n) => n; return n2; }\n");
    assert!(t.m.shadowed(t.sym("e", 1), false));
    assert!(!t.m.shadowed(t.sym("e", 0), false));
    assert!(t.m.shadowed(t.sym("n", 1), false), "the arrow's param shadows h's");
    assert_eq!(t.m.reference_at(t.at("e", 2)).unwrap().symbol, Some(t.sym("e", 1)));
}

#[test]
fn jsx_components_are_references_intrinsics_are_not() {
    let t = model(Language::Jsx, "import Foo from './foo';\nimport * as UI from './ui';\nconst el = <div><Foo a={x} /><UI.Button /></div>;\n");
    assert!(!t.unused("Foo"));
    assert!(!t.unused("UI"));
    assert!(t.m.reference_at(t.at("div", 0)).is_none());
    assert!(t.undeclared("x", 0));
}

#[test]
fn typescript_types_and_values() {
    let t = model(
        Language::Typescript,
        "import type { T } from './t';\nimport { U, V } from './u';\ninterface I<P> { [key: string]: P; m(arg: T): void }\ntype A = Partial<U>;\nexport const k: I<number> = null!;\nclass C { constructor(private readonly s: string) {} }\nenum E { X = 1 }\nfunction over(a: number): void;\nfunction over(a: any) {}\nnamespace NS { export const z = 1; }\n",
    );
    assert!(!t.unused("T"), "used as a type");
    assert!(!t.unused("U"));
    assert!(t.unused("V"));
    assert!(!t.unused("I"));
    assert!(!t.unused("P"));
    assert!(t.unused("A"));
    assert!(!t.undeclared("Partial", 0));
    assert!(t.m.reference_at(t.at("key", 0)).is_none());
    assert!(!t.unused("s"), "a parameter property is used");
    assert!(t.unused("E"));
    assert_eq!(t.m.symbol(t.sym("over", 0)).decls.len(), 2, "overloads are one symbol");
    assert!(t.unused("NS"));
}

#[test]
fn js_imports_and_exports() {
    let t = model(
        Language::Typescript,
        "import React, { useState as useS } from 'react';\nimport * as path from 'node:path';\nimport './side';\nexport function a() {}\nexport const b = 1, c = 2;\nexport default class {}\nexport { a as alias };\nexport * from './all';\nexport * as ns from './ns';\nexport type { Q } from './q';\ninterface Q {}\n",
    );
    let names: Vec<String> = t.m.exports.iter().map(|e| e.name.clone()).collect();
    assert_eq!(names, ["a", "b", "c", "default", "alias", "*", "ns", "Q"]);
    assert_eq!(t.m.exports[4].local, Some(t.sym("a", 0)));
    assert!(t.m.imported_from(t.sym("useS", 0), "react", Some("useState")));
    assert!(t.m.imported_from(t.sym("React", 0), "react", Some("default")));
    assert!(!t.m.imported_from(t.sym("React", 0), "preact", None));
    assert!(t.m.imported_from(t.sym("path", 0), "node:path", None));
    assert!(t.m.imports.iter().any(|i| i.source == "./side" && i.imported == Imported::SideEffect));
}

#[test]
fn js_assignment_targets() {
    let t = model(Language::Javascript, "let a, b, c;\n[a, { k: b }] = arr;\nc += 1;\nfor (a of xs) {}\n");
    assert!(t.m.is_reassigned(t.sym("a", 0)));
    assert!(t.m.is_reassigned(t.sym("b", 0)));
    assert!(t.unused("b"), "destructured into, never read");
    assert!(!t.unused("c"), "a compound assignment reads");
}

#[test]
fn python_unused_imports_and_locals() {
    let t = model(
        Language::Python,
        "import os\nimport sys as system\nfrom a import b, c as c\nfrom __future__ import annotations\n\ndef f(x, y=1):\n    z = 2\n    w = 3\n    return x + w\n\nTOP = os.sep\n",
    );
    assert!(!t.unused("os"));
    assert!(t.unused("system"));
    assert!(t.unused("b"));
    assert!(!t.m.is_unused(t.sym("c", 1)), "a redundant alias re-exports");
    assert!(!t.unused("annotations"));
    assert!(t.unused("z"));
    assert!(!t.unused("w"));
    assert!(t.unused("y"));
    assert!(!t.unused("TOP"), "module-level names are the module's surface");
}

#[test]
fn python_all_marks_imports_used() {
    let t = model(Language::Python, "from .x import a, b\n__all__ = ['a', 'z']\n");
    assert!(!t.unused("a"));
    assert!(t.unused("b"));
    assert_eq!(t.m.exported_names(), ["a".to_string(), "z".to_string()].into());
}

#[test]
fn python_legb_class_scope_and_comprehensions() {
    let t = model(
        Language::Python,
        "class A:\n    k = 1\n    def m(self):\n        return k\n    j = [k for _ in range(3)]\n    n = k\n\ndef outer():\n    v = 1\n    def inner():\n        return v + len([])\n    return inner\n\nsquares = [q * q for q in range(10)]\nq\n",
    );
    assert!(t.undeclared("k", 1), "a method cannot see class names");
    assert!(t.undeclared("k", 2), "nor can a comprehension's body");
    assert!(!t.undeclared("k", 3), "the class body can");
    assert!(!t.undeclared("v", 1));
    assert!(!t.undeclared("len", 0));
    assert!(t.undeclared("q", 3), "the comprehension variable does not leak");
}

#[test]
fn python_global_nonlocal_walrus() {
    let t = model(
        Language::Python,
        "count = 0\n\ndef bump():\n    global count\n    count += 1\n\ndef make():\n    total = 0\n    def add():\n        nonlocal total\n        total = 5\n    return add\n\nif (n := 10) > 5:\n    print(n)\n",
    );
    assert!(t.m.is_reassigned(t.sym("count", 0)));
    assert!(t.m.is_reassigned(t.sym("total", 0)));
    assert!(!t.undeclared("n", 1));
    assert_eq!(t.m.symbol(t.sym("n", 0)).scope, 0);
}

#[test]
fn python_patterns_with_except_lambda() {
    let t = model(
        Language::Python,
        "try:\n    pass\nexcept (ValueError, KeyError) as err:\n    print(err)\ndef g():\n    with open('f') as (fh, other):\n        return fh\nfn = lambda a, *rest, **kw: a\nmatch cmd:\n    case Point(x=px, y=[py, *more]) | {\"k\": dv} as whole:\n        print(px, py, more, dv, whole)\n    case Color.RED:\n        pass\nf'{fh!r} {undefined_name:>10}'\n",
    );
    assert!(!t.undeclared("err", 1));
    assert!(t.unused("other"));
    assert!(t.unused("rest"));
    assert!(!t.undeclared("px", 1));
    assert!(!t.undeclared("whole", 1));
    assert!(t.undeclared("Point", 0));
    assert!(t.undeclared("Color", 0));
    assert!(t.undeclared("cmd", 0));
    assert!(t.undeclared("undefined_name", 0), "f-string interpolations are references");
    assert!(t.undeclared("fh", 2), "g's fh is g's");
}

#[test]
fn python_star_import_silences_undeclared() {
    let t = model(Language::Python, "from os.path import *\njoin('a')\n");
    assert!(!t.undeclared("join", 0));
}

#[test]
fn python_shadowing_builtins() {
    let t = model(Language::Python, "def f(list):\n    id = 1\n    return id\n");
    assert!(!t.m.shadowed(t.sym("list", 0), false));
    assert!(t.m.shadowed(t.sym("list", 0), true));
}

#[test]
fn python_format_spec_fields_are_read() {
    let t = model(Language::Python, "def f(x):\n    width = 3\n    return f\"{x:{width}d}\"\n");
    assert!(!t.unused("width"));
}

#[test]
fn python_generic_subscripts_are_references() {
    let t = model(
        Language::Python,
        "from typing import Any, List, Optional\n\ndef main(args: Optional[List[str]] = None) -> List[Any]:\n    return []\n\ndef f[T: int](x: T) -> T:\n    return x\n",
    );
    assert!(!t.unused("List"));
    assert!(!t.unused("Any"));
    assert!(!t.unused("Optional"));
    assert!(t.m.symbols.iter().all(|s| s.name != "str"), "`str` in `List[str]` is a reference, not a declaration");
    assert!(t.m.symbols.iter().any(|s| s.name == "T"), "a PEP 695 type parameter is declared");
}

#[test]
fn python_pep695_type_params_scope() {
    // W-17 row 23: annotations, return types, bases and methods see the
    // parameters; defaults and the module do not.
    let t = model(
        Language::Python,
        "def first[T](xs: list[T], d=T) -> T:\n    y: T = xs[0]\n    return y\n\nclass Box[U](Base[U]):\n    def get(self) -> U:\n        return self.u\n\nx: T\n",
    );
    assert!(!t.undeclared("T", 1), "a parameter annotation");
    assert!(t.undeclared("T", 2), "a default is evaluated outside");
    assert!(!t.undeclared("T", 3), "the return type");
    assert!(!t.undeclared("T", 4), "the body");
    assert!(t.undeclared("T", 5), "the module");
    assert!(!t.undeclared("U", 1), "a base");
    assert!(!t.undeclared("U", 2), "a method");
}

#[test]
fn python_generic_type_alias_binds() {
    // W-17 row 28: `type Alias[T] = ...` binds `Alias`; `T` is the alias's own.
    let t = model(Language::Python, "type Alias[T] = list[T]\nx: Alias[int]\ny: T\n");
    assert!(!t.undeclared("Alias", 1));
    assert!(!t.undeclared("T", 1));
    assert!(t.undeclared("T", 2));
}

#[test]
fn python_dynamic_all() {
    // W-17 row 24: a non-literal part makes the export list unknowable.
    let t = model(Language::Python, "from . import sub\nfrom .x import a, b\n__all__ = ['sub'] + sub.__all__\n");
    assert!(!t.unused("a"), "may be re-exported");
    assert!(!t.unused("b"));
    let t = model(Language::Python, "from .x import a, b, c\n__all__ = ['a']\n__all__.extend(['b'])\n__all__ += ('c',)\n");
    assert!(!t.unused("a"));
    assert!(!t.unused("b"), "extended");
    assert!(!t.unused("c"), "added");
    let t = model(Language::Python, "from .x import a, b\n__all__ = []\n__all__.append('a')\n");
    assert!(!t.unused("a"), "appended");
    assert!(t.unused("b"), "a literal list stays exact");
}

#[test]
fn typescript_merged_export_and_enums() {
    // W-17 row 25: `export { Color }` exports the value and the type.
    let t = model(Language::Typescript, "const Color = 1;\ntype Color = number;\nexport { Color };\n");
    let ids = &t.m.scopes[0].bindings["Color"];
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().all(|&id| !t.m.is_unused(id)), "both namespaces are exported");
    // W-17 row 33: members are declared; `NS.T` reads `NS` as a type.
    let t = model(Language::Typescript, "enum E { A = 1, B = A << 1, C }\nlet v: NS.T;\nlet w: E.A = E.B;\n");
    assert!(!t.undeclared("A", 1), "a member in an initializer");
    assert!(t.m.reference_at(t.at("NS", 0)).is_some_and(|r| r.type_only));
    assert!(!t.unused("E"));
}
