use crate::*;
use codelaw_core::Language;
use std::path::Path;

fn graph(files: &[(&str, &str)], config: GraphConfig) -> Graph {
    let mut inputs = Vec::new();
    let mut manifests = Vec::new();
    for (p, t) in files {
        let name = p.rsplit('/').next().unwrap();
        if is_manifest(name) {
            manifests.push(Manifest { path: p.to_string(), text: t.to_string() });
        }
        let facts = Language::from_path(Path::new(p)).and_then(|l| extract(l, t));
        inputs.push(GraphInput { path: p.to_string(), facts });
    }
    Graph::build(inputs, &manifests, config)
}

fn unused_rule() -> GraphRule {
    let v: serde_json::Value = serde_json::json!({
        "symbol": {"exported": true, "kind": ["function", "class", "variable", "type", "interface", "enum"]},
        "unless": [{"reachable_from": "entry"}, {"tagged": ["public", "keep"]}],
        "when": {"external_references": 0},
        "certainty_from": "module.dynamic",
    });
    let fix = serde_json::json!([
        {"when": {"local_references": ">0"}, "do": ["strip-export-keyword"]},
        {"else": null, "do": ["remove-declaration"]},
    ]);
    compile(&v, Some(&fix)).unwrap()
}

const ALL: &[Language] = &[Language::Javascript, Language::Typescript, Language::Tsx, Language::Jsx, Language::Python];

fn unused(files: &[(&str, &str)]) -> Vec<String> {
    let g = graph(files, GraphConfig::default());
    run(&unused_rule(), &g, ALL).into_iter().map(|f| format!("{}#{}", f.file, f.captures[0].1)).collect()
}

#[test]
fn the_record_example() {
    let files = [
        ("packages/utils/package.json", r#"{"name":"@acme/utils","private":true,"main":"src/index.ts"}"#),
        ("packages/utils/src/index.ts", "export * from './date';\n"),
        ("packages/utils/src/date.ts", "export function formatDate(d: Date) { return d.toISOString(); }\nexport function parseDate(s: string) { return new Date(s); }\n"),
        ("packages/web/package.json", r#"{"name":"web","private":true}"#),
        ("packages/web/src/app.ts", "import { formatDate } from \"@acme/utils\";\nconsole.log(formatDate(new Date()));\n"),
    ];
    // A private package's `main` is reachable, but `parseDate` has no importer.
    assert_eq!(unused(&files), vec!["packages/utils/src/date.ts#parseDate"]);
    let mut public = files;
    public[0].1 = r#"{"name":"@acme/utils","exports":{".":"./dist/index.js"}}"#;
    // Not private: its whole public surface counts as used.
    assert!(unused(&public).is_empty());
}

#[test]
fn unused_export_through_a_barrel() {
    let files = [
        ("src/main.ts", "import { a } from './lib';\nimport * as ns from './ns';\nconsole.log(a, ns.used);\n"),
        ("src/lib/index.ts", "export { a, b } from './impl';\n"),
        ("src/lib/impl.ts", "export const a = 1;\nexport const b = 2;\nexport function c() {}\nfunction local() { return c; }\n"),
        ("src/ns.ts", "export const used = 1;\nexport const unused = 2;\n"),
    ];
    let got = unused(&files);
    assert_eq!(got, vec!["src/lib/impl.ts#b", "src/lib/impl.ts#c", "src/ns.ts#unused"]);
}

#[test]
fn fixes_strip_or_remove() {
    let files = [("src/a.ts", "import { x } from './b';\nx();\n"), ("src/b.ts", "export function x() {}\nexport function y() {}\nexport const z = 1;\nconsole.log(z);\n")];
    let g = graph(&files, GraphConfig::default());
    let f = run(&unused_rule(), &g, ALL);
    assert_eq!(f.len(), 2);
    let text = files[1].1;
    let apply = |e: &codelaw_core::TextEdit| format!("{}{}{}", &text[..e.start], e.text, &text[e.end..]);
    assert_eq!(apply(&f[0].edits[0]), "export function x() {}\nexport const z = 1;\nconsole.log(z);\n");
    assert_eq!(apply(&f[1].edits[0]), "export function x() {}\nexport function y() {}\nconst z = 1;\nconsole.log(z);\n");
}

#[test]
fn keep_tags_registrars_and_dynamic() {
    let files = [
        ("app/main.py", "from app import views\nfrom app.plugins import loader\nviews.index()\nloader.load()\n"),
        ("app/__init__.py", ""),
        (
            "app/views.py",
            "__all__ = ['index', 'about', 'kept', 'route']\n\ndef index(): pass\n\ndef about(): pass\n\ndef kept():  # codelaw: keep\n    pass\n\n@app.get('/x')\ndef route(): pass\n",
        ),
        ("app/plugins/__init__.py", ""),
        ("app/plugins/loader.py", "import importlib\n__all__ = ['load', 'register']\ndef load(name=None):\n    importlib.import_module(name)\ndef register(): pass\n"),
    ];
    let g = graph(&files, GraphConfig::default());
    let f = run(&unused_rule(), &g, ALL);
    let got: Vec<(String, bool)> = f.iter().map(|f| (format!("{}#{}", f.file, f.captures[0].1), f.uncertain.is_some())).collect();
    assert_eq!(got, vec![("app/plugins/loader.py#register".into(), true), ("app/views.py#about".into(), false)]);
}

#[test]
fn python_without_all_reports_only_dead_names() {
    let files = [
        ("pkg/__init__.py", ""),
        ("pkg/run.py", "from pkg.util import helper\nhelper()\n"),
        ("pkg/util.py", "def helper(): return _x()\ndef _x(): pass\ndef used_here(): pass\nused_here()\ndef dead(): pass\n"),
    ];
    assert_eq!(unused(&files), vec!["pkg/util.py#dead"]);
}

#[test]
fn python_packages_reexports_and_submodules() {
    let files = [
        ("src/acme/__init__.py", "from .billing import compute\n"),
        ("src/acme/billing.py", "__all__ = ['compute', 'other']\ndef compute(): pass\ndef other(): pass\n"),
        ("src/acme/tools/__init__.py", ""),
        ("src/acme/tools/fmt.py", "__all__ = ['f', 'g']\ndef f(): pass\ndef g(): pass\n"),
        ("tests/test_it.py", "from acme import compute\nimport acme.tools.fmt\nfrom acme.tools import fmt as m\ncompute()\nacme.tools.fmt.f()\n"),
    ];
    // `m` is bound and never read, so nothing reads `g`.
    assert_eq!(unused(&files), vec!["src/acme/billing.py#other", "src/acme/tools/fmt.py#g"]);
}

#[test]
fn no_removal_that_breaks_an_export_list_or_a_reexport() {
    let files = [
        ("src/main.ts", "import { used } from './a';\nimport './barrel';\nconsole.log(used);\n"),
        ("src/a.ts", "const listed = 1;\nexport const used = 2;\nexport function reexported() {}\nexport { listed };\n"),
        ("src/barrel.ts", "export { reexported } from './a';\n"),
    ];
    let g = graph(&files, GraphConfig::default());
    let f = run(&unused_rule(), &g, ALL);
    let got: Vec<(String, usize)> = f.iter().map(|f| (f.captures[0].1.clone(), f.edits.len())).collect();
    assert_eq!(got, vec![("listed".into(), 0), ("reexported".into(), 0)]);
}

#[test]
fn unimported_modules_are_not_reported() {
    // Whether a file is used at all is `graph/no-unused-files`' question.
    assert!(unused(&[("src/lonely.ts", "export const x = 1;\n")]).is_empty());
}

#[test]
fn entries_from_presets_html_and_config() {
    let files = [
        ("index.html", "<script type=\"module\" src=\"/src/main.ts\"></script>"),
        ("src/main.ts", "import { a } from './a';\nexport const fromMain = a;\n"),
        ("src/a.ts", "export const a = 1;\nexport const b = 2;\n"),
        ("src/a.test.ts", "import { b } from './a';\nexport const t = b;\n"),
    ];
    assert!(unused(&files).is_empty());
}

#[test]
fn unused_css_classes() {
    let rule = compile(&serde_json::json!({"class": {"defined_in": "css"}}), Some(&serde_json::json!({"do": ["remove-rule"]}))).unwrap();
    let files = [
        ("styles.css", ".used { color: red }\n.unused { color: blue }\n.btn-primary {}\n.from-js {}\n.a, .b {}\n"),
        ("index.html", "<div class=\"used a\"></div>"),
        ("app.ts", "el.className = `btn-${kind}`; el.classList.add('from-js');\n"),
    ];
    let g = graph(&files, GraphConfig::default());
    let f = run(&rule, &g, &[Language::Css]);
    let names: Vec<&str> = f.iter().map(|f| f.captures[0].1.as_str()).collect();
    assert_eq!(names, vec!["unused", "b"]);
    assert_eq!(f[0].edits.len(), 1);
    assert!(f[1].edits.is_empty(), "a rule with two selectors is not removed");
}

#[test]
fn a_stylesheet_linked_from_html_is_still_checked() {
    // Corpus run (W-11): a `<link href>` made the stylesheet an entry, and
    // entries are skipped, so a plain site's CSS was never checked.
    let rule = compile(&serde_json::json!({"class": {"defined_in": "css"}}), None).unwrap();
    let files = [
        ("index.html", "<link rel=\"stylesheet\" href=\"css/site.css\"><div class=\"used\"></div>"),
        ("css/site.css", ".used { color: red }\n.dead { color: blue }\n"),
    ];
    let g = graph(&files, GraphConfig::default());
    let names: Vec<String> = run(&rule, &g, &[Language::Css]).into_iter().map(|f| f.captures[0].1.clone()).collect();
    assert_eq!(names, vec!["dead"]);
}

#[test]
fn framework_hooks_and_unittest_modules_are_used() {
    // Corpus run (W-11): Flask hook decorators, Django template-library
    // decorators and unittest's `test*.py` (`tests.py`) were reported.
    let files = [
        ("app/__init__.py", ""),
        ("app/main.py", "from app import auth, tags, tests\n"),
        ("app/auth.py", "@bp.before_app_request\ndef load_user(): pass\n\n@app.errorhandler(404)\ndef not_found(e): pass\n"),
        ("app/tags.py", "@register.inclusion_tag('x.html')\ndef snippet(): pass\n\n@register.filter\ndef shout(s): pass\n"),
        ("app/tests.py", "class SmokeTests: pass\n"),
    ];
    assert!(unused(&files).is_empty(), "{:?}", unused(&files));
}

#[test]
fn module_augmentation_uses_the_interface() {
    // Corpus run (W-11, zustand): middleware merges into `StoreMutators`
    // with `declare module '../vanilla'`; that is a use, not dead code.
    let files = [
        ("src/main.ts", "import { create } from './vanilla';\nimport './devtools';\ncreate();\n"),
        ("src/vanilla.ts", "export function create() {}\nexport interface StoreMutators<S, A> {}\nexport interface Unused {}\n"),
        ("src/devtools.ts", "export {};\ndeclare module './vanilla' {\n  interface StoreMutators<S, A> { devtools: S }\n}\n"),
    ];
    assert_eq!(unused(&files), vec!["src/vanilla.ts#Unused"]);
}

#[test]
fn the_two_first_party_copies_are_one_rule() {
    // SR-RULE-PACKS decision 2 merges the js and python copies into one rule
    // whose body is the later pack's: the bodies must not differ, or a TS
    // repo loses `interface` / `enum` when the python pack loads last.
    let block = |s: &str| s.split("```codelaw").nth(1).and_then(|b| b.split("```").next()).unwrap().to_owned();
    let js = block(include_str!("../../../packs/js/rules/graph/no-unused-exports.md"));
    let py = block(include_str!("../../../packs/python/rules/graph/no-unused-exports.md"));
    assert_eq!(js, py);
}

#[test]
fn why_unused_and_refs() {
    let files = [("src/a.ts", "import { x } from './b';\nx();\n"), ("src/b.ts", "export function x() {}\nexport function y() {}\n")];
    let g = graph(&files, GraphConfig::default());
    let r = refs(&g, "src/b.ts#x").unwrap();
    assert!(r.contains("src/a.ts  import"), "{r}");
    let w = why_unused(&g, "src/b.ts#y", &[DeclKind::Function], &["public".into()]).unwrap();
    assert!(w.contains("reported: exported and unused"), "{w}");
    let w = why_unused(&g, "src/b.ts#x", &[DeclKind::Function], &[]).unwrap();
    assert!(w.contains("keeps  when external_references: 0"), "{w}");
    assert!(locate_err(&g, "nope"));
}

fn locate_err(g: &Graph, spec: &str) -> bool {
    query::locate(g, spec).is_err()
}

fn import_findings(block: serde_json::Value, files: &[(&str, &str)]) -> Vec<String> {
    let rule = compile(&serde_json::json!({ "import": block }), None).unwrap();
    let g = graph(files, GraphConfig::default());
    run(&rule, &g, ALL).into_iter().map(|f| format!("{}:{} {}", f.file, f.span.0, f.captures[0].1)).collect()
}

#[test]
fn missing_imports() {
    let files = [
        ("tsconfig.json", r#"{"compilerOptions":{"paths":{"@/*":["src/*"]}}}"#),
        ("src/a.ts", "import { b } from './b';\nimport { c } from './c';\nimport x from '@/nope';\nimport y from 'react';\nimport s from './s.svg';\nexport * from './gone.js';\n"),
        ("src/b.ts", "export const b = require('./missing');\n"),
    ];
    let esm = import_findings(serde_json::json!({"form": ["import", "export", "dynamic"], "resolves": false}), &files);
    assert_eq!(esm, vec!["src/a.ts:43 ./c", "src/a.ts:64 @/nope", "src/a.ts:136 ./gone.js"]);
    let cjs = import_findings(serde_json::json!({"form": ["require"], "resolves": false}), &files);
    assert_eq!(cjs, vec!["src/b.ts:25 ./missing"]);
}

#[test]
fn undeclared_imports() {
    let files = [
        ("package.json", r#"{"name":"app","dependencies":{"react":"19"},"devDependencies":{"@types/node":"22","@acme/ui":"*"}}"#),
        ("src/a.ts", "import React from 'react';\nimport fs from 'node:fs';\nimport path from 'path';\nimport lodash from 'lodash/fp';\nimport { Ui } from '@acme/ui/button';\nimport type { X } from 'node';\nimport self from 'app/x';\nimport alias from '~/x';\nimport v from 'virtual:pwa';\n"),
    ];
    let got = import_findings(serde_json::json!({"declared": false}), &files);
    assert_eq!(got, vec!["src/a.ts:97 lodash/fp"]);
}

#[test]
fn unpublished_imports() {
    let files = [
        ("package.json", r#"{"name":"lib","main":"lib/index.js","files":["lib"],"dependencies":{"a":"1"},"devDependencies":{"b":"1"}}"#),
        ("lib/index.js", "const a = require('a');\nconst b = require('b');\nconst t = require('../test/helper');\nconst u = require('./util');\n"),
        ("lib/util.js", "module.exports = 1;\n"),
        ("test/helper.js", "const b = require('b');\n"),
    ];
    let got = import_findings(serde_json::json!({"form": ["require"], "published": false}), &files);
    assert_eq!(got, vec!["lib/index.js:42 b", "lib/index.js:66 ../test/helper"]);
}

#[test]
fn deterministic() {
    let files = [("src/a.ts", "import { x } from './b';\nx();\n"), ("src/b.ts", "export function x() {}\nexport function y() {}\n")];
    let a = run(&unused_rule(), &graph(&files, GraphConfig::default()), ALL);
    let mut rev = files;
    rev.reverse();
    let b = run(&unused_rule(), &graph(&rev, GraphConfig::default()), ALL);
    assert_eq!(a, b);
}

#[test]
fn deprecated_uses_by_identity() {
    let files = [
        ("packages/utils/package.json", r#"{"name":"@acme/utils","private":true,"main":"src/index.ts"}"#),
        ("packages/utils/src/index.ts", "export * from './date';\n"),
        ("packages/utils/src/date.ts", "/** @deprecated use formatISO */\nexport function formatDate(d: Date) { return d.toISOString(); }\nexport function formatISO(d: Date) { return d.toISOString(); }\n"),
        ("packages/web/package.json", r#"{"name":"web","private":true}"#),
        ("packages/web/src/a.ts", "import { formatDate as fd } from \"@acme/utils\";\nfd(new Date());\nfd(new Date());\n"),
        ("packages/web/src/b.ts", "function formatDate() {}\nformatDate();\n"),
        ("packages/web/src/c.ts", "import { format } from \"date-fns\";\nformat(1);\n"),
    ];
    let g = graph(&files, GraphConfig::default());
    let (module, name) = deprecated::declared(&g, "@acme/utils#formatDate").unwrap();
    assert_eq!((module.as_str(), name.as_str()), ("packages/utils/src/date.ts", "formatDate"));
    let uses = deprecated::uses_of_decl(&g, &module, &name, "@acme/utils#formatDate");
    // Two reads through the alias in a.ts; b.ts's own `formatDate` is not it.
    assert_eq!(uses.iter().map(|u| u.file.as_str()).collect::<Vec<_>>(), vec!["packages/web/src/a.ts", "packages/web/src/a.ts"]);
    let marked = deprecated::marked(&g);
    assert_eq!(marked.len(), 1);
    assert_eq!(marked[0].symbol, "packages/utils/src/date.ts#formatDate");
    assert_eq!(marked[0].message, "use formatISO");
    // A package outside the project: by its import specifier.
    assert!(deprecated::declared(&g, "date-fns#format").is_none());
    let ext = deprecated::uses_of_external(&g, "date-fns#format");
    assert_eq!(ext.len(), 1);
    assert_eq!(ext[0].file, "packages/web/src/c.ts");
}

#[test]
fn deprecated_python_alias() {
    let files = [
        ("acme/__init__.py", ""),
        ("acme/billing/__init__.py", ""),
        ("acme/billing/legacy.py", "def compute_tax(x):\n    return x\n"),
        ("app/billing.py", "from acme.billing.legacy import compute_tax as ct\nct(1)\n"),
    ];
    let g = graph(&files, GraphConfig::default());
    let (module, name) = deprecated::declared(&g, "acme.billing.legacy:compute_tax").unwrap();
    let uses = deprecated::uses_of_decl(&g, &module, &name, "acme.billing.legacy:compute_tax");
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].file, "app/billing.py");
}

fn module_rule(classes: &str) -> GraphRule {
    compile(&serde_json::json!({"module": {"classes": classes}}), Some(&serde_json::json!({"do": ["remove-rule"]}))).unwrap()
}

fn module_findings(classes: &str, files: &[(&str, &str)], config: GraphConfig) -> Vec<String> {
    let g = graph(files, config);
    let langs: &[Language] = if classes == "unused" { &[Language::Css, Language::Scss] } else { ALL };
    run(&module_rule(classes), &g, langs).into_iter().map(|f| format!("{}:{}", f.file, f.captures[0].1)).collect()
}

#[test]
fn css_modules_unused_and_undefined_classes() {
    let files = [
        ("src/a.module.css", ".used {}
.unused {}
:global(.g) {}
.btn-primary {}
"),
        ("src/a.tsx", "import s from './a.module.css';
export const x = [s.used, s['btn-primary'], s.nope];
"),
        ("src/plain.css", ".whatever {}
"),
        ("src/plain.tsx", "import p from './plain.css';
export const y = p.zzz;
"),
    ];
    assert_eq!(module_findings("unused", &files, GraphConfig::default()), vec!["src/a.module.css:unused"]);
    assert_eq!(module_findings("undefined", &files, GraphConfig::default()), vec!["src/a.tsx:nope"]);
}

#[test]
fn css_modules_destructuring_and_named_imports_are_reads() {
    let files = [
        ("a.module.css", ".one {}
.two {}
.three {}
"),
        ("a.js", "import s from './a.module.css';
const { one, two: renamed } = s;
"),
        ("b.js", "import { three, four } from './a.module.css';
"),
    ];
    assert!(module_findings("unused", &files, GraphConfig::default()).is_empty());
    assert_eq!(module_findings("undefined", &files, GraphConfig::default()), vec!["b.js:four"]);
}

#[test]
fn css_modules_computed_access_and_unimported_modules_are_silent() {
    let files = [
        ("a.module.css", ".one {}
.two {}
"),
        ("a.js", "import s from './a.module.css';
export const f = (n) => s[n];
"),
        ("b.module.css", ".lonely {}
"),
        ("c.module.css", ".x {}
.y {}
"),
        ("c.js", "import s from './c.module.css';
export const all = { ...s };
export const g = s.x;
"),
        ("d.module.css", ".x {}
.y {}
"),
        ("d.js", "import s from './d.module.css';
register(s);
"),
    ];
    assert!(module_findings("unused", &files, GraphConfig::default()).is_empty());
}

#[test]
fn css_modules_composes_counts_as_a_use_and_incomplete_modules_are_skipped() {
    let files = [
        ("a.module.css", ".base {}
.top { composes: base; }
.other { composes: remote from './b.module.css'; }
"),
        ("b.module.css", ".remote {}
.stray {}
"),
        ("use.js", "import a from './a.module.css';
import b from './b.module.css';
export const t = [a.top, a.other, b.stray];
"),
        ("c.module.scss", ".card { &-title {} }
.list, .row { &-x {} }
"),
        ("c.js", "import c from './c.module.scss';
export const k = c.card;
"),
    ];
    // `.card-title` is unused, but `&-x` on a selector list could not be named, so c.module.scss is skipped.
    assert!(module_findings("unused", &files, GraphConfig::default()).is_empty());
    let sass = [("c.module.scss", ".card { &-title {} }
"), ("c.js", "import c from './c.module.scss';
export const k = c.card;
")];
    assert_eq!(module_findings("unused", &sass, GraphConfig::default()), vec!["c.module.scss:card-title"]);
}

#[test]
fn css_modules_convention_and_globs() {
    let files = [("a.module.css", ".btn-primary {}
:export { gutter: 1px }
"), ("a.js", "import s from './a.module.css';
export const k = [s.btnPrimary, s.gutter];
")];
    assert_eq!(module_findings("undefined", &files, GraphConfig::default()), vec!["a.js:btnPrimary"]);
    let camel = GraphConfig { css_modules_convention: "camelCase".into(), ..Default::default() };
    assert!(module_findings("undefined", &files, camel.clone()).is_empty());
    assert!(module_findings("unused", &files, camel).is_empty());
    let named = [("src/a.styles.css", ".x {}
.y {}
"), ("src/a.js", "import s from './a.styles.css';
export const k = s.x;
")];
    assert!(module_findings("unused", &named, GraphConfig::default()).is_empty(), "not a module without [css] modules");
    let globbed = GraphConfig { css_modules: vec!["src/**/*.styles.css".into()], ..Default::default() };
    assert_eq!(module_findings("unused", &named, globbed), vec!["src/a.styles.css:y"]);
}

#[test]
fn test_files_and_what_they_reach() {
    for p in ["src/a.test.ts", "src/a.spec.jsx", "src/__tests__/a.ts", "tests/test_a.py", "a_test.py", "conftest.py"] {
        assert!(is_test_file(p), "{p}");
    }
    for p in ["src/a.ts", "src/testing.ts", "src/a.test.css", "app/contest.py"] {
        assert!(!is_test_file(p), "{p}");
    }
    let g = graph(
        &[
            ("src/a.ts", "export const viaAlias = () => 1;\nexport function direct() {}\nexport function unreached() {}\nfunction privateHelper() {}\n"),
            ("src/index.ts", "export { viaAlias as renamed } from './a';\n"),
            ("src/a.test.ts", "import { renamed as r } from './index';\nimport { direct } from './a';\nr(); direct();\n"),
            ("app/m.py", "def f():\n    pass\n\ndef g():\n    pass\n"),
            ("tests/test_m.py", "import app.m as mod\nfrom unittest import mock\nmod.f()\nmock.patch('app.m.g')\n"),
        ],
        GraphConfig::default(),
    );
    assert!(g.test_reaches("src/a.ts", "viaAlias", true), "a renamed re-export is identity");
    assert!(g.test_reaches("src/a.ts", "direct", true));
    assert!(!g.test_reaches("src/a.ts", "unreached", true));
    assert!(g.test_reaches("src/a.ts", "privateHelper", false), "not importable: its module is imported by a test");
    assert!(g.test_reaches("app/m.py", "f", true));
    assert!(!g.test_reaches("app/m.py", "g", true), "a string is not identity");
}

#[test]
fn terms_query_compiles_and_is_the_engines_to_run() {
    let rule = compile(&serde_json::json!({"terms": "inconsistent"}), None).unwrap();
    assert_eq!(rule.query, crate::Query::InconsistentTerms);
    let graph = Graph::build(Vec::new(), &[], GraphConfig::default());
    assert!(crate::run(&rule, &graph, &[Language::Markdown]).is_empty());
    assert!(compile(&serde_json::json!({"terms": "unused"}), None).unwrap_err().contains("`inconsistent`"));
    let respell = serde_json::json!({"do": ["respell"]});
    assert!(compile(&serde_json::json!({"terms": "inconsistent"}), Some(&respell)).is_ok());
    assert!(compile(&serde_json::json!({"terms": "inconsistent"}), Some(&serde_json::json!({"do": ["remove-rule"]}))).is_err());
    assert!(compile(&serde_json::json!({"class": {"defined_in": "css"}}), Some(&respell)).is_err());
}

#[test]
fn the_flags_query_compiles() {
    assert_eq!(compile(&serde_json::json!({"flags": "reads"}), None).unwrap().query, crate::Query::Flags(crate::flags::FlagCheck::Reads));
    assert_eq!(compile(&serde_json::json!({"flags": "registry"}), None).unwrap().query, crate::Query::Flags(crate::flags::FlagCheck::Registry));
    assert!(compile(&serde_json::json!({"flags": "all"}), None).unwrap_err().contains("`reads`, `markers` or `registry`"));
    assert!(compile(&serde_json::json!({"flags": "reads"}), Some(&serde_json::json!({"do": ["remove-rule"]}))).is_err());
    assert!(compile(&serde_json::json!({"flags": "reads"}), Some(&serde_json::json!({"do": ["keep-branch"]}))).is_ok());
    assert!(compile(&serde_json::json!({"flags": "registry"}), Some(&serde_json::json!({"do": ["keep-branch"]}))).unwrap_err().contains("remove-entry"));
    assert!(compile(&serde_json::json!({"symbol": {"exported": true}, "when": {"external_references": 0}}), Some(&serde_json::json!({"do": ["fix-marker"]}))).is_err());
}
