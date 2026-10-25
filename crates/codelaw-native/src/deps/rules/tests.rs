//! W-52 DoD 3–5: each kind of finding at its severity, `min_release_age`,
//! and the fix keeping the range's operator.

use super::*;
use crate::FileContext;
use serde_json::json;

struct Repo {
    path: String,
    files: BTreeMap<String, String>,
    as_of: Option<String>,
}

impl FileContext for Repo {
    fn path(&self) -> &str {
        &self.path
    }
    fn bytes(&self) -> &[u8] {
        self.files[&self.path].as_bytes()
    }
    fn exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }
    fn is_dir(&self, _: &str) -> bool {
        false
    }
    fn read(&self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }
    fn list(&self, _: &str) -> Option<Vec<String>> {
        None
    }
    fn as_of(&self) -> Option<String> {
        self.as_of.clone()
    }
}

const SNAPSHOT: &str = r#"{
  "fetched": "2026-10-25",
  "registries": { "npm": "https://registry.npmjs.org", "pypi": "https://pypi.org/simple" },
  "packages": {
    "npm:react": { "latest": "19.2.0", "versions": { "18.3.1": "2024-04-26", "18.3.2": "2024-06-01", "19.2.0": "2025-10-01", "19.3.0": "2026-10-22" } },
    "npm:lodash": { "latest": "4.17.21", "versions": { "4.17.20": "2020-08-13", "4.17.21": "2021-02-20" } },
    "npm:chalk": { "latest": "5.4.0", "versions": { "5.3.0": "2023-06-29", "5.4.0": "2024-12-01" } },
    "npm:left-pad": { "latest": "1.3.0", "versions": { "1.3.0": "2018-04-09" }, "deprecated": { "1.3.0": "use String.prototype.padStart()" } },
    "npm:tilde": { "latest": "2.5.0", "versions": { "1.2.3": "2020-01-01", "2.5.0": "2024-01-01" } },
    "npm:pinned": { "latest": "3.0.1", "versions": { "2.0.0": "2020-01-01", "3.0.1": "2024-01-01" } },
    "npm:floor": { "latest": "2.0.0", "versions": { "1.0.0": "2020-01-01", "2.0.0": "2024-01-01" } },
    "pypi:requests": { "latest": "2.32.3", "versions": { "2.31.0": "2023-05-22", "2.32.3": "2024-05-29" } },
    "pypi:django": { "latest": "5.1.2", "versions": { "4.2.0": "2023-04-03", "5.1.2": "2024-10-08" } },
    "pypi:flask": { "latest": "3.0.3", "versions": { "2.0.0": "2021-05-11", "3.0.3": "2024-04-07" } },
    "pypi:bad": { "latest": "1.1", "versions": { "1.0": "2020-01-01", "1.1": "2021-01-01" }, "yanked": { "1.0": "broken wheel" } }
  }
}"#;

fn run(rule: &dyn NativeRule, path: &str, src: &str, extra: &[(&str, &str)], options: Options, as_of: Option<&str>) -> Vec<(String, Option<Severity>, String)> {
    let mut files: BTreeMap<String, String> = extra.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect();
    files.insert(path.into(), src.into());
    let repo = Repo { path: path.into(), files, as_of: as_of.map(str::to_owned) };
    let lang = if path.ends_with(".json") { Language::Json } else { Language::Toml };
    let root = codelaw_lang::parse(lang, src);
    let data = BTreeMap::new();
    let ctx = NativeContext { source: src, language: lang, semantic: None, options: &options, data: &data, file: &repo };
    let mut out = Vec::new();
    rule.check(root.root(), &ctx, &mut out);
    out.into_iter()
        .map(|f| {
            let sev = rule.severity_of(&f, &options);
            let mut fixed = src.to_owned();
            for e in f.edits.iter().rev() {
                fixed.replace_range(e.start..e.end, &e.text);
            }
            (f.message, sev, if f.edits.is_empty() { String::new() } else { fixed })
        })
        .collect()
}

fn opts(v: Value) -> Options {
    v.as_object().unwrap().clone().into_iter().collect()
}

#[test]
fn each_kind_has_its_severity() {
    let pkg = r#"{
  "dependencies": { "react": "^18.3.1", "lodash": "^4.17.20", "left-pad": "1.3.0", "chalk": "^5.3.0" }
}"#;
    let lock = r#"{ "packages": { "node_modules/react": { "version": "18.3.1" }, "node_modules/lodash": { "version": "4.17.20" } } }"#;
    let got = run(&OutdatedDependency, "package.json", pkg, &[(SNAPSHOT_PATH, SNAPSHOT), ("package-lock.json", lock)], Options::new(), None);
    let m: Vec<(&str, Option<Severity>)> = got.iter().map(|(m, s, _)| (m.as_str(), *s)).collect();
    assert_eq!(
        m,
        [
            ("`react` ^18.3.1 — installed 18.3.1, latest 19.3.0 (major)", Some(Severity::Warn)),
            ("`lodash` ^4.17.20 allows 4.17.21, but the lockfile has 4.17.20: run your install to pick it up (lockfile)", Some(Severity::Info)),
            ("`left-pad` 1.3.0 — 1.3.0 is deprecated: use String.prototype.padStart() (deprecated)", Some(Severity::Error)),
        ],
        "chalk's ^5.3.0 allows 5.4.0 and there is no locked chalk, so there is nothing to say"
    );
    let minor = run(&OutdatedDependency, "package.json", r#"{ "dependencies": { "chalk": "~5.3.0" } }"#, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), None);
    assert_eq!(minor[0].0, "`chalk` ~5.3.0 — latest 5.4.0 (minor)");
    assert_eq!(minor[0].1, Some(Severity::Info));
    let patch = run(&OutdatedDependency, "package.json", r#"{ "dependencies": { "lodash": "4.17.20" } }"#, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), None);
    assert_eq!(patch[0].0, "`lodash` 4.17.20 — latest 4.17.21 (patch)");
    assert_eq!(patch[0].1, Some(Severity::Info));

    let py = "[project]\ndependencies = [\"bad==1.0\", \"requests>=2.31,<2.32\"]\n";
    let got = run(&OutdatedDependency, "pyproject.toml", py, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), None);
    assert_eq!(got[0].0, "`bad==1.0` — 1.0 is yanked: broken wheel; latest 1.1 (yanked)");
    assert_eq!(got[0].1, Some(Severity::Error));
    assert_eq!(got[1].0, "`requests>=2.31,<2.32` — latest 2.32.3 (minor)");
    assert_eq!(got[1].2, "", "the upper bound refuses 2.32.3, so no bump is offered");

    // `[deps] severity` regrades and switches off.
    let o = opts(json!({ "severity": { "major": "error", "lockfile": "off" } }));
    let got = run(&OutdatedDependency, "package.json", pkg, &[(SNAPSHOT_PATH, SNAPSHOT), ("package-lock.json", lock)], o, None);
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].1, Some(Severity::Error));
    assert!(got[1].0.ends_with("(deprecated)"));
}

#[test]
fn min_release_age_hides_a_young_release() {
    let pkg = r#"{ "dependencies": { "react": "^18.3.1" } }"#;
    // 19.3.0 was published 2026-10-22, three days before `fetched`.
    let open = run(&OutdatedDependency, "package.json", pkg, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), None);
    assert!(open[0].0.contains("latest 19.3.0"), "{open:?}");
    let held = run(&OutdatedDependency, "package.json", pkg, &[(SNAPSHOT_PATH, SNAPSHOT)], opts(json!({ "min_release_age": "7d" })), None);
    assert!(held[0].0.contains("latest 19.2.0"), "{held:?}");
    let exact = run(&OutdatedDependency, "package.json", pkg, &[(SNAPSHOT_PATH, SNAPSHOT)], opts(json!({ "min_release_age": 3 })), None);
    assert!(exact[0].0.contains("latest 19.3.0"), "three days old is old enough for 3: {exact:?}");
}

#[test]
fn the_fix_keeps_the_operator() {
    let cases = [
        (r#"{ "dependencies": { "react": "^18.3.1" } }"#, r#"{ "dependencies": { "react": "^19.3.0" } }"#),
        (r#"{ "dependencies": { "tilde": "~1.2.3" } }"#, r#"{ "dependencies": { "tilde": "~2.5.0" } }"#),
        (r#"{ "dependencies": { "tilde": "~1.2" } }"#, r#"{ "dependencies": { "tilde": "~2.5" } }"#),
        (r#"{ "dependencies": { "pinned": "2.0.0" } }"#, r#"{ "dependencies": { "pinned": "3.0.1" } }"#),
        (r#"{ "dependencies": { "pinned": "=2.0.0" } }"#, r#"{ "dependencies": { "pinned": "=3.0.1" } }"#),
    ];
    for (src, want) in cases {
        let got = run(&OutdatedDependency, "package.json", src, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), None);
        assert_eq!(got[0].2, want, "{src}");
    }
    let py = [
        ("[project]\ndependencies = [\"requests>=2.31\"]\n", ""),
        ("[project]\ndependencies = [\"Django~=4.2\"]\n", "[project]\ndependencies = [\"Django~=5.1\"]\n"),
        ("[project]\ndependencies = [\"flask==2.0.0\"]\n", "[project]\ndependencies = [\"flask==3.0.3\"]\n"),
        ("[dependency-groups]\ndev = [\"flask >= 2.0 ; python_version > '3.8'\"]\n", ""),
    ];
    for (src, want) in py {
        let got = run(&OutdatedDependency, "pyproject.toml", src, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), None);
        assert_eq!(got.first().map(|g| g.2.as_str()).unwrap_or(""), want, "{src}: {got:?}");
    }
    // `>=` already allows the latest: no lockfile, so nothing to say.
    let floor = run(&OutdatedDependency, "package.json", r#"{ "dependencies": { "floor": ">=1.0.0" } }"#, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), None);
    assert!(floor.is_empty());
}

#[test]
fn ignore_holds_until_it_expires() {
    let pkg = r#"{ "dependencies": { "react": "^18.3.1" } }"#;
    let files = [(SNAPSHOT_PATH, SNAPSHOT)];
    assert!(run(&OutdatedDependency, "package.json", pkg, &files, opts(json!({ "ignore": ["npm:react"] })), None).is_empty());
    let dated = opts(json!({ "ignore": [{ "package": "npm:react", "expires": "2026-11-01" }] }));
    assert!(run(&OutdatedDependency, "package.json", pkg, &files, dated.clone(), Some("2026-10-31")).is_empty());
    assert_eq!(run(&OutdatedDependency, "package.json", pkg, &files, dated, Some("2026-11-01")).len(), 1);
}

#[test]
fn no_snapshot_no_finding() {
    let pkg = r#"{ "dependencies": { "react": "^18.3.1" } }"#;
    assert!(run(&OutdatedDependency, "package.json", pkg, &[], Options::new(), None).is_empty());
    assert!(run(&StaleDepsSnapshot, "package.json", pkg, &[], Options::new(), Some("2027-01-01")).is_empty());
}

#[test]
fn a_stale_snapshot_is_reported_once_at_the_root() {
    let pkg = "{}\n";
    let py = "[project]\nname = \"a\"\n";
    let files = [(SNAPSHOT_PATH, SNAPSHOT), ("package.json", pkg)];
    assert!(run(&StaleDepsSnapshot, "package.json", pkg, &files, Options::new(), Some("2026-11-08")).is_empty(), "14 days is not more than 14");
    let stale = run(&StaleDepsSnapshot, "package.json", pkg, &files, Options::new(), Some("2026-11-09"));
    assert_eq!(stale.len(), 1);
    assert!(stale[0].0.contains("15 days before the as-of date 2026-11-09"), "{stale:?}");
    assert!(run(&StaleDepsSnapshot, "pyproject.toml", py, &files, Options::new(), Some("2026-11-09")).is_empty(), "the root package.json carries it");
    assert_eq!(run(&StaleDepsSnapshot, "pyproject.toml", py, &[(SNAPSHOT_PATH, SNAPSHOT)], Options::new(), Some("2026-11-09")).len(), 1);
    assert!(run(&StaleDepsSnapshot, "web/package.json", pkg, &files, Options::new(), Some("2026-11-09")).is_empty());
    assert!(run(&StaleDepsSnapshot, "package.json", pkg, &files, opts(json!({ "max_snapshot_age": "30d" })), Some("2026-11-09")).is_empty());
    assert!(run(&StaleDepsSnapshot, "package.json", pkg, &files, Options::new(), None).is_empty(), "no as-of date, nothing to judge by");
    let broken = run(&StaleDepsSnapshot, "package.json", pkg, &[(SNAPSHOT_PATH, "{ not json")], Options::new(), None);
    assert!(broken[0].0.contains("cannot be read"));
}
