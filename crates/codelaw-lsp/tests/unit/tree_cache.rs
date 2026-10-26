//! `codelaw-lsp`'s unit tests of the closed-file tree cache
//! (SR-ARCHITECTURE decision 24), compiled as a module of the library so
//! they reach the server's state. They write a fixture repo, so they live
//! here rather than in `src/`, which must have no write path.

use super::*;
use serde_json::json;
use std::fs;

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dest)
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

fn open(server: &mut Server, u: &str, text: &str) {
    let params = json!({ "textDocument": { "uri": u, "languageId": "javascript", "version": 1, "text": text } });
    server.notification(Notification::new(DidOpenTextDocument::METHOD.into(), params)).unwrap();
}

fn close(server: &mut Server, u: &str) {
    server.notification(Notification::new(DidCloseTextDocument::METHOD.into(), json!({ "textDocument": { "uri": u } }))).unwrap();
}

/// What a fresh parse of `text` finds, from a project loaded afresh.
fn fresh(root: &Path, file: &Path, text: &str) -> Vec<Diagnostic> {
    let config = codelaw_config::find_config(root).unwrap();
    let project = Project::from_config(config, EngineOptions { no_cache: true, ..EngineOptions::from_env() }).unwrap();
    project.check_document(&project.entry_for(file).unwrap(), text)
}

/// SR-ARCHITECTURE decision 24: closed documents keep their trees in an
/// LRU bounded by `[lsp] tree_cache`; a reopened file with the same text
/// gets its tree back; the diagnostics are a fresh parse's every time.
#[test]
fn closed_trees_are_an_lru_bounded_by_tree_cache() {
    let root = std::env::temp_dir().join(format!("codelaw-lsp-tree-cache-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/l3/rules"), &root.join(".codelaw/rules"));
    let root = canonical(&root);
    fs::write(root.join("codelaw.toml"), "[lsp]\ntree_cache = 2\n").unwrap();
    let text = "if (a == b) {\n  go();\n}\n";
    let files: Vec<PathBuf> = ["a.js", "b.js", "c.js"].iter().map(|n| root.join(n)).collect();
    for f in &files {
        fs::write(f, text).unwrap();
    }
    let uris: Vec<String> = files.iter().map(|f| uri::from_path(f)).collect();

    let (conn, _client) = Connection::memory();
    let mut server = Server::new(&conn, vec![root.clone()]);
    let want: Vec<Vec<Diagnostic>> = files.iter().map(|f| fresh(&root, f, text)).collect();
    assert!(!want[0].is_empty(), "the fixture rule fires");
    let mut trees = Vec::new();
    for (u, want) in uris.iter().zip(&want) {
        open(&mut server, u, text);
        assert_eq!(&server.docs[u].diagnostics, want);
        trees.push(server.docs[u].tree.as_ref().unwrap().text().as_ptr());
    }
    for u in &uris {
        close(&mut server, u);
    }
    let cache = &server.closed[&root];
    assert_eq!((cache.capacity(), cache.len()), (2, 2), "the bound is respected");
    assert!(!cache.contains(&uris[0]) && cache.contains(&uris[1]) && cache.contains(&uris[2]), "a.js was the least recently touched");

    // A recently closed file reopened with the same text reuses its tree.
    open(&mut server, &uris[2], text);
    assert_eq!(server.docs[&uris[2]].tree.as_ref().unwrap().text().as_ptr(), trees[2], "c.js's kept tree is reused");
    assert!(!server.closed[&root].contains(&uris[2]));
    assert_eq!(server.docs[&uris[2]].diagnostics, want[2]);
    // An evicted file parses afresh, with the same verdict.
    open(&mut server, &uris[0], text);
    assert_eq!(server.docs[&uris[0]].diagnostics, want[0]);
    // A kept tree of other text is never used.
    let edited = "if (a == b) {}\nif (c == d) {}\n";
    open(&mut server, &uris[1], edited);
    assert_ne!(server.docs[&uris[1]].tree.as_ref().unwrap().text().as_ptr(), trees[1]);
    assert_eq!(server.docs[&uris[1]].diagnostics, fresh(&root, &files[1], edited));
    assert!(server.closed[&root].is_empty());

    // A bound of 0, after a reload, keeps no closed tree.
    fs::write(root.join("codelaw.toml"), "[lsp]\ntree_cache = 0\n").unwrap();
    server.reload("test").unwrap();
    close(&mut server, &uris[0]);
    assert_eq!(server.closed[&root].capacity(), 0);
    assert!(server.closed[&root].is_empty());
    let _ = fs::remove_dir_all(&root);
}
