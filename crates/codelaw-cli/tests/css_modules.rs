//! W-63: CSS Modules (`*.module.css`, `*.module.scss`, `[css] modules`) in the
//! `css` pack and the `css-modules` pack, through the real binary.
//!
//! `fixtures/css-modules/react` is a small React app: an unused class, an
//! undefined `styles.missing`, a bad `composes`, a module read by a computed
//! name, and a `.module.scss`. Each finding is reported once, in the right file.

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

/// The React fixture, with the packs linked where `codelaw:css-modules/…` finds them.
fn app(name: &str) -> PathBuf {
    let root = repo_root();
    let dir = std::env::temp_dir().join(format!("codelaw-css-modules-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("node_modules/@codelaw")).unwrap();
    copy_dir(&root.join("fixtures/css-modules/react"), &dir);
    for pack in ["css", "scss", "css-modules"] {
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("packs").join(pack), dir.join(format!("node_modules/@codelaw/rules-{pack}"))).unwrap();
    }
    dir
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_CACHE_DIR").output().unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// `(file, rule)` of every diagnostic, sorted.
fn findings(cwd: &Path, extra: &[&str]) -> Vec<(String, String)> {
    let mut args = vec!["check", "--format", "json", "--no-cache"];
    args.extend_from_slice(extra);
    let (_, out) = codelaw(cwd, &args);
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    let mut got: Vec<(String, String)> =
        v["diagnostics"].as_array().unwrap().iter().map(|d| (d["file"].as_str().unwrap().to_owned(), d["rule"].as_str().unwrap().to_owned())).collect();
    got.sort();
    got
}

fn pair(file: &str, rule: &str) -> (String, String) {
    (file.to_owned(), rule.to_owned())
}

#[test]
fn each_problem_is_reported_once_in_the_right_file() {
    let dir = app("fixture");
    let got = findings(&dir, &[]);
    assert_eq!(
        got,
        [
            pair("src/Button.module.css", "css-modules/composes-valid"),
            pair("src/Button.module.css", "css-modules/no-unused-class"),
            pair("src/Button.tsx", "css-modules/no-undefined-class"),
            pair("src/Panel.module.scss", "css-modules/no-unused-class"),
        ],
        "`:global`, `composes`, `:export`, `@value` and `:import` give no css/* finding; `styles[size]` silences Dynamic.module.css"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unsafe_fix_removes_an_unused_class_and_only_that() {
    let dir = app("fix");
    let (_, out) = codelaw(&dir, &["fix", "--unsafe", "--no-cache"]);
    let css = fs::read_to_string(dir.join("src/Button.module.css")).unwrap();
    assert!(!css.contains(".unused"), "{css}\n{out}");
    assert!(css.contains(".broken") && css.contains(".button"), "{css}");
    let scss = fs::read_to_string(dir.join("src/Panel.module.scss")).unwrap();
    assert!(!scss.contains(".old") && scss.contains("&-title"), "{scss}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn modules_convention_decides_what_styles_x_reaches() {
    let dir = app("convention");
    let toml = fs::read_to_string(dir.join("codelaw.toml")).unwrap();
    fs::write(dir.join("src/Kebab.module.css"), ".btn-primary { color: red; }\n").unwrap();
    fs::write(dir.join("src/Kebab.tsx"), "import styles from \"./Kebab.module.css\";\nexport const k = styles.btnPrimary;\n").unwrap();
    // asIs: `.btn-primary` is `styles["btn-primary"]`, so `styles.btnPrimary` is undefined.
    let asis = findings(&dir, &[]);
    assert!(asis.contains(&pair("src/Kebab.tsx", "css-modules/no-undefined-class")), "{asis:?}");
    assert!(asis.contains(&pair("src/Kebab.module.css", "css-modules/no-unused-class")), "{asis:?}");
    fs::write(dir.join("codelaw.toml"), format!("{toml}\n[css]\nmodules_convention = \"camelCase\"\n")).unwrap();
    let mapped = findings(&dir, &[]);
    assert!(!mapped.iter().any(|(f, _)| f.starts_with("src/Kebab")), "{mapped:?}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn css_modules_globs_name_modules_that_are_not_called_module() {
    let dir = app("globs");
    fs::write(dir.join("src/Card.styles.css"), ".card { composes: gone; }\n:global(.x) { color: red; }\n").unwrap();
    fs::write(dir.join("src/Card.tsx"), "import styles from \"./Card.styles.css\";\nexport const c = styles.card;\n").unwrap();
    let before = findings(&dir, &[]);
    assert!(!before.iter().any(|(f, r)| f == "src/Card.styles.css" && r.starts_with("css-modules/")), "{before:?}");
    // As a plain stylesheet, `composes` is an unknown property and `:global` an unknown pseudo-class.
    assert!(before.iter().any(|(f, r)| f == "src/Card.styles.css" && r == "css/no-unknown-property"), "{before:?}");
    let toml = fs::read_to_string(dir.join("codelaw.toml")).unwrap();
    fs::write(dir.join("codelaw.toml"), format!("{toml}\n[css]\nmodules = [\"src/**/*.styles.css\"]\n")).unwrap();
    let after = findings(&dir, &[]);
    assert!(after.contains(&pair("src/Card.styles.css", "css-modules/composes-valid")), "{after:?}");
    assert!(!after.iter().any(|(f, r)| f == "src/Card.styles.css" && r.starts_with("css/")), "{after:?}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_bad_modules_convention_is_a_config_error() {
    let dir = app("bad-convention");
    let toml = fs::read_to_string(dir.join("codelaw.toml")).unwrap();
    fs::write(dir.join("codelaw.toml"), format!("{toml}\n[css]\nmodules_convention = \"snake\"\n")).unwrap();
    let (code, out) = codelaw(&dir, &["check", "--no-cache"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("modules_convention") && out.contains("camelCase"), "{out}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn strict_adds_naming_and_no_global() {
    let dir = app("strict");
    let toml = fs::read_to_string(dir.join("codelaw.toml")).unwrap().replace("css-modules/recommended", "css-modules/strict");
    fs::write(dir.join("codelaw.toml"), toml).unwrap();
    let got = findings(&dir, &[]);
    assert!(got.contains(&pair("src/Button.module.css", "css-modules/no-global")), "{got:?}");
    assert!(got.contains(&pair("src/Panel.module.scss", "css-modules/class-naming")), "{got:?}");
    let _ = fs::remove_dir_all(&dir);
}
