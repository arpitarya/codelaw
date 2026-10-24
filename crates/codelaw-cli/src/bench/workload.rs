//! The reference workload of L8 decision 2: a deterministic, seeded,
//! generated repo spanning every L3 language. The same seed and file count
//! give the same bytes on every machine and every run — no network, no
//! clock, no randomness from the OS — so only timing varies between runs
//! (L5). `fixtures/perf/README.md` names its shape; the files themselves
//! are generated, never committed.
//!
//! The mix is that of a polyglot product repo: TypeScript and Python
//! first, then JavaScript, Markdown, JSON, YAML, CSS, HTML and TOML. Files
//! sit 25 to a folder, import their siblings, and about one in eight
//! carries a finding the recommended presets report.

use std::fmt::Write as _;
use std::path::Path;

/// The seed of the reference workload; L8's budgets are measured on it.
pub const SEED: u64 = 20_261_018;
/// The file count of the `*_10k` operations.
pub const FILES: usize = 10_000;
/// The digest of `generate(SEED, FILES)`: the bytes L8's budgets bind.
pub const REFERENCE_DIGEST: &str = "bf9c5bea0d8b31ee";
/// Files per folder.
const PER_DIR: usize = 25;

/// Each language's share of the workload, in files per 10,000.
const MIX: &[(Kind, usize)] = &[
    (Kind::Ts, 2200),
    (Kind::Tsx, 800),
    (Kind::Js, 1300),
    (Kind::Jsx, 300),
    (Kind::Py, 2000),
    (Kind::Json, 800),
    (Kind::Yaml, 500),
    (Kind::Toml, 200),
    (Kind::Md, 1000),
    (Kind::Css, 500),
    (Kind::Html, 400),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Ts,
    Tsx,
    Js,
    Jsx,
    Py,
    Json,
    Yaml,
    Toml,
    Md,
    Css,
    Html,
}

impl Kind {
    fn ext(self) -> &'static str {
        match self {
            Kind::Ts => "ts",
            Kind::Tsx => "tsx",
            Kind::Js => "js",
            Kind::Jsx => "jsx",
            Kind::Py => "py",
            Kind::Json => "json",
            Kind::Yaml => "yaml",
            Kind::Toml => "toml",
            Kind::Md => "md",
            Kind::Css => "css",
            Kind::Html => "html",
        }
    }

    /// The folder a language's files live under.
    fn area(self) -> &'static str {
        match self {
            Kind::Ts | Kind::Tsx => "packages/ts",
            Kind::Js | Kind::Jsx => "packages/js",
            Kind::Py => "services",
            Kind::Json | Kind::Yaml | Kind::Toml => "config",
            Kind::Md => "docs",
            Kind::Css | Kind::Html => "web",
        }
    }
}

/// SplitMix64: small, fast, and the same everywhere.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// In `lo..=hi`.
    fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + (self.next() % (hi - lo + 1) as u64) as usize
    }

    /// True `n` times in `d`.
    fn chance(&mut self, n: u64, d: u64) -> bool {
        self.next() % d < n
    }

    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[self.range(0, xs.len() - 1)]
    }
}

const WORDS: &[&str] = &[
    "account", "batch", "cache", "draft", "event", "filter", "group", "header", "index", "job", "kernel", "ledger", "metric", "node", "order",
    "payload", "queue", "record", "session", "ticket", "user", "vector", "widget", "zone",
];

/// One generated file: its path relative to the workload root, and its bytes.
pub struct File {
    pub path: String,
    pub text: String,
}

/// Every file of the workload, sorted by path.
pub fn generate(seed: u64, files: usize) -> Vec<File> {
    let mut rng = Rng(seed);
    let mut out = Vec::with_capacity(files);
    let total: usize = MIX.iter().map(|(_, n)| n).sum();
    let mut counts: Vec<(Kind, usize)> = MIX.iter().map(|&(k, n)| (k, n * files / total)).collect();
    // The rounding remainder goes to the first language.
    let short = files - counts.iter().map(|(_, n)| n).sum::<usize>();
    counts[0].1 += short;
    for (kind, count) in counts {
        for i in 0..count {
            let (dir, slot) = (i / PER_DIR, i % PER_DIR);
            let sibling = if slot > 0 { Some(rng.range(0, slot - 1)) } else { None };
            let finding = rng.chance(1, 8);
            let path = if kind == Kind::Py && slot == 0 {
                format!("{}/s{dir}/__init__.py", kind.area())
            } else {
                format!("{}/{}{dir}/m{slot}.{}", kind.area(), kind.ext(), kind.ext())
            };
            let text = body(kind, &mut rng, slot, sibling, finding);
            out.push(File { path, text });
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The BLAKE3 digest of the workload's config, paths and bytes: names it
/// in a report, and tells a reused folder from a stale one. The packs are
/// not in it: they are the repo's own at the commit measured.
pub fn digest(files: &[File]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(CONFIG.as_bytes());
    for f in files {
        h.update(f.path.as_bytes());
        h.update(&[0]);
        h.update(f.text.as_bytes());
        h.update(&[0]);
    }
    h.finalize().to_hex()[..16].to_owned()
}

/// The project config: every first-party pack's recommended presets, with
/// the preview rules on (SR-RULE-PACKS decision 9) — every pack rule is
/// preview today, and a budget set on a run that lints nothing would break
/// the day the rules turn stable.
pub const CONFIG: &str = "preview = true\nextends = [\n  \"codelaw:js/recommended\",\n  \"codelaw:js/typescript\",\n  \"codelaw:ts/recommended\",\n  \"codelaw:react/recommended\",\n  \"codelaw:py/recommended\",\n  \"codelaw:data/recommended\",\n  \"codelaw:md/recommended\",\n  \"codelaw:css/recommended\",\n  \"codelaw:html/recommended\",\n]\n";

/// Where the workload installs a first-party pack: npm's `node_modules`
/// when the pack is on npm, else the workload's `.venv` (L6 decision 11).
fn install_path(pack: &codelaw_rules::FirstPartyPack) -> std::path::PathBuf {
    match (pack.npm, pack.module()) {
        (Some(npm), _) => Path::new("node_modules").join(npm),
        (None, Some(module)) => Path::new(".venv/lib/python3.12/site-packages").join(module),
        (None, None) => unreachable!("every first-party pack is on a registry"),
    }
}

/// The marker naming what a workload folder holds.
const MARKER: &str = ".codelaw-bench";

/// Write the workload into `dir` with the packs from `packs`: the files,
/// `codelaw.toml`, `node_modules/@codelaw/rules-*` and the PyPI-only pack in `.venv`. A folder whose
/// marker names this digest is reused as it is. Returns the digest.
pub fn write(dir: &Path, packs: &Path, seed: u64, files: usize) -> Result<String, String> {
    let set = generate(seed, files);
    let digest = digest(&set);
    // `packs 9`: the nine-pack layout of 2026-10-24; a folder written with
    // the five-pack layout is rewritten, not reused.
    let marker = format!("seed {seed}\nfiles {files}\ndigest {digest}\npacks {}\n", codelaw_rules::FIRST_PARTY_PACKS.len());
    if std::fs::read_to_string(dir.join(MARKER)).is_ok_and(|m| m == marker) {
        return Ok(digest);
    }
    if dir.exists() {
        // Only a folder this command made, or an empty one, is replaced.
        let ours = dir.join(MARKER).is_file();
        let empty = std::fs::read_dir(dir).map(|mut d| d.next().is_none()).unwrap_or(false);
        if !ours && !empty {
            return Err(format!("{} is not empty and holds no workload; name another folder with --workload", codelaw_core::normalize_path(dir)));
        }
        std::fs::remove_dir_all(dir).map_err(|e| format!("cannot clear {}: {e}", codelaw_core::normalize_path(dir)))?;
    }
    let io = |e: std::io::Error| format!("cannot write the workload in {}: {e}", codelaw_core::normalize_path(dir));
    for f in &set {
        let path = dir.join(&f.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(io)?;
        }
        std::fs::write(&path, &f.text).map_err(io)?;
    }
    std::fs::write(dir.join("codelaw.toml"), CONFIG).map_err(io)?;
    for pack in codelaw_rules::FIRST_PARTY_PACKS {
        let from = packs.join(pack.short);
        if !from.join("codelaw-pack.toml").is_file() {
            return Err(format!("no rule pack in {}; name the packs folder with --packs", codelaw_core::normalize_path(&from)));
        }
        copy_dir(&from, &dir.join(install_path(pack))).map_err(io)?;
    }
    std::fs::write(dir.join(MARKER), marker).map_err(io)?;
    Ok(digest)
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), target)?;
        }
    }
    Ok(())
}

/// The bytes of one file. `units` blocks of the language's own shapes, an
/// import of a sibling where the language has one, and with `finding` one
/// thing the recommended presets report.
fn body(kind: Kind, rng: &mut Rng, slot: usize, sibling: Option<usize>, finding: bool) -> String {
    let units = rng.range(2, 12);
    let mut s = String::new();
    match kind {
        Kind::Ts | Kind::Tsx | Kind::Js | Kind::Jsx => script(&mut s, kind, rng, slot, sibling, finding, units),
        Kind::Py => python(&mut s, rng, slot, sibling, finding, units),
        Kind::Json => json(&mut s, rng, finding, units),
        Kind::Yaml => yaml(&mut s, rng, finding, units),
        Kind::Toml => toml(&mut s, rng, finding, units),
        Kind::Md => markdown(&mut s, rng, finding, units),
        Kind::Css => css(&mut s, rng, finding, units),
        Kind::Html => html(&mut s, rng, finding, units),
    }
    // One final newline, as the formatters leave it.
    let end = s.trim_end_matches('\n').len();
    s.truncate(end);
    s.push('\n');
    s
}

fn script(s: &mut String, kind: Kind, rng: &mut Rng, slot: usize, sibling: Option<usize>, finding: bool, units: usize) {
    let typed = matches!(kind, Kind::Ts | Kind::Tsx);
    let jsx = matches!(kind, Kind::Tsx | Kind::Jsx);
    if let Some(j) = sibling {
        let _ = writeln!(s, "import {{ helper{j} }} from \"./m{j}\";");
    }
    if jsx {
        s.push_str("import { useState } from \"react\";\n");
    }
    s.push('\n');
    for u in 0..units {
        let w = rng.pick(WORDS);
        let k = rng.range(1, 99);
        let name = format!("{w}{slot}x{u}");
        match rng.range(0, 3) {
            0 if typed => {
                let _ = writeln!(s, "export interface {}{u} {{\n  id: number;\n  name: string;\n  tags: string[];\n}}\n", cap(w));
                let _ = writeln!(
                    s,
                    "export function {name}(items: {}{u}[], factor: number): number {{\n  let total = 0;\n  for (const item of items) {{\n    if (item.tags.length > {k}) {{\n      total += item.id * factor;\n    }} else {{\n      total -= {k};\n    }}\n  }}\n  return total;\n}}\n",
                    cap(w)
                );
            }
            0 | 1 => {
                let p = if typed { ": number[]" } else { "" };
                let r = if typed { ": number" } else { "" };
                let _ = writeln!(
                    s,
                    "export function {name}(values{p}){r} {{\n  const kept = values.filter((v) => v % {k} !== 0);\n  return kept.reduce((a, b) => a + b, 0);\n}}\n"
                );
            }
            2 if typed => {
                let _ = writeln!(
                    s,
                    "export class {}{slot}x{u} {{\n  constructor(\n    private readonly name: string,\n    private readonly size: number,\n  ) {{}}\n\n  scaled(factor: number): number {{\n    return this.size > {k} ? this.size * factor : this.size + {k};\n  }}\n}}\n",
                    cap(w)
                );
            }
            2 => {
                let _ = writeln!(
                    s,
                    "export class {}{slot}x{u} {{\n  constructor(name, size) {{\n    this.name = name;\n    this.size = size;\n  }}\n\n  scaled(factor) {{\n    return this.size > {k} ? this.size * factor : this.size + {k};\n  }}\n}}\n",
                    cap(w)
                );
            }
            _ if jsx => {
                let _ = writeln!(
                    s,
                    "export function {}View{u}() {{\n  const [count, setCount] = useState({k});\n  return (\n    <section className=\"{w}\">\n      <h2>{w} {u}</h2>\n      <button onClick={{() => setCount(count + 1)}}>{{count}}</button>\n    </section>\n  );\n}}\n",
                    cap(w)
                );
            }
            _ => {
                let _ = writeln!(s, "export const {name} = Object.freeze({{\n  key: \"{w}\",\n  limit: {k},\n  enabled: {},\n}});\n", k.is_multiple_of(2));
            }
        }
    }
    if jsx {
        let _ = writeln!(
            s,
            "export function View{slot}() {{\n  const [open, setOpen] = useState(false);\n  return (\n    <button onClick={{() => setOpen(!open)}}>{{open ? \"close\" : \"open\"}}</button>\n  );\n}}\n"
        );
    }
    let t = if typed { ": number" } else { "" };
    let base = sibling.map(|j| format!("helper{j}(value)")).unwrap_or_else(|| "value".into());
    let _ = writeln!(s, "export function helper{slot}(value{t}){t} {{\n  return {base} * {};\n}}", slot + 1);
    if finding {
        match rng.range(0, 2) {
            0 => s.push_str("\nexport function debugHere() {\n  debugger;\n}\n"),
            1 => s.push_str("\nexport function check(a, b) {\n  if (a) {\n  }\n  return b;\n}\n"),
            _ => s.push_str("\nexport function loose(a, b) {\n  return a == b;\n}\n"),
        }
    }
}

fn python(s: &mut String, rng: &mut Rng, slot: usize, sibling: Option<usize>, finding: bool, units: usize) {
    // Standard library, then local: the order isort keeps.
    let sibling = sibling.filter(|j| *j > 0);
    s.push_str("import os\n");
    if let Some(j) = sibling {
        let _ = writeln!(s, "\nfrom .m{j} import helper_{j}");
    }
    s.push_str("\n\n");
    for u in 0..units {
        let w = rng.pick(WORDS);
        let k = rng.range(1, 99);
        if rng.chance(1, 2) {
            let _ = writeln!(
                s,
                "class {}{slot}x{u}:\n    def __init__(self, name: str, size: int) -> None:\n        self.name = name\n        self.size = size\n\n    def scaled(self, factor: int) -> int:\n        if self.size > {k}:\n            return self.size * factor\n        return self.size + {k}\n\n",
                cap(w)
            );
        } else {
            let _ = writeln!(
                s,
                "def {w}_{slot}_{u}(values: list[int]) -> int:\n    total = 0\n    for value in values:\n        if value % {k} == 0:\n            continue\n        total += value\n    return total + len(os.sep)\n\n"
            );
        }
    }
    let base = sibling.map(|j| format!("helper_{j}(value)")).unwrap_or_else(|| "value".into());
    let _ = writeln!(s, "def helper_{slot}(value: int) -> int:\n    return {base} * {} + len(os.sep)", slot + 1);
    if finding {
        match rng.range(0, 1) {
            0 => s.push_str("\n\ndef risky(path):\n    try:\n        return open(path).read()\n    except:\n        return None\n"),
            _ => s.push_str("\n\ndef same(a):\n    return a == None\n"),
        }
    }
}

fn json(s: &mut String, rng: &mut Rng, finding: bool, units: usize) {
    s.push_str("{\n");
    for u in 0..units {
        let w = rng.pick(WORDS);
        let k = rng.range(1, 999);
        let _ = write!(s, "  \"{w}{u}\": {{\n    \"enabled\": {},\n    \"limit\": {k},\n    \"tags\": [\"{w}\", \"t{k}\"]\n  }}", k.is_multiple_of(2));
        s.push_str(if u + 1 < units || finding { ",\n" } else { "\n" });
    }
    if finding {
        // A duplicate key.
        s.push_str("  \"dup\": 1,\n  \"dup\": 2\n");
    }
    s.push_str("}\n");
}

fn yaml(s: &mut String, rng: &mut Rng, finding: bool, units: usize) {
    s.push_str("name: service\n");
    if finding {
        s.push_str("name: duplicate\n");
    }
    s.push_str("jobs:\n");
    for u in 0..units {
        let w = rng.pick(WORDS);
        let k = rng.range(1, 60);
        let _ = writeln!(s, "  {w}{u}:\n    runs-on: ubuntu-latest\n    timeout-minutes: {k}\n    steps:\n      - run: make {w}\n      - run: make test-{w}");
    }
}

fn toml(s: &mut String, rng: &mut Rng, finding: bool, units: usize) {
    s.push_str("[package]\nname = \"service\"\nversion = \"1.0.0\"\n\n");
    for u in 0..units {
        let w = rng.pick(WORDS);
        let k = rng.range(1, 999);
        let _ = writeln!(s, "[{w}{u}]\nenabled = {}\nlimit = {k}\ntags = [\"{w}\", \"t{k}\"]\n", k.is_multiple_of(2));
    }
    if finding {
        s.push_str("# TODO: remove the old limits\n");
    }
}

fn markdown(s: &mut String, rng: &mut Rng, finding: bool, units: usize) {
    s.push_str("# Guide\n\n");
    for u in 0..units {
        let w = rng.pick(WORDS);
        let k = rng.range(1, 99);
        let _ = writeln!(
            s,
            "## The {w} step {u}\n\nEach {w} is checked before it is stored. A {w} older than {k} days is moved\nto the archive, and its owner is told.\n\n- read the {w}\n- check its limit\n- store it\n\n```sh\nmake {w}\n```\n"
        );
    }
    if finding {
        s.push_str("#### Skipped level\n\nThis heading skips a level.\n");
    }
}

fn css(s: &mut String, rng: &mut Rng, finding: bool, units: usize) {
    for u in 0..units {
        let w = rng.pick(WORDS);
        let k = rng.range(1, 64);
        let _ = writeln!(s, ".{w}-{u} {{\n  display: flex;\n  margin: {k}px 0;\n  padding: 0 {}px;\n  color: #333;\n}}\n", k / 2);
    }
    if finding {
        s.push_str(".loud {\n  color: red !important;\n}\n");
    }
}

fn html(s: &mut String, rng: &mut Rng, finding: bool, units: usize) {
    s.push_str("<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\">\n    <title>Page</title>\n  </head>\n  <body>\n");
    for u in 0..units {
        let w = rng.pick(WORDS);
        let _ = writeln!(s, "    <section id=\"{w}-{u}\">\n      <h2>{w}</h2>\n      <p>Each {w} is listed here.</p>\n      <a href=\"/{w}\">More</a>\n    </section>");
    }
    if finding {
        s.push_str("    <img src=\"/logo.png\">\n");
    }
    s.push_str("  </body>\n</html>\n");
}

fn cap(w: &str) -> String {
    let mut c = w.chars();
    c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_bytes() {
        let a = generate(SEED, 600);
        let b = generate(SEED, 600);
        assert_eq!(digest(&a), digest(&b));
        assert_ne!(digest(&a), digest(&generate(SEED + 1, 600)));
        assert_eq!(a.len(), 600);
    }

    #[test]
    fn every_l3_language_is_in_the_mix() {
        let set = generate(SEED, 1000);
        for ext in ["ts", "tsx", "js", "jsx", "py", "json", "yaml", "toml", "md", "css", "html"] {
            assert!(set.iter().any(|f| f.path.ends_with(&format!(".{ext}"))), "no .{ext} file");
        }
        let mut paths: Vec<&str> = set.iter().map(|f| f.path.as_str()).collect();
        paths.dedup();
        assert_eq!(paths.len(), 1000, "a path is generated twice");
    }

    /// The reference workload is pinned: budgets measured on one set of
    /// bytes bind only that set. A change to the generator changes this
    /// digest, and every budget must then be re-measured (L8 decision 5).
    #[test]
    fn the_reference_workload_is_pinned() {
        let set = generate(SEED, FILES);
        assert_eq!(set.len(), FILES);
        assert_eq!(digest(&set), REFERENCE_DIGEST, "the reference workload changed: every L8 budget must be re-measured on it");
    }
}
