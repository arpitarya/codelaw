//! W-62: SCSS is a CSS dialect (L3, amended 2026-10-26). The `css/*` rules
//! read `.scss` without calling Sass syntax an error, the `scss/*` rules fire
//! where they should, and `codelaw format` lays `.scss` out and is idempotent.

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

/// Every rule of a pack over `fixtures/scss`: the count of findings each gives is
/// the snapshot's. `modern.scss` is valid Sass that the `css/*` rules must read
/// without a false positive; `mistakes.scss` holds real mistakes they still catch.
#[test]
fn the_css_and_scss_packs_hold_their_counts_on_the_scss_corpus() {
    let root = repo_root();
    for (pack, snapshot) in [("packs/css", "fixtures/scss/css-pack.toml"), ("packs/scss", "fixtures/scss/scss-pack.toml")] {
        let (code, out) = codelaw(&root, &["test", pack, "--corpus", "fixtures/scss/corpus", "--snapshot", snapshot]);
        assert_eq!(code, 0, "{pack}: {out}");
        assert!(out.contains("corpus: 2 files"), "{pack}: {out}");
    }
}

/// A project that installs both packs reads Sass without a `css/*` false positive.
#[test]
fn css_rules_stay_quiet_on_sass_syntax() {
    let root = repo_root();
    let dir = std::env::temp_dir().join(format!("codelaw-scss-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("node_modules/@codelaw")).unwrap();
    for pack in ["css", "scss"] {
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("packs").join(pack), dir.join(format!("node_modules/@codelaw/rules-{pack}"))).unwrap();
    }
    fs::write(
        dir.join("codelaw.toml"),
        "extends = [\"codelaw:css/recommended\", \"codelaw:css/standard\", \"codelaw:scss/recommended\"]\npreview = true\n[format]\ncheck = false\n",
    )
    .unwrap();
    let sass = "@use \"sass:map\";\n$gap: 8px !default;\n%reset {\n  margin: 0;\n}\n@mixin m($a: 1) {\n  &:hover {\n    top: $a;\n  }\n}\n.nav {\n  @extend %reset;\n  font: {\n    family: Arial;\n    size: 14px;\n  }\n  &-wide {\n    width: map.get($m, w);\n  }\n  &__item {\n    padding: #{$gap};\n  }\n  // a comment\n  @include m;\n}\n";
    fs::write(dir.join("ok.scss"), sass).unwrap();
    let (code, out) = codelaw(&dir, &["check", "--no-cache"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("0 problems"), "{out}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_lays_out_scss_and_is_idempotent() {
    let dir = std::env::temp_dir().join(format!("codelaw-scss-fmt-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let messy = "$a:1px;\n.b{&-c{margin:$a;font:{family:Arial}}}\n@mixin m($x:1){top:$x}\n// note\n@include m(2);\n";
    fs::write(dir.join("a.scss"), messy).unwrap();
    let (code, out) = codelaw(&dir, &["format", "a.scss"]);
    assert_eq!(code, 0, "{out}");
    let once = fs::read_to_string(dir.join("a.scss")).unwrap();
    assert_ne!(once, messy);
    assert!(once.contains("  &-c {") && once.contains("$a: 1px;"), "{once}");
    let (code, out) = codelaw(&dir, &["format", "a.scss"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(fs::read_to_string(dir.join("a.scss")).unwrap(), once, "formatting twice changed the file");
    let (code, out) = codelaw(&dir, &["format", "--check", "a.scss"]);
    assert_eq!(code, 0, "{out}");
    let _ = fs::remove_dir_all(&dir);
}
