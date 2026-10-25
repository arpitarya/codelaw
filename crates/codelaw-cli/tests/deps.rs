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

const TOKEN: &str = "s3cret-token";

/// A registry on 127.0.0.1: npm packuments at `/<name>`, a private scope
/// that wants `Bearer s3cret-token`, and PEP 691 pages at `/simple/<name>/`.
fn registry() -> (u16, Seen) {
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
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                continue;
            }
            let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
            let mut auth = None;
            loop {
                let mut h = String::new();
                if reader.read_line(&mut h).unwrap_or(0) == 0 || h == "\r\n" {
                    break;
                }
                if let Some((k, v)) = h.split_once(':')
                    && k.eq_ignore_ascii_case("authorization")
                {
                    auth = Some(v.trim().to_owned());
                }
            }
            log.lock().unwrap().push((path.clone(), auth.clone()));
            let private = path.starts_with("/@acme");
            let (status, body) = match pages.get(path.as_str()) {
                Some(_) if private && auth.as_deref() != Some(&format!("Bearer {TOKEN}")) => ("401 Unauthorized", "{}".to_owned()),
                Some(b) => ("200 OK", b.clone()),
                None => ("404 Not Found", "{}".to_owned()),
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
    fs::write(d.join("codelaw.toml"), "extends = [\"codelaw:data/package-json\", \"codelaw:data/pyproject\"]\npreview = true\n\n[deps]\nmin_release_age = \"7d\"\n").unwrap();
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
    assert!(out.contains("wrote .codelaw/deps-snapshot.json: 3 npm and 2 PyPI package(s), fetched 2026-10-25"), "{out}");
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
    let (_, out, err) = codelaw(&d, &["check", "--no-cache", "--format", "json", "--as-of", "2026-10-25", "--rule", "data/outdated-dependency"], &[], port);
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
    let (_, _, err) = codelaw(&d, &["fix", "--no-cache", "--unsafe", "--as-of", "2026-10-25", "--rule", "data/outdated-dependency"], &[], port);
    let pkg = fs::read_to_string(d.join("package.json")).unwrap();
    assert!(pkg.contains("\"react\": \"^19.2.0\"") && pkg.contains("\"@acme/ui\": \"^2.1.0\"") && pkg.contains("\"left-pad\": \"1.3.0\""), "{pkg}{err}");
    assert!(fs::read_to_string(d.join("pyproject.toml")).unwrap().contains("\"bad==1.1\""), "{err}");

    // data/stale-deps-snapshot, judged on the as-of date.
    let stale = |date: &str| {
        let (_, out, _) = codelaw(&d, &["check", "--no-cache", "--format", "json", "--as-of", date, "--rule", "data/stale-deps-snapshot"], &[], port);
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
