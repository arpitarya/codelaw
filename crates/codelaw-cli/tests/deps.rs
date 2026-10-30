//! W-52 through the binary: `codelaw deps refresh` against a registry this
//! test serves on 127.0.0.1 (never the real network), then `check` and
//! `fix --unsafe` reading the snapshot it wrote (SR-RULE-PACKS decision 15).
//!
//! DoD 1: a sorted snapshot, the same bytes on a second run. DoD 6: a
//! private npm registry named in `.npmrc`, its token from the environment,
//! sent and never written.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

/// One request the registry saw: its path and its `Authorization` header.
type Seen = Arc<Mutex<Vec<(String, Option<String>)>>>;

/// What the server answers a `POST`: the path and body give the status and
/// body (W-128's advisory endpoints).
type Post = Arc<dyn Fn(&str, &str) -> Option<(&'static str, String)> + Send + Sync>;

const TOKEN: &str = "s3cret-token";

/// A registry on 127.0.0.1: npm packuments at `/<name>`, a private scope
/// that wants `Bearer s3cret-token`, and PEP 691 pages at `/simple/<name>/`.
fn registry() -> (u16, Seen) {
    serve(BTreeMap::new(), Arc::new(|path: &str, _: &str| (path == "/-/npm/v1/security/advisories/bulk").then(|| ("200 OK", "{}".to_owned()))))
}

/// The registry of [`registry`], with more pages and a `POST` handler.
fn serve(extra: BTreeMap<&'static str, String>, post: Post) -> (u16, Seen) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let mut pages: BTreeMap<&str, String> = BTreeMap::new();
    pages.insert(
        "/react",
        r#"{"name":"react","dist-tags":{"latest":"19.2.0"},
            "versions":{"17.0.2":{},"18.3.1":{},"19.2.0":{},"19.3.0-canary.1":{}},
            "time":{"created":"2013-05-29T00:00:00.000Z","17.0.2":"2021-03-22T00:00:00.000Z","18.3.1":"2024-04-26T16:42:00.000Z","19.2.0":"2025-10-01T00:00:00.000Z","19.3.0-canary.1":"2026-10-20T00:00:00.000Z"}}"#
            .into(),
    );
    pages.insert(
        "/left-pad",
        r#"{"name":"left-pad","versions":{"1.3.0":{"deprecated":"use String.prototype.padStart()"}},"time":{"1.3.0":"2018-04-09T00:00:00.000Z"}}"#.into(),
    );
    pages.insert("/@acme%2fui", r#"{"name":"@acme/ui","versions":{"1.0.0":{},"2.1.0":{}},"time":{"1.0.0":"2025-01-01T00:00:00Z","2.1.0":"2026-09-01T00:00:00Z"}}"#.into());
    pages.insert(
        "/simple/requests/",
        r#"{"meta":{"api-version":"1.1"},"name":"requests","versions":["2.31.0","2.32.3","2.33.0rc1"],
            "files":[
              {"filename":"requests-2.31.0-py3-none-any.whl","url":"x","hashes":{},"upload-time":"2023-05-22T15:12:44Z"},
              {"filename":"requests-2.31.0.tar.gz","url":"x","hashes":{},"upload-time":"2023-05-22T15:12:40Z"},
              {"filename":"requests-2.32.3-py3-none-any.whl","url":"x","hashes":{},"upload-time":"2024-05-29T15:37:47Z"},
              {"filename":"requests-2.33.0rc1-py3-none-any.whl","url":"x","hashes":{},"upload-time":"2026-10-01T00:00:00Z"}]}"#
            .into(),
    );
    pages.insert(
        "/simple/bad/",
        r#"{"meta":{"api-version":"1.1"},"name":"bad","versions":["1.0","1.1"],
            "files":[{"filename":"bad-1.0.tar.gz","url":"x","hashes":{},"upload-time":"2020-01-01T00:00:00Z","yanked":"broken wheel"},
                     {"filename":"bad-1.1.tar.gz","url":"x","hashes":{},"upload-time":"2021-01-01T00:00:00Z","yanked":false}]}"#
            .into(),
    );
    pages.extend(extra);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                continue;
            }
            let method = line.split_whitespace().next().unwrap_or("").to_owned();
            let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
            let mut auth = None;
            let mut length = 0usize;
            loop {
                let mut h = String::new();
                if reader.read_line(&mut h).unwrap_or(0) == 0 || h == "\r\n" {
                    break;
                }
                if let Some((k, v)) = h.split_once(':') {
                    if k.eq_ignore_ascii_case("authorization") {
                        auth = Some(v.trim().to_owned());
                    } else if k.eq_ignore_ascii_case("content-length") {
                        length = v.trim().parse().unwrap_or(0);
                    }
                }
            }
            let mut body = vec![0u8; length];
            if length > 0 {
                let _ = std::io::Read::read_exact(&mut reader, &mut body);
            }
            let body = String::from_utf8_lossy(&body).into_owned();
            log.lock().unwrap().push((if method == "POST" { format!("POST {path} {body}") } else { path.clone() }, auth.clone()));
            let private = path.starts_with("/@acme");
            let (status, body) = match (method.as_str(), pages.get(path.as_str())) {
                ("POST", _) => post(&path, &body).unwrap_or(("404 Not Found", "{}".to_owned())),
                (_, Some(_)) if private && auth.as_deref() != Some(&format!("Bearer {TOKEN}")) => ("401 Unauthorized", "{}".to_owned()),
                (_, Some(b)) => ("200 OK", b.clone()),
                (_, None) => ("404 Not Found", "{}".to_owned()),
            };
            let _ = write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        }
    });
    (port, seen)
}

/// A project with npm and Python manifests, a lockfile, the data pack and
/// an `.npmrc` that names the private scope and its token variable.
fn project(name: &str, port: u16) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-deps-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    let d = fs::canonicalize(d).unwrap();
    copy_dir(&repo_root().join("packs/data"), &d.join("node_modules/@codelaw/rules-data"));
    copy_dir(&repo_root().join("packs/deps"), &d.join("node_modules/@codelaw/rules-deps"));
    // One advisory source, on this test's registry: the tests never reach the network.
    fs::write(
        d.join("codelaw.toml"),
        "extends = [\"codelaw:data/package-json\", \"codelaw:data/pyproject\", \"codelaw:deps/recommended\"]\npreview = true\n\n[deps]\nmin_release_age = \"7d\"\nadvisory_sources = [{ name = \"registry\", kind = \"registry\" }]\n",
    )
    .unwrap();
    fs::write(
        d.join("package.json"),
        "{\n  \"name\": \"app\",\n  \"dependencies\": {\n    \"react\": \"^18.3.1\",\n    \"left-pad\": \"1.3.0\",\n    \"@acme/ui\": \"^1.0.0\",\n    \"local\": \"workspace:*\"\n  }\n}\n",
    )
    .unwrap();
    fs::write(d.join("package-lock.json"), r#"{"packages":{"node_modules/react":{"version":"18.3.1"}}}"#).unwrap();
    fs::write(d.join("pyproject.toml"), "[project]\nname = \"app\"\ndependencies = [\"requests>=2.31,<3\", \"bad==1.0\", \"missing>=1\"]\n").unwrap();
    fs::write(d.join(".npmrc"), format!("registry=http://127.0.0.1:{port}/\n@acme:registry=http://127.0.0.1:{port}/\n//127.0.0.1:{port}/:_authToken=${{ACME_NPM_TOKEN}}\n")).unwrap();
    d
}

fn codelaw(cwd: &Path, args: &[&str], env: &[(&str, &str)], port: u16) -> (i32, String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.args(args).current_dir(cwd).env("HOME", cwd).env("NO_COLOR", "1");
    for k in ["VIRTUAL_ENV", "CODELAW_AS_OF", "CODELAW_CACHE_DIR", "CODELAW_SITE_PACKAGES", "UV_DEFAULT_INDEX", "UV_INDEX_URL", "NPM_CONFIG_REGISTRY", "npm_config_registry"] {
        c.env_remove(k);
    }
    for k in ["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "all_proxy"] {
        c.env_remove(k);
    }
    c.env("NO_PROXY", "*").env("PIP_INDEX_URL", format!("http://127.0.0.1:{port}/simple"));
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
}

#[test]
fn refresh_writes_a_sorted_snapshot_and_check_reads_it() {
    let (port, seen) = registry();
    let d = project("refresh", port);
    let token = [("ACME_NPM_TOKEN", TOKEN)];
    let (code, out, err) = codelaw(&d, &["deps", "refresh", "--as-of", "2026-10-25"], &token, port);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("wrote .codelaw/deps-snapshot.json: 3 npm and 2 PyPI package(s), fetched 2026-10-25; 0 advisories on 0 vulnerable package(s) (sources: registry ok)"), "{out}");
    assert!(err.contains("`pypi:missing` is not on its registry"), "{err}");
    let first = fs::read_to_string(d.join(".codelaw/deps-snapshot.json")).unwrap();

    // DoD 1: the same registry gives the same bytes.
    let (code, _, err) = codelaw(&d, &["deps", "refresh", "--as-of", "2026-10-25"], &token, port);
    assert_eq!(code, 0, "{err}");
    assert_eq!(fs::read_to_string(d.join(".codelaw/deps-snapshot.json")).unwrap(), first, "a second refresh wrote different bytes");

    let snap: serde_json::Value = serde_json::from_str(&first).unwrap();
    let keys: Vec<&String> = snap["packages"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["npm:@acme/ui", "npm:left-pad", "npm:react", "pypi:bad", "pypi:requests"], "sorted, and `workspace:*` is not the registry's");
    let react = &snap["packages"]["npm:react"];
    assert_eq!(react["latest"], "19.2.0");
    assert_eq!(react["versions"], serde_json::json!({ "18.3.1": "2024-04-26", "19.2.0": "2025-10-01" }), "17.0.2 is below the version in use; the canary is a prerelease");
    assert_eq!(snap["packages"]["npm:left-pad"]["deprecated"]["1.3.0"], "use String.prototype.padStart()");
    assert_eq!(snap["packages"]["pypi:requests"]["latest"], "2.32.3");
    assert_eq!(snap["packages"]["pypi:bad"]["yanked"]["1.0"], "broken wheel");
    assert_eq!(snap["registries"]["npm:@acme"], format!("http://127.0.0.1:{port}/"));
    assert_eq!(snap["registries"]["pypi"], format!("http://127.0.0.1:{port}/simple/"));

    // DoD 6: the token went to the private scope, and nowhere on disk.
    let log = seen.lock().unwrap().clone();
    assert!(log.iter().any(|(p, a)| p == "/@acme%2fui" && a.as_deref() == Some(&format!("Bearer {TOKEN}") as &str)), "{log:?}");
    assert!(!first.contains(TOKEN), "the token was written into the snapshot");
    let (code, _, err) = codelaw(&d, &["deps", "refresh", "--as-of", "2026-10-26"], &[], port);
    assert_eq!(code, 2, "without the token the private registry refuses: {err}");
    assert!(err.contains("refused the request (401)") && err.contains("the snapshot was not written"), "{err}");
    assert_eq!(fs::read_to_string(d.join(".codelaw/deps-snapshot.json")).unwrap(), first, "a failed refresh leaves the snapshot alone");

    // check reads the snapshot: every kind of finding, at its severity.
    let (_, out, err) = codelaw(&d, &["check", "--no-cache", "--format", "json", "--as-of", "2026-10-25", "--rule", "deps/outdated"], &[], port);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    let got: Vec<(String, String, String)> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["file"].as_str().unwrap().to_owned(), d["severity"].as_str().unwrap().to_owned(), d["message"].as_str().unwrap().to_owned()))
        .collect();
    let want = [
        ("package.json", "warn", "`react` ^18.3.1 — installed 18.3.1, latest 19.2.0 (major)"),
        ("package.json", "error", "`left-pad` 1.3.0 — 1.3.0 is deprecated: use String.prototype.padStart() (deprecated)"),
        ("package.json", "warn", "`@acme/ui` ^1.0.0 — latest 2.1.0 (major)"),
        ("pyproject.toml", "error", "`bad==1.0` — 1.0 is yanked: broken wheel; latest 1.1 (yanked)"),
    ];
    let want: Vec<(String, String, String)> = want.iter().map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string())).collect();
    assert_eq!(got, want, "requests>=2.31,<3 already allows 2.32.3 and nothing is locked");

    // The suggest fix keeps each range's operator.
    let (_, _, err) = codelaw(&d, &["fix", "--no-cache", "--unsafe", "--as-of", "2026-10-25", "--rule", "deps/outdated"], &[], port);
    let pkg = fs::read_to_string(d.join("package.json")).unwrap();
    assert!(pkg.contains("\"react\": \"^19.2.0\"") && pkg.contains("\"@acme/ui\": \"^2.1.0\"") && pkg.contains("\"left-pad\": \"1.3.0\""), "{pkg}{err}");
    assert!(fs::read_to_string(d.join("pyproject.toml")).unwrap().contains("\"bad==1.1\""), "{err}");

    // deps/stale-snapshot, judged on the as-of date.
    let stale = |date: &str| {
        let (_, out, _) = codelaw(&d, &["check", "--no-cache", "--format", "json", "--as-of", date, "--rule", "deps/stale-snapshot"], &[], port);
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["diagnostics"].as_array().unwrap().len()
    };
    assert_eq!(stale("2026-11-08"), 0);
    assert_eq!(stale("2026-11-09"), 1, "15 days old is past the default 14");
    let _ = fs::remove_dir_all(d);
}

#[test]
fn refresh_refuses_a_bad_date_and_a_dead_registry() {
    let d = project("dead", 9);
    let (code, _, err) = codelaw(&d, &["deps", "refresh", "--as-of", "25-10-2026"], &[], 9);
    assert_eq!(code, 2);
    assert!(err.contains("is not a date"), "{err}");
    // Port 9 (discard) on 127.0.0.1 has nothing listening: the request fails.
    let (code, _, err) = codelaw(&d, &["deps", "refresh", "--as-of", "2026-10-25"], &[], 9);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("the snapshot was not written"), "{err}");
    assert!(!d.join(".codelaw/deps-snapshot.json").exists());
    let _ = fs::remove_dir_all(d);
}

/// W-128 DoD 3 and 6: three advisory sources on 127.0.0.1 — the registry,
/// an OSV that disagrees with it, and one that refuses (403) — and one
/// turned off. The refresh merges what answers, records the refusal, and
/// the rules read the result offline.
#[test]
fn advisories_from_every_enabled_source_are_merged() {
    let mut pages: BTreeMap<&'static str, String> = BTreeMap::new();
    pages.insert(
        "/lodash",
        r#"{"name":"lodash","versions":{"4.17.15":{},"4.17.20":{},"4.17.21":{}},"time":{"4.17.15":"2019-07-19T00:00:00Z","4.17.20":"2020-08-13T00:00:00Z","4.17.21":"2021-02-20T00:00:00Z"}}"#.into(),
    );
    pages.insert(
        "/minimist",
        r#"{"name":"minimist","versions":{"1.2.5":{},"1.2.6":{},"1.2.8":{}},"time":{"1.2.5":"2020-03-12T00:00:00Z","1.2.6":"2022-03-21T00:00:00Z","1.2.8":"2023-02-01T00:00:00Z"}}"#.into(),
    );
    pages.insert("/lodash/4.17.15", r#"{"name":"lodash","version":"4.17.15","license":"MIT"}"#.into());
    pages.insert(
        "/simple/jinja2/",
        r#"{"meta":{"api-version":"1.1"},"name":"jinja2","versions":["3.1.2","3.1.3","3.1.4"],"files":[
            {"filename":"jinja2-3.1.2-py3-none-any.whl","url":"x","hashes":{},"upload-time":"2022-04-28T00:00:00Z"},
            {"filename":"jinja2-3.1.3-py3-none-any.whl","url":"x","hashes":{},"upload-time":"2024-01-10T00:00:00Z"},
            {"filename":"jinja2-3.1.4-py3-none-any.whl","url":"x","hashes":{},"upload-time":"2024-05-05T00:00:00Z"}]}"#
            .into(),
    );
    pages.insert(
        "/pypi/jinja2/3.1.2/json",
        r#"{"info":{"license_expression":"BSD-3-Clause"},"vulnerabilities":[
            {"id":"PYSEC-2024-9","aliases":["CVE-2024-22195"],"details":"never kept","fixed_in":["3.1.3"],"link":"https://osv.dev/vulnerability/PYSEC-2024-9","withdrawn":null},
            {"id":"PYSEC-2020-1","aliases":[],"fixed_in":["3.1.3"],"withdrawn":"2021-01-01T00:00:00Z"}]}"#
            .into(),
    );
    pages.insert("/pypi/jinja2/3.1.3/json", r#"{"info":{"license":"BSD-3-Clause"},"vulnerabilities":[]}"#.into());
    pages.insert("/pypi/jinja2/3.1.4/json", r#"{"info":{"license":"BSD-3-Clause"},"vulnerabilities":[]}"#.into());
    let post: Post = Arc::new(|path: &str, body: &str| {
        let ask: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        match path {
            "/-/npm/v1/security/advisories/bulk" => {
                let mut out = serde_json::Map::new();
                if ask.get("lodash").is_some() {
                    out.insert("lodash".into(), serde_json::json!([{ "id": 1, "url": "https://github.com/advisories/GHSA-35jh-r3h4-6jhm", "title": "never kept", "severity": "high", "vulnerable_versions": "<4.17.21" }]));
                }
                if ask.get("minimist").is_some() {
                    out.insert("minimist".into(), serde_json::json!([{ "id": 2, "url": "https://github.com/advisories/GHSA-xvch-5gv4-984h", "severity": "critical", "vulnerable_versions": "<1.2.6" }]));
                }
                Some(("200 OK", serde_json::Value::Object(out).to_string()))
            }
            "/osv/v1/query" => {
                let name = ask["package"]["name"].as_str().unwrap_or_default();
                let eco = ask["package"]["ecosystem"].as_str().unwrap_or_default();
                let vulns = match (eco, name) {
                    // Disagrees: a lower severity and an earlier fix than the registry's.
                    ("npm", "lodash") => serde_json::json!([{ "id": "GHSA-35jh-r3h4-6jhm", "aliases": ["CVE-2021-23337"], "database_specific": { "severity": "MODERATE" },
                        "affected": [{ "package": { "ecosystem": "npm", "name": "lodash" }, "ranges": [{ "type": "SEMVER", "events": [{ "introduced": "0" }, { "fixed": "4.17.20" }] }] }] }]),
                    ("PyPI", "jinja2") => serde_json::json!([{ "id": "GHSA-h5c8-rqwp-cp95", "aliases": ["CVE-2024-22195", "PYSEC-2024-9"], "database_specific": { "severity": "MODERATE" },
                        "affected": [{ "package": { "ecosystem": "PyPI", "name": "jinja2" }, "ranges": [{ "type": "ECOSYSTEM", "events": [{ "introduced": "0" }, { "fixed": "3.1.3" }] }] }] }]),
                    _ => serde_json::json!([]),
                };
                Some(("200 OK", serde_json::json!({ "vulns": vulns }).to_string()))
            }
            "/blocked/v1/query" => Some(("403 Forbidden", "{}".to_owned())),
            _ => None,
        }
    });
    let (port, seen) = serve(pages, post);
    let d = std::env::temp_dir().join(format!("codelaw-deps-advisories-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    let d = fs::canonicalize(d).unwrap();
    copy_dir(&repo_root().join("packs/deps"), &d.join("node_modules/@codelaw/rules-deps"));
    let sources = format!(
        "advisory_sources = [\n  {{ name = \"registry\", kind = \"registry\" }},\n  {{ name = \"osv\", kind = \"osv\", url = \"http://127.0.0.1:{port}/osv\" }},\n  {{ name = \"blocked\", kind = \"osv\", url = \"http://127.0.0.1:{port}/blocked\" }},\n  {{ name = \"off\", kind = \"osv\", url = \"http://127.0.0.1:{port}/never\", enabled = false }},\n]\n"
    );
    fs::write(d.join("codelaw.toml"), format!("extends = [\"codelaw:deps/recommended\"]\npreview = true\n\n[deps]\n{sources}")).unwrap();
    fs::write(d.join("package.json"), "{\n  \"name\": \"app\",\n  \"dependencies\": {\n    \"lodash\": \"^4.17.15\"\n  }\n}\n").unwrap();
    fs::write(
        d.join("package-lock.json"),
        "{\n  \"lockfileVersion\": 3,\n  \"packages\": {\n    \"\": { \"dependencies\": { \"lodash\": \"^4.17.15\" } },\n    \"node_modules/lodash\": { \"version\": \"4.17.15\" },\n    \"node_modules/minimist\": { \"version\": \"1.2.5\" }\n  }\n}\n",
    )
    .unwrap();
    fs::write(d.join("pyproject.toml"), "[project]\nname = \"app\"\ndependencies = [\"jinja2>=3.1.2\"]\n").unwrap();
    fs::write(d.join(".npmrc"), format!("registry=http://127.0.0.1:{port}/\n")).unwrap();

    let (code, out, err) = codelaw(&d, &["deps", "refresh", "--as-of", "2026-10-29"], &[], port);
    assert_eq!(code, 0, "{out}{err}");
    assert!(err.contains("the advisory source `blocked` is unavailable") && err.contains("HTTP 403"), "{err}");
    assert!(out.contains("3 advisories on 3 vulnerable package(s) (sources: blocked unavailable, osv ok, registry ok)"), "{out}");
    let log = seen.lock().unwrap().clone();
    assert!(!log.iter().any(|(p, _)| p.contains("/never")), "a source turned off was read: {log:?}");
    let text = fs::read_to_string(d.join(".codelaw/deps-snapshot.json")).unwrap();
    assert!(!text.contains("never kept"), "advisory text was kept (CC-BY): {text}");
    let snap: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(snap["sources"]["blocked"]["state"], "unavailable");
    assert!(snap["sources"]["blocked"]["error"].as_str().unwrap().contains("403"));
    assert!(snap["sources"].get("off").is_none());

    // The merge: one record per vulnerability, the safer answer winning.
    let lodash = &snap["packages"]["npm:lodash"]["advisories"][0];
    assert_eq!(lodash["id"], "GHSA-35jh-r3h4-6jhm");
    assert_eq!(lodash["severity"], "high", "the higher severity");
    assert_eq!(lodash["sources"], serde_json::json!(["osv", "registry"]));
    assert_eq!(lodash["affected"], serde_json::json!(["<4.17.20", "<4.17.21"]), "affected if any source says so");
    assert_eq!(snap["packages"]["npm:lodash"]["licence"]["4.17.15"], "MIT");
    let jinja = &snap["packages"]["pypi:jinja2"];
    assert_eq!(jinja["advisories"].as_array().unwrap().len(), 1, "the withdrawn record is dropped; PYSEC and GHSA are one: {jinja}");
    assert_eq!(jinja["advisories"][0]["id"], "GHSA-h5c8-rqwp-cp95");
    assert_eq!(jinja["advisories"][0]["aliases"], serde_json::json!(["CVE-2024-22195", "PYSEC-2024-9"]));
    assert_eq!(jinja["scanned"]["registry"], serde_json::json!(["3.1.2", "3.1.3", "3.1.4"]), "the version in use, each fixed_in and the latest");
    assert_eq!(jinja["licence"]["3.1.2"], "BSD-3-Clause");
    assert!(snap["packages"]["npm:minimist"]["advisories"][0]["id"] == "GHSA-xvch-5gv4-984h", "the transitive one: {text}");

    // The rules read it offline.
    let (_, out, err) = codelaw(&d, &["check", "--no-cache", "--format", "json", "--as-of", "2026-10-29"], &[], port);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    let got: Vec<(String, String, String)> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["rule"].as_str().unwrap().starts_with("deps/") && d["rule"] != "deps/outdated")
        .map(|d| (d["rule"].as_str().unwrap().to_owned(), d["file"].as_str().unwrap().to_owned(), d["severity"].as_str().unwrap().to_owned()))
        .collect();
    let want: Vec<(String, String, String)> = [
        ("deps/incomplete-scan", "package.json", "warn"),
        ("deps/vulnerable", "package.json", "error"),
        ("deps/vulnerable", "package.json", "error"),
        ("deps/vulnerable", "pyproject.toml", "warn"),
    ]
    .iter()
    .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
    .collect();
    let mut got = got;
    got.sort();
    assert_eq!(got, want, "{out}");
    // `rules_for_path`'s twin names a vulnerable package a source file imports (B-052).
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/index.js"), "import _ from 'lodash';\n").unwrap();
    let (_, out, err) = codelaw(&d, &["rules", "--for", "src/index.js", "--format", "json"], &[], port);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}{err}"));
    assert_eq!((v["dependencies"][0]["package"].as_str(), v["dependencies"][0]["target"].as_str()), (Some("lodash"), Some("4.17.21")), "{out}");
    // MCP `check_dependency`, offline, the same object as `deps check --format json`.
    let mut mcp = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .arg("mcp")
        .current_dir(&d)
        .env("HOME", &d)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let call = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "check_dependency", "arguments": { "ecosystem": "npm", "name": "lodash", "version": "4.17.15" } } });
    mcp.stdin.take().unwrap().write_all(format!("{call}\n").as_bytes()).unwrap();
    let out = mcp.wait_with_output().unwrap();
    let reply: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or("{}")).unwrap();
    assert_eq!(reply["result"]["structuredContent"]["target"], "4.17.21", "{reply}");
    let (_, _, err) = codelaw(&d, &["fix", "--no-cache", "--unsafe", "--as-of", "2026-10-29", "--rule", "deps/vulnerable"], &[], port);
    assert!(fs::read_to_string(d.join("package.json")).unwrap().contains("\"lodash\": \"^4.17.21\""), "{err}");
    assert!(fs::read_to_string(d.join("pyproject.toml")).unwrap().contains("\"jinja2>=3.1.3\""), "{err}");
    assert!(fs::read_to_string(d.join("package-lock.json")).unwrap().contains("\"version\": \"1.2.5\""), "a lockfile is never edited");
    assert!(fs::read_to_string(d.join("package.json")).unwrap().contains("\"overrides\": {\n    \"minimist\": \"1.2.6\"\n  }"), "the transitive one's override: {err}");

    // `deps check`, the CLI twin of MCP `check_dependency`.
    let (code, out, err) = codelaw(&d, &["deps", "check", "npm", "minimist", "1.2.5", "--format", "json"], &[], port);
    assert_eq!(code, 0, "{err}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!((v["target"].as_str(), v["vulnerable"].as_bool(), v["complete"].as_bool()), (Some("1.2.6"), Some(true), Some(false)), "{out}");
    let (_, out, _) = codelaw(&d, &["deps", "check", "npm", "minimist", "1.2.5"], &[], port);
    assert_eq!(out, "minimist 1.2.5 is vulnerable (GHSA-xvch-5gv4-984h, critical): upgrade to 1.2.6\n");

    // Every enabled source failing: nothing is written.
    fs::write(d.join("codelaw.toml"), format!("extends = [\"codelaw:deps/recommended\"]\npreview = true\n\n[deps]\nadvisory_sources = [{{ name = \"blocked\", kind = \"osv\", url = \"http://127.0.0.1:{port}/blocked\" }}]\n")).unwrap();
    let before = fs::read_to_string(d.join(".codelaw/deps-snapshot.json")).unwrap();
    let (code, _, err) = codelaw(&d, &["deps", "refresh", "--as-of", "2026-10-30"], &[], port);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("every advisory source failed"), "{err}");
    assert_eq!(fs::read_to_string(d.join(".codelaw/deps-snapshot.json")).unwrap(), before);
    let _ = fs::remove_dir_all(d);
}
