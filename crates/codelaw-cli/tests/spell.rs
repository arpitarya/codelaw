//! W-103: the spell pack through the real binary (DoD 3 and 4).
//!
//! `fixtures/spell/repo` holds one file per L3 language, each with a known
//! typo in a comment, a prose string, a declared name or a key, and the same
//! words where they are not prose: a URL, a hash, a JSON value that is an id,
//! a Markdown code span and link target, an HTML `class`, a name's uses.
//! `check` reports each typo once, where it is written or declared; `fix`
//! corrects the prose and the file-local names, leaves every exported name,
//! key, class and id as it is, and a second `fix` changes nothing.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
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

/// The fixture, with the spell pack linked where `codelaw:spell/…` finds it.
fn repo(name: &str) -> PathBuf {
    let root = repo_root();
    let dir = std::env::temp_dir().join(format!("codelaw-spell-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("node_modules/@codelaw")).unwrap();
    copy_dir(&root.join("fixtures/spell/repo"), &dir);
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("packs/spell"), dir.join("node_modules/@codelaw/rules-spell")).unwrap();
    dir
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_CACHE_DIR").output().unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// `file:line rule` of every diagnostic, sorted.
fn findings(cwd: &Path) -> Vec<String> {
    let (_, out) = codelaw(cwd, &["check", "--format", "json", "--no-cache"]);
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    let mut got: Vec<String> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| format!("{}:{} {}", d["file"].as_str().unwrap(), d["range"]["start"]["line"], d["rule"].as_str().unwrap()))
        .collect();
    got.sort();
    got
}

/// What `fix` cannot do alone: exported and module-level names, members,
/// keys, classes and ids, which other files read.
const MANUAL: &[&str] = &[
    "config/app.toml:3 spell/known-typo-key",
    "config/app.yaml:2 spell/known-typo-key",
    "config/settings.json:2 spell/known-typo-key",
    "src/app.js:2 spell/known-typo-identifier",
    "src/index.html:6 spell/known-typo-key",
    "src/jobs.py:2 spell/known-typo-identifier",
    "src/model.ts:5 spell/known-typo-identifier",
    "src/site.css:2 spell/known-typo-key",
    "src/site.scss:2 spell/known-typo-key",
];

#[test]
fn each_typo_is_reported_once_where_it_is_written_or_declared() {
    let dir = repo("check");
    let mut expected: Vec<String> = [
        "config/app.toml:1 spell/known-typo",
        "config/app.toml:4 spell/known-typo",
        "config/app.yaml:1 spell/known-typo",
        "config/app.yaml:3 spell/known-typo",
        "config/settings.json:3 spell/known-typo",
        "docs/guide.md:3 spell/known-typo",
        "docs/guide.md:3 spell/known-typo",
        "docs/guide.md:3 spell/known-typo",
        "src/Card.jsx:1 spell/known-typo",
        "src/Card.jsx:3 spell/known-typo-identifier",
        "src/Card.jsx:4 spell/known-typo",
        "src/List.tsx:1 spell/known-typo",
        "src/List.tsx:3 spell/known-typo-identifier",
        "src/List.tsx:5 spell/known-typo",
        "src/app.js:1 spell/known-typo",
        "src/app.js:5 spell/known-typo-identifier",
        "src/app.js:6 spell/known-typo",
        "src/index.html:3 spell/known-typo",
        "src/index.html:5 spell/known-typo",
        "src/index.html:7 spell/known-typo",
        "src/index.html:8 spell/known-typo",
        "src/jobs.py:1 spell/known-typo",
        "src/jobs.py:6 spell/known-typo",
        "src/jobs.py:7 spell/known-typo-identifier",
        "src/model.ts:1 spell/known-typo",
        "src/model.ts:2 spell/known-typo-identifier",
        "src/model.ts:9 spell/known-typo",
        "src/site.css:1 spell/known-typo",
        "src/site.scss:1 spell/known-typo",
    ]
    .iter()
    .chain(MANUAL)
    .map(|s| s.to_string())
    .collect();
    expected.sort();
    assert_eq!(findings(&dir), expected, "a URL, a hash, an id value, a code span, a link target, an HTML class and a name's uses are not reported");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn fix_corrects_prose_and_file_local_names_and_nothing_exported() {
    let dir = repo("fix");
    let (code, out) = codelaw(&dir, &["fix", "--no-cache"]);
    assert!(code == 0 || code == 1, "{out}");
    assert_eq!(findings(&dir), MANUAL, "{out}");
    let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
    assert_eq!(
        read("src/app.js"),
        "// We receive the payload here.\nexport const recieveTimeout = 30;\n\nexport function handle(input) {\n  const lengthLimit = 10;\n  const message = \"the address is missing\";\n  const link = \"https://example.com/recieve\";\n  const id = \"recieve\";\n  return [input, lengthLimit, message, link, id];\n}\n"
    );
    assert_eq!(
        read("src/jobs.py"),
        "# Jobs run until the queue is empty.\nrecieve_timeout = 30\n\n\ndef drain(queue):\n    \"\"\"Drain the queue until it is empty.\"\"\"\n    length = len(queue)\n    hash_ = \"3f2a1b4cuntill\"\n    return length, hash_\n"
    );
    assert!(read("src/model.ts").contains("type AddressBook") && read("src/model.ts").contains("as AddressBook") && read("src/model.ts").contains("  adress: string;"));
    assert!(read("config/settings.json").contains("\"recieveTimeout\": 30") && read("config/settings.json").contains("\"id\": \"recieve\""));
    assert!(read("docs/guide.md").contains("[the address docs](https://example.com/adress) and `recieve()`"));
    assert!(read("src/index.html").contains("id=\"adress-form\" class=\"recieve\"") && read("src/index.html").contains("href=\"#adress-form\">receive</a>"));
    // A fixpoint: the second run changes nothing.
    let before: Vec<String> = ["src/app.js", "src/model.ts", "docs/guide.md"].iter().map(|f| read(f)).collect();
    codelaw(&dir, &["fix", "--no-cache"]);
    let after: Vec<String> = ["src/app.js", "src/model.ts", "docs/guide.md"].iter().map(|f| read(f)).collect();
    assert_eq!(before, after);
    let _ = fs::remove_dir_all(&dir);
}

/// W-112: a repo `init` has not seen, with the packs it will want already
/// in place (`init --no-install` then finds them): prose with two unknown
/// words and one that is one edit from a common word.
fn fresh(name: &str) -> PathBuf {
    let root = repo_root();
    let dir = std::env::temp_dir().join(format!("codelaw-spell-init-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("node_modules/@codelaw")).unwrap();
    fs::create_dir_all(dir.join("docs")).unwrap();
    fs::write(dir.join("package.json"), "{\"name\": \"x\"}\n").unwrap();
    fs::write(dir.join("docs/notes.md"), "# Notes\n\nWe deploy the service with kubectl and the frobnicator each morning before lunch.\nThe servise answers every request from the client before noon.\n").unwrap();
    #[cfg(unix)]
    for pack in ["data", "spell"] {
        std::os::unix::fs::symlink(root.join("packs").join(pack), dir.join("node_modules/@codelaw").join(format!("rules-{pack}"))).unwrap();
    }
    dir
}

const TEMPLATE: &str = include_str!("../../../templates/init/.codelaw/spell/words.txt");

#[test]
fn init_adds_the_spell_pack_and_fills_the_words_file() {
    let a = fresh("a");
    let (code, out) = codelaw(&a, &["init", "--no-install"]);
    assert_eq!(code, 0, "{out}");
    let config = fs::read_to_string(a.join("codelaw.toml")).unwrap();
    assert!(config.contains("\"codelaw:spell/recommended\""), "{config}");
    assert!(out.contains("npm install -D @codelaw/rules-data @codelaw/rules-spell"), "{out}");
    let words = fs::read_to_string(a.join(".codelaw/spell/words.txt")).unwrap();
    // `codelaw` is from the rules folder's README, which init wrote.
    assert_eq!(words, format!("{TEMPLATE}codelaw\nfrobnicator\nkubectl\n"), "the template, then the learned words, sorted");
    assert!(out.contains("1 likely typos were not added") && out.contains("  servise: one edit from the common word `service`"), "{out}");
    assert!(out.contains("added 3 words to .codelaw/spell/words.txt:\n  codelaw\n  frobnicator\n  kubectl\n"), "{out}");
    // The learned words excuse the dictionary check.
    fs::write(a.join("codelaw.toml"), config.replace("codelaw:spell/recommended", "codelaw:spell/strict") + "preview = true\n").unwrap();
    let (_, out) = codelaw(&a, &["check", "--rule", "spell/unknown-word", "--no-cache"]);
    assert!(out.contains("servise") && !out.contains("kubectl") && !out.contains("frobnicator"), "{out}");

    // The same repo gives the same bytes.
    let b = fresh("b");
    let (code, out) = codelaw(&b, &["init", "--no-install"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(fs::read(b.join(".codelaw/spell/words.txt")).unwrap(), words.as_bytes());
    assert_eq!(fs::read_to_string(b.join("codelaw.toml")).unwrap(), config);

    // --no-spell: no pack, no file, no learn.
    let c = fresh("c");
    let (code, out) = codelaw(&c, &["init", "--no-install", "--no-spell"]);
    assert_eq!(code, 0, "{out}");
    assert!(!fs::read_to_string(c.join("codelaw.toml")).unwrap().contains("spell"));
    assert!(!out.contains("rules-spell") && !out.contains("words"), "{out}");
    assert!(!c.join(".codelaw/spell").exists());
    let d = fresh("d");
    assert_eq!(codelaw(&d, &["init", "--no-install", "--no-spell"]).1, out, "byte-identical across runs");
    for dir in [a, b, c, d] {
        let _ = fs::remove_dir_all(dir);
    }
}

#[test]
fn learn_appends_new_words_only_and_keeps_every_line() {
    let dir = fresh("learn");
    fs::write(dir.join("codelaw.toml"), "extends = [\"codelaw:md/recommended\"]\n").unwrap();
    fs::create_dir_all(dir.join(".codelaw/spell")).unwrap();
    let mine = "# our words\nKubectl\n\n# the rest\n";
    fs::write(dir.join(".codelaw/spell/words.txt"), mine).unwrap();
    let (code, out) = codelaw(&dir, &["spell", "learn"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(fs::read_to_string(dir.join(".codelaw/spell/words.txt")).unwrap(), format!("{mine}frobnicator\n"), "kubectl was already accepted");
    assert!(out.contains("servise"), "{out}");
    // A second run adds nothing and changes no byte.
    let (code, out) = codelaw(&dir, &["spell", "learn"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("no new words; .codelaw/spell/words.txt is up to date"), "{out}");
    assert_eq!(fs::read_to_string(dir.join(".codelaw/spell/words.txt")).unwrap(), format!("{mine}frobnicator\n"));
    // A new word is added after the others.
    fs::write(dir.join("docs/more.md"), "# More\n\nThe gizmotron writes a short summary when the build finishes.\n").unwrap();
    let (_, out) = codelaw(&dir, &["spell", "learn"]);
    assert!(out.contains("added 1 word to .codelaw/spell/words.txt:\n  gizmotron\n"), "{out}");
    assert_eq!(fs::read_to_string(dir.join(".codelaw/spell/words.txt")).unwrap(), format!("{mine}frobnicator\ngizmotron\n"));
    // A `words_file` the config sets must be there.
    fs::write(dir.join("codelaw.toml"), "[spell]\nwords_file = \"spelling.txt\"\n").unwrap();
    let (code, out) = codelaw(&dir, &["spell", "learn"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("`[spell] words_file`") && out.contains("spelling.txt"), "{out}");
    let _ = fs::remove_dir_all(&dir);
}
