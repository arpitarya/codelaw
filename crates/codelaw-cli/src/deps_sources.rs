//! The advisory sources of `codelaw deps refresh` (W-128; SR-RULE-PACKS
//! decision 15): each enabled `[deps] advisory_sources` entry is read, every
//! record is merged per package (`codelaw_native::deps::advisory::merge`,
//! the safer answer winning), and each source's state goes into the
//! snapshot. A source that cannot be read is `unavailable`, never silent;
//! the refresh fails only when every enabled source does.
//!
//! | kind | request | gives |
//! |---|---|---|
//! | `registry` · npm | `POST <registry>/-/npm/v1/security/advisories/bulk` | GHSA id and URL, `vulnerable_versions`, severity — the fixed version derived from the range |
//! | `registry` · PyPI | `GET <index host>/pypi/<name>/<version>/json` | `vulnerabilities[]` of that version: id, aliases, `fixed_in`, `withdrawn`; and the licence |
//! | `osv` | `POST <url>/v1/query` by package | every advisory, `affected[].ranges[].events`, aliases, severity |
//!
//! Nothing here keeps an advisory's text: ids, ranges, versions, a
//! severity, a URL and the source's name (OSV and GHSA data is CC-BY 4.0).

use crate::deps_cmd::{Registries, public, request};
use codelaw_native::deps::Ecosystem;
use codelaw_native::deps::advisory::{Ver, ranges_from_events, severity_word};
use codelaw_native::deps::pep440::normalize;
use codelaw_native::deps::snapshot::Advisory;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

/// One `[deps] advisory_sources` entry, as the refresh uses it.
#[derive(Clone, Debug)]
pub struct Source {
    pub name: String,
    pub kind: String,
    pub url: Option<String>,
    pub npm_url: Option<String>,
    pub pypi_url: Option<String>,
}

/// The enabled sources, in the config's order; `registry` then `osv` when
/// the config names none.
pub fn sources(options: &BTreeMap<String, Value>) -> Vec<Source> {
    let Some(list) = options.get("advisory_sources").and_then(Value::as_array) else {
        return vec![
            Source { name: "registry".into(), kind: "registry".into(), url: None, npm_url: None, pypi_url: None },
            Source { name: "osv".into(), kind: "osv".into(), url: None, npm_url: None, pypi_url: None },
        ];
    };
    list.iter()
        .filter(|s| s.get("enabled").and_then(Value::as_bool) != Some(false))
        .map(|s| {
            let text = |k: &str| s.get(k).and_then(Value::as_str).map(str::to_owned);
            Source { name: text("name").unwrap_or_default(), kind: text("kind").unwrap_or_default(), url: text("url"), npm_url: text("npm_url"), pypi_url: text("pypi_url") }
        })
        .collect()
}

/// Run `f` over `items`, `jobs` at a time, keeping their order.
pub fn parallel<T: Sync, R: Send>(items: &[T], jobs: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let mut out = Vec::with_capacity(items.len());
    for chunk in items.chunks(jobs.max(1)) {
        let done: Vec<R> = std::thread::scope(|s| {
            let handles: Vec<_> = chunk.iter().map(|i| s.spawn(|| f(i))).collect();
            handles.into_iter().map(|h| h.join().expect("a request thread")).collect()
        });
        out.extend(done);
    }
    out
}

/// What one source gave: records per package key, the versions it covered
/// per package key, and its first failure.
#[derive(Default)]
pub struct Reading {
    pub records: BTreeMap<String, Vec<Advisory>>,
    pub scanned: BTreeMap<String, BTreeSet<String>>,
    pub error: Option<String>,
}

impl Reading {
    fn fail(&mut self, e: String) {
        if self.error.is_none() {
            self.error = Some(e);
        }
    }
}

/// The package key of a name.
pub fn key(eco: Ecosystem, name: &str) -> String {
    match eco {
        Ecosystem::Npm => format!("npm:{name}"),
        Ecosystem::Pypi => format!("pypi:{}", normalize(name)),
    }
}

/// The PyPI JSON API beside a simple index: `…/simple/` → `…/pypi/`.
pub fn pypi_json_base(simple: &str) -> String {
    let s = simple.trim_end_matches('/');
    match s.strip_suffix("/simple") {
        Some(base) => format!("{base}/pypi/"),
        None => format!("{s}/../pypi/"),
    }
}

/// One PyPI answer per `(name, version)`.
type Answers = BTreeMap<(String, String), Result<Option<Value>, String>>;

/// A PyPI version's JSON, cached for the whole refresh: the registry
/// source and the licence read the same answer.
pub struct PypiCache {
    base: String,
    token: Option<String>,
    answers: Mutex<Answers>,
}

impl PypiCache {
    pub fn new(registries: &Registries, pypi_url: Option<&str>) -> PypiCache {
        let index = pypi_url.map(str::to_owned).unwrap_or_else(|| registries.pypi.clone());
        PypiCache { base: pypi_json_base(&index), token: None, answers: Mutex::new(BTreeMap::new()) }
    }

    /// `None` when the index has no such version (404).
    pub fn version(&self, name: &str, version: &str) -> Result<Option<Value>, String> {
        let k = (name.to_owned(), version.to_owned());
        if let Some(a) = self.answers.lock().expect("cache").get(&k) {
            return a.clone();
        }
        let url = format!("{}{name}/{version}/json", self.base);
        let a = match request("GET", &url, "application/json", self.token.as_deref(), None) {
            Ok((200, body)) => serde_json::from_str(&body).map(Some).map_err(|_| format!("{} did not answer with JSON", public(&url))),
            Ok((404, _)) => Ok(None),
            Ok((code, _)) => Err(format!("{} answered HTTP {code}", public(&url))),
            Err(e) => Err(e),
        };
        self.answers.lock().expect("cache").insert(k, a.clone());
        a
    }
}

/// The fixed versions an npm `vulnerable_versions` range implies: each
/// alternative's `<X` bound.
pub fn npm_fixed(range: &str) -> Vec<String> {
    range
        .split("||")
        .filter_map(|alt| {
            let toks: Vec<&str> = alt.split_whitespace().collect();
            toks.iter().enumerate().find_map(|(i, t)| match t.strip_prefix('<') {
                Some(r) if !r.starts_with('=') && !r.is_empty() => Some(r.to_owned()),
                Some("") => toks.get(i + 1).map(|s| s.to_string()),
                _ => None,
            })
        })
        .collect()
}

/// npm's bulk advisory endpoint for `packages` (name → versions): one
/// request per hundred packages.
pub fn npm_bulk(registries: &Registries, npm_url: Option<&str>, packages: &BTreeMap<String, BTreeSet<String>>, name: &str, jobs: usize, out: &mut Reading) {
    let names: Vec<&String> = packages.keys().collect();
    let chunks: Vec<Vec<&String>> = names.chunks(100).map(<[_]>::to_vec).collect();
    let answers = parallel(&chunks, jobs, |chunk| {
        let base = npm_url.map(crate::deps_cmd::slash).unwrap_or_else(|| registries.npm.clone());
        let url = format!("{base}-/npm/v1/security/advisories/bulk");
        let body: serde_json::Map<String, Value> = chunk.iter().map(|n| ((*n).clone(), json!(packages[*n]))).collect();
        match request("POST", &url, "application/json", registries.token(&url), Some(&Value::Object(body).to_string())) {
            Ok((200, text)) => serde_json::from_str::<Value>(&text).map_err(|_| format!("{} did not answer with JSON", public(&url))),
            Ok((code, _)) => Err(format!("{} answered HTTP {code}", public(&url))),
            Err(e) => Err(e),
        }
    });
    for (chunk, answer) in chunks.iter().zip(answers) {
        let doc = match answer {
            Ok(d) => d,
            Err(e) => {
                out.fail(e);
                continue;
            }
        };
        for n in chunk {
            let k = key(Ecosystem::Npm, n);
            out.scanned.entry(k.clone()).or_default().extend(packages[*n].iter().cloned());
            for a in doc[n.as_str()].as_array().into_iter().flatten() {
                let Some(range) = a["vulnerable_versions"].as_str() else { continue };
                let url = a["url"].as_str().map(str::to_owned);
                let ghsa = url.as_deref().and_then(|u| u.rsplit('/').next()).filter(|s| s.starts_with("GHSA-")).map(str::to_owned);
                let id = ghsa.unwrap_or_else(|| format!("NPM-{}", a["id"].as_u64().map(|i| i.to_string()).or_else(|| a["id"].as_str().map(str::to_owned)).unwrap_or_default()));
                out.records.entry(k.clone()).or_default().push(Advisory {
                    id,
                    aliases: Vec::new(),
                    severity: a["severity"].as_str().and_then(severity_word).unwrap_or("").to_owned(),
                    affected: range.split("||").map(|r| r.trim().to_owned()).filter(|r| !r.is_empty()).collect(),
                    fixed: npm_fixed(range),
                    url,
                    sources: vec![name.to_owned()],
                });
            }
        }
    }
}

/// The registry source's PyPI half: each `(name, version)` asked of the
/// index's JSON API; the queried version is affected from itself up to the
/// lowest `fixed_in` above it.
pub fn pypi_registry(cache: &PypiCache, asks: &[(String, String)], name: &str, jobs: usize, out: &mut Reading) {
    let answers = parallel(asks, jobs, |(n, v)| cache.version(n, v));
    for ((n, v), answer) in asks.iter().zip(answers) {
        let doc = match answer {
            Ok(Some(d)) => d,
            Ok(None) => continue,
            Err(e) => {
                out.fail(e);
                continue;
            }
        };
        let k = key(Ecosystem::Pypi, n);
        out.scanned.entry(k.clone()).or_default().insert(v.clone());
        let Some(here) = Ver::parse(Ecosystem::Pypi, v) else { continue };
        for a in doc["vulnerabilities"].as_array().into_iter().flatten() {
            if !a["withdrawn"].is_null() {
                continue;
            }
            let Some(id) = a["id"].as_str() else { continue };
            let fixed: Vec<String> = a["fixed_in"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect();
            let above = fixed.iter().filter_map(|f| Ver::parse(Ecosystem::Pypi, f).map(|p| (p, f))).filter(|(p, _)| *p > here).min_by(|a, b| a.0.cmp(&b.0)).map(|(_, f)| f.clone());
            let range = match above {
                Some(f) => format!(">={v} <{f}"),
                None => format!(">={v}"),
            };
            out.records.entry(k.clone()).or_default().push(Advisory {
                id: id.to_owned(),
                aliases: a["aliases"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect(),
                severity: String::new(),
                affected: vec![range],
                fixed,
                url: a["link"].as_str().map(str::to_owned),
                sources: vec![name.to_owned()],
            });
        }
    }
}

/// An OSV record's severity word: `database_specific.severity` (GHSA's),
/// else an affected entry's `ecosystem_specific.severity`.
fn osv_severity(rec: &Value) -> String {
    let word = rec["database_specific"]["severity"].as_str().or_else(|| rec["affected"].as_array().into_iter().flatten().find_map(|a| a["ecosystem_specific"]["severity"].as_str()));
    word.and_then(severity_word).unwrap_or("").to_owned()
}

/// One OSV record as the snapshot keeps it, for this package; `None` when
/// it is withdrawn or names no range of it.
pub fn osv_record(rec: &Value, eco: Ecosystem, package: &str, source: &str) -> Option<Advisory> {
    if !rec["withdrawn"].is_null() {
        return None;
    }
    let id = rec["id"].as_str()?;
    let want = key(eco, package);
    let osv_eco = match eco {
        Ecosystem::Npm => "npm",
        Ecosystem::Pypi => "PyPI",
    };
    let mut affected = Vec::new();
    let mut fixed = Vec::new();
    for a in rec["affected"].as_array().into_iter().flatten() {
        let p = &a["package"];
        if p["ecosystem"].as_str() != Some(osv_eco) || p["name"].as_str().map(|n| key(eco, n)) != Some(want.clone()) {
            continue;
        }
        for r in a["ranges"].as_array().into_iter().flatten() {
            if !matches!(r["type"].as_str(), Some("SEMVER" | "ECOSYSTEM")) {
                continue;
            }
            let events: Vec<(String, String)> = r["events"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|e| e.as_object()?.iter().next().and_then(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_owned()))))
                .collect();
            fixed.extend(events.iter().filter(|(k, _)| k == "fixed").map(|(_, v)| v.clone()));
            affected.extend(ranges_from_events(&events));
        }
        affected.extend(a["versions"].as_array().into_iter().flatten().filter_map(Value::as_str).map(|v| format!("={v}")));
    }
    if affected.is_empty() {
        return None;
    }
    let url = rec["references"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| r["type"].as_str() == Some("ADVISORY"))
        .and_then(|r| r["url"].as_str())
        .map(str::to_owned)
        .or_else(|| Some(format!("https://osv.dev/vulnerability/{id}")));
    Some(Advisory {
        id: id.to_owned(),
        aliases: rec["aliases"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect(),
        severity: osv_severity(rec),
        affected,
        fixed,
        url,
        sources: vec![source.to_owned()],
    })
}

/// An `osv` source: `POST <url>/v1/query` per package, every version
/// covered.
pub fn osv(url: &str, packages: &[(Ecosystem, String)], name: &str, jobs: usize, out: &mut Reading) {
    let endpoint = format!("{}/v1/query", url.trim_end_matches('/'));
    let answers = parallel(packages, jobs, |(eco, n)| {
        let ecosystem = if *eco == Ecosystem::Npm { "npm" } else { "PyPI" };
        let body = json!({ "package": { "name": n, "ecosystem": ecosystem } }).to_string();
        match request("POST", &endpoint, "application/json", None, Some(&body)) {
            Ok((200, text)) => serde_json::from_str::<Value>(&text).map_err(|_| format!("{} did not answer with JSON", public(&endpoint))),
            Ok((code, _)) => Err(format!("{} answered HTTP {code}", public(&endpoint))),
            Err(e) => Err(e),
        }
    });
    for ((eco, n), answer) in packages.iter().zip(answers) {
        let doc = match answer {
            Ok(d) => d,
            Err(e) => {
                out.fail(e);
                continue;
            }
        };
        let k = key(*eco, n);
        out.scanned.entry(k.clone()).or_default().insert("*".into());
        for rec in doc["vulns"].as_array().into_iter().flatten() {
            if let Some(a) = osv_record(rec, *eco, n, name) {
                out.records.entry(k.clone()).or_default().push(a);
            }
        }
    }
}

/// A licence from a PyPI version's JSON: PEP 639's `license_expression`,
/// else a short `license`, else the `License ::` classifiers.
pub fn pypi_licence(doc: &Value) -> Option<String> {
    let info = &doc["info"];
    if let Some(e) = info["license_expression"].as_str().filter(|e| !e.trim().is_empty()) {
        return Some(e.trim().to_owned());
    }
    if let Some(l) = info["license"].as_str().map(str::trim).filter(|l| !l.is_empty() && l.len() <= 80 && !l.contains('\n')) {
        return Some(l.to_owned());
    }
    let classes: Vec<&str> = info["classifiers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|c| c.strip_prefix("License :: "))
        .map(|c| c.rsplit(" :: ").next().unwrap_or(c))
        .collect();
    (!classes.is_empty()).then(|| classes.join(" OR "))
}

/// A licence from an npm version manifest: `license`, a `{ type }`, or the
/// old `licenses` list.
pub fn npm_licence(manifest: &Value) -> Option<String> {
    match &manifest["license"] {
        Value::String(s) if !s.trim().is_empty() => return Some(s.trim().to_owned()),
        Value::Object(o) => return o.get("type").and_then(Value::as_str).map(str::to_owned),
        _ => {}
    }
    let list: Vec<&str> = manifest["licenses"].as_array().into_iter().flatten().filter_map(|l| l["type"].as_str()).collect();
    (!list.is_empty()).then(|| list.join(" OR "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_shapes() {
        assert_eq!(npm_fixed("<4.17.21"), ["4.17.21"]);
        assert_eq!(npm_fixed(">=1.0.0 <1.2.3 || >=2.0.0 < 2.1.1 || <=0.5.0"), ["1.2.3", "2.1.1"]);
        assert_eq!(pypi_json_base("https://pypi.org/simple/"), "https://pypi.org/pypi/");
        assert_eq!(pypi_json_base("http://127.0.0.1:9/simple"), "http://127.0.0.1:9/pypi/");
        let rec = json!({
            "id": "GHSA-aaaa", "aliases": ["CVE-1"], "database_specific": { "severity": "MODERATE" },
            "affected": [
                { "package": { "ecosystem": "PyPI", "name": "Jinja2" }, "ranges": [ { "type": "ECOSYSTEM", "events": [ { "introduced": "0" }, { "fixed": "3.1.3" } ] } ] },
                { "package": { "ecosystem": "npm", "name": "jinja2" }, "ranges": [ { "type": "SEMVER", "events": [ { "introduced": "0" }, { "fixed": "9" } ] } ] }
            ],
            "details": "long text that is never kept"
        });
        let a = osv_record(&rec, Ecosystem::Pypi, "jinja2", "osv").unwrap();
        assert_eq!((a.affected.as_slice(), a.fixed.as_slice(), a.severity.as_str()), (&["<3.1.3".to_string()][..], &["3.1.3".to_string()][..], "moderate"));
        assert_eq!(a.url.as_deref(), Some("https://osv.dev/vulnerability/GHSA-aaaa"));
        assert!(osv_record(&json!({ "id": "X", "withdrawn": "2024-01-01", "affected": [] }), Ecosystem::Pypi, "jinja2", "osv").is_none());
        assert_eq!(pypi_licence(&json!({ "info": { "license_expression": "MIT" } })).as_deref(), Some("MIT"));
        assert_eq!(pypi_licence(&json!({ "info": { "license": "Copyright …\nlong", "classifiers": ["License :: OSI Approved :: BSD License"] } })).as_deref(), Some("BSD License"));
        assert_eq!(npm_licence(&json!({ "license": { "type": "ISC" } })).as_deref(), Some("ISC"));
        assert_eq!(npm_licence(&json!({ "licenses": [{ "type": "MIT" }, { "type": "Apache-2.0" }] })).as_deref(), Some("MIT OR Apache-2.0"));
    }
}
