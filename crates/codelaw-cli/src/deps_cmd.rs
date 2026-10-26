//! `codelaw deps refresh` (SR-CLI-MCP decision 1; SR-RULE-PACKS decision
//! 15): the one command that reads the network (L5 decision 3). It reads
//! the project's manifests and lockfiles, asks npm and the Python index for
//! each dependency's versions, and writes `.codelaw/deps-snapshot.json`,
//! which the offline rules read. Nothing in the engine calls this.
//!
//! The requests go through `curl` on the PATH, as `git` serves the as-of
//! date: the binary carries no HTTP or TLS stack (L2), and a token is
//! passed on curl's stdin, never on its command line or into the snapshot.

use crate::{Common, engine_options, fail, load};
use clap::{Args, Subcommand};
use codelaw_core::Language;
use codelaw_native::deps::pep440::{Pep440, normalize, requirement};
use codelaw_native::deps::snapshot::{Package, SNAPSHOT_PATH, Snapshot, date_of, days};
use codelaw_native::deps::{Ecosystem, folder_of, lock, manifest_deps, npm::Range};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

#[derive(Args)]
pub struct DepsArgs {
    #[command(subcommand)]
    action: DepsAction,
}

#[derive(Subcommand)]
enum DepsAction {
    /// Read npm and the Python index for every dependency the manifests
    /// name and write `.codelaw/deps-snapshot.json` (the network's only door).
    Refresh(RefreshArgs),
}

#[derive(Args)]
struct RefreshArgs {
    /// The day written as `fetched` (YYYY-MM-DD); today, UTC, by default.
    #[arg(long, value_name = "DATE")]
    as_of: Option<String>,
    /// Requests in flight at once.
    #[arg(long, default_value_t = 8)]
    jobs: usize,
}

pub fn deps(args: DepsArgs) -> ExitCode {
    match args.action {
        DepsAction::Refresh(a) => match refresh(&a) {
            Ok(out) => {
                eprint!("{}", out.warnings);
                print!("{}", out.summary);
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        },
    }
}

/// One package to ask a registry about.
#[derive(Clone, Debug)]
struct Wanted {
    eco: Ecosystem,
    /// The registry's name for it: the npm name, or the normalised PyPI name.
    name: String,
    /// The lowest version any manifest or lockfile uses, when one is known.
    floor: Option<String>,
}

struct Refreshed {
    summary: String,
    warnings: String,
}

fn today() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    date_of((secs / 86_400) as i64)
}

fn refresh(args: &RefreshArgs) -> Result<Refreshed, String> {
    let fetched = match &args.as_of {
        Some(d) if days(d).is_some() && d.len() == 10 => d.clone(),
        Some(d) => return Err(format!("`--as-of {d}` is not a date; write it as YYYY-MM-DD")),
        None => today(),
    };
    let options = engine_options(&Common::default())?;
    let mut project = load(&[], options).map_err(|e| e.to_string())?;
    let root = project.root().to_path_buf();
    let settings = project.config.section_options("deps");
    let prerelease = settings.get("prerelease").and_then(Value::as_bool).unwrap_or(false);
    let ignored: BTreeSet<String> = settings
        .get("ignore")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|e| e.as_str().or_else(|| e.get("package").and_then(Value::as_str)))
        .map(str::to_owned)
        .collect();
    let files = project.files(&[]).map_err(|e| e.to_string())?;
    let read = |p: &str| std::fs::read_to_string(root.join(p)).ok();
    let mut wanted: BTreeMap<String, Wanted> = BTreeMap::new();
    for f in &files {
        let manifest = matches!((f.language, f.rel.rsplit('/').next()), (Language::Json, Some("package.json")) | (Language::Toml, Some("pyproject.toml")));
        if !manifest {
            continue;
        }
        let Some(text) = read(&f.rel) else { continue };
        let tree = codelaw_lang::parse(f.language, &text);
        let deps = manifest_deps(f.language, &f.rel, tree.root().get_inner_node(), &text);
        let dir = folder_of(&f.rel);
        for eco in [Ecosystem::Npm, Ecosystem::Pypi] {
            let specs: BTreeMap<String, String> = deps
                .iter()
                .filter(|d| d.ecosystem == eco)
                .map(|d| (if eco == Ecosystem::Npm { d.name.clone() } else { normalize(&d.name) }, d.spec.clone()))
                .collect();
            if specs.is_empty() {
                continue;
            }
            let locked = lock::installed(eco, dir, &specs, &read);
            for (name, spec) in &specs {
                let key = format!("{}:{name}", eco.prefix());
                if ignored.contains(&key) {
                    continue;
                }
                let written = match eco {
                    Ecosystem::Npm => match Range::parse(spec) {
                        Some(r) => Some(r.base().to_string()),
                        // A local, git or tag dependency is not the registry's.
                        None if spec.contains(':') || spec.contains('/') => continue,
                        None => None,
                    },
                    Ecosystem::Pypi => match requirement(spec) {
                        Some(r) => r.clauses.iter().find(|c| matches!(c.op.as_str(), "==" | "===" | ">=" | "~=" | ">")).map(|c| c.version.clone()),
                        None => continue,
                    },
                };
                let mut floors: Vec<String> = written.into_iter().chain(locked.get(name).cloned()).collect();
                let entry = wanted.entry(key).or_insert_with(|| Wanted { eco, name: name.clone(), floor: None });
                floors.extend(entry.floor.take());
                entry.floor = lowest(eco, floors);
            }
        }
    }
    let registries = Registries::read(&root);
    let jobs = args.jobs.max(1);
    let list: Vec<(String, Wanted)> = wanted.into_iter().collect();
    let mut results: Vec<(String, Result<Option<Package>, String>)> = Vec::new();
    for chunk in list.chunks(jobs) {
        let done: Vec<_> = std::thread::scope(|s| {
            let handles: Vec<_> = chunk.iter().map(|(k, w)| (k.clone(), s.spawn(|| fetch(&registries, w, prerelease)))).collect();
            handles.into_iter().map(|(k, h)| (k, h.join().unwrap_or_else(|_| Err("the request thread failed".into())))).collect()
        });
        results.extend(done);
    }
    let mut snap = Snapshot { fetched, registries: BTreeMap::new(), packages: BTreeMap::new() };
    let mut warnings = String::new();
    let mut errors = Vec::new();
    let mut counts: BTreeMap<Ecosystem, usize> = BTreeMap::new();
    for (key, r) in results {
        match r {
            Ok(Some(p)) => {
                let eco = if key.starts_with("npm:") { Ecosystem::Npm } else { Ecosystem::Pypi };
                *counts.entry(eco).or_default() += 1;
                let (label, url) = registries.label_for(eco, key.split_once(':').map(|(_, n)| n).unwrap_or(""));
                snap.registries.insert(label, url);
                snap.packages.insert(key, p);
            }
            Ok(None) => warnings.push_str(&format!("warning: `{key}` is not on its registry; it is left out of the snapshot\n")),
            Err(e) => errors.push(format!("`{key}`: {e}")),
        }
    }
    if !errors.is_empty() {
        return Err(format!("the snapshot was not written; {} request(s) failed:\n  {}", errors.len(), errors.join("\n  ")));
    }
    let path = root.join(SNAPSHOT_PATH);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&path, snap.to_text()).map_err(|e| format!("{}: {e}", path.display()))?;
    let summary = format!(
        "wrote {SNAPSHOT_PATH}: {} npm and {} PyPI package(s), fetched {}\n",
        counts.get(&Ecosystem::Npm).copied().unwrap_or(0),
        counts.get(&Ecosystem::Pypi).copied().unwrap_or(0),
        snap.fetched
    );
    Ok(Refreshed { summary, warnings })
}

/// The lowest of some versions, by the ecosystem's order.
fn lowest(eco: Ecosystem, vs: Vec<String>) -> Option<String> {
    match eco {
        Ecosystem::Npm => vs.into_iter().filter_map(|v| semver::Version::parse(&v).ok().map(|p| (p, v))).min().map(|(_, v)| v),
        Ecosystem::Pypi => vs.into_iter().filter_map(|v| Pep440::parse(&v).map(|p| (p, v))).min().map(|(_, v)| v),
    }
}

/// Where each ecosystem's packages come from, and the tokens to send.
struct Registries {
    npm: String,
    /// `@scope` → registry.
    scopes: BTreeMap<String, String>,
    /// `//host/path/` → token.
    tokens: BTreeMap<String, String>,
    pypi: String,
}

fn slash(url: &str) -> String {
    if url.ends_with('/') { url.to_owned() } else { format!("{url}/") }
}

/// `${VAR}` in an `.npmrc` value, from the environment.
fn expand(v: &str) -> String {
    let mut out = String::new();
    let mut rest = v;
    while let Some(i) = rest.find("${") {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i..].find('}') else {
            out.push_str(&rest[i..]);
            return out;
        };
        out.push_str(&std::env::var(&rest[i + 2..i + j]).unwrap_or_default());
        rest = &rest[i + j + 1..];
    }
    out.push_str(rest);
    out
}

/// A URL without its credentials, for the snapshot.
fn public(url: &str) -> String {
    match url.split_once("://") {
        Some((scheme, rest)) => {
            let (host, path) = rest.split_once('/').map(|(h, p)| (h, format!("/{p}"))).unwrap_or((rest, String::new()));
            let host = host.rsplit_once('@').map(|(_, h)| h).unwrap_or(host);
            format!("{scheme}://{host}{path}")
        }
        None => url.to_owned(),
    }
}

impl Registries {
    fn read(root: &Path) -> Registries {
        let mut r = Registries { npm: "https://registry.npmjs.org/".into(), scopes: BTreeMap::new(), tokens: BTreeMap::new(), pypi: "https://pypi.org/simple/".into() };
        // The user's `.npmrc`, then the project's: the project wins.
        let home = std::env::var_os("HOME").map(|h| Path::new(&h).join(".npmrc"));
        for file in home.into_iter().chain([root.join(".npmrc")]) {
            let Ok(text) = std::fs::read_to_string(file) else { continue };
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with(['#', ';']) {
                    continue;
                }
                let Some((k, v)) = line.split_once('=') else { continue };
                let (k, v) = (k.trim(), expand(v.trim().trim_matches('"')));
                if k == "registry" {
                    r.npm = slash(&v);
                } else if let Some(scope) = k.strip_suffix(":registry").filter(|s| s.starts_with('@')) {
                    r.scopes.insert(scope.to_owned(), slash(&v));
                } else if let Some(prefix) = k.strip_suffix(":_authToken").filter(|p| p.starts_with("//")) {
                    r.tokens.insert(slash(prefix), v);
                }
            }
        }
        if let Ok(v) = std::env::var("NPM_CONFIG_REGISTRY").or_else(|_| std::env::var("npm_config_registry"))
            && !v.is_empty()
        {
            r.npm = slash(&v);
        }
        r.pypi = match ["PIP_INDEX_URL", "UV_DEFAULT_INDEX", "UV_INDEX_URL"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty())) {
            Some(v) => slash(&v),
            None => uv_default_index(root).map(|v| slash(&v)).unwrap_or(r.pypi),
        };
        r
    }

    fn npm_registry(&self, name: &str) -> &str {
        name.split_once('/').and_then(|(scope, _)| self.scopes.get(scope)).map(String::as_str).unwrap_or(&self.npm)
    }

    /// The token for a URL: the longest `//host/path/` it starts with.
    fn token(&self, url: &str) -> Option<&str> {
        let bare = url.split_once("://").map(|(_, r)| format!("//{r}"))?;
        self.tokens.iter().filter(|(p, _)| bare.starts_with(p.as_str())).max_by_key(|(p, _)| p.len()).map(|(_, t)| t.as_str()).filter(|t| !t.is_empty())
    }

    /// The snapshot's `registries` entry for a package.
    fn label_for(&self, eco: Ecosystem, name: &str) -> (String, String) {
        match eco {
            Ecosystem::Npm => match name.split_once('/').filter(|(s, _)| self.scopes.contains_key(*s)) {
                Some((scope, _)) => (format!("npm:{scope}"), public(&self.scopes[scope])),
                None => ("npm".into(), public(&self.npm)),
            },
            Ecosystem::Pypi => ("pypi".into(), public(&self.pypi)),
        }
    }
}

/// `[[tool.uv.index]]` with `default = true`, in the root `pyproject.toml`.
fn uv_default_index(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join("pyproject.toml")).ok()?;
    let doc: toml::Table = toml::from_str(&text).ok()?;
    let indexes = doc.get("tool")?.get("uv")?.get("index")?.as_array()?;
    indexes.iter().find(|i| i.get("default").and_then(|d| d.as_bool()) == Some(true))?.get("url")?.as_str().map(str::to_owned)
}

/// A body and its HTTP status, through `curl`.
fn get(url: &str, accept: &str, token: Option<&str>) -> Result<(u16, String), String> {
    let quote = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    let mut config = format!("url = {}\nheader = {}\nwrite-out = \"\\n%{{http_code}}\"\n", quote(url), quote(&format!("Accept: {accept}")));
    if let Some(t) = token {
        config.push_str(&format!("header = {}\n", quote(&format!("Authorization: Bearer {t}"))));
    }
    let mut child = Command::new("curl")
        .args(["--silent", "--show-error", "--location", "--compressed", "--max-time", "60", "--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("`deps refresh` reads the registries with `curl`, which could not be run: {e}"))?;
    child.stdin.take().expect("piped").write_all(config.as_bytes()).map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("{}: {}", public(url), String::from_utf8_lossy(&out.stderr).trim()));
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let (body, code) = text.rsplit_once('\n').ok_or_else(|| format!("{}: no response", public(url)))?;
    let code = code.trim().parse().map_err(|_| format!("{}: no status in the response", public(url)))?;
    Ok((code, body.to_owned()))
}

/// The registry's facts about one package; `None` when it has no such
/// package.
fn fetch(r: &Registries, w: &Wanted, prerelease: bool) -> Result<Option<Package>, String> {
    let (url, accept) = match w.eco {
        Ecosystem::Npm => (format!("{}{}", r.npm_registry(&w.name), w.name.replace('/', "%2f")), "application/json"),
        Ecosystem::Pypi => (format!("{}{}/", r.pypi, w.name), "application/vnd.pypi.simple.v1+json"),
    };
    let (code, body) = get(&url, accept, r.token(&url))?;
    match code {
        200 => {}
        404 => return Ok(None),
        401 | 403 => return Err(format!("{} refused the request ({code}): set its token in `.npmrc` (`_authToken=${{TOKEN}}`) or the index URL's credentials", public(&url))),
        _ => return Err(format!("{} answered {code}", public(&url))),
    }
    let doc: Value = serde_json::from_str(&body).map_err(|_| format!("{} did not answer with JSON (a Python index must serve PEP 691's JSON simple API)", public(&url)))?;
    let facts = match w.eco {
        Ecosystem::Npm => npm_facts(&doc),
        Ecosystem::Pypi => pypi_facts(&doc, &w.name),
    };
    Ok(Some(keep(w, facts, prerelease)))
}

/// What a registry says of each version: its day, and why it is withdrawn.
struct Facts {
    versions: BTreeMap<String, String>,
    withdrawn: BTreeMap<String, String>,
}

fn npm_facts(doc: &Value) -> Facts {
    let mut f = Facts { versions: BTreeMap::new(), withdrawn: BTreeMap::new() };
    for (v, meta) in doc["versions"].as_object().into_iter().flatten() {
        let day = doc["time"][v].as_str().and_then(|t| t.get(..10)).unwrap_or("").to_owned();
        f.versions.insert(v.clone(), day);
        if let Some(msg) = meta["deprecated"].as_str().filter(|m| !m.is_empty()) {
            f.withdrawn.insert(v.clone(), msg.to_owned());
        }
    }
    f
}

/// A distribution file's version: `name-1.0-py3-none-any.whl`, `name-1.0.tar.gz`.
fn file_version(filename: &str) -> Option<String> {
    if let Some(stem) = filename.strip_suffix(".whl") {
        return stem.split('-').nth(1).map(str::to_owned);
    }
    let stem = [".tar.gz", ".zip", ".tar.bz2", ".tgz"].iter().find_map(|s| filename.strip_suffix(s))?;
    stem.rsplit_once('-').map(|(_, v)| v.to_owned())
}

/// PEP 691's JSON simple API, with PEP 700's `versions` and `upload-time`.
fn pypi_facts(doc: &Value, _name: &str) -> Facts {
    let mut f = Facts { versions: BTreeMap::new(), withdrawn: BTreeMap::new() };
    // version -> (earliest day, files, yanked files, a reason)
    let mut files: BTreeMap<String, (String, usize, usize, String)> = BTreeMap::new();
    for file in doc["files"].as_array().into_iter().flatten() {
        let Some(v) = file["filename"].as_str().and_then(file_version) else { continue };
        let Some(norm) = Pep440::parse(&v).map(|p| p.to_string()) else { continue };
        let e = files.entry(norm).or_insert_with(|| (String::new(), 0, 0, String::new()));
        if let Some(day) = file["upload-time"].as_str().and_then(|t| t.get(..10))
            && (e.0.is_empty() || day < e.0.as_str())
        {
            e.0 = day.to_owned();
        }
        e.1 += 1;
        match &file["yanked"] {
            Value::Bool(true) => e.2 += 1,
            Value::String(why) => {
                e.2 += 1;
                if e.3.is_empty() {
                    e.3 = why.clone();
                }
            }
            _ => {}
        }
    }
    let listed: Vec<String> = doc["versions"].as_array().into_iter().flatten().filter_map(Value::as_str).filter_map(|v| Pep440::parse(v).map(|p| p.to_string())).collect();
    let all: BTreeSet<String> = if listed.is_empty() { files.keys().cloned().collect() } else { listed.into_iter().collect() };
    for v in all {
        let (day, n, yanked, why) = files.get(&v).cloned().unwrap_or_default();
        if n > 0 && yanked == n {
            f.withdrawn.insert(v.clone(), why);
        }
        f.versions.insert(v, day);
    }
    f
}

/// The snapshot entry: the highest stable version as `latest`, and every
/// version from the lowest one in use up (prereleases only when asked for).
fn keep(w: &Wanted, facts: Facts, prerelease: bool) -> Package {
    let mut p = Package::default();
    match w.eco {
        Ecosystem::Npm => {
            let floor = w.floor.as_deref().and_then(|v| semver::Version::parse(v).ok());
            let parsed: Vec<(semver::Version, &String)> = facts.versions.keys().filter_map(|v| semver::Version::parse(v).ok().map(|p| (p, v))).collect();
            p.latest = parsed.iter().filter(|(v, s)| v.pre.is_empty() && !facts.withdrawn.contains_key(*s)).max_by(|a, b| a.0.cmp(&b.0)).map(|(v, _)| v.to_string());
            for (v, s) in parsed {
                if (prerelease || v.pre.is_empty()) && floor.as_ref().is_none_or(|f| v >= *f) {
                    p.versions.insert(s.clone(), facts.versions[s].clone());
                    if let Some(m) = facts.withdrawn.get(s) {
                        p.deprecated.insert(s.clone(), m.clone());
                    }
                }
            }
        }
        Ecosystem::Pypi => {
            let floor = w.floor.as_deref().and_then(Pep440::parse);
            let parsed: Vec<(Pep440, &String)> = facts.versions.keys().filter_map(|v| Pep440::parse(v).map(|p| (p, v))).collect();
            p.latest = parsed.iter().filter(|(v, s)| !v.is_prerelease() && !facts.withdrawn.contains_key(*s)).max_by(|a, b| a.0.cmp(&b.0)).map(|(v, _)| v.to_string());
            for (v, s) in parsed {
                if (prerelease || !v.is_prerelease()) && floor.as_ref().is_none_or(|f| v >= *f) {
                    p.versions.insert(s.clone(), facts.versions[s].clone());
                    if let Some(m) = facts.withdrawn.get(s) {
                        p.yanked.insert(s.clone(), m.clone());
                    }
                }
            }
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_and_files() {
        assert_eq!(public("https://u:p@pypi.acme.dev/simple/"), "https://pypi.acme.dev/simple/");
        assert_eq!(file_version("requests-2.32.3-py3-none-any.whl").as_deref(), Some("2.32.3"));
        assert_eq!(file_version("python-dateutil-2.8.2.tar.gz").as_deref(), Some("2.8.2"));
        // SAFETY: a test of this module only; nothing else reads the variable.
        unsafe { std::env::set_var("CODELAW_DEPS_TEST_TOKEN", "t0k") };
        assert_eq!(expand("${CODELAW_DEPS_TEST_TOKEN}"), "t0k");
        let r = Registries {
            npm: "https://registry.npmjs.org/".into(),
            scopes: [("@acme".to_owned(), "https://npm.acme.dev/r/".to_owned())].into(),
            tokens: [("//npm.acme.dev/r/".to_owned(), "t0k".to_owned())].into(),
            pypi: "https://pypi.org/simple/".into(),
        };
        assert_eq!(r.npm_registry("@acme/ui"), "https://npm.acme.dev/r/");
        assert_eq!(r.token("https://npm.acme.dev/r/@acme%2fui"), Some("t0k"));
        assert_eq!(r.token("https://registry.npmjs.org/react"), None);
    }
}
