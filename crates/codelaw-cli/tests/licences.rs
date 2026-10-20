//! L2's licence check (SR-LAW-2): every resolved Cargo dependency,
//! transitive ones included, is OSI-approved and permissive, and so is every
//! npm and PyPI package the build installs — maturin and what it pulls in,
//! npm itself (decision 4: build dependencies, in every scope). A
//! multi-licensed dependency qualifies through any one licence that does
//! (decision 6).

use serde_json::Value;
use std::process::Command;

/// Permissive and on the OSI-approved list (decisions 1 and 2).
const ALLOWED: &[&str] = &[
    "MIT",
    "MIT-0",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "0BSD",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Unlicense",
    "BSL-1.0",
];

fn tokens(expr: &str) -> Vec<String> {
    let spaced = expr.replace('/', " OR ").replace('(', " ( ").replace(')', " ) ");
    let mut out: Vec<String> = Vec::new();
    for t in spaced.split_whitespace() {
        // `WITH` binds to the licence before it.
        if let Some(last) = out.last_mut()
            && (t == "WITH" || last.ends_with(" WITH"))
        {
            last.push(' ');
            last.push_str(t);
            continue;
        }
        out.push(t.to_owned());
    }
    out
}

/// Evaluate an SPDX expression: `OR` takes any side, `AND` needs both.
fn allowed(tokens: &[String], pos: &mut usize) -> bool {
    let mut result = term(tokens, pos);
    while *pos < tokens.len() && tokens[*pos] == "OR" {
        *pos += 1;
        let rhs = term(tokens, pos);
        result = result || rhs;
    }
    result
}

fn term(tokens: &[String], pos: &mut usize) -> bool {
    let mut result = atom(tokens, pos);
    while *pos < tokens.len() && tokens[*pos] == "AND" {
        *pos += 1;
        let rhs = atom(tokens, pos);
        result = result && rhs;
    }
    result
}

fn atom(tokens: &[String], pos: &mut usize) -> bool {
    let Some(t) = tokens.get(*pos) else { return false };
    *pos += 1;
    if t == "(" {
        let v = allowed(tokens, pos);
        if tokens.get(*pos).is_some_and(|c| c == ")") {
            *pos += 1;
        }
        return v;
    }
    ALLOWED.contains(&t.as_str())
}

pub fn expression_allowed(expr: &str) -> bool {
    let t = tokens(expr);
    let mut pos = 0;
    allowed(&t, &mut pos) && pos == t.len()
}

#[test]
fn spdx_expressions() {
    assert!(expression_allowed("MIT"));
    assert!(expression_allowed("MIT OR Apache-2.0"));
    assert!(expression_allowed("MIT/Apache-2.0"));
    assert!(expression_allowed("(MIT OR Apache-2.0) AND Unicode-3.0"));
    assert!(expression_allowed("CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception"));
    assert!(!expression_allowed("GPL-3.0"));
    assert!(!expression_allowed("MIT AND MPL-2.0"));
    assert!(!expression_allowed("CC0-1.0"), "CC0 is not OSI-approved");
    assert!(!expression_allowed("LGPL-2.1 OR GPL-2.0"));
    assert!(expression_allowed("MPL-2.0 OR MIT"), "one qualifying licence is enough");
}

#[test]
fn every_cargo_dependency_is_free_and_permissive() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        // Every platform's dependencies, not only this machine's. Under the
        // L5 network fence, run `cargo fetch` first so this reads the cache.
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let meta: Value = serde_json::from_slice(&out.stdout).unwrap();
    let members: Vec<&str> = meta["workspace_members"].as_array().unwrap().iter().map(|m| m.as_str().unwrap()).collect();
    let mut refused = Vec::new();
    let mut checked = 0;
    for p in meta["packages"].as_array().unwrap() {
        if members.contains(&p["id"].as_str().unwrap()) {
            continue;
        }
        checked += 1;
        let name = format!("{} {}", p["name"].as_str().unwrap(), p["version"].as_str().unwrap());
        match p["license"].as_str() {
            Some(l) if expression_allowed(l) => {}
            Some(l) => refused.push(format!("{name}: `{l}`")),
            None => refused.push(format!("{name}: no licence expression")),
        }
    }
    assert!(checked > 50, "only {checked} dependencies seen");
    assert!(refused.is_empty(), "L2 refuses:\n{}", refused.join("\n"));
}

/// A package the build installs from npm or PyPI, with the licence its
/// registry page states for the version the build resolves (decision 1).
struct BuildTool {
    registry: &'static str,
    name: &'static str,
    licence: &'static str,
    /// What installing it installs too (always, or on some interpreters).
    requires: &'static [&'static str],
    /// `name[extra]` → what that extra installs.
    extras: &'static [(&'static str, &'static [&'static str])],
    source: &'static str,
}

/// Every npm and PyPI package the build, the CI and the release install.
/// A new install in `.github/`, `scripts/` or `packaging/` fails
/// `every_build_tool_is_declared_and_free_and_permissive` until it is added
/// here with its licence.
const BUILD_TOOLS: &[BuildTool] = &[
    BuildTool {
        registry: "pypi",
        name: "maturin",
        licence: "MIT OR Apache-2.0",
        requires: &["tomli"],
        extras: &[("zig", &["ziglang"])],
        source: "https://pypi.org/pypi/maturin/1.15.0/json (license_expression)",
    },
    BuildTool { registry: "pypi", name: "tomli", licence: "MIT", requires: &[], extras: &[], source: "https://pypi.org/project/tomli/ (maturin needs it below Python 3.11)" },
    BuildTool { registry: "pypi", name: "ziglang", licence: "MIT", requires: &[], extras: &[], source: "https://pypi.org/pypi/ziglang/json (license_expression)" },
    BuildTool {
        registry: "npm",
        name: "npm",
        licence: "Artistic-2.0",
        // npm ships its dependencies bundled inside its own tarball.
        requires: &[],
        extras: &[],
        source: "https://registry.npmjs.org/npm/latest (license)",
    },
];

/// Build tools whose licence the allow-list does not name and the decider
/// has not ruled on yet. Each is printed on every run, and the list may only
/// shrink. npm's Artistic-2.0 is OSI-approved and asks no derived work to
/// carry its terms (decision 2's test), but it is not among decision 2's
/// examples, so whether it passes is the decider's call (W-20).
const AWAITING_RULING: &[(&str, &str)] =
    &[("npm", "Artistic-2.0 — OSI-approved and not copyleft by decision 2's test, but not on its list: the decider rules (W-20)")];

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

/// The files that install things at build, CI or release time.
fn build_files() -> Vec<std::path::PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    let mut stack = vec![root.join(".github/workflows"), root.join("scripts"), root.join("packaging")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| ["yml", "yaml", "sh", "py", "toml", "json"].iter().any(|y| x == *y)) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// GitHub's `${{ … }}` becomes the `'[extra]'` literals it can produce, so
/// `"maturin${{ x && '[zig]' || '' }}==1"` reads as `maturin[zig]==1`.
fn expand_expressions(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(i) = rest.find("${{") {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i..].find("}}") else { break };
        let expr = &rest[i + 3..i + j];
        for lit in expr.split('\'').skip(1).step_by(2) {
            if lit.starts_with('[') && lit.ends_with(']') {
                out.push_str(lit);
            }
        }
        rest = &rest[i + j + 2..];
    }
    out.push_str(rest);
    out
}

fn ours(name: &str) -> bool {
    name == "codelaw" || name.starts_with("@codelaw/") || name.starts_with("codelaw-rules-") || name.starts_with("codelaw_rules_")
}

type Install = (String, String, Vec<String>);

/// `(registry, name, extras)` for one install argument; None for a local
/// file, a variable, a flag or one of codelaw's own packages.
fn package_of(registry: &str, arg: &str) -> Option<Install> {
    let a = arg.trim_matches(|c| c == '"' || c == '\'');
    if a.is_empty() || a.starts_with(['-', '$', '.', '/', '~']) || [".tgz", ".whl", ".tar.gz", ".zip"].iter().any(|x| a.ends_with(x)) {
        return None;
    }
    let (name, extras) = if registry == "npm" {
        let end = a[1..].find('@').map(|i| i + 1).unwrap_or(a.len());
        (a[..end].to_owned(), Vec::new())
    } else {
        let end = a.find(|c: char| "[=<>!~;@ ".contains(c)).unwrap_or(a.len());
        let extras = a
            .find('[')
            .and_then(|i| a[i + 1..].find(']').map(|j| &a[i + 1..i + 1 + j]))
            .map(|x| x.split(',').map(|e| e.trim().to_owned()).collect())
            .unwrap_or_default();
        (a[..end].to_lowercase().replace('_', "-"), extras)
    };
    if ours(&name) { None } else { Some((registry.to_owned(), name, extras)) }
}

/// Every package an install command in `text` names.
fn installs(text: &str) -> Vec<Install> {
    let joined = text.replace("\\\r\n", " ").replace("\\\n", " ");
    let mut out = Vec::new();
    for line in joined.lines() {
        let line = expand_expressions(line);
        for cmd in line.split(['&', ';', '|', '(', ')']) {
            let t: Vec<&str> = cmd.split_whitespace().collect();
            for (i, tok) in t.iter().enumerate() {
                let tool = tok.trim_matches('"').rsplit('/').next().unwrap_or(tok);
                let next = t.get(i + 1).copied().unwrap_or("");
                let (registry, args): (&str, &[&str]) = match (tool, next) {
                    ("pip" | "pip3" | "pipx", "install") | ("uv", "add") => ("pypi", &t[i + 2..]),
                    ("npm" | "pnpm" | "yarn" | "bun", "i" | "install" | "add") => ("npm", &t[i + 2..]),
                    ("npx", _) => ("npm", &t[i + 1..]),
                    _ => continue,
                };
                if tool == "npx" {
                    // npx runs one package; the rest are its arguments.
                    out.extend(args.iter().find(|a| !a.starts_with('-')).and_then(|a| package_of(registry, a)));
                } else {
                    out.extend(args.iter().filter_map(|a| package_of(registry, a)));
                }
            }
        }
    }
    out
}

/// `[build-system] requires`, `[project] dependencies` and package.json's.
fn manifest_deps(path: &std::path::Path, text: &str) -> Vec<Install> {
    let mut out = Vec::new();
    if path.file_name().is_some_and(|n| n == "pyproject.toml") {
        let v: toml::Value = toml::from_str(text).expect("pyproject.toml parses");
        for (table, key) in [("build-system", "requires"), ("project", "dependencies")] {
            for r in v.get(table).and_then(|b| b.get(key)).and_then(|r| r.as_array()).into_iter().flatten() {
                out.extend(r.as_str().and_then(|r| package_of("pypi", r)));
            }
        }
    } else if path.file_name().is_some_and(|n| n == "package.json") {
        let v: Value = serde_json::from_str(text).expect("package.json parses");
        for k in ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"] {
            for name in v.get(k).and_then(|d| d.as_object()).into_iter().flat_map(|d| d.keys()) {
                out.extend(package_of("npm", name));
            }
        }
    }
    out
}

#[test]
fn the_install_scanner_reads_what_the_build_writes() {
    let names = |t: &str| {
        installs(t)
            .into_iter()
            .map(|(r, n, e)| format!("{r}:{n}{}", if e.is_empty() { String::new() } else { format!("[{}]", e.join(",")) }))
            .collect::<Vec<_>>()
    };
    assert_eq!(names(r#"pip install -q "maturin${{ contains(m, 'zig') && '[zig]' || '' }}==$V""#), ["pypi:maturin[zig]"]);
    assert_eq!(names("npm i -g npm@latest --no-audit && npm --version"), ["npm:npm"]);
    assert_eq!(names("npx -y npm@12 trust github \"$name\""), ["npm:npm"]);
    assert_eq!(names("npm i --no-save \\\n  \"$dist\"/npm/codelaw-[0-9]*.tgz @scope/tool@1"), ["npm:@scope/tool"]);
    assert_eq!(names("\"$work/venv/bin/pip\" install -q --no-index \"$dist\"/pypi/codelaw-*.whl"), Vec::<String>::new());
    assert_eq!(names("python3 -m pip install Foo_Bar>=2 codelaw-rules-js"), ["pypi:foo-bar"]);
}

#[test]
fn every_build_tool_is_declared_and_free_and_permissive() {
    let mut found: Vec<(Install, String)> = Vec::new();
    for f in build_files() {
        let text = std::fs::read_to_string(&f).unwrap_or_default();
        let at = f.strip_prefix(repo_root()).unwrap().to_string_lossy().replace('\\', "/");
        found.extend(installs(&text).into_iter().chain(manifest_deps(&f, &text)).map(|i| (i, at.clone())));
    }
    // The scan sees what the build is known to install; an empty scan is a broken one.
    for must in ["maturin", "npm"] {
        assert!(found.iter().any(|f| f.0.1 == must), "the scan no longer sees `{must}`: {found:?}");
    }
    let tool = |r: &str, n: &str| BUILD_TOOLS.iter().position(|t| t.registry == r && t.name == n);
    let mut undeclared = Vec::new();
    let mut queue: Vec<(String, String, String)> = Vec::new();
    for ((r, n, extras), at) in &found {
        let Some(t) = tool(r, n) else {
            undeclared.push(format!("{r} `{n}` (installed in {at})"));
            continue;
        };
        queue.push((r.clone(), n.clone(), at.clone()));
        for e in extras {
            match BUILD_TOOLS[t].extras.iter().find(|x| x.0 == e) {
                Some((_, more)) => queue.extend(more.iter().map(|m| (r.clone(), m.to_string(), format!("{n}[{e}]")))),
                None => undeclared.push(format!("{r} `{n}[{e}]`: the extra is not declared (installed in {at})")),
            }
        }
    }
    let mut reached = vec![false; BUILD_TOOLS.len()];
    while let Some((r, n, via)) = queue.pop() {
        let Some(t) = tool(&r, &n) else {
            undeclared.push(format!("{r} `{n}` (required by {via})"));
            continue;
        };
        if !std::mem::replace(&mut reached[t], true) {
            queue.extend(BUILD_TOOLS[t].requires.iter().map(|m| (r.clone(), m.to_string(), n.clone())));
        }
    }
    assert!(undeclared.is_empty(), "L2: declare each in BUILD_TOOLS with its licence and source:\n{}", undeclared.join("\n"));
    let unused: Vec<&str> = BUILD_TOOLS.iter().zip(&reached).filter(|(_, r)| !**r).map(|(t, _)| t.name).collect();
    assert!(unused.is_empty(), "BUILD_TOOLS declares what nothing installs: {unused:?}");
    let mut refused = Vec::new();
    for t in BUILD_TOOLS {
        if expression_allowed(t.licence) {
            continue;
        }
        match AWAITING_RULING.iter().find(|a| a.0 == t.name) {
            Some((_, why)) => eprintln!("⚠ L2: {} `{}` is {} ({})", t.registry, t.name, why, t.source),
            None => refused.push(format!("{} `{}`: `{}` ({})", t.registry, t.name, t.licence, t.source)),
        }
    }
    assert!(refused.is_empty(), "L2 refuses:\n{}", refused.join("\n"));
    for (name, _) in AWAITING_RULING {
        let t = BUILD_TOOLS.iter().find(|t| t.name == *name).expect("an awaited tool is declared");
        assert!(!expression_allowed(t.licence), "`{name}` passes now: remove it from AWAITING_RULING");
    }
}
