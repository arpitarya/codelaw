//! The law checks W-02 owes (DoD 7): the L3 fixture matrix and isolation
//! diff, the L4 loader and fix-field checks, and the L5 determinism,
//! parity and no-model checks — run through the real `codelaw` binary.

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LANGUAGES: &[&str] = &["javascript", "typescript", "python", "json", "yaml", "toml", "markdown", "css", "html"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn fixtures() -> PathBuf {
    repo_root().join("fixtures/l3")
}

fn copy_dir(from: &Path, to: &Path) {
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-laws-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

/// A repo of the fixture rules plus the files of `languages`.
fn build_repo(name: &str, languages: &[&str]) -> PathBuf {
    let d = temp(name);
    fs::create_dir_all(d.join(".codelaw/rules")).unwrap();
    copy_dir(&fixtures().join("rules"), &d.join(".codelaw/rules"));
    for l in languages {
        copy_dir(&fixtures().join(l), &d);
    }
    d
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_SITE_PACKAGES")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn check_json(cwd: &Path, extra: &[&str]) -> Value {
    let mut args = vec!["check", "--format", "json", "--no-cache"];
    args.extend_from_slice(extra);
    let (_, out, err) = codelaw(cwd, &args);
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}\n{err}"))
}

/// Diagnostics per file, for the isolation diff.
fn by_file(v: &Value) -> BTreeMap<String, Vec<Value>> {
    let mut out: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for d in v["diagnostics"].as_array().unwrap() {
        out.entry(d["file"].as_str().unwrap().to_owned()).or_default().push(d.clone());
    }
    out
}

fn files_under(root: &Path, rel_root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for e in fs::read_dir(root).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == ".codelaw") {
                continue;
            }
            out.extend(files_under(&p, rel_root));
        } else {
            out.insert(p.strip_prefix(rel_root).unwrap().to_string_lossy().replace('\\', "/"), fs::read(&p).unwrap());
        }
    }
    out
}

#[test]
fn l3_each_language_alone_and_together() {
    let combined = build_repo("combined", LANGUAGES);
    let together = check_json(&combined, &[]);
    let all = by_file(&together);
    // Together: one invocation, one report, one exit code (L3 decision 3).
    let rules: std::collections::BTreeSet<&str> = together["diagnostics"].as_array().unwrap().iter().map(|d| d["rule"].as_str().unwrap()).collect();
    assert_eq!(rules.len(), 9, "every language's rule fires in the combined repo: {rules:?}");
    for lang in LANGUAGES {
        let single = build_repo(lang, &[lang]);
        let v = check_json(&single, &[]);
        let alone = by_file(&v);
        // Equal capability (L3 decision 6): every language's rule runs.
        assert!(!alone.is_empty(), "{lang}: no diagnostics in its own repo: {v}");
        // Isolation (L3 decision 5): each file's diagnostics are the same
        // alone and in the combined repo.
        for (file, diags) in &alone {
            assert_eq!(Some(diags), all.get(file), "{lang}: {file} differs alone and together");
        }
        // `test` works in every single-language repo.
        let (code, out, err) = codelaw(&single, &["test"]);
        assert_eq!(code, 0, "{lang}: codelaw test\n{out}{err}");
        // `fix` writes the same bytes alone and together.
        let (code, _, err) = codelaw(&single, &["fix", "--no-cache"]);
        assert!(code == 0 || code == 1, "{lang}: fix exited {code}: {err}");
        let _ = fs::remove_dir_all(&single);
    }
    let (code, _, err) = codelaw(&combined, &["fix", "--no-cache"]);
    assert!(code == 0 || code == 1, "combined fix exited {code}: {err}");
    let fixed_together = files_under(&combined, &combined);
    for lang in LANGUAGES {
        let single = build_repo(&format!("{lang}-fix"), &[lang]);
        codelaw(&single, &["fix", "--no-cache"]);
        for (file, bytes) in files_under(&single, &single) {
            assert_eq!(Some(&bytes), fixed_together.get(&file), "{lang}: {file} fixes differently alone and together");
        }
        let _ = fs::remove_dir_all(&single);
    }
    let _ = fs::remove_dir_all(&combined);
}

#[test]
fn l3_a_language_switched_off_costs_nothing() {
    let repo = build_repo("lang-off", LANGUAGES);
    let v = check_json(&repo, &["--lang", "python"]);
    let files: Vec<&str> = v["diagnostics"].as_array().unwrap().iter().map(|d| d["file"].as_str().unwrap()).collect();
    assert_eq!(files, ["pkg/mod.py"]);
    assert_eq!(v["summary"]["files_checked"], 1);
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn l4_every_diagnostic_carries_its_fix() {
    let repo = build_repo("l4-fields", LANGUAGES);
    let v = check_json(&repo, &[]);
    let diags = v["diagnostics"].as_array().unwrap();
    assert!(!diags.is_empty());
    for d in diags {
        let class = d["fix"]["class"].as_str().unwrap();
        let edits = d["fix"]["edits"].as_array().unwrap();
        let steps = d["fix"]["steps"].as_array().unwrap();
        for field in ["rule", "file", "message"] {
            assert!(d[field].is_string(), "{d}");
        }
        assert!(d["range"]["start"]["line"].is_u64(), "{d}");
        match class {
            "auto" | "suggest" => assert!(!edits.is_empty(), "an {class} diagnostic without edits: {d}"),
            "manual" => assert!(!steps.is_empty(), "a manual diagnostic without steps: {d}"),
            other => panic!("unknown fix class {other}"),
        }
    }
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn l4_the_loader_refuses_a_rule_without_a_fix() {
    let repo = temp("l4-loader");
    let rule = fs::read_to_string(fixtures().join("rules/fixture/js-eqeqeq.md")).unwrap().replace("fix: auto\n", "");
    fs::create_dir_all(repo.join(".codelaw/rules/fixture")).unwrap();
    fs::write(repo.join(".codelaw/rules/fixture/js-eqeqeq.md"), rule).unwrap();
    fs::write(repo.join("a.js"), "a == b;\n").unwrap();
    let (code, _, err) = codelaw(&repo, &["check"]);
    assert_eq!(code, 2);
    assert!(err.contains("`fix:` is required"), "{err}");
    let (code, _, err) = codelaw(&repo, &["rules", "lint"]);
    assert_eq!(code, 1, "{err}");
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn l5_same_output_from_any_folder_and_thread_count() {
    let repo = build_repo("l5", LANGUAGES);
    let a = check_json(&repo, &["--threads", "1"]);
    let b = check_json(&repo.join("src"), &["--threads", "8", repo.to_str().unwrap()]);
    assert_eq!(a, b);
    let (_, text_a, _) = codelaw(&repo, &["check", "--no-cache"]);
    let (_, text_b, _) = codelaw(&repo, &["check", "--no-cache"]);
    assert_eq!(text_a, text_b);
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn l5_cli_and_library_agree() {
    let repo = build_repo("parity", LANGUAGES);
    let cli = check_json(&repo, &[]);
    let mut project = codelaw_engine::Project::load(&repo, codelaw_engine::EngineOptions { no_cache: true, ..Default::default() }).unwrap();
    let lib = codelaw_report::check_json(&project.check(&[]).unwrap());
    assert_eq!(cli, lib);
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn l5_no_model_in_the_dependency_graph() {
    let lock = fs::read_to_string(repo_root().join("Cargo.lock")).unwrap();
    let banned = ["openai", "anthropic", "llm", "langchain", "ollama", "tokenizers", "candle", "onnx", "tch", "async-openai", "llama"];
    for line in lock.lines().filter(|l| l.starts_with("name = ")) {
        let name = line.trim_start_matches("name = ").trim_matches('"');
        assert!(!banned.iter().any(|b| name == *b || name.starts_with(&format!("{b}-")) || name.ends_with(&format!("-{b}"))), "model dependency `{name}` (L5)");
    }
}

/// One unformatted file per L3 language, for the L7 check.
const MESSY: &[(&str, &str, &str)] = &[
    ("javascript", "m.js", "const a = {b:1}\n"),
    ("typescript", "m.ts", "let a:number=1\n"),
    ("python", "m.py", "x = {  'a':1 }\n"),
    ("json", "m.json", "{\"a\":1}\n"),
    ("yaml", "m.yaml", "a:   1\n"),
    ("toml", "m.toml", "a=1\n"),
    ("markdown", "m.md", "# T\n* a\n"),
    ("css", "m.css", "a{color:red}\n"),
    ("html", "m.html", "<div><p>x</p>\n\n\n</div>\n"),
];

/// SR-LAW-7 §How L7 is enforced: over the L3 fixture matrix, every
/// language has a lint rule and a formatter that round-trips its fixtures.
#[test]
fn l7_every_language_has_a_lint_rule_and_a_formatter_that_round_trips_its_fixtures() {
    assert_eq!(MESSY.len(), LANGUAGES.len());
    for lang in LANGUAGES {
        let repo = build_repo(&format!("l7-{lang}"), &[lang]);
        // A lint rule: the language's fixture rule fires on its files.
        let v = check_json(&repo, &[]);
        let rules: Vec<&str> = v["diagnostics"].as_array().unwrap().iter().map(|d| d["rule"].as_str().unwrap()).collect();
        assert!(rules.iter().any(|r| r.starts_with("fixture/")), "{lang}: no lint rule fires: {rules:?}");
        // A formatter that round-trips the fixtures: they are formatted,
        // and formatting them changes no byte.
        assert!(!rules.iter().any(|r| r.starts_with("format/")), "{lang}: the fixtures are not formatted: {v}");
        let before = files_under(&repo, &repo);
        let (code, out, err) = codelaw(&repo, &["format", "--check", "--no-cache"]);
        assert_eq!(code, 0, "{lang}: format --check\n{out}{err}");
        let (code, out, err) = codelaw(&repo, &["format", "--no-cache"]);
        assert_eq!(code, 0, "{lang}: format\n{out}{err}");
        assert_eq!(files_under(&repo, &repo), before, "{lang}: formatting the fixtures changed them");
        // And it formats: an unformatted file is reported, rewritten, then clean.
        let (_, file, text) = MESSY.iter().find(|(l, _, _)| l == lang).unwrap();
        fs::write(repo.join(file), text).unwrap();
        let (code, out, _) = codelaw(&repo, &["format", "--check", "--no-cache"]);
        assert_eq!((code, out.trim()), (1, format!("would format {file}").as_str()), "{lang}");
        let v = check_json(&repo, &[]);
        assert!(v["diagnostics"].as_array().unwrap().iter().any(|d| d["rule"] == "format/unformatted" && d["file"] == *file), "{lang}: check misses {file}");
        let (code, _, err) = codelaw(&repo, &["format", "--no-cache"]);
        assert_eq!(code, 0, "{lang}: {err}");
        assert_ne!(fs::read_to_string(repo.join(file)).unwrap(), *text, "{lang}: {file} was not formatted");
        let (code, out, err) = codelaw(&repo, &["format", "--check", "--no-cache"]);
        assert_eq!(code, 0, "{lang}: not idempotent\n{out}{err}");
        let _ = fs::remove_dir_all(&repo);
    }
}

/// SR-FORMATTER decision 4: a file the formatter cannot handle is reported
/// and left as it was.
#[test]
fn l7_a_format_that_fails_is_reported_and_the_file_kept() {
    let repo = temp("l7-fail");
    fs::write(repo.join("bad.js"), "const = ;\n").unwrap();
    let (code, out, _) = codelaw(&repo, &["format", "--no-cache"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("format/failed"), "{out}");
    assert_eq!(fs::read_to_string(repo.join("bad.js")).unwrap(), "const = ;\n");
    let _ = fs::remove_dir_all(&repo);
}
