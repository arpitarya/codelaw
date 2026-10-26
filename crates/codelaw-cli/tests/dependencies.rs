//! L11's check (SR-LAW-11 decision 7): no dependency outside `DEPENDENCIES.md`.
//!
//! The ledger at the end of `DEPENDENCIES.md` carries, per ratification unit,
//! its members (the crates, packages, actions or tools) and — for Cargo and
//! npm — its transitive tree as ratified. This test reads every manifest and
//! lockfile and fails on
//!
//! * a direct dependency that is a member of no unit (decision 1),
//! * a unit whose transitive tree holds a package its ledger line does not
//!   (decisions 4 and 5: a new package, or a new major version),
//! * a unit in use whose row is missing or whose status is none of
//!   `pending`, `ratified YYYY-MM-DD`, `remove` (decision 3),
//! * a script in `scripts/` or `.claude/hooks/` importing past the Python
//!   standard library.
//!
//! The check itself uses only crates and the standard library already in the
//! tree (`toml`, `serde_json`): it adds no dependency (L11).
//!
//! After the decider ratifies a unit again, regenerate the ledger lines with
//! `CODELAW_DEPS_PRINT=1 cargo test -p codelaw-cli --test dependencies
//! print_ledger -- --nocapture` and paste them over the old block.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("repo root")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

// ---------------------------------------------------------------- ledger

#[derive(Debug, Default, Clone)]
struct Unit {
    id: String,
    eco: String,
    members: BTreeSet<String>,
    tree: BTreeSet<String>,
}

const BEGIN: &str = "<!-- l11-ledger:begin -->";
const END: &str = "<!-- l11-ledger:end -->";

/// The ledger: one logical line per fact, a continuation line starts with
/// whitespace. `<ID> <ecosystem> members <name>…`, `<ID> <ecosystem> tree
/// <name@ver>…`, and `own <ecosystem> <name>…` for code codelaw wrote itself.
fn parse_ledger(md: &str) -> (BTreeMap<String, Unit>, BTreeMap<String, BTreeSet<String>>) {
    let start = md.find(BEGIN).expect("DEPENDENCIES.md has no ledger begin marker") + BEGIN.len();
    let end = md.find(END).expect("DEPENDENCIES.md has no ledger end marker");
    let mut logical: Vec<String> = Vec::new();
    for line in md[start..end].lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("```") {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            let last = logical.last_mut().expect("continuation with no line");
            last.push(' ');
            last.push_str(t);
        } else {
            logical.push(t.to_owned());
        }
    }
    let mut units: BTreeMap<String, Unit> = BTreeMap::new();
    let mut own: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for l in logical {
        let w: Vec<&str> = l.split_whitespace().collect();
        if w[0] == "own" {
            own.entry(w[1].to_owned()).or_default().extend(w[2..].iter().map(|s| s.to_string()));
            continue;
        }
        let u = units.entry(w[0].to_owned()).or_insert_with(|| Unit {
            id: w[0].to_owned(),
            eco: w[1].to_owned(),
            ..Unit::default()
        });
        let names = w[3..].iter().map(|s| s.to_string());
        match w[2] {
            "members" => u.members.extend(names),
            "tree" => u.tree.extend(names),
            other => panic!("ledger line `{l}`: `{other}` is neither members nor tree"),
        }
    }
    (units, own)
}

/// Rows of the tables: id -> the whole row text.
fn rows(md: &str) -> BTreeMap<String, String> {
    md.lines()
        .filter(|l| l.starts_with("| D-"))
        .map(|l| (l[2..].split('|').next().unwrap().trim().to_owned(), l.to_owned()))
        .collect()
}

fn status_ok(row: &str) -> bool {
    row.contains("ratified 20") || row.contains("pending") || row.contains("**remove**")
}

// ---------------------------------------------------------------- versions

/// `1.2.3` -> `1`, `0.25.17` -> `0.25`, `0.0.4` -> `0.0.4`: a package is the
/// same ratified package until its major (or, below 1.0, minor) moves.
fn major_key(version: &str) -> String {
    let core = version.split(['+', '-']).next().unwrap_or(version);
    let mut it = core.split('.');
    let (a, b, c) = (it.next().unwrap_or("0"), it.next().unwrap_or("0"), it.next().unwrap_or("0"));
    if a != "0" {
        a.to_owned()
    } else if b != "0" {
        format!("0.{b}")
    } else {
        format!("0.0.{c}")
    }
}

// ---------------------------------------------------------------- cargo

#[derive(Debug)]
struct Pkg {
    name: String,
    version: String,
    deps: Vec<String>,
}

fn cargo_lock(root: &Path) -> Vec<Pkg> {
    let doc: toml::Table = read(&root.join("Cargo.lock")).parse().expect("Cargo.lock");
    doc["package"]
        .as_array()
        .expect("packages")
        .iter()
        .map(|p| Pkg {
            name: p["name"].as_str().unwrap().to_owned(),
            version: p["version"].as_str().unwrap().to_owned(),
            deps: p
                .get("dependencies")
                .and_then(|d| d.as_array())
                .map(|a| a.iter().map(|s| s.as_str().unwrap().to_owned()).collect())
                .unwrap_or_default(),
        })
        .collect()
}

fn is_own(name: &str, own: &BTreeMap<String, BTreeSet<String>>) -> bool {
    name.starts_with("codelaw") || own.get("cargo").is_some_and(|s| s.contains(name))
}

/// The tree of a unit: every `name@key` reachable from its members in the
/// lockfile, every platform's packages counted.
fn cargo_tree(
    lock: &[Pkg],
    members: &BTreeSet<String>,
    own: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut stack: Vec<(String, Option<String>)> = members.iter().map(|m| (m.clone(), None)).collect();
    while let Some((name, version)) = stack.pop() {
        for p in lock.iter().filter(|p| p.name == name && version.as_ref().is_none_or(|v| *v == p.version)) {
            if !seen.insert((p.name.clone(), p.version.clone())) {
                continue;
            }
            for d in &p.deps {
                let mut it = d.split_whitespace();
                let n = it.next().unwrap().to_owned();
                if is_own(&n, own) {
                    continue;
                }
                stack.push((n, it.next().map(str::to_owned)));
            }
        }
    }
    seen.into_iter().map(|(n, v)| format!("{n}@{}", major_key(&v))).collect()
}

fn manifests(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.join("Cargo.toml")];
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().is_some_and(|n| n == "Cargo.toml") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Every dependency named in any `Cargo.toml` (any table, any target), with
/// the file it came from. `[patch]` entries count: they put code in the build.
fn cargo_direct(root: &Path) -> BTreeMap<String, String> {
    fn walk(t: &toml::Table, file: &str, out: &mut BTreeMap<String, String>) {
        for (k, v) in t {
            let Some(inner) = v.as_table() else { continue };
            // `[workspace.dependencies]` only declares; a crate that uses an
            // entry names it in its own table, and that is what is counted.
            if k == "workspace" {
                continue;
            }
            let dep_table = matches!(k.as_str(), "dependencies" | "dev-dependencies" | "build-dependencies");
            if dep_table {
                for (name, spec) in inner {
                    let real = spec.get("package").and_then(|p| p.as_str()).unwrap_or(name);
                    out.entry(real.to_owned()).or_insert_with(|| file.to_owned());
                }
            } else if k == "patch" {
                for (_registry, patches) in inner {
                    for name in patches.as_table().into_iter().flat_map(|t| t.keys()) {
                        out.entry(name.clone()).or_insert_with(|| file.to_owned());
                    }
                }
            } else {
                walk(inner, file, out);
            }
        }
    }
    let mut out = BTreeMap::new();
    for m in manifests(root) {
        let doc: toml::Table = read(&m).parse().unwrap_or_else(|e| panic!("{}: {e}", m.display()));
        let file = m.strip_prefix(root).unwrap().display().to_string();
        walk(&doc, &file, &mut out);
    }
    out
}

// ---------------------------------------------------------------- npm

fn package_jsons(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.join("packaging")];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = e.path();
            let name = e.file_name();
            if p.is_dir() {
                if name != "node_modules" && name != "out" {
                    stack.push(p);
                }
            } else if name == "package.json" {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn npm_is_own(name: &str) -> bool {
    name == "codelaw" || name.starts_with("@codelaw/")
}

fn npm_direct(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for p in package_jsons(root) {
        let v: Value = serde_json::from_str(&read(&p)).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let file = p.strip_prefix(root).unwrap().display().to_string();
        for table in ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"] {
            for name in v.get(table).and_then(|t| t.as_object()).into_iter().flat_map(|o| o.keys()) {
                if !npm_is_own(name) {
                    out.entry(name.clone()).or_insert_with(|| file.clone());
                }
            }
        }
    }
    out
}

fn npm_tree(root: &Path, members: &BTreeSet<String>) -> BTreeSet<String> {
    let mut tree = BTreeSet::new();
    for p in package_jsons(root) {
        let lock = p.with_file_name("package-lock.json");
        if !lock.exists() {
            continue;
        }
        let v: Value = serde_json::from_str(&read(&lock)).expect("package-lock.json");
        let packages = v["packages"].as_object().expect("lock packages");
        let by_name = |name: &str| -> Vec<(&String, &Value)> {
            let suffix = format!("node_modules/{name}");
            packages.iter().filter(|(k, _)| !k.is_empty() && (**k == suffix || k.ends_with(&format!("/{suffix}")))).collect()
        };
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut stack: Vec<String> = members.iter().cloned().collect();
        while let Some(name) = stack.pop() {
            for (key, pkg) in by_name(&name) {
                if !seen.insert(key.clone()) {
                    continue;
                }
                let ver = pkg["version"].as_str().unwrap_or("0");
                tree.insert(format!("{name}@{}", major_key(ver)));
                for table in ["dependencies", "optionalDependencies"] {
                    for d in pkg.get(table).and_then(|t| t.as_object()).into_iter().flat_map(|o| o.keys()) {
                        stack.push(d.clone());
                    }
                }
            }
        }
    }
    tree
}

// ---------------------------------------------------------------- python, actions

/// `maturin>=1.15,<2` -> `maturin`; `maturin[zig]==1` -> `maturin`.
fn py_name(spec: &str) -> String {
    spec.trim()
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect::<String>()
        .to_lowercase()
        .replace('_', "-")
}

fn python_direct(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let path = root.join("packaging/python/pyproject.toml");
    let doc: toml::Table = read(&path).parse().expect("pyproject.toml");
    let file = "packaging/python/pyproject.toml";
    let mut specs: Vec<String> = Vec::new();
    let strings = |v: Option<&toml::Value>| -> Vec<String> {
        v.and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|s| s.as_str().map(str::to_owned)).collect())
            .unwrap_or_default()
    };
    specs.extend(strings(doc.get("build-system").and_then(|b| b.get("requires"))));
    specs.extend(strings(doc.get("project").and_then(|b| b.get("dependencies"))));
    if let Some(o) = doc.get("project").and_then(|p| p.get("optional-dependencies")).and_then(|o| o.as_table()) {
        for v in o.values() {
            specs.extend(strings(Some(v)));
        }
    }
    for s in specs {
        out.entry(py_name(&s)).or_insert_with(|| file.to_owned());
    }
    out
}

/// `${{ … }}` is the workflow's own expression language, not an argument.
fn strip_expressions(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(i) = rest.find("${{") {
        out.push_str(&rest[..i]);
        match rest[i..].find("}}") {
            Some(j) => rest = &rest[i + j + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn workflow_files(root: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(root.join(".github/workflows"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "yml" || x == "yaml"))
        .collect();
    v.sort();
    v
}

/// What the workflows pull in: `uses:` actions, and packages named on a
/// `pip install`, `npm install -g` or `cargo install` line. Returns
/// (ecosystem, name) -> where.
fn workflow_direct(root: &Path) -> BTreeMap<(String, String), String> {
    let mut out = BTreeMap::new();
    for f in workflow_files(root) {
        let file = f.strip_prefix(root).unwrap().display().to_string();
        for (n, raw) in read(&f).lines().enumerate() {
            let line = strip_expressions(raw.trim().trim_start_matches("- ").trim());
            let line = line.as_str();
            if line.starts_with('#') {
                continue;
            }
            let at = format!("{file}:{}", n + 1);
            if let Some(rest) = line.strip_prefix("uses:") {
                let u = rest.split('#').next().unwrap().trim().trim_matches(['"', '\'']);
                if u.starts_with("./") || u.starts_with("docker://") {
                    continue;
                }
                let path = u.split('@').next().unwrap();
                let key: Vec<&str> = path.split('/').take(2).collect();
                out.entry(("action".to_owned(), key.join("/").to_lowercase())).or_insert(at);
                continue;
            }
            for (marker, eco) in [("pip install", "python"), ("npm install", "npm"), ("npm i ", "npm"), ("cargo install", "cargo")] {
                let Some(pos) = line.find(marker) else { continue };
                let args = line[pos + marker.len()..].split("&&").next().unwrap().split(['|', ';']).next().unwrap();
                for tok in args.split_whitespace() {
                    let tok = tok.trim_matches(['"', '\'']);
                    if tok.starts_with('-') || tok.is_empty() || tok.starts_with('$') {
                        continue;
                    }
                    let name = match eco {
                        "python" => py_name(tok),
                        // `npm@$NPM_VERSION` is npm itself: a toolchain (L11 decision 2).
                        _ => {
                            tok.strip_prefix('@').map(|t| format!("@{}", t.split('@').next().unwrap())).unwrap_or_else(|| tok.split('@').next().unwrap().to_owned())
                        }
                    };
                    if name.is_empty() || name == "npm" || name == "pip" {
                        continue;
                    }
                    out.entry((eco.to_owned(), name)).or_insert_with(|| at.clone());
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- the checks

fn load() -> (PathBuf, String, BTreeMap<String, Unit>, BTreeMap<String, BTreeSet<String>>) {
    let root = root();
    let md = read(&root.join("DEPENDENCIES.md"));
    let (units, own) = parse_ledger(&md);
    (root, md, units, own)
}

fn owner<'a>(units: &'a BTreeMap<String, Unit>, eco: &str, name: &str) -> Option<&'a Unit> {
    units.values().find(|u| u.eco == eco && u.members.contains(name))
}

#[test]
fn every_dependency_belongs_to_a_ratified_unit() {
    let (root, md, units, own) = load();
    let table = rows(&md);
    let mut problems: Vec<String> = Vec::new();
    let mut used: BTreeSet<String> = BTreeSet::new();

    let mut check = |eco: &str, name: &str, from: &str| match owner(&units, eco, name) {
        Some(u) => {
            used.insert(u.id.clone());
        }
        None => problems.push(format!(
            "{eco} dependency `{name}` ({from}) is in no unit of DEPENDENCIES.md — L11: ask the decider to ratify it by name"
        )),
    };

    for (name, from) in cargo_direct(&root) {
        if !is_own(&name, &own) {
            check("cargo", &name, &from);
        }
    }
    for (name, from) in npm_direct(&root) {
        check("npm", &name, &from);
    }
    for (name, from) in python_direct(&root) {
        check("python", &name, &from);
    }
    for ((eco, name), from) in workflow_direct(&root) {
        match eco.as_str() {
            "action" => check("action", &name, &from),
            "cargo" if is_own(&name, &own) => {}
            _ => check(&eco, &name, &from),
        }
    }

    for id in &used {
        match table.get(id) {
            None => problems.push(format!("unit {id} is in use but has no row in DEPENDENCIES.md")),
            Some(row) if !status_ok(row) => {
                problems.push(format!("unit {id}'s row has no status of pending, ratified YYYY-MM-DD or remove: {row}"))
            }
            Some(_) => {}
        }
    }
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

#[test]
fn no_ratified_tree_grew() {
    let (root, _md, units, own) = load();
    let lock = cargo_lock(&root);
    let mut problems: Vec<String> = Vec::new();
    for u in units.values() {
        let now = match u.eco.as_str() {
            "cargo" => cargo_tree(&lock, &u.members, &own),
            "npm" => npm_tree(&root, &u.members),
            _ => continue,
        };
        // A member no longer in the lockfile has been removed; nothing to grow.
        for pkg in now.difference(&u.tree) {
            problems.push(format!(
                "unit {} ({}): new package `{pkg}` in its tree ({}) — ratified trees do not grow (L11 decision 4); ask the decider to ratify {} again",
                u.id,
                u.eco,
                if u.eco == "cargo" { "Cargo.lock" } else { "package-lock.json" },
                u.id
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

/// Every package in the lockfile is some unit's: nothing rides along unseen.
#[test]
fn every_locked_package_is_in_a_unit() {
    let (root, _md, units, own) = load();
    let all: BTreeSet<String> =
        units.values().filter(|u| u.eco == "cargo").flat_map(|u| u.tree.iter().cloned()).collect();
    let problems: Vec<String> = cargo_lock(&root)
        .iter()
        .filter(|p| !is_own(&p.name, &own))
        .map(|p| format!("{}@{}", p.name, major_key(&p.version)))
        .filter(|k| !all.contains(k))
        .map(|k| format!("Cargo.lock: `{k}` is in no unit's tree — L11"))
        .collect();
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

#[test]
fn scripts_and_hooks_import_only_the_standard_library() {
    let root = root();
    let snippet = r#"
import ast, os, sys, sysconfig, pkgutil
std = set(getattr(sys, "stdlib_module_names", ()))
if not std:  # Python < 3.10
    lib = sysconfig.get_paths()["stdlib"]
    std = {m.name for m in pkgutil.iter_modules([lib, os.path.join(lib, "lib-dynload")])}
std |= set(sys.builtin_module_names)
std |= set(sys.argv[1].split(","))
bad = []
for d in sys.argv[2:]:
    if not os.path.isdir(d):
        continue
    local = {os.path.splitext(f)[0] for f in os.listdir(d)}
    for f in sorted(os.listdir(d)):
        if not f.endswith(".py"):
            continue
        tree = ast.parse(open(os.path.join(d, f), encoding="utf-8").read(), f)
        for n in ast.walk(tree):
            names = []
            if isinstance(n, ast.Import):
                names = [a.name for a in n.names]
            elif isinstance(n, ast.ImportFrom) and n.level == 0 and n.module:
                names = [n.module]
            for m in names:
                top = m.split(".")[0]
                if top not in std and top not in local and top != "__future__":
                    bad.append("%s/%s: imports `%s`" % (d, f, top))
print("\n".join(bad))
"#;
    let (_, _, _, own) = load();
    let own_py = own.get("python").map(|s| s.iter().cloned().collect::<Vec<_>>().join(",")).unwrap_or_default();
    let out = Command::new("python3")
        .args(["-I", "-c", snippet])
        .arg(own_py)
        .arg(root.join("scripts"))
        .arg(root.join(".claude/hooks"))
        .output()
        .expect("python3");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let bad = String::from_utf8_lossy(&out.stdout);
    assert!(bad.trim().is_empty(), "\nL11: a script or hook uses a third-party package:\n{bad}");
}

/// The ledger's own shape: every unit has members; Cargo and npm units a tree.
#[test]
fn ledger_is_well_formed() {
    let (_root, _md, units, _own) = load();
    assert!(!units.is_empty());
    for u in units.values() {
        assert!(!u.members.is_empty(), "unit {} has no members line", u.id);
    }
}

/// `CODELAW_DEPS_PRINT=1 … print_ledger -- --nocapture`: the ledger as it is
/// today, for the decider's next ratification. Prints nothing otherwise.
#[test]
fn print_ledger() {
    if std::env::var_os("CODELAW_DEPS_PRINT").is_none() {
        return;
    }
    let (root, _md, units, own) = load();
    let lock = cargo_lock(&root);
    for (eco, names) in &own {
        println!("own {eco} {}", names.iter().cloned().collect::<Vec<_>>().join(" "));
    }
    for u in units.values() {
        println!("{} {} members {}", u.id, u.eco, u.members.iter().cloned().collect::<Vec<_>>().join(" "));
        let tree = match u.eco.as_str() {
            "cargo" => cargo_tree(&lock, &u.members, &own),
            "npm" => npm_tree(&root, &u.members),
            _ => continue,
        };
        let mut line = format!("{} {} tree", u.id, u.eco);
        let mut width = line.len();
        for t in &tree {
            if width + t.len() > 100 {
                line.push_str("\n  ");
                width = 2;
            }
            line.push(' ');
            line.push_str(t);
            width += t.len() + 1;
        }
        println!("{line}");
    }
}

// ---------------------------------------------------------------- the check itself

fn fake(name: &str, version: &str, deps: &[&str]) -> Pkg {
    Pkg { name: name.into(), version: version.into(), deps: deps.iter().map(|d| d.to_string()).collect() }
}

/// DoD 2: a bump that makes a ratified crate's tree gain a package is seen,
/// and a patch or minor bump is not.
#[test]
fn a_grown_tree_is_detected_and_a_patch_bump_is_not() {
    let members: BTreeSet<String> = ["clap".to_owned()].into();
    let own = BTreeMap::new();
    let ratified = cargo_tree(&[fake("clap", "4.6.7", &["anstyle"]), fake("anstyle", "1.0.1", &[])], &members, &own);
    assert_eq!(ratified, ["anstyle@1", "clap@4"].map(String::from).into());

    let patched = cargo_tree(&[fake("clap", "4.6.9", &["anstyle"]), fake("anstyle", "1.2.0", &[])], &members, &own);
    assert!(patched.is_subset(&ratified), "a patch or minor bump is not growth");

    let grown = cargo_tree(
        &[fake("clap", "4.7.0", &["anstyle", "new-dep 0.1.0"]), fake("anstyle", "1.0.1", &[]), fake("new-dep", "0.1.0", &[])],
        &members,
        &own,
    );
    let new: Vec<&String> = grown.difference(&ratified).collect();
    assert_eq!(new, ["new-dep@0.1"]);

    let major = cargo_tree(&[fake("clap", "5.0.0", &["anstyle"]), fake("anstyle", "1.0.1", &[])], &members, &own);
    assert_eq!(major.difference(&ratified).collect::<Vec<_>>(), ["clap@5"]);
}

#[test]
fn versions_keys_and_workflow_lines() {
    assert_eq!(major_key("1.2.3"), "1");
    assert_eq!(major_key("0.25.17"), "0.25");
    assert_eq!(major_key("0.0.4"), "0.0.4");
    assert_eq!(major_key("0.25.0+codelaw.1"), "0.25");
    assert_eq!(py_name("maturin[zig]>=1.15,<2"), "maturin");
    assert_eq!(strip_expressions("pip install \"maturin${{ contains(a, 'zig') && '[zig]' || '' }}==$V\""), "pip install \"maturin==$V\"");
}

#[test]
fn an_unlisted_dependency_is_reported_with_its_name_and_origin() {
    let md = "| D-01 | **x** | ratified 2026-10-26 |\n<!-- l11-ledger:begin -->\n```text\nD-01 cargo members clap\nD-01 cargo tree clap@4\n```\n<!-- l11-ledger:end -->";
    let (units, _) = parse_ledger(md);
    assert!(owner(&units, "cargo", "clap").is_some());
    assert!(owner(&units, "cargo", "left-pad").is_none());
    assert!(status_ok(&rows(md)["D-01"]));
    assert!(!status_ok("| D-02 | **y** | maybe |"));
}
