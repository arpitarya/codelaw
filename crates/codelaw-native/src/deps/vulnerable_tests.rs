//! W-128 DoD 4: direct and transitive findings, the target, the fix edits,
//! the four severities, an ignored advisory, and the incomplete scan.

use super::*;
use crate::FileContext;
use serde_json::json;

struct Repo {
    path: String,
    files: BTreeMap<String, String>,
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
        Some("2026-10-29".into())
    }
}

const SNAPSHOT: &str = r#"{
  "fetched": "2026-10-29",
  "packages": {
    "npm:lodash": {
      "latest": "4.17.21",
      "versions": { "4.17.15": "2019-07-19", "4.17.20": "2020-08-13", "4.17.21": "2021-02-20" },
      "advisories": [
        { "id": "GHSA-35jh-r3h4-6jhm", "aliases": ["CVE-2021-23337"], "severity": "high", "affected": ["<4.17.21"], "fixed": ["4.17.21"], "sources": ["osv", "registry"] },
        { "id": "GHSA-p6mc-m468-83gw", "severity": "moderate", "affected": ["<4.17.20"], "fixed": ["4.17.20"], "sources": ["registry"] }
      ],
      "scanned": { "osv": ["*"] }
    },
    "npm:minimist": {
      "versions": { "1.2.5": "2020-03-12", "1.2.6": "2022-03-21", "1.2.8": "2023-02-01" },
      "advisories": [ { "id": "GHSA-xvch-5gv4-984h", "severity": "critical", "affected": ["<1.2.6"], "fixed": ["1.2.6"], "sources": ["osv"] } ],
      "scanned": { "osv": ["*"] }
    },
    "pypi:jinja2": {
      "versions": { "3.1.2": "2022-04-28", "3.1.3": "2024-01-10", "3.1.4": "2024-05-05" },
      "advisories": [ { "id": "GHSA-h5c8-rqwp-cp95", "aliases": ["CVE-2024-22195"], "severity": "moderate", "affected": [">=3.1.2 <3.1.3"], "fixed": ["3.1.3"], "sources": ["registry"] } ],
      "scanned": { "registry": ["3.1.2", "3.1.3"] }
    },
    "pypi:requests": {
      "latest": "2.32.3",
      "versions": { "2.31.0": "2023-05-22", "2.32.0": "2024-05-20", "2.32.3": "2024-05-29" },
      "advisories": [ { "id": "PYSEC-2024-1", "affected": [">=2.31.0 <2.32.0"], "fixed": ["2.32.0"], "sources": ["registry"] } ],
      "scanned": { "registry": ["2.31.0", "2.32.0", "2.32.3"] }
    }
  },
  "sources": {
    "osv": { "kind": "osv", "state": "unavailable", "on": "2026-10-29", "error": "HTTP 403" },
    "registry": { "kind": "registry", "state": "ok", "on": "2026-10-29" }
  }
}"#;

fn run(rule: &dyn NativeRule, path: &str, files: &[(&str, &str)], options: Value) -> Vec<(String, Option<Severity>, String)> {
    let mut map: BTreeMap<String, String> = files.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect();
    map.entry(SNAPSHOT_PATH.into()).or_insert_with(|| SNAPSHOT.into());
    let src = map[path].clone();
    let repo = Repo { path: path.into(), files: map };
    let lang = if path.ends_with(".json") {
        Language::Json
    } else if path.ends_with(".yaml") {
        Language::Yaml
    } else {
        Language::Toml
    };
    let options: Options = options.as_object().unwrap().clone().into_iter().collect();
    let root = codelaw_lang::parse(lang, &src);
    let data = BTreeMap::new();
    let ctx = NativeContext { source: &src, language: lang, semantic: None, options: &options, data: &data, file: &repo };
    let mut out = Vec::new();
    rule.check(root.root(), &ctx, &mut out);
    out.into_iter()
        .map(|f| {
            let sev = rule.severity_of(&f, &options).or(Some(Severity::Error));
            let mut fixed = src.clone();
            for e in f.edits.iter().rev() {
                fixed.replace_range(e.start..e.end, &e.text);
            }
            (f.message, sev, if f.edits.is_empty() { String::new() } else { fixed })
        })
        .collect()
}

#[test]
fn a_direct_dependency_moves_to_the_lowest_clean_version() {
    let pkg = "{\n  \"dependencies\": {\n    \"lodash\": \"^4.17.15\"\n  }\n}\n";
    let got = run(&Vulnerable, "package.json", &[("package.json", pkg)], json!({}));
    assert_eq!(got.len(), 1, "{got:?}");
    let (msg, sev, fixed) = &got[0];
    assert_eq!(
        msg,
        "`lodash` 4.17.15 is vulnerable — GHSA-35jh-r3h4-6jhm (high, CVE-2021-23337) and 1 more; upgrade to 4.17.21, then run `npm install` (snapshot 2026-10-29; incomplete: osv unavailable)"
    );
    assert_eq!(*sev, Some(Severity::Error));
    assert!(fixed.contains("\"lodash\": \"^4.17.21\""), "{fixed}");
    // `[deps] severity` grades it; `off` drops it.
    let got = run(&Vulnerable, "package.json", &[("package.json", pkg)], json!({ "severity": { "high": "warn" } }));
    assert_eq!(got[0].1, Some(Severity::Warn));
    assert!(run(&Vulnerable, "package.json", &[("package.json", pkg)], json!({ "severity": { "high": "off" } })).is_empty());
}

#[test]
fn a_dated_ignore_names_one_advisory() {
    let pkg = "{ \"dependencies\": { \"lodash\": \"4.17.15\" } }";
    let one = json!({ "ignore": [ { "package": "npm:lodash", "advisory": "CVE-2021-23337", "expires": "2027-01-01" } ] });
    let got = run(&Vulnerable, "package.json", &[("package.json", pkg)], one);
    assert_eq!(got.len(), 1);
    assert!(got[0].0.contains("GHSA-p6mc-m468-83gw (moderate)") && got[0].0.contains("upgrade to 4.17.21"), "{}", got[0].0);
    assert_eq!(got[0].1, Some(Severity::Warn), "the moderate one is left");
    let expired = json!({ "ignore": [ { "package": "npm:lodash", "advisory": "GHSA-35jh-r3h4-6jhm", "expires": "2026-10-01" } ] });
    assert!(run(&Vulnerable, "package.json", &[("package.json", pkg)], expired)[0].0.contains("GHSA-35jh"));
}

#[test]
fn a_transitive_dependency_sits_on_its_lockfile_entry() {
    let pkg = "{ \"dependencies\": { \"lodash\": \"^4.17.21\" } }";
    let lock = "{\n  \"lockfileVersion\": 3,\n  \"packages\": {\n    \"\": { \"dependencies\": { \"lodash\": \"^4.17.21\" } },\n    \"node_modules/lodash\": { \"version\": \"4.17.21\" },\n    \"node_modules/minimist\": { \"version\": \"1.2.5\" }\n  }\n}\n";
    let files = [("package.json", pkg), ("package-lock.json", lock)];
    assert!(run(&Vulnerable, "package.json", &files, json!({})).is_empty(), "the direct one is clean");
    let got = run(&Vulnerable, "package-lock.json", &files, json!({}));
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].0.starts_with("`minimist` 1.2.5 (installed through another package) is vulnerable — GHSA-xvch-5gv4-984h (critical)"), "{}", got[0].0);
    assert!(got[0].0.contains("add `\"overrides\": { \"minimist\": \"1.2.6\" }` to package.json, then run `npm install` — an override changes the install of the whole tree"), "{}", got[0].0);
    assert!(got[0].2.is_empty(), "a lockfile is never edited");
}

#[test]
fn yarn_and_uv_get_the_override_as_an_edit_on_the_manifest() {
    let pkg = "{\n  \"name\": \"app\",\n  \"dependencies\": {\n    \"lodash\": \"^4.17.21\"\n  }\n}\n";
    let yarn = "lodash@^4.17.21:\n  version \"4.17.21\"\n\nminimist@^1.2.0:\n  version \"1.2.5\"\n";
    let got = run(&Vulnerable, "package.json", &[("package.json", pkg), ("yarn.lock", yarn)], json!({}));
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].0.contains("in yarn.lock") && got[0].0.contains("through `resolutions` in package.json, then run `yarn install`"), "{}", got[0].0);
    assert!(got[0].2.contains("  \"resolutions\": {\n    \"minimist\": \"1.2.6\"\n  }"), "{}", got[0].2);
    assert!(serde_json::from_str::<Value>(&got[0].2).is_ok());
    assert_eq!(yarn_entry_line(yarn, "minimist", "1.2.5"), Some(4), "the copy on yarn.lock's entry (W-136)");
    assert_eq!(yarn_entry_line(yarn, "minimist", "9.9.9"), None);

    let py = "[project]\nname = \"app\"\ndependencies = [\"flask>=3\"]\n";
    let uv = "[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = { editable = \".\" }\n\n[[package]]\nname = \"Jinja2\"\nversion = \"3.1.2\"\n";
    let got = run(&Vulnerable, "pyproject.toml", &[("pyproject.toml", py), ("uv.lock", uv)], json!({}));
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].0.contains("upgrade to 3.1.3") && got[0].0.contains("`uv lock`"), "{}", got[0].0);
    assert!(got[0].2.ends_with("\n[tool.uv]\noverride-dependencies = [\"jinja2==3.1.3\"]\n"), "{}", got[0].2);
    let with_table = "[project]\nname = \"app\"\n\n[tool.uv]\noverride-dependencies = [\"x==1\"]\n";
    let got = run(&Vulnerable, "pyproject.toml", &[("pyproject.toml", with_table), ("uv.lock", uv)], json!({}));
    assert!(got[0].2.contains("override-dependencies = [\"jinja2==3.1.3\", \"x==1\"]"), "{}", got[0].2);
}

#[test]
fn an_ungraded_advisory_is_high_and_a_python_range_moves() {
    let py = "[project]\nname = \"app\"\ndependencies = [\"requests>=2.31.0\"]\n";
    let got = run(&Vulnerable, "pyproject.toml", &[("pyproject.toml", py)], json!({}));
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].0.contains("PYSEC-2024-1 (high — no source grades it); upgrade to 2.32.0"), "{}", got[0].0);
    assert_eq!(got[0].1, Some(Severity::Error));
    assert!(got[0].2.contains("\"requests>=2.32.0\""), "{}", got[0].2);
}

#[test]
fn the_scan_is_incomplete_while_an_enabled_source_is_unavailable() {
    let pkg = "{ \"name\": \"app\" }\n";
    let got = run(&IncompleteScan, "package.json", &[("package.json", pkg)], json!({}));
    assert_eq!(got.len(), 1);
    assert!(got[0].0.contains("`osv` (HTTP 403, on 2026-10-29)"), "{}", got[0].0);
    // Turned off in the config: complete as far as it goes.
    let off = json!({ "advisory_sources": [ { "name": "registry", "kind": "registry", "enabled": true }, { "name": "osv", "kind": "osv", "enabled": false } ] });
    assert!(run(&IncompleteScan, "package.json", &[("package.json", pkg)], off).is_empty());
    // Once per project: not on a nested manifest.
    assert!(run(&IncompleteScan, "web/package.json", &[("web/package.json", pkg), ("package.json", pkg)], json!({})).is_empty());
}

#[test]
fn check_dependency_answers_offline() {
    let snap = Snapshot::parse(SNAPSHOT).unwrap();
    let o = Options::new();
    let v = check_dependency(Some(&snap), &o, Ecosystem::Npm, "lodash", Some("4.17.15"), None);
    assert_eq!(v["vulnerable"], true);
    assert_eq!(v["target"], "4.17.21");
    assert_eq!(v["advisories"][0]["id"], "GHSA-35jh-r3h4-6jhm");
    assert_eq!(v["complete"], false);
    let v = check_dependency(Some(&snap), &o, Ecosystem::Npm, "lodash", None, None);
    assert_eq!((v["version"].as_str(), v["vulnerable"].as_bool()), (Some("4.17.21"), Some(false)), "no version: the latest");
    assert_eq!(check_dependency(Some(&snap), &o, Ecosystem::Pypi, "Nope", None, None)["known"], false);
    assert_eq!(check_dependency(None, &o, Ecosystem::Pypi, "x", None, None)["known"], false);
}
