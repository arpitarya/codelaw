//! SR-CONSUMER-LAYOUT's veto check: `codelaw init` in a fixture repo, then
//! `git status` compared with §1's tree, read from the record itself — so
//! the tree and the check cannot drift apart.
//!
//! - every path init (and a later `check`) adds is one §1 lists (decision 1);
//! - nothing is gitignored but `.codelaw/cache/`, and init's only change to
//!   `.gitignore` is that one line (decision 3);
//! - with no `--agents`, no generated path exists (decision 5);
//! - `--agents` writes only §1's paths, or, while it is refused, nothing.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

const CLASSES: &[&str] = &["you write", "codelaw writes", "generated", "derived", "installed", "your code"];

/// A path of §1's tree: `<x>` is one path segment, `*` part of one, `…` any
/// run of characters.
fn matches(pattern: &str, path: &str) -> bool {
    fn go(p: &[char], s: &[char]) -> bool {
        match p.split_first() {
            None => s.is_empty(),
            Some(('…', rest)) => (0..=s.len()).any(|i| go(rest, &s[i..])),
            Some(('*', rest)) => (0..=s.len()).take_while(|i| *i == 0 || s[i - 1] != '/').any(|i| go(rest, &s[i..])),
            Some(('<', rest)) => {
                let Some(close) = rest.iter().position(|c| *c == '>') else { return false };
                (1..=s.len()).take_while(|i| s[i - 1] != '/').any(|i| go(&rest[close + 1..], &s[i..]))
            }
            Some((c, rest)) => s.first() == Some(c) && go(rest, &s[1..]),
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let s: Vec<char> = path.chars().collect();
    // The path itself, or anything under it: an installed pack is a folder.
    (1..=s.len()).any(|i| (i == s.len() || s[i] == '/' || s[i - 1] == '/') && go(&p, &s[..i]))
}

/// §1's tree as `(path pattern, class)`, read from the record.
fn layout() -> Vec<(String, String)> {
    let record = fs::read_to_string(repo_root().join("records/0111_consumer-layout.md")).unwrap();
    let section = &record[record.find("## §1").unwrap()..record.find("## §2").unwrap()];
    let start = section.find("```text\n").unwrap() + "```text\n".len();
    let block = &section[start..start + section[start..].find("```").unwrap()];
    let class_at = |text: &str| -> Option<(usize, String)> {
        CLASSES
            .iter()
            .filter_map(|c| {
                text.match_indices(c).find(|(i, _)| (*i == 0 || text[..*i].ends_with(' ')) && text[i + c.len()..].chars().next().is_none_or(|n| n == ' ' || n == ',')).map(|(i, _)| (i, c.to_string()))
            })
            .min()
    };
    let lines: Vec<&str> = block.lines().skip(1).collect(); // the first line is the repo itself
    let mut out = Vec::new();
    let mut parents: Vec<(usize, String)> = Vec::new();
    for (n, line) in lines.iter().enumerate() {
        let Some(glyph) = line.find(['├', '└']) else { continue }; // a continuation line
        let depth = line[..glyph].chars().count();
        let text = line[glyph..].trim_start_matches(['├', '└', '─', ' ']);
        let (path_col, class) = match class_at(text) {
            Some((i, c)) => (&text[..i], Some(c)),
            // A long path puts its class on the next line.
            None => (text, lines.get(n + 1).filter(|l| !l.contains(['├', '└'])).and_then(|l| class_at(l.trim_start_matches(['│', ' ']))).map(|c| c.1)),
        };
        parents.retain(|(d, _)| *d < depth);
        let prefix: String = parents.iter().map(|p| p.1.as_str()).collect();
        let paths: Vec<&str> = path_col.split_whitespace().filter(|t| !["·", "or", "…"].contains(t)).collect();
        match class {
            Some(c) => out.extend(paths.iter().map(|p| (format!("{prefix}{p}"), c.clone()))),
            None => parents.push((depth, format!("{prefix}{}", paths.concat()))),
        }
    }
    out
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.name=codelaw-test", "-c", "user.email=test@codelaw.invalid", "-c", "core.excludesFile=", "-c", "init.defaultBranch=main"])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .env("CODELAW_SKILLS", skills_package())
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// The skills package `init --agents` reads (W-77), outside the consumer so
/// it adds no path to its tree: `templates/agents/` and its manifest.
fn skills_package() -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-layout-skills-{}", std::process::id()));
    if !d.join("codelaw-skills.json").is_file() {
        copy_dir(&repo_root().join("templates/agents"), &d);
        fs::write(d.join("codelaw-skills.json"), format!("{{\"name\": \"codelaw-skills\", \"version\": \"{}\"}}\n", env!("CARGO_PKG_VERSION"))).unwrap();
    }
    d
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        if e.path().is_dir() {
            copy_dir(&e.path(), &to.join(e.file_name()));
        } else {
            fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}

/// A committed consumer repo: TypeScript, Python and JSON sources, and the
/// team's own `.gitignore`.
fn consumer(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-layout-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("src")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    fs::write(d.join("package.json"), "{\"name\": \"my-app\", \"private\": true}\n").unwrap();
    fs::write(d.join(".gitignore"), "node_modules/\n").unwrap();
    fs::write(d.join("src/app.ts"), "export const answer: number = 42;\n").unwrap();
    fs::write(d.join("src/tool.py"), "ANSWER = 42\n").unwrap();
    git(&d, &["init", "-q"]);
    git(&d, &["add", "-A"]);
    git(&d, &["commit", "-qm", "the app"]);
    d
}

/// `git status` as `(code, path)`, ignored paths included.
fn status(d: &Path) -> Vec<(String, String)> {
    git(d, &["status", "--porcelain", "--untracked-files=all", "--ignored"]).lines().map(|l| (l[..2].to_owned(), l[3..].to_owned())).collect()
}

fn class_of<'a>(tree: &'a [(String, String)], path: &str) -> Option<&'a str> {
    tree.iter().find(|(p, _)| matches(p, path)).map(|(_, c)| c.as_str())
}

#[test]
fn the_tree_reads_from_the_record() {
    let tree = layout();
    let class = |p: &str| class_of(&tree, p);
    assert_eq!(class("codelaw.toml"), Some("you write"));
    assert_eq!(class(".codelaw/rules/acme/no-moment.md"), Some("you write"));
    assert_eq!(class(".codelaw/rules/README.md"), Some("codelaw writes"));
    assert_eq!(class(".codelaw/rules/acme/no-moment.cases.md"), Some("you write"));
    assert_eq!(class(".codelaw/baseline/graph--no-deprecated-usage.toml"), Some("codelaw writes"));
    assert_eq!(class(".codelaw/deprecations.toml"), Some("you write"));
    assert_eq!(class(".codelaw/cache/"), Some("derived"));
    assert_eq!(class(".codelaw/cache/v1/index.bin"), Some("derived"));
    assert_eq!(class(".mcp.json"), Some("generated"));
    assert_eq!(class(".claude/skills/codelaw/SKILL.md"), Some("generated"));
    assert_eq!(class("node_modules/@codelaw/rules-js/codelaw-pack.toml"), Some("installed"));
    assert_eq!(class(".venv/lib/python3.12/site-packages/codelaw_rules_js/x"), Some("installed"));
    assert_eq!(class("src/app.ts"), Some("your code"));
    assert_eq!(class("Makefile"), None);
    assert_eq!(class(".codelaw/secrets.txt"), None);
}

#[test]
fn init_writes_only_the_paths_the_consumer_layout_lists() {
    let tree = layout();
    let d = consumer("init");
    let (code, out) = codelaw(&d, &["init", "--no-install"]);
    assert_eq!(code, 0, "{out}");
    // What `npm install -D @codelaw/rules-…` would put in place: the
    // Python pack is PyPI-only, so this npm repo's init leaves it out.
    assert!(out.contains("note: codelaw-rules-python is published on PyPI only"), "{out}");
    for pack in ["data", "deps", "js", "spell", "ts"] {
        copy_dir(&repo_root().join("packs").join(pack), &d.join("node_modules/@codelaw").join(format!("rules-{pack}")));
    }
    let (code, out) = codelaw(&d, &["check"]);
    assert!(code == 0 || code == 1, "check after init: {out}");

    let mut stray = Vec::new();
    for (code, path) in status(&d) {
        if path == ".gitignore" {
            assert_eq!(code, " M", ".gitignore");
            continue;
        }
        let class = class_of(&tree, &path);
        match (code.as_str(), class) {
            // Decision 3: nothing is ignored but the cache, beside what the team ignored itself.
            ("!!", Some("derived")) => {}
            // The team's own `node_modules/` line, holding the installed packs.
            ("!!", _) if path.starts_with("node_modules/") => {}
            ("!!", _) => stray.push(format!("ignored: {path} ({class:?})")),
            // Decision 5: no agents, no agent files.
            (_, Some("generated")) => stray.push(format!("generated with no --agents: {path}")),
            (_, Some("you write" | "codelaw writes")) => {}
            (_, _) => stray.push(format!("{code} {path} ({class:?})")),
        }
    }
    assert!(stray.is_empty(), "init wrote paths SR-CONSUMER-LAYOUT §1 does not list:\n{}", stray.join("\n"));
    for must in ["codelaw.toml", ".codelaw/rules/README.md", ".codelaw/spell/words.txt"] {
        assert!(d.join(must).is_file(), "init did not write {must}");
    }
    assert!(d.join(".codelaw/cache").is_dir(), "check wrote no cache, so decision 3 went unexercised");
    // Decision 3: init adds `.codelaw/cache/` to `.gitignore`, and nothing else.
    let diff = git(&d, &["diff", "--unified=0", "--", ".gitignore"]);
    let changed: Vec<&str> = diff.lines().filter(|l| l.starts_with(['+', '-']) && !l.starts_with("+++") && !l.starts_with("---")).collect();
    assert_eq!(changed, ["+.codelaw/cache/"], "{diff}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn init_with_agents_writes_only_the_listed_paths_or_nothing() {
    let tree = layout();
    let d = consumer("agents");
    let (code, out) = codelaw(&d, &["init", "--no-install", "--agents", "claude,copilot,codex,kiro"]);
    let after = status(&d);
    if code != 0 {
        // Refused (W-10 has not landed `init --agents`): a refusal writes nothing.
        assert!(after.is_empty(), "a refused init left {after:?}: {out}");
    } else {
        let stray: Vec<String> = after
            .iter()
            .filter(|(c, p)| p != ".gitignore" && (c == "!!" || !matches!(class_of(&tree, p), Some("you write" | "codelaw writes" | "generated"))))
            .map(|(c, p)| format!("{c} {p}"))
            .collect();
        assert!(stray.is_empty(), "init --agents wrote paths SR-CONSUMER-LAYOUT §1 does not list:\n{}", stray.join("\n"));
        assert!(after.iter().any(|(_, p)| class_of(&tree, p) == Some("generated")), "init --agents generated nothing: {out}");
    }
    let _ = fs::remove_dir_all(&d);
}
