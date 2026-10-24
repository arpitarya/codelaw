//! SR-RULE-AUTHORING decisions 4 and 6: `codelaw test --corpus` runs level 4
//! after levels 1–3, and no pack rule is `stable` without a corpus count.
//!
//! A pack's corpus counts live in `packs/<pack>/corpus.toml` (the
//! `--snapshot` format). A rule reaches `stable` only once its id is there.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").output().unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

#[test]
fn test_corpus_runs_level_four_and_holds_the_snapshot() {
    let root = repo_root();
    let snap = std::env::temp_dir().join(format!("codelaw-corpus-snapshot-{}.toml", std::process::id()));
    let snap_s = snap.to_str().unwrap();
    let args = ["test", "fixtures/l3/rules", "--corpus", "fixtures/l3", "--snapshot", snap_s];

    let (code, out) = codelaw(&root, &[&args[..], &["--update-snapshot"]].concat());
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("corpus: ") && out.contains(" findings"), "{out}");
    let written = fs::read_to_string(&snap).unwrap();
    assert!(written.contains("\"fixture/js-eqeqeq\" = "), "{written}");

    // The same corpus gives the same counts.
    let (code, out) = codelaw(&root, &args);
    assert_eq!(code, 0, "{out}");

    // A count that moved fails, and names the rule.
    fs::write(&snap, written.replace("\"fixture/js-eqeqeq\" = ", "\"fixture/js-eqeqeq\" = 1")).unwrap();
    let (code, out) = codelaw(&root, &args);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("fixture/js-eqeqeq") && out.contains("corpus · snapshot"), "{out}");

    // A rule with no count fails too.
    fs::write(&snap, "[findings]\n").unwrap();
    let (code, out) = codelaw(&root, &args);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("no count in the snapshot"), "{out}");
    let _ = fs::remove_file(&snap);

    // --snapshot needs --corpus; an empty corpus is refused.
    let (code, _) = codelaw(&root, &["test", "fixtures/l3/rules", "--snapshot", snap_s]);
    assert_eq!(code, 2);
    let empty = std::env::temp_dir().join(format!("codelaw-corpus-empty-{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap();
    let (code, out) = codelaw(&root, &["test", "fixtures/l3/rules", "--corpus", empty.to_str().unwrap()]);
    assert_eq!(code, 2, "{out}");
    let _ = fs::remove_dir_all(&empty);
}

fn rule_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            rule_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "md") && !p.to_string_lossy().ends_with(".cases.md") {
            out.push(p);
        }
    }
}

/// The frontmatter's `id:` and `stability:`.
fn front(text: &str) -> (Option<String>, Option<String>) {
    let Some(rest) = text.strip_prefix("---\n") else { return (None, None) };
    let fm = &rest[..rest.find("\n---").unwrap_or(0)];
    let field = |k: &str| fm.lines().find_map(|l| l.strip_prefix(k)).map(|v| v.trim().trim_matches(['"', '\'']).to_owned());
    (field("id:"), field("stability:"))
}

#[test]
fn no_pack_rule_is_stable_without_a_corpus_count() {
    let packs = repo_root().join("packs");
    let mut stable = 0;
    let mut unearned = Vec::new();
    let mut packs_seen = 0;
    for pack in fs::read_dir(&packs).unwrap().flatten().filter(|e| e.path().is_dir()) {
        packs_seen += 1;
        let snapshot = fs::read_to_string(pack.path().join("corpus.toml")).unwrap_or_default();
        let mut files = Vec::new();
        rule_files(&pack.path().join("rules"), &mut files);
        for f in files {
            let (id, stability) = front(&fs::read_to_string(&f).unwrap());
            if stability.as_deref() != Some("stable") {
                continue;
            }
            stable += 1;
            let id = id.unwrap_or_else(|| f.display().to_string());
            if !snapshot.lines().any(|l| l.trim_start().starts_with(&format!("\"{id}\" ="))) {
                unearned.push(format!("{id} ({})", f.strip_prefix(&packs).unwrap().display()));
            }
        }
    }
    assert_eq!(packs_seen, codelaw_rules::FIRST_PARTY_PACKS.len(), "every first-party pack");
    eprintln!("{stable} stable pack rules, each with a corpus count");
    assert!(
        unearned.is_empty(),
        "stable without a corpus run (SR-RULE-AUTHORING decision 6) — run `codelaw test packs/<pack> --corpus <pinned corpus> --snapshot packs/<pack>/corpus.toml --update-snapshot` first:\n{}",
        unearned.join("\n")
    );
}
