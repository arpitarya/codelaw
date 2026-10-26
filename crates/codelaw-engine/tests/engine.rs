//! The check and fix pipelines on real folders.

use codelaw_core::{Diagnostic, FixClass, Language};
use codelaw_engine::{EngineError, EngineOptions, FixOptions, Project};
use std::fs;
use std::path::{Path, PathBuf};

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Repo {
        let d = std::env::temp_dir().join(format!("codelaw-engine-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        Repo(fs::canonicalize(d).unwrap())
    }

    fn file(&self, rel: &str, text: &str) -> &Repo {
        let p = self.0.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
        self
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.0.join(rel)).unwrap()
    }

    fn project(&self) -> Result<Project, EngineError> {
        Project::load(&self.0, EngineOptions { no_cache: true, ..Default::default() })
    }

    fn check(&self) -> Vec<Diagnostic> {
        self.project().unwrap().check(&[]).unwrap().diagnostics
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A rule file with examples for each language given.
fn rule(id: &str, languages: &[Language], fix: &str, block: &str, examples: &[(Language, &str, &str)]) -> String {
    let langs: Vec<&str> = languages.iter().map(|l| l.name()).collect();
    let reason = if fix == "auto" { String::new() } else { "fix_reason: test\n".into() };
    let mut s = format!(
        "---\nid: {id}\ntitle: test rule\nlanguages: [{}]\nfix: {fix}\n{reason}stability: stable\n---\n\nSummary.\n\n**Why:** test.\n\n```codelaw\n{block}\n```\n\n",
        langs.join(", ")
    );
    if fix == "manual" {
        s.push_str("## How to fix\n\n1. Fix it by hand.\n\n");
    }
    s.push_str("## Examples\n\n");
    for (l, dont, do_) in examples {
        s.push_str(&format!("### {}\n\n#### Don't\n\n```{}\n{dont}\n```\n\n#### Do\n\n```{}\n{do_}\n```\n\n", l.display(), l.extension(), l.extension()));
    }
    // One line break at the end, as the Markdown formatter leaves it.
    s.truncate(s.trim_end().len() + 1);
    s
}

fn eqeq() -> String {
    rule(
        "acme/eqeqeq",
        &[Language::Javascript, Language::Typescript],
        "auto",
        "match:\n  pattern: $A == $B\nmessage: \"Use === instead of ==\"\nfix: \"$A === $B\"",
        &[(Language::Javascript, "a == b;", "a === b;"), (Language::Typescript, "a == b;", "a === b;")],
    )
}

fn summary(d: &[Diagnostic]) -> Vec<String> {
    d.iter().map(|d| format!("{}:{}:{} {} {}", d.file, d.start.line, d.start.column, d.rule, d.fix)).collect()
}

#[test]
fn no_rules_is_one_info_diagnostic() {
    let r = Repo::new("norules");
    r.file("a.js", "x == y;\n");
    let d = r.check();
    assert_eq!(summary(&d), ["./1:1 codelaw/no-rules manual".replace("./", ".:")]);
}

#[test]
fn local_rule_check_and_fix() {
    let r = Repo::new("local");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("src/a.js", "if (a == b) {\n}\nc == d;\n").file("src/b.py", "a == b\n");
    let d = r.check();
    assert_eq!(summary(&d), ["src/a.js:1:5 acme/eqeqeq auto", "src/a.js:3:1 acme/eqeqeq auto"]);
    assert_eq!(d[0].edits.len(), 1);
    assert_eq!(d[0].source, "local");

    let mut p = r.project().unwrap();
    let report = p.fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(report.files.len(), 1);
    assert!(report.files[0].written);
    assert_eq!(r.read("src/a.js"), "if (a === b) {\n}\nc === d;\n");
    assert!(report.check.diagnostics.is_empty(), "{:?}", report.check.diagnostics);
    // A second fix is a fixpoint.
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert!(report.files.is_empty());
}

/// SR-FORMATTER decision 5: `.editorconfig` is the base layer, the
/// nearest file and section win, and a key `codelaw.toml` sets wins over it.
#[test]
fn editorconfig_is_the_base_layer_under_format() {
    let r = Repo::new("editorconfig");
    r.file("codelaw.toml", "[format]\nline_width = 30\n")
        .file(".editorconfig", "root = true\n[*]\nindent_style = space\nindent_size = 4\nmax_line_length = 120\nend_of_line = crlf\n[*.py]\nindent_size = 2\n")
        .file("sub/.editorconfig", "[*.js]\nindent_style = tab\n")
        .file("a.js", "if (a) { b(1) }\n")
        .file("sub/b.js", "if (a) { b(1) }\n")
        .file("c.py", "if a:\n    b(1)\n");
    let p = r.project().unwrap();
    let a = p.formatting.options_for(&r.0.join("a.js"));
    assert_eq!((a.indent_width, a.line_width, a.end_of_line), (4, 30, Some(codelaw_format::EndOfLine::Crlf)));
    assert_eq!(p.formatting.options_for(&r.0.join("sub/b.js")).indent_style, codelaw_format::IndentStyle::Tab);
    assert_eq!(p.formatting.options_for(&r.0.join("c.py")).python_indent_width, 2);
    assert_eq!(p.format_source("a.js", Language::Javascript, "if (a) { b(1) }\n").unwrap(), "if (a) {\r\n    b(1);\r\n}\r\n");
    assert_eq!(p.format_source("sub/b.js", Language::Javascript, "if (a) { b(1) }\n").unwrap(), "if (a) {\r\n\tb(1);\r\n}\r\n");
    assert_eq!(p.format_source("c.py", Language::Python, "if a:\n    b(1)\n").unwrap(), "if a:\r\n  b(1)\r\n");
    // `codelaw format` writes what `format_source` gives.
    r.project().unwrap().format(&[], &Default::default()).unwrap();
    assert_eq!(r.read("sub/b.js"), "if (a) {\r\n\tb(1);\r\n}\r\n");
}

#[test]
fn dry_run_writes_nothing() {
    let r = Repo::new("dry");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("a.js", "a == b;\n");
    let report = r.project().unwrap().fix(&[], &FixOptions { dry_run: true, unsafe_fixes: false, trace: false }).unwrap();
    assert_eq!(report.files[0].fixed, "a === b;\n");
    assert!(!report.files[0].written);
    assert_eq!(r.read("a.js"), "a == b;\n");
}

fn pack(r: &Repo, dir: &str, name: &str, min: &str, rules: &[(&str, String)], presets: &[(&str, &str)]) {
    let preset_names: Vec<String> = presets.iter().map(|(p, _)| format!("\"{p}\"")).collect();
    r.file(
        &format!("{dir}/codelaw-pack.toml"),
        &format!("name = \"{name}\"\nversion = \"0.1.0\"\nmin_codelaw = \"{min}\"\nlanguages = [\"javascript\"]\npresets = [{}]\n", preset_names.join(", ")),
    );
    for (id, text) in rules {
        r.file(&format!("{dir}/rules/{id}.md"), text);
    }
    for (p, body) in presets {
        r.file(&format!("{dir}/presets/{p}.toml"), body);
    }
}

fn js_eqeq() -> String {
    eqeq().replace("acme/eqeqeq", "js/eqeqeq")
}

#[test]
fn first_party_pack_from_node_modules() {
    let r = Repo::new("npm");
    pack(&r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/eqeqeq", js_eqeq())], &[("js/recommended", "[rules]\n\"js/eqeqeq\" = \"warn\"\n")]);
    r.file("codelaw.toml", "extends = [\"codelaw:js/recommended\"]\n").file("a.js", "a == b;\n");
    let d = r.check();
    assert_eq!(summary(&d), ["a.js:1:1 js/eqeqeq auto"]);
    assert_eq!(d[0].severity, codelaw_core::Severity::Warn);
    assert_eq!(d[0].source, "@codelaw/rules-js@0.1.0");
}

#[test]
fn first_party_pack_from_the_python_environment() {
    let r = Repo::new("pypi");
    pack(&r, ".venv/lib/python3.12/site-packages/codelaw_rules_js", "@codelaw/rules-js", "0.1.0", &[("js/eqeqeq", js_eqeq())], &[("js/recommended", "[rules]\n\"js/eqeqeq\" = \"error\"\n")]);
    r.file("pyproject.toml", "[tool.codelaw]\nextends = [\"codelaw:js/recommended\"]\n").file("a.js", "a == b;\n");
    assert_eq!(summary(&r.check()), ["a.js:1:1 js/eqeqeq auto"]);
}

#[test]
fn pack_errors_stop_the_run() {
    let r = Repo::new("packerr");
    r.file("codelaw.toml", "extends = [\"codelaw:js/recommended\"]\n");
    let e = r.project().err().unwrap().to_string();
    assert!(e.contains("npm i -D @codelaw/rules-js") && e.contains("pip install codelaw-rules-js"), "{e}");

    pack(&r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "99.0.0", &[("js/eqeqeq", js_eqeq())], &[("js/recommended", "")]);
    let e = r.project().err().unwrap().to_string();
    assert!(e.contains("needs codelaw 99.0.0 or newer") && e.contains("upgrade codelaw"), "{e}");

    let native = rule("js/future", &[Language::Javascript], "manual", "native: js/not-yet-built", &[(Language::Javascript, "x", "y")]);
    pack(&r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/eqeqeq", js_eqeq()), ("js/future", native)], &[("js/recommended", "")]);
    let e = r.project().err().unwrap().to_string();
    assert!(e.contains("`native: js/not-yet-built` is not an engine rule") && e.contains("needs codelaw 0.1.0 or newer"), "{e}");
}

#[test]
fn native_rule_from_a_first_party_pack() {
    let r = Repo::new("native");
    let text = rule(
        "js/no-unreachable",
        &[Language::Javascript],
        "suggest",
        "native: js/no-unreachable",
        &[(Language::Javascript, "function f() { return 1; g(); }", "function f() { return 1; }")],
    );
    pack(&r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/no-unreachable", text)], &[("js/recommended", "[rules]\n\"js/no-unreachable\" = \"error\"\n")]);
    r.file("codelaw.toml", "extends = [\"codelaw:js/recommended\"]\n").file("a.js", "function f() {\n  return 1;\n  g();\n}\n");
    let d = r.check();
    assert_eq!(summary(&d), ["a.js:3:3 js/no-unreachable suggest"]);
    assert!(d[0].message.starts_with("Unreachable code"));
    let fixed = r.project().unwrap().fix(&[], &FixOptions { unsafe_fixes: true, dry_run: false, trace: false }).unwrap();
    assert_eq!(fixed.files.len(), 1);
    assert_eq!(r.read("a.js"), "function f() {\n  return 1;\n}\n");
}

#[test]
fn a_kind_no_grammar_has_is_refused() {
    let r = Repo::new("badkind");
    let text = rule("acme/typo", &[Language::Javascript, Language::Typescript], "manual", "match:\n  kind: lexical_declaraton\nmessage: x", &[(Language::Javascript, "let a;", "a;"), (Language::Typescript, "let a;", "a;")]);
    r.file(".codelaw/rules/acme/typo.md", &text);
    let e = r.project().err().unwrap().to_string();
    assert!(e.contains("`kind: lexical_declaraton` is not a node kind of javascript, typescript"), "{e}");
}

#[test]
fn local_rules_cannot_take_reserved_namespaces() {
    let r = Repo::new("reserved");
    r.file(".codelaw/rules/js/eqeqeq.md", &js_eqeq());
    let e = r.project().err().unwrap().to_string();
    assert!(e.contains("namespace `js` is reserved"), "{e}");
}

#[test]
fn ejected_rule_shadows_the_pack_rule() {
    let r = Repo::new("eject");
    pack(&r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/eqeqeq", js_eqeq())], &[("js/recommended", "[rules]\n\"js/eqeqeq\" = \"warn\"\n")]);
    r.file(".codelaw/rules/js/eqeqeq.md", &js_eqeq().replace("Use === instead of ==", "Local: use ==="));
    r.file("codelaw.toml", "extends = [\"codelaw:js/recommended\"]\n").file("a.js", "a == b;\n");
    let p = r.project().unwrap();
    let loaded = p.rules.get("js/eqeqeq").unwrap();
    assert_eq!(loaded.source.label(), "local");
    assert_eq!(loaded.shadows, ["@codelaw/rules-js@0.1.0"]);
    assert_eq!(r.check()[0].message, "Local: use ===");
}

#[test]
fn config_severity_overrides_and_unknown_rules() {
    let r = Repo::new("severity");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq())
        .file("codelaw.toml", "[rules]\n\"acme/eqeqeq\" = \"warn\"\n[[overrides]]\nfiles = [\"legacy/**\"]\nrules = { \"acme/eqeqeq\" = \"off\" }\n")
        .file("a.js", "a == b;\n")
        .file("legacy/b.js", "a == b;\n");
    let d = r.check();
    assert_eq!(summary(&d), ["a.js:1:1 acme/eqeqeq auto"]);
    assert_eq!(d[0].severity, codelaw_core::Severity::Warn);
    r.file("codelaw.toml", "[rules]\n\"acme/eqeq\" = \"warn\"\n");
    let e = r.project().err().unwrap().to_string();
    assert!(e.contains("unknown rule `acme/eqeq`; did you mean `acme/eqeqeq`?"), "{e}");
}

#[test]
fn suppressions() {
    let r = Repo::new("suppress");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file(
        "a.js",
        "// codelaw-ignore acme/eqeqeq: comparing across types on purpose\na == b;\nc == d; // codelaw-ignore acme/eqeqeq\n// codelaw-ignore acme/eqeqeq: nothing here\nlet x = 1;\n// codelaw-ignore acme/nope: typo\n",
    );
    let d = r.check();
    assert_eq!(
        summary(&d),
        [
            "a.js:3:1 acme/eqeqeq auto",
            "a.js:3:9 codelaw/suppression-needs-reason manual",
            "a.js:4:1 codelaw/unused-suppression auto",
            "a.js:6:1 codelaw/unknown-suppression manual",
        ]
    );
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert!(report.files[0].written);
    assert_eq!(
        r.read("a.js"),
        "// codelaw-ignore acme/eqeqeq: comparing across types on purpose\na == b;\nc === d; // codelaw-ignore acme/eqeqeq\nlet x = 1;\n// codelaw-ignore acme/nope: typo\n"
    );
}

#[test]
fn html_regions_lint_and_fix_in_the_host() {
    let r = Repo::new("regions");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("index.html", "<p>x</p>\n<script>\n  if (a == b) go();\n</script>\n<script type=\"text/template\">a == b\n</script>\n");
    let d = r.check();
    assert_eq!(summary(&d), ["index.html:3:7 acme/eqeqeq auto"]);
    r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert!(r.read("index.html").contains("if (a === b) go();"));
    // A language switched off is off inside hosts too.
    r.file("index.html", "<script>\n  a == b;\n</script>\n").file("codelaw.toml", "languages = [\"html\"]\n");
    assert!(r.check().is_empty());
}

#[test]
fn markdown_inline_rules_run_on_inline_content() {
    let r = Repo::new("mdinline");
    let text = rule(
        "acme/no-http-links",
        &[Language::Markdown],
        "manual",
        "match:\n  kind: link_destination\n  regex: ^http://\nmessage: \"Use https: $$MATCH\"",
        &[(Language::Markdown, "[a](http://x.org)", "[a](https://x.org)")],
    );
    r.file(".codelaw/rules/acme/no-http-links.md", &text).file("README.md", "# Title\n\nSee [the site](http://example.org) and [docs](https://ok.org).\n");
    let d = r.check();
    assert_eq!(summary(&d), ["README.md:3:16 acme/no-http-links manual"]);
    assert_eq!(d[0].message, "Use https: http://example.org");
}

#[test]
fn skipped_files_are_reported() {
    let r = Repo::new("skips");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq())
        .file("codelaw.toml", "[files]\nmax_file_size = \"4 KiB\"\n")
        .file("gen.js", "// @generated by a tool\na == b;\n")
        .file("big.js", &format!("a == b;\n{}\n", "x;".repeat(3000)));
    fs::write(r.0.join("bad.js"), [0x61, 0xff, 0x3b]).unwrap();
    let report = r.project().unwrap().check(&[]).unwrap();
    assert_eq!(summary(&report.diagnostics), ["bad.js:1:1 codelaw/not-utf8 manual", "big.js:1:1 codelaw/file-too-large manual", "gen.js:1:1 codelaw/generated manual"]);
    let skipped: Vec<&str> = report.skipped.iter().map(|s| s.file.as_str()).collect();
    assert_eq!(skipped, ["bad.js", "big.js", "gen.js"]);
    assert_eq!(report.files_checked, 2, "the rule file and codelaw.toml");
}

#[test]
fn gitignore_and_default_excludes() {
    let r = Repo::new("ignore");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq())
        .file(".gitignore", "dist/\n")
        .file("dist/a.js", "a == b;\n")
        .file("node_modules/x/a.js", "a == b;\n")
        .file(".github/a.js", "a == b;\n")
        .file("src/a.js", "a == b;\n");
    assert_eq!(summary(&r.check()), [".github/a.js:1:1 acme/eqeqeq auto", "src/a.js:1:1 acme/eqeqeq auto"]);
}

#[test]
fn crlf_and_bom_are_preserved() {
    let r = Repo::new("crlf");
    let collect = rule(
        "acme/exports-at-end",
        &[Language::Typescript],
        "auto",
        "match:\n  kind: export_statement\n  has: { field: declaration, pattern: $DECL }\nmessage: export at the end\nfix:\n  - strip-export-keyword: $DECL\n  - collect-into-block: { target: $DECL, block: \"export { $$$NAMES };\", type_block: \"export type { $$$TYPE_NAMES };\" }",
        &[(Language::Typescript, "export const a = 1;", "const a = 1;\nexport { a };")],
    );
    r.file(".codelaw/rules/acme/exports-at-end.md", &collect).file("a.ts", "\u{feff}export const a = 1;\r\nexport function f() {}\r\n");
    r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(fs::read(r.0.join("a.ts")).unwrap(), "\u{feff}const a = 1;\r\nfunction f() {}\r\nexport { a, f };\r\n".as_bytes());
}

#[test]
fn cold_warm_and_partial_cache_agree() {
    let r = Repo::new("cache");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq());
    for i in 0..20 {
        r.file(&format!("src/f{i}.js"), &format!("a{i} == b;\nok();\n"));
    }
    let opts = || EngineOptions { cache_dir_env: Some(r.0.join("cache").to_string_lossy().into()), ..Default::default() };
    let run = || Project::load(&r.0, opts()).unwrap().check(&[]).unwrap().diagnostics;
    let cold = run();
    let warm = run();
    assert_eq!(cold, warm);
    // Delete some entries: still identical (SR-CACHE decision 7).
    let entries = r.0.join("cache/entries");
    for (i, bucket) in fs::read_dir(&entries).unwrap().flatten().enumerate() {
        if i % 2 == 0 {
            fs::remove_dir_all(bucket.path()).unwrap();
        }
    }
    assert_eq!(run(), cold);
    let p = Project::load(&r.0, opts()).unwrap();
    let mut p = p;
    p.check(&[]).unwrap();
    assert!(p.cache.stats().0 > 0, "warm runs hit the cache");
}

#[test]
fn deterministic_across_threads_and_order() {
    let r = Repo::new("determinism");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq());
    for i in 0..30 {
        r.file(&format!("d{}/f{i}.ts", i % 3), &format!("x == {i};\n"));
    }
    let run = |threads: usize, paths: Vec<PathBuf>| {
        Project::load(&r.0, EngineOptions { no_cache: true, threads: Some(threads), ..Default::default() }).unwrap().check(&paths).unwrap().diagnostics
    };
    let a = run(1, vec![]);
    let b = run(8, vec![r.0.join("d2"), r.0.join("d0"), r.0.join("d1")]);
    assert_eq!(a, b);
    assert_eq!(a.len(), 30);
}

#[test]
fn check_source_matches_check() {
    let r = Repo::new("snippet");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("a.js", "a == b;\n");
    let mut p = r.project().unwrap();
    let from_disk = p.check(&[]).unwrap().diagnostics;
    assert_eq!(p.check_source("a.js", Language::Javascript, "a == b;\n"), from_disk);
    let fixed = p.fix_source("a.js", Language::Javascript, "a == b;\n", false);
    assert_eq!(fixed.fixed, "a === b;\n");
    let _ = Path::new("");
    let _ = FixClass::Auto;
}

// ---- W-17: regression tests for the 2026-10-20 review ----

/// The panic fixture (SR-ARCHITECTURE decision 25): a first-party pack whose
/// `js/test-panic` rule runs the `test/panic` native rule, which panics.
fn panic_pack(r: &Repo, severity: &str) {
    let text = rule("js/test-panic", &[Language::Javascript], "manual", "native: test/panic", &[(Language::Javascript, "x", "y")]);
    let preset = format!("[rules]\n\"js/test-panic\" = \"{severity}\"\n\"js/eqeqeq\" = \"warn\"\n");
    pack(r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/test-panic", text), ("js/eqeqeq", js_eqeq())], &[("js/recommended", &preset)]);
    r.file("codelaw.toml", "extends = [\"codelaw:js/recommended\"]\n");
}

#[test]
fn a_rule_panic_is_an_internal_error_never_the_rules_own() {
    let r = Repo::new("panic");
    panic_pack(&r, "warn");
    // A suppression of the rule itself does not hide its failure.
    r.file("a.js", "// codelaw-ignore js/test-panic: hiding the failure would be wrong\na == b;\n");
    let opts = || EngineOptions { cache_dir_env: Some(r.0.join("cache").to_string_lossy().into()), ..Default::default() };
    for run in ["cold", "warm"] {
        let report = Project::load(&r.0, opts()).unwrap().check(&[]).unwrap();
        let s = summary(&report.diagnostics);
        assert!(s.contains(&"a.js:1:1 codelaw/internal-error manual".to_owned()), "{run}: {s:?}");
        assert!(s.contains(&"a.js:2:1 js/eqeqeq auto".to_owned()), "the other rules still run ({run}): {s:?}");
        assert!(!report.diagnostics.iter().any(|d| d.rule == "js/test-panic"), "{run}: {s:?}");
        assert_eq!(report.internal_errors(), 1, "{run}");
        let d = report.diagnostics.iter().find(|d| d.rule == "codelaw/internal-error").unwrap();
        assert_eq!(d.severity, codelaw_core::Severity::Error);
        assert!(d.message.contains("js/test-panic") && d.message.contains("always panics"), "{}", d.message);
    }
    // `fix` reports it too, keeps the suppression (the rule did not run, so
    // it is not unused) and still applies the other rules' fixes.
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(report.check.internal_errors(), 1);
    assert_eq!(r.read("a.js"), "// codelaw-ignore js/test-panic: hiding the failure would be wrong\na === b;\n");
}

#[test]
fn a_failed_write_reports_the_file_as_it_is_on_disk() {
    let r = Repo::new("writefail");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("a.js", "a == b;\n");
    // A folder takes the temp file's name, so the write fails even for
    // root, who ignores read-only permissions.
    let tmp = r.0.join(format!(".a.js.codelaw-{}.tmp", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(r.read("a.js"), "a == b;\n");
    assert_eq!(report.files.len(), 1);
    assert!(!report.files[0].written);
    assert_eq!(report.fixed_count(), 0);
    let s = summary(&report.check.diagnostics);
    assert_eq!(s, ["a.js:1:1 acme/eqeqeq auto", "a.js:1:1 codelaw/write-failed manual"], "the pre-fix findings, and the failure");
    let failed = report.check.diagnostics.iter().find(|d| d.rule == "codelaw/write-failed").unwrap();
    assert_eq!(failed.severity, codelaw_core::Severity::Error, "a failed write fails the run");
    assert!(tmp.is_dir(), "the folder in the way is left alone");
}

#[cfg(unix)]
#[test]
fn a_fix_writes_through_a_symlink() {
    let r = Repo::new("symlink");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("codelaw.toml", "[files]\nfollow_symlinks = true\n").file("real/a.js", "a == b;\n");
    std::os::unix::fs::symlink(r.0.join("real/a.js"), r.0.join("b.js")).unwrap();
    std::os::unix::fs::symlink(r.0.join("real"), r.0.join("linked")).unwrap();
    let report = r.project().unwrap().fix(&[r.0.join("linked")], &FixOptions::default()).unwrap();
    assert!(report.files.iter().all(|f| f.written), "{:?}", report.check.diagnostics);
    assert!(fs::symlink_metadata(r.0.join("linked")).unwrap().file_type().is_symlink(), "the link stays a link");
    assert_eq!(r.read("real/a.js"), "a === b;\n", "the file it points to is fixed");
    // One file reached by three paths is fixed once.
    fs::write(r.0.join("real/a.js"), "c == d;\n").unwrap();
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert!(report.files.iter().all(|f| f.written), "{:?}", report.check.diagnostics);
    assert!(fs::symlink_metadata(r.0.join("b.js")).unwrap().file_type().is_symlink(), "a file link stays a link");
    assert_eq!(r.read("real/a.js"), "c === d;\n");
}

#[test]
fn a_pack_that_copies_a_first_party_name_is_still_third_party() {
    let r = Repo::new("fakefirstparty");
    let native = || {
        rule("js/no-unreachable", &[Language::Javascript], "suggest", "native: js/no-unreachable", &[(Language::Javascript, "function f() { return 1; g(); }", "function f() { return 1; }")])
    };
    pack(&r, "vendor/js", "@codelaw/rules-js", "0.1.0", &[("js/no-unreachable", native())], &[("js/recommended", "")]);
    r.file("codelaw.toml", "extends = [\"path:vendor/js\"]\n");
    let e = r.project().err().unwrap().to_string();
    assert!(e.contains("`native:` is accepted only in first-party packs"), "{e}");
    // The same pack where a first-party install lives is first-party.
    let r2 = Repo::new("realfirstparty");
    pack(&r2, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/no-unreachable", native())], &[("js/recommended", "")]);
    r2.file("codelaw.toml", "extends = [\"npm:@codelaw/rules-js\"]\n");
    assert!(r2.project().is_ok());
}

#[cfg(unix)]
#[test]
fn a_symlink_cycle_in_pack_data_ends() {
    let r = Repo::new("datacycle");
    pack(&r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/eqeqeq", js_eqeq())], &[("js/recommended", "[rules]\n\"js/eqeqeq\" = \"warn\"\n")]);
    r.file("node_modules/@codelaw/rules-js/data/globals/x.json", "{\"a\": 1}");
    std::os::unix::fs::symlink(r.0.join("node_modules/@codelaw/rules-js/data"), r.0.join("node_modules/@codelaw/rules-js/data/globals/loop")).unwrap();
    r.file("codelaw.toml", "extends = [\"codelaw:js/recommended\"]\n").file("a.js", "a == b;\n");
    assert_eq!(summary(&r.check()), ["a.js:1:1 js/eqeqeq auto"]);
}

#[test]
fn the_verifier_refuses_a_fix_that_makes_a_new_rule_fire() {
    let r = Repo::new("verifynew");
    let strict = rule("acme/no-strict-eq", &[Language::Javascript], "manual", "match:\n  pattern: $A === $B\nmessage: no ===", &[(Language::Javascript, "a === b;", "a == b;")]);
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file(".codelaw/rules/acme/no-strict-eq.md", &strict).file("a.js", "a == b;\n");
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(r.read("a.js"), "a == b;\n", "the fix would make acme/no-strict-eq fire");
    let d = &report.check.diagnostics;
    assert_eq!(summary(d), ["a.js:1:1 acme/eqeqeq manual"]);
    assert_eq!(d[0].note.as_deref(), Some("fix-verify-failed"));

    // A rule that fires and is fixed in a later pass is not new at the end.
    let chain = rule(
        "acme/strict-to-call",
        &[Language::Javascript],
        "auto",
        "match:\n  pattern: $A === $B\nmessage: use eq()\nfix: \"eq($A, $B)\"",
        &[(Language::Javascript, "a === b;", "eq(a, b);")],
    );
    fs::remove_file(r.0.join(".codelaw/rules/acme/no-strict-eq.md")).unwrap();
    r.file(".codelaw/rules/acme/strict-to-call.md", &chain);
    r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(r.read("a.js"), "eq(a, b);\n");
}

#[test]
fn the_verifier_counts_syntax_errors_inside_regions() {
    let r = Repo::new("verifyregion");
    let breaking = rule("acme/break", &[Language::Javascript], "auto", "match:\n  pattern: $A == $B\nmessage: x\nfix: \"$A == ($B\"", &[(Language::Javascript, "a == b;", "a == (b;")]);
    r.file(".codelaw/rules/acme/break.md", &breaking).file("a.html", "<p>x</p>\n<script>\n  a == b;\n</script>\n");
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(r.read("a.html"), "<p>x</p>\n<script>\n  a == b;\n</script>\n", "the JavaScript would no longer parse");
    assert_eq!(report.check.diagnostics[0].note.as_deref(), Some("fix-verify-failed"));
}

#[test]
fn suppressions_are_checked_where_no_rule_runs_and_only_open_a_comment() {
    let r = Repo::new("suppressnorule");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq())
        .file("a.py", "# codelaw-ignore acme/nope: typo\nx = 1\n# codelaw-ignore acme/eqeqeq\ny = 2\n")
        .file("b.js", "// see codelaw-ignore docs for the syntax\nlet x = 1;\n");
    assert_eq!(summary(&r.check()), ["a.py:1:1 codelaw/unknown-suppression manual", "a.py:3:1 codelaw/suppression-needs-reason manual"]);
}

#[test]
fn a_file_named_on_the_command_line_gets_its_nested_config() {
    let r = Repo::new("nestedexplicit");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq())
        .file("codelaw.toml", "")
        .file("pkg/codelaw.toml", "[rules]\n\"acme/eqeqeq\" = \"off\"\n")
        .file("pkg/src/a.js", "a == b;\n")
        .file("b.js", "a == b;\n");
    assert_eq!(summary(&r.check()), ["b.js:1:1 acme/eqeqeq auto"]);
    for path in ["pkg/src/a.js", "pkg/src", "pkg"] {
        let named = r.project().unwrap().check(&[r.0.join(path)]).unwrap().diagnostics;
        assert!(named.is_empty(), "`codelaw check {path}` reads pkg/codelaw.toml: {:?}", summary(&named));
    }
}

#[test]
fn ignore_files_above_the_project_are_read_only_inside_its_repository() {
    let outer = Repo::new("aboveroot");
    outer.file(".gitignore", "*.js\n");
    let r = outer.0.join("proj");
    fs::create_dir_all(r.join(".codelaw/rules/acme")).unwrap();
    fs::write(r.join(".codelaw/rules/acme/eqeqeq.md"), eqeq()).unwrap();
    fs::write(r.join("codelaw.toml"), "").unwrap();
    fs::create_dir_all(r.join("src")).unwrap();
    fs::write(r.join("src/a.js"), "a == b;\n").unwrap();
    let check = |paths: &[PathBuf]| Project::load(&r, EngineOptions { no_cache: true, ..Default::default() }).unwrap().check(paths).unwrap().diagnostics;
    // No repository: a `.gitignore` above the project (say `~/.gitignore`)
    // is a machine's, not the project's.
    assert_eq!(summary(&check(&[])), ["src/a.js:1:1 acme/eqeqeq auto"]);
    assert_eq!(summary(&check(&[r.join("src")])), ["src/a.js:1:1 acme/eqeqeq auto"]);
    // Inside a repository whose root is above the project, its ignore files apply.
    fs::create_dir_all(outer.0.join(".git")).unwrap();
    assert!(check(&[]).is_empty());
    assert!(check(&[r.join("src")]).is_empty());
    // And the project's own ignore files apply to a walk of a subfolder.
    fs::remove_dir_all(outer.0.join(".git")).unwrap();
    fs::write(r.join(".gitignore"), "src/gen/\n").unwrap();
    fs::create_dir_all(r.join("src/gen")).unwrap();
    fs::write(r.join("src/gen/b.js"), "a == b;\n").unwrap();
    assert_eq!(summary(&check(&[r.join("src")])), ["src/a.js:1:1 acme/eqeqeq auto"]);
}

#[cfg(unix)]
#[test]
fn walk_errors_are_reported_as_skipped() {
    let r = Repo::new("walkerr");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("codelaw.toml", "[files]\nfollow_symlinks = true\n").file("d/a.js", "a == b;\n");
    std::os::unix::fs::symlink(r.0.join("d"), r.0.join("d/loop")).unwrap();
    let report = r.project().unwrap().check(&[]).unwrap();
    assert_eq!(summary(&report.diagnostics), ["d/a.js:1:1 acme/eqeqeq auto"]);
    let skipped: Vec<&str> = report.skipped.iter().map(|s| s.file.as_str()).collect();
    assert_eq!(skipped, ["d/loop"], "{:?}", report.skipped);
}

#[test]
fn files_outside_the_root_are_relative_paths() {
    let r = Repo::new("outsideroot");
    r.file("proj/.codelaw/rules/acme/eqeqeq.md", &eqeq()).file("proj/codelaw.toml", "").file("other/a.js", "a == b;\n");
    let mut p = Project::load(&r.0.join("proj"), EngineOptions { no_cache: true, ..Default::default() }).unwrap();
    let d = p.check(&[r.0.join("other/a.js")]).unwrap().diagnostics;
    assert_eq!(summary(&d), ["../other/a.js:1:1 acme/eqeqeq auto"]);
}

#[test]
fn bom_offsets_are_file_offsets_and_fix_reports_like_check() {
    let r = Repo::new("bomoffsets");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("a.js", "\u{feff}x;\na == b;\n").file("b.js", "x;\na == b;\n");
    let report = r.project().unwrap().check(&[]).unwrap();
    let a = report.diagnostics.iter().find(|d| d.file == "a.js").unwrap();
    let b = report.diagnostics.iter().find(|d| d.file == "b.js").unwrap();
    assert_eq!((a.start_byte, a.end_byte), (b.start_byte + 3, b.end_byte + 3), "offsets count the BOM's bytes");
    assert_eq!(a.edits[0].start, b.edits[0].start + 3);
    let bytes = fs::read(r.0.join("a.js")).unwrap();
    assert_eq!(&bytes[a.start_byte..a.end_byte], b"a == b");
    let dry = r.project().unwrap().fix(&[], &FixOptions { dry_run: true, unsafe_fixes: false, trace: false }).unwrap();
    assert_eq!(dry.fixed_count(), 2);
    assert!(dry.check.diagnostics.is_empty(), "{:?}", dry.check.diagnostics);
}

/// `target:` + `assert:` rules through check and fix (SR-RULE-FORMAT
/// decision 3e): `when`/`else` picks the steps, and every byte outside the
/// edit is kept.
#[test]
fn target_rules_check_and_fix_json_yaml_and_toml() {
    let r = Repo::new("target");
    let strict = rule(
        "acme/strict",
        &[Language::Json, Language::Yaml, Language::Toml],
        "auto",
        "target: options\nassert: { path: { strict: { present: true, equals: true } } }\nmessage: \"`$PATH` turns on `strict`\"\nfix:\n  - when: { path: { strict: { present: true } } }\n    then: [{ ensure-key: { key: strict, value: true } }]\n    else: [{ ensure-key: { key: strict, value: true } }, sort-keys]",
        &[
            (Language::Json, "{ \"options\": {} }", "{ \"options\": { \"strict\": true } }"),
            (Language::Yaml, "options: {}", "options: { strict: true }"),
            (Language::Toml, "[options]", "[options]\nstrict = true"),
        ],
    );
    r.file(".codelaw/rules/acme/strict.md", &strict)
        .file("a.json", "{\n  \"options\": {\n    \"strict\": false\n  }\n}\n")
        .file("b.yaml", "# top\noptions:\n  # z comes first here\n  z: 1\n  a: 2 # two\nother: x\n")
        .file("c.toml", "name = \"x\"\n\n[other]\nb = 1\n")
        .file("d.json", "{\n  \"options\": {\n    \"strict\": true\n  }\n}\n");
    let d = r.check();
    // The rule file's own front matter is YAML too, but a `target:` rule
    // without `host:` reads whole files only.
    assert_eq!(summary(&d), ["a.json:2:3 acme/strict auto", "b.yaml:2:1 acme/strict auto", "c.toml:1:1 acme/strict auto"], "{:?}", d.iter().map(|d| &d.note).collect::<Vec<_>>());
    assert_eq!(d[0].message, "`options` turns on `strict`");
    r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(r.read("a.json"), "{\n  \"options\": {\n    \"strict\": true\n  }\n}\n");
    // `else`: the key is written and the keys sorted, each comment moving
    // with its key; the rest of the file is as it was.
    assert_eq!(r.read("b.yaml"), "# top\noptions:\n  a: 2 # two\n  strict: true\n  # z comes first here\n  z: 1\nother: x\n");
    assert_eq!(r.read("c.toml"), "name = \"x\"\n\n[other]\nb = 1\n\n[options]\nstrict = true\n");
    assert!(r.check().is_empty());
}

/// A `target:` fix that cannot be verified is reported `manual` with the
/// reason in its note, and a `target:` rule for another language is refused.
#[test]
fn target_rules_refuse_what_they_cannot_do() {
    let r = Repo::new("targetrefuse");
    let sorted = rule(
        "acme/sorted",
        &[Language::Json],
        "auto",
        "target: deps\nassert: { sorted: true }\nmessage: Sort `$PATH`\nfix: [sort-keys]",
        &[(Language::Json, "{ \"deps\": { \"b\": 1, \"a\": 2 } }", "{ \"deps\": { \"a\": 2, \"b\": 1 } }")],
    );
    r.file(".codelaw/rules/acme/sorted.md", &sorted).file("a.json", "{ \"deps\": { \"b\": 1, /* keep */ \"a\": 2 } }\n");
    let d = r.check();
    assert_eq!(summary(&d), ["a.json:1:3 acme/sorted manual"]);
    assert!(d[0].note.as_deref().is_some_and(|n| n.contains("comments")), "{:?}", d[0].note);
    let js = rule("acme/js-target", &[Language::Javascript], "manual", "target: x\nassert: { present: true }\nmessage: m", &[(Language::Javascript, "a;", "b;")]);
    r.file(".codelaw/rules/acme/js-target.md", &js);
    let Err(EngineError::Load(e)) = r.project() else { panic!("a `target:` rule for JavaScript loaded") };
    assert!(e.iter().any(|e| e.message.contains("`target:` rules read JSON, YAML and TOML")), "{e:?}");
}

/// W-58: a size rule's finding carries the measured value and the limit, and
/// an option of the wrong type is a load error.
#[test]
fn a_size_rule_reports_its_measured_value() {
    let text = rule("js/max-lines", &[Language::Javascript], "manual", "native: js/max-lines", &[(Language::Javascript, "x", "y")])
        .replace("stability: stable\n", "stability: stable\noptions:\n  max: { type: \"integer\", default: 300 }\n");
    let build = |name: &str, options: &str| {
        let r = Repo::new(name);
        pack(&r, "node_modules/@codelaw/rules-js", "@codelaw/rules-js", "0.1.0", &[("js/max-lines", text.clone())], &[("js/size", "[rules]\n\"js/max-lines\" = \"warn\"\n")]);
        r.file("codelaw.toml", &format!("extends = [\"codelaw:js/size\"]\n[rules]\n\"js/max-lines\" = {{ severity = \"warn\", options = {{ {options} }} }}\n")).file("a.js", "a();\nb();\nc();\nd();\n");
        r
    };
    let r = build("size", "max = 2");
    let d = r.check();
    assert_eq!(summary(&d), ["a.js:3:1 js/max-lines manual"]);
    assert_eq!(d[0].data, Some(serde_json::json!({ "lines": 4, "max": 2 })));
    assert_eq!(d[0].message, "4 lines (max 2; blank lines and comments not counted)");
    let wrong = build("size-wrong", "max = \"2\"");
    let e = wrong.project().err().unwrap().to_string();
    assert!(e.contains("option `max` must be a `integer`"), "{e}");
}
