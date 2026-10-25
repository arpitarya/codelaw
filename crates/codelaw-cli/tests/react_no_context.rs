//! W-41: `react/no-context`'s adoption recipe, end to end on
//! `fixtures/react-no-context/` — old context warns through a baseline, new
//! context fails (SR-BASELINE decisions 1–2; no engine code of its own).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "react/no-context";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_AS_OF")
        .env_remove("CODELAW_CACHE_DIR")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap() + &String::from_utf8(out.stderr).unwrap())
}

/// Each `react/no-context` finding as `file severity`.
fn findings(cwd: &Path) -> (i32, Vec<String>) {
    let (code, out) = codelaw(cwd, &["check", "--as-of", "2026-10-25", "--rule", RULE, "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    let mut found: Vec<String> = v["diagnostics"].as_array().unwrap().iter().map(|d| format!("{} {}", d["path"].as_str().or(d["file"].as_str()).unwrap_or("?"), d["severity"].as_str().unwrap())).collect();
    found.sort();
    (code, found)
}

#[test]
fn old_context_warns_new_context_fails() {
    let d = std::env::temp_dir().join(format!("codelaw-react-no-context-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    copy_dir(&repo().join("fixtures/react-no-context"), &d);
    copy_dir(&repo().join("packs/react"), &d.join("node_modules/@codelaw/rules-react"));
    let d = fs::canonicalize(&d).unwrap();

    // Before the baseline: every use is an error.
    let (code, found) = findings(&d);
    assert_eq!(code, 1, "{found:?}");
    assert_eq!(found, ["src/Header.jsx error", "src/theme.js error", "src/theme.js error"]);

    // The recipe: grandfather today's uses, with no deadline.
    let (code, out) = codelaw(&d, &["baseline", "add", "--rule", RULE, "--warn-from", "2026-10-25", "--on-touch", "keep", "--reason", "legacy context, grandfathered"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("2 entries, 3 findings allowed"), "{out}");

    // Untouched old uses: warnings, exit 0.
    let (code, found) = findings(&d);
    assert_eq!(code, 0, "{found:?}");
    assert_eq!(found, ["src/Header.jsx warn", "src/theme.js warn", "src/theme.js warn"]);

    // A new file using context: an error; the old ones still warn.
    fs::write(d.join("src/Footer.jsx"), "import { useContext } from \"react\";\nimport { ThemeContext } from \"./theme\";\n\nexport function Footer() {\n  return <p>{useContext(ThemeContext)}</p>;\n}\n").unwrap();
    let (code, found) = findings(&d);
    assert_eq!(code, 1, "{found:?}");
    assert_eq!(found, ["src/Footer.jsx error", "src/Header.jsx warn", "src/theme.js warn", "src/theme.js warn"]);
    fs::remove_file(d.join("src/Footer.jsx")).unwrap();

    // One more use in an old file: every use in that file fails.
    let header = fs::read_to_string(d.join("src/Header.jsx")).unwrap();
    fs::write(d.join("src/Header.jsx"), header.replace("  return <h1", "  const again = React.useContext(ThemeContext);\n  return <h1 title={again}")).unwrap();
    let (code, out) = codelaw(&d, &["check", "--as-of", "2026-10-25", "--rule", RULE]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("(2 in this file; the baseline allows 1)"), "{out}");
    let (_, found) = findings(&d);
    assert_eq!(found, ["src/Header.jsx error", "src/Header.jsx error", "src/theme.js warn", "src/theme.js warn"]);

    // `explain` shows the recipe.
    let (code, out) = codelaw(&d, &["explain", RULE]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("codelaw baseline add --rule react/no-context --warn-from <today> --on-touch keep"), "{out}");

    let _ = fs::remove_dir_all(&d);
}
