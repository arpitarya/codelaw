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
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("src/a.js", "if (a == b) {}\nc == d;\n").file("src/b.py", "a == b\n");
    let d = r.check();
    assert_eq!(summary(&d), ["src/a.js:1:5 acme/eqeqeq auto", "src/a.js:2:1 acme/eqeqeq auto"]);
    assert_eq!(d[0].edits.len(), 1);
    assert_eq!(d[0].source, "local");

    let mut p = r.project().unwrap();
    let report = p.fix(&[], &FixOptions::default()).unwrap();
    assert_eq!(report.files.len(), 1);
    assert!(report.files[0].written);
    assert_eq!(r.read("src/a.js"), "if (a === b) {}\nc === d;\n");
    assert!(report.check.diagnostics.is_empty(), "{:?}", report.check.diagnostics);
    // A second fix is a fixpoint.
    let report = r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert!(report.files.is_empty());
}

#[test]
fn dry_run_writes_nothing() {
    let r = Repo::new("dry");
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("a.js", "a == b;\n");
    let report = r.project().unwrap().fix(&[], &FixOptions { dry_run: true, unsafe_fixes: false }).unwrap();
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
    let fixed = r.project().unwrap().fix(&[], &FixOptions { unsafe_fixes: true, dry_run: false }).unwrap();
    assert_eq!(fixed.files.len(), 1);
    assert_eq!(r.read("a.js"), "function f() {\n  return 1;\n  \n}\n");
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
    r.file(".codelaw/rules/acme/eqeqeq.md", &eqeq()).file("index.html", "<p>x</p>\n<script>\n  if (a == b) go();\n</script>\n<script type=\"text/template\">a == b</script>\n");
    let d = r.check();
    assert_eq!(summary(&d), ["index.html:3:7 acme/eqeqeq auto"]);
    r.project().unwrap().fix(&[], &FixOptions::default()).unwrap();
    assert!(r.read("index.html").contains("if (a === b) go();"));
    // A language switched off is off inside hosts too.
    r.file("index.html", "<script>a == b</script>\n").file("codelaw.toml", "languages = [\"html\"]\n");
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
