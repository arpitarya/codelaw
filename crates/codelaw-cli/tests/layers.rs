//! B-020: every crate depends only on crates in its own layer or the ones
//! above it (SR-ARCHITECTURE decision 1). The layers are SR-PACKAGES
//! decision 1's table; a crate that is not in it fails here.

use std::path::Path;

/// SR-PACKAGES decision 1, top layer first.
const LAYERS: &[(&str, &[&str])] = &[
    ("foundation", &["codelaw-core", "codelaw-cache"]),
    ("inputs", &["codelaw-config", "codelaw-lang", "codelaw-rules"]),
    (
        "engines",
        &[
            "codelaw-semantic",
            "codelaw-match",
            "codelaw-native",
            "codelaw-fix",
            "codelaw-format",
            "codelaw-graph",
            "codelaw-baseline",
            "codelaw-adapters",
            "codelaw-types",
        ],
    ),
    ("orchestration", &["codelaw-engine", "codelaw-report", "codelaw-test"]),
    ("surfaces", &["codelaw-cli", "codelaw-mcp", "codelaw-lsp", "codelaw-agents"]),
];

fn layer(name: &str) -> Option<usize> {
    LAYERS.iter().position(|(_, crates)| crates.contains(&name))
}

#[test]
fn crates_respect_the_layer_order() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut seen = 0;
    let mut problems = Vec::new();
    for entry in std::fs::read_dir(crates_dir).unwrap().flatten() {
        let manifest = entry.path().join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest) else { continue };
        let doc: toml::Table = toml::from_str(&text).unwrap();
        let name = doc["package"]["name"].as_str().unwrap().to_owned();
        seen += 1;
        let Some(own) = layer(&name) else {
            problems.push(format!("{name} is not in SR-PACKAGES decision 1"));
            continue;
        };
        // Dev-dependencies are tests, not the shipped graph.
        let Some(deps) = doc.get("dependencies").and_then(|d| d.as_table()) else { continue };
        for dep in deps.keys().filter(|d| d.starts_with("codelaw-")) {
            match layer(dep) {
                Some(l) if l <= own => {}
                Some(l) => problems.push(format!("{name} ({}) depends on {dep} ({}), a lower layer", LAYERS[own].0, LAYERS[l].0)),
                None => problems.push(format!("{name} depends on {dep}, which is not in SR-PACKAGES decision 1")),
            }
        }
    }
    assert!(seen >= 13, "found only {seen} crates");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
