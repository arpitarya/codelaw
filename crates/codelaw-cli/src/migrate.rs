//! `codelaw migrate eslint|prettier|biome|ruff|stylelint [--write]
//! [--format text|json]` (SR-CLI-MCP decision 15, W-94): read another
//! tool's config and write the `codelaw.toml` that runs the same rules.
//!
//! Every source rule maps through the `ported_from:` of the rules in the
//! installed first-party packs, a Biome rule through the sources Biome
//! names (`codelaw-engine/data/biome-sources.json`), a preset through
//! `data/migrate-presets.json`. Nothing is guessed: what does not map is
//! in the report, by name, with the reason. A config that is a program is
//! never run (L5): a flat `eslint.config.*` is read only where it is
//! literal, through the JavaScript grammar; the rest is reported.

use codelaw_core::{Language, template};
use codelaw_engine::suppress::{Alias, upstream_keys};
use codelaw_format::import::{Imported, PRETTIER_FILES, PRETTIER_PROGRAMS};
use codelaw_lang::SyntaxNode;
use codelaw_rules::{FIRST_PARTY_PACKS, PackSearch, RuleFile, RulePack, Stability, load_pack_from, namespace_pack};
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Tool {
    Eslint,
    Prettier,
    Biome,
    Ruff,
    Stylelint,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Tool::Eslint => "eslint",
            Tool::Prettier => "prettier",
            Tool::Biome => "biome",
            Tool::Ruff => "ruff",
            Tool::Stylelint => "stylelint",
        }
    }

    /// The tool whose line suppressions the written config accepts.
    fn alias(self) -> Option<Alias> {
        match self {
            Tool::Eslint => Some(Alias::Eslint),
            Tool::Biome => Some(Alias::Biome),
            Tool::Ruff => Some(Alias::Ruff),
            Tool::Prettier | Tool::Stylelint => None,
        }
    }
}

#[derive(clap::Args)]
pub struct MigrateArgs {
    /// The tool whose config is read.
    tool: Tool,
    /// Write `codelaw.toml`; without it the file is printed, not written.
    #[arg(long)]
    write: bool,
    /// text or json.
    #[arg(long, default_value = "text")]
    format: String,
    /// The project folder; the current one by default.
    #[arg(long, value_name = "DIR")]
    dir: Option<PathBuf>,
}

const PRESETS: &str = include_str!("../data/migrate-presets.json");

/// One source rule as the tool's config sets it.
#[derive(Clone, Debug)]
struct SourceRule {
    /// The tool's id: `no-console`, `noDebugger`, `F401`, `color-no-invalid-hex`.
    id: String,
    /// `error`, `warn` or `info`; `None` is off. `Some("")` is the tool's
    /// "on" with no severity: the codelaw rule's own.
    severity: Option<String>,
    /// The tool's options, when it gave any.
    options: Option<Json>,
}

/// One `files` group of the source (an ESLint override, a Biome override,
/// a Ruff per-file ignore), written as a `[scope]`.
#[derive(Clone, Debug, Default)]
struct SourceScope {
    files: Vec<String>,
    rules: Vec<SourceRule>,
}

/// What a tool's config holds, before mapping.
#[derive(Debug, Default)]
struct Input {
    sources: Vec<String>,
    presets: Vec<String>,
    rules: Vec<SourceRule>,
    scopes: Vec<SourceScope>,
    /// Ruff: the selected code prefixes, expanded over the codes codelaw ports.
    select: Option<RuffSelect>,
    /// `<where>: <setting>` and why it has no codelaw equivalent.
    ignored: Vec<(String, String)>,
    format: Imported,
}

#[derive(Debug, Default)]
struct RuffSelect {
    select: Vec<String>,
    ignore: Vec<String>,
    /// `select` was not set: Ruff's default.
    default: bool,
}

/// One line of the report.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
    pub source: String,
    pub codelaw: Vec<String>,
    pub note: String,
}

/// What the migration found and would write.
#[derive(Debug, Default)]
pub struct Report {
    pub tool: &'static str,
    pub sources: Vec<String>,
    pub mapped: Vec<Line>,
    pub changed: Vec<Line>,
    pub unmapped: Vec<Line>,
    pub ignored: Vec<Line>,
    /// First-party packs not installed, so not searched.
    pub not_installed: Vec<String>,
    pub config: String,
    pub written: Option<PathBuf>,
    /// What an existing `codelaw.toml` already set, so kept as it was.
    pub kept: Vec<String>,
    /// It added to an existing `codelaw.toml` rather than writing one.
    pub merged: bool,
}

/// The installed first-party packs, and `ported_from:` id → codelaw ids.
struct Catalog {
    rules: BTreeMap<String, Arc<RuleFile>>,
    by_upstream: BTreeMap<String, Vec<String>>,
    packs: BTreeMap<&'static str, Arc<RulePack>>,
    not_installed: Vec<String>,
}

impl Catalog {
    fn load(root: &Path) -> Catalog {
        let search = PackSearch::new(root, root, std::env::var("CODELAW_SITE_PACKAGES").ok().as_deref(), std::env::var("VIRTUAL_ENV").ok().as_deref());
        let mut c = Catalog { rules: BTreeMap::new(), by_upstream: BTreeMap::new(), packs: BTreeMap::new(), not_installed: Vec::new() };
        for p in FIRST_PARTY_PACKS {
            let Some(pack) = search.find_first_party(p.short).and_then(|d| load_pack_from(&d, true).ok()) else {
                c.not_installed.push(p.npm.or(p.pypi).unwrap_or(p.short).to_owned());
                continue;
            };
            let pack = Arc::new(pack);
            for r in &pack.rules {
                let r = Arc::new(r.clone());
                for up in &r.ported_from {
                    let ids = c.by_upstream.entry(up.clone()).or_default();
                    if !ids.contains(&r.id) {
                        ids.push(r.id.clone());
                    }
                }
                c.rules.insert(r.id.clone(), r);
            }
            c.packs.insert(p.short, pack);
        }
        c
    }

    /// The codelaw rules that port a source rule.
    fn lookup(&self, tool: Tool, id: &str) -> Vec<String> {
        let keys: Vec<String> = match tool {
            Tool::Stylelint => match id.strip_prefix("scss/") {
                Some(rest) => vec![format!("stylelint-scss/{rest}")],
                None => vec![format!("stylelint/{id}")],
            },
            Tool::Prettier => Vec::new(),
            Tool::Eslint => upstream_keys(Alias::Eslint, id),
            Tool::Biome => upstream_keys(Alias::Biome, id),
            // A Ruff code is a prefix of the codes it selects: `E`, `F4`, `F401`.
            Tool::Ruff => {
                let prefix = id.to_ascii_uppercase();
                self.by_upstream.keys().filter_map(|k| k.strip_prefix("ruff/")).filter(|c| prefix == "ALL" || c.starts_with(prefix.as_str())).map(|c| format!("ruff/{c}")).collect()
            }
        };
        let mut out: Vec<String> = Vec::new();
        for k in keys {
            for id in self.by_upstream.get(&k).into_iter().flatten() {
                if !out.contains(id) {
                    out.push(id.clone());
                }
            }
        }
        out
    }

    fn pack_of(&self, id: &str) -> Option<&'static str> {
        id.split_once('/').and_then(|(ns, _)| namespace_pack(ns))
    }

    /// The rules a first-party preset turns on, its own `extends` included.
    fn preset_rules(&self, spec: &str, out: &mut BTreeSet<String>) -> bool {
        let Some((ns, name)) = spec.strip_prefix("codelaw:").and_then(|s| s.split_once('/')) else { return false };
        let Some(pack) = namespace_pack(ns).and_then(|p| self.packs.get(p)) else { return false };
        let Some(preset) = pack.presets.get(&format!("{ns}/{name}")) else { return false };
        for e in &preset.extends {
            self.preset_rules(e, out);
        }
        for (id, s) in &preset.rules {
            if s.severity() != Some("off") {
                out.insert(id.clone());
            }
        }
        true
    }
}

/// The ESLint config files, flat first, in the order ESLint looks.
const ESLINT_FLAT: &[&str] = &["eslint.config.js", "eslint.config.mjs", "eslint.config.cjs", "eslint.config.ts", "eslint.config.mts", "eslint.config.cts"];
const ESLINTRC: &[&str] = &[".eslintrc.json", ".eslintrc.yaml", ".eslintrc.yml", ".eslintrc"];
const ESLINTRC_PROGRAMS: &[&str] = &[".eslintrc.js", ".eslintrc.cjs"];
const STYLELINT_FILES: &[&str] = &[".stylelintrc", ".stylelintrc.json", ".stylelintrc.yaml", ".stylelintrc.yml"];
const STYLELINT_PROGRAMS: &[&str] = &[".stylelintrc.js", ".stylelintrc.cjs", ".stylelintrc.mjs", "stylelint.config.js", "stylelint.config.cjs", "stylelint.config.mjs", "stylelint.config.ts"];
const BIOME_FILES: &[&str] = &["biome.json", "biome.jsonc"];
const RUFF_FILES: &[&str] = &["ruff.toml", ".ruff.toml"];
/// Ruff's default rule selection when a config sets none.
const RUFF_DEFAULT: &[&str] = &["E4", "E7", "E9", "F"];

fn read(root: &Path, name: &str) -> Option<String> {
    std::fs::read_to_string(root.join(name)).ok()
}

/// JSON with comments and trailing commas (`.eslintrc`, `biome.jsonc`), or YAML.
fn parse_data(name: &str, text: &str) -> Result<Json, String> {
    if name.ends_with(".yaml") || name.ends_with(".yml") {
        return serde_saphyr::from_str::<Json>(text).map_err(|e| format!("{name}: {e}"));
    }
    if let Some(v) = codelaw_graph::resolve::jsonc(text) {
        return Ok(v);
    }
    if name.ends_with(".json5") {
        return Err(format!("{name}: JSON5 that is not plain JSON is not read; convert it to JSON first"));
    }
    // `.eslintrc` and `.stylelintrc` may be YAML.
    serde_saphyr::from_str::<Json>(text).map_err(|_| format!("{name}: neither JSON nor YAML"))
}

fn package_json_key(root: &Path, key: &str) -> Option<Json> {
    let text = read(root, "package.json")?;
    serde_json::from_str::<Json>(&text).ok()?.get(key).cloned()
}

/// Why a config that is a program is not read (L5).
fn program(name: &str) -> String {
    format!("{name} is a program, and codelaw never runs one (L5): convert it to JSON first, or list the rules by hand")
}

fn strings(v: Option<&Json>) -> Vec<String> {
    match v {
        Some(Json::String(s)) => vec![s.clone()],
        Some(Json::Array(a)) => a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------- ESLint

/// An ESLint rule setting: `2`, `"warn"`, `["error", { … }]`.
fn eslint_rule(id: &str, v: &Json) -> Option<SourceRule> {
    let (level, options) = match v {
        Json::Array(a) => (a.first()?, (a.len() > 1).then(|| if a.len() == 2 { a[1].clone() } else { Json::Array(a[1..].to_vec()) })),
        other => (other, None),
    };
    let severity = match level {
        Json::Number(n) => match n.as_i64()? {
            0 => None,
            1 => Some("warn".to_owned()),
            2 => Some("error".to_owned()),
            _ => return None,
        },
        Json::String(s) => match s.as_str() {
            "off" => None,
            "warn" => Some("warn".to_owned()),
            "error" => Some("error".to_owned()),
            _ => return None,
        },
        _ => return None,
    };
    Some(SourceRule { id: id.to_owned(), severity, options })
}

fn eslint_rules(source: &str, rules: Option<&Json>, input: &mut Input) -> Vec<SourceRule> {
    let mut out = Vec::new();
    for (id, v) in rules.and_then(Json::as_object).into_iter().flatten() {
        match eslint_rule(id, v) {
            Some(r) => out.push(r),
            None => input.ignored.push((format!("{source}: rules.{id} = {v}"), "not a severity ESLint takes".into())),
        }
    }
    out
}

/// `.eslintrc*` and `package.json`'s `eslintConfig`.
fn eslintrc(source: &str, v: &Json, input: &mut Input) {
    input.sources.push(source.to_owned());
    let Some(map) = v.as_object() else {
        input.ignored.push((source.to_owned(), "not an object".into()));
        return;
    };
    for (k, val) in map {
        match k.as_str() {
            "extends" => input.presets.extend(strings(Some(val))),
            "rules" => {
                let r = eslint_rules(source, Some(val), input);
                input.rules.extend(r);
            }
            "overrides" => {
                for (i, o) in val.as_array().into_iter().flatten().enumerate() {
                    let files = strings(o.get("files"));
                    let rules = eslint_rules(&format!("{source} overrides[{i}]"), o.get("rules"), input);
                    for (ok, ov) in o.as_object().into_iter().flatten() {
                        if !matches!(ok.as_str(), "files" | "rules") {
                            input.ignored.push((format!("{source} overrides[{i}]: {ok} = {ov}"), "only an override's files and rules map".into()));
                        }
                    }
                    if !files.is_empty() && !rules.is_empty() {
                        input.scopes.push(SourceScope { files, rules });
                    }
                }
            }
            "plugins" | "root" => {}
            "ignorePatterns" => input.ignored.push((format!("{source}: ignorePatterns = {val}"), "write them in codelaw's `exclude`".into())),
            other => input.ignored.push((format!("{source}: {other} = {val}"), "codelaw has no setting for it: it parses every file itself".into())),
        }
    }
}

fn read_eslint(root: &Path, input: &mut Input) -> Result<(), String> {
    for name in ESLINT_FLAT {
        if let Some(text) = read(root, name) {
            return flat::read(name, &text, input);
        }
    }
    for name in ESLINTRC {
        if let Some(text) = read(root, name) {
            let v = parse_data(name, &text)?;
            eslintrc(name, &v, input);
            return Ok(());
        }
    }
    if let Some(name) = ESLINTRC_PROGRAMS.iter().find(|n| root.join(n).is_file()) {
        return Err(program(name));
    }
    if let Some(v) = package_json_key(root, "eslintConfig") {
        eslintrc("package.json \"eslintConfig\"", &v, input);
        return Ok(());
    }
    Err("no ESLint config here: eslint.config.*, .eslintrc.* or package.json \"eslintConfig\"".into())
}

/// A flat config read without running it: the literal parts of the array
/// it exports, through the JavaScript or TypeScript grammar.
mod flat {
    use super::*;

    /// A node's literal value, or `None` when it is computed.
    fn literal(n: &SyntaxNode<'_>) -> Option<Json> {
        match n.kind().as_ref() {
            "string" => {
                let t = n.text();
                Some(Json::String(t[1..t.len().saturating_sub(1)].to_owned()))
            }
            "number" => n.text().parse::<f64>().ok().and_then(|f| if f.fract() == 0.0 { Some(json!(f as i64)) } else { serde_json::Number::from_f64(f).map(Json::Number) }),
            "true" => Some(Json::Bool(true)),
            "false" => Some(Json::Bool(false)),
            "null" => Some(Json::Null),
            "array" => n.children().filter(|c| c.is_named() && c.kind() != "comment").map(|c| literal(&c)).collect::<Option<Vec<_>>>().map(Json::Array),
            "object" => {
                let mut m = serde_json::Map::new();
                for c in n.children().filter(|c| c.is_named() && c.kind() != "comment") {
                    if c.kind() != "pair" {
                        return None;
                    }
                    let k = key(&c.field("key")?)?;
                    m.insert(k, literal(&c.field("value")?)?);
                }
                Some(Json::Object(m))
            }
            "parenthesized_expression" | "as_expression" | "satisfies_expression" => n.children().find(|c| c.is_named()).and_then(|c| literal(&c)),
            _ => None,
        }
    }

    fn key(n: &SyntaxNode<'_>) -> Option<String> {
        match n.kind().as_ref() {
            "property_identifier" | "identifier" => Some(n.text().into_owned()),
            "string" => literal(n).and_then(|v| v.as_str().map(str::to_owned)),
            _ => None,
        }
    }

    /// `x.configs.recommended`, `x.configs["flat/recommended"]` → (`x`, `configs.recommended`).
    fn path(n: &SyntaxNode<'_>) -> Option<(String, String)> {
        match n.kind().as_ref() {
            "identifier" => Some((n.text().into_owned(), String::new())),
            "member_expression" => {
                let (base, p) = path(&n.field("object")?)?;
                let prop = n.field("property")?.text().into_owned();
                Some((base, if p.is_empty() { prop } else { format!("{p}.{prop}") }))
            }
            "subscript_expression" => {
                let (base, p) = path(&n.field("object")?)?;
                let prop = literal(&n.field("index")?)?.as_str()?.to_owned();
                Some((base, if p.is_empty() { prop } else { format!("{p}.{prop}") }))
            }
            _ => None,
        }
    }

    /// `import x from "m"`, `import * as x from "m"`, `const x = require("m")`.
    fn imports(program: &SyntaxNode<'_>) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for n in program.dfs() {
            match n.kind().as_ref() {
                "import_statement" => {
                    let Some(m) = n.field("source").and_then(|s| literal(&s)).and_then(|v| v.as_str().map(str::to_owned)) else { continue };
                    for c in n.dfs() {
                        if c.kind() == "identifier" && c.parent().is_some_and(|p| matches!(p.kind().as_ref(), "import_clause" | "namespace_import")) {
                            out.insert(c.text().into_owned(), m.clone());
                        }
                    }
                }
                "variable_declarator" => {
                    if let (Some(name), Some(v)) = (n.field("name"), n.field("value"))
                        && name.kind() == "identifier"
                        && v.kind() == "call_expression"
                        && v.field("function").is_some_and(|f| f.text() == "require")
                        && let Some(m) = v.field("arguments").and_then(|a| a.children().find(|c| c.kind() == "string")).and_then(|s| literal(&s))
                    {
                        out.insert(name.text().into_owned(), m.as_str().unwrap_or_default().to_owned());
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// The value a top-level `const name = …` gives `name`.
    fn binding<'r>(program: &SyntaxNode<'r>, name: &str) -> Option<SyntaxNode<'r>> {
        program.children().filter(|c| matches!(c.kind().as_ref(), "lexical_declaration" | "variable_declaration")).find_map(|d| {
            d.children().filter(|c| c.kind() == "variable_declarator").find(|c| c.field("name").is_some_and(|n| n.text() == name)).and_then(|c| c.field("value"))
        })
    }

    /// The exported config: `export default …` or `module.exports = …`.
    fn exported<'r>(program: &SyntaxNode<'r>) -> Option<SyntaxNode<'r>> {
        for c in program.children() {
            match c.kind().as_ref() {
                "export_statement" if c.children().any(|x| x.kind() == "default") => {
                    return c.field("value").or_else(|| c.children().filter(|x| x.is_named()).last());
                }
                "expression_statement" => {
                    if let Some(a) = c.children().find(|x| x.kind() == "assignment_expression")
                        && a.field("left").is_some_and(|l| l.text().replace(char::is_whitespace, "") == "module.exports")
                    {
                        return a.field("right");
                    }
                }
                _ => {}
            }
        }
        None
    }

    struct Reader<'a> {
        name: &'a str,
        imports: BTreeMap<String, String>,
        input: &'a mut Input,
    }

    impl Reader<'_> {
        /// The elements of the exported array, through `defineConfig(…)`,
        /// `tseslint.config(…)` and a top-level binding.
        fn elements<'r>(&self, program: &SyntaxNode<'r>, n: SyntaxNode<'r>, depth: usize) -> Option<Vec<SyntaxNode<'r>>> {
            if depth > 4 {
                return None;
            }
            match n.kind().as_ref() {
                "array" => Some(n.children().filter(|c| c.is_named() && c.kind() != "comment").collect()),
                "object" => Some(vec![n]),
                "identifier" => binding(program, &n.text()).and_then(|v| self.elements(program, v, depth + 1)),
                "parenthesized_expression" | "as_expression" | "satisfies_expression" => n.children().find(|c| c.is_named()).and_then(|c| self.elements(program, c, depth + 1)),
                // `defineConfig([…])`, `tseslint.config(…)`: the calls that
                // only gather configs. Any other call computes one.
                "call_expression" => {
                    let callee = n.field("function")?.text().into_owned();
                    if callee != "defineConfig" && !callee.ends_with(".config") {
                        return None;
                    }
                    let args: Vec<SyntaxNode<'r>> = n.field("arguments")?.children().filter(|c| c.is_named() && c.kind() != "comment").collect();
                    let mut out = Vec::new();
                    for a in args {
                        match a.kind().as_ref() {
                            "array" => out.extend(self.elements(program, a, depth + 1)?),
                            _ => out.push(a),
                        }
                    }
                    Some(out)
                }
                _ => None,
            }
        }

        fn preset(&mut self, n: &SyntaxNode<'_>) {
            let n = if n.kind() == "spread_element" { n.children().find(|c| c.is_named()) } else { Some(n.clone()) };
            let named = n.as_ref().and_then(path).and_then(|(base, p)| self.imports.get(&base).map(|m| format!("{m}:{p}")));
            match named {
                Some(p) => self.input.presets.push(p),
                None => self.input.ignored.push((format!("{}: {}", self.name, one_line(n.as_ref().map(|n| n.text().into_owned()).unwrap_or_default())), "computed, so not read: codelaw never runs a config (L5); list its rules by hand".into())),
            }
        }

        fn object(&mut self, n: &SyntaxNode<'_>, i: usize) {
            let mut files = Vec::new();
            let mut rules = Vec::new();
            for c in n.children().filter(|c| c.is_named() && c.kind() != "comment") {
                let k = if c.kind() == "pair" { c.field("key").and_then(|k| key(&k)) } else { None };
                let at = format!("{} [{i}]", self.name);
                match (k.as_deref(), c.field("value")) {
                    (Some("rules"), Some(v)) => {
                        if v.kind() != "object" {
                            self.input.ignored.push((format!("{at}: rules = {}", one_line(v.text().into_owned())), "computed, so not read (L5); list the rules by hand".into()));
                            continue;
                        }
                        for p in v.children().filter(|p| p.is_named() && p.kind() != "comment") {
                            let id = if p.kind() == "pair" { p.field("key").and_then(|k| key(&k)) } else { None };
                            let val = p.field("value").and_then(|x| literal(&x));
                            match (id, val) {
                                (Some(id), Some(val)) => match eslint_rule(&id, &val) {
                                    Some(r) => rules.push(r),
                                    None => self.input.ignored.push((format!("{at}: rules.{id} = {val}"), "not a severity ESLint takes".into())),
                                },
                                _ => self.input.ignored.push((format!("{at}: {}", one_line(p.text().into_owned())), "computed, so not read (L5); set it by hand".into())),
                            }
                        }
                    }
                    (Some("files"), Some(v)) => match literal(&v) {
                        Some(f) => files = strings(Some(&f)),
                        None => self.input.ignored.push((format!("{at}: files"), "computed, so not read (L5)".into())),
                    },
                    (Some("extends"), Some(v)) => {
                        for e in v.children().filter(|e| e.is_named() && e.kind() != "comment") {
                            match literal(&e) {
                                Some(Json::String(s)) => self.input.presets.push(s),
                                _ => self.preset(&e),
                            }
                        }
                    }
                    (Some(other @ ("ignores" | "languageOptions" | "plugins" | "settings" | "linterOptions" | "name" | "processor" | "language")), _) => {
                        let why = if other == "ignores" { "write them in codelaw's `exclude`" } else { "codelaw has no setting for it: it parses every file itself" };
                        self.input.ignored.push((format!("{at}: {other}"), why.into()));
                    }
                    _ => self.input.ignored.push((format!("{at}: {}", one_line(c.text().into_owned())), "not a key codelaw reads".into())),
                }
            }
            if files.is_empty() {
                self.input.rules.extend(rules);
            } else if !rules.is_empty() {
                self.input.scopes.push(SourceScope { files, rules });
            }
        }
    }

    fn one_line(s: String) -> String {
        let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
        if s.chars().count() > 60 { format!("{}…", s.chars().take(60).collect::<String>()) } else { s }
    }

    pub(super) fn read(name: &str, text: &str, input: &mut Input) -> Result<(), String> {
        let language = if name.ends_with("ts") { Language::Typescript } else { Language::Javascript };
        let root = codelaw_lang::parse(language, text);
        let program = root.root();
        let mut r = Reader { name, imports: imports(&program), input };
        let Some(elements) = exported(&program).and_then(|e| r.elements(&program, e, 0)) else {
            return Err(format!("{name}: what it exports is computed, and codelaw never runs a config (L5): convert it to JSON first, or list the rules by hand"));
        };
        r.input.sources.push(name.to_owned());
        for (i, e) in elements.iter().enumerate() {
            if e.kind() == "object" {
                r.object(e, i);
            } else {
                r.preset(e);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- Biome

fn biome_level(v: &Json) -> Option<(Option<String>, Option<Json>)> {
    let (level, options) = match v {
        Json::String(s) => (s.as_str(), None),
        Json::Object(o) => (o.get("level").and_then(Json::as_str).unwrap_or("on"), o.get("options").cloned()),
        _ => return None,
    };
    let severity = match level {
        "off" => None,
        "on" => Some(String::new()),
        "error" | "warn" | "info" => Some(level.to_owned()),
        _ => return None,
    };
    Some((severity, options))
}

fn biome_rules(source: &str, rules: &Json, input: &mut Input) -> Vec<SourceRule> {
    let mut out = Vec::new();
    for (group, v) in rules.as_object().into_iter().flatten() {
        match (group.as_str(), v) {
            ("recommended", Json::Bool(b)) => {
                if *b {
                    input.presets.push("recommended".into());
                }
            }
            ("all", _) => input.ignored.push((format!("{source}: linter.rules.all = {v}"), "codelaw has no every-rule switch; list the rules by hand".into())),
            (_, Json::Object(rs)) => {
                for (name, r) in rs {
                    if name == "recommended" || name == "all" {
                        input.ignored.push((format!("{source}: linter.rules.{group}.{name} = {r}"), "a group switch: codelaw maps Biome's rules one by one".into()));
                        continue;
                    }
                    match biome_level(r) {
                        Some((severity, options)) => out.push(SourceRule { id: name.clone(), severity, options }),
                        None => input.ignored.push((format!("{source}: linter.rules.{group}.{name} = {r}"), "not a level Biome takes".into())),
                    }
                }
            }
            _ => input.ignored.push((format!("{source}: linter.rules.{group} = {v}"), "a group switch: codelaw maps Biome's rules one by one".into())),
        }
    }
    out
}

fn biome_format(source: &str, f: &Json, prefix: &str, input: &mut Input) {
    for (k, v) in f.as_object().into_iter().flatten() {
        let to = match (k.as_str(), v) {
            ("lineWidth", Json::Number(n)) => n.as_i64().map(|n| ("line_width", toml::Value::Integer(n))),
            ("indentWidth", Json::Number(n)) => n.as_i64().map(|n| ("indent_width", toml::Value::Integer(n))),
            ("indentStyle", Json::String(s)) if s == "tab" || s == "space" => Some(("indent_style", toml::Value::String(s.clone()))),
            ("lineEnding", Json::String(s)) if s == "lf" || s == "crlf" => Some(("end_of_line", toml::Value::String(s.clone()))),
            ("quoteStyle", Json::String(s)) if s == "single" || s == "double" => Some(("quote_style", toml::Value::String(s.clone()))),
            ("semicolons", Json::String(s)) if s == "always" || s == "asNeeded" => Some(("semicolons", toml::Value::String(if s == "always" { "always" } else { "as-needed" }.into()))),
            ("trailingCommas", Json::String(s)) if matches!(s.as_str(), "all" | "es5" | "none") => Some(("trailing_commas", toml::Value::String(s.clone()))),
            _ => None,
        };
        match to {
            Some((key, value)) => {
                if !input.format.keys.iter().any(|(k, _)| k == key) {
                    input.format.keys.push((key.to_owned(), value));
                }
            }
            None => input.ignored.push((format!("{source}: {prefix}{k} = {v}"), "no [format] key holds it".into())),
        }
    }
}

fn read_biome(root: &Path, input: &mut Input) -> Result<(), String> {
    let Some((name, text)) = BIOME_FILES.iter().find_map(|n| read(root, n).map(|t| (*n, t))) else {
        return Err("no Biome config here: biome.json or biome.jsonc".into());
    };
    let v = parse_data(name, &text)?;
    input.sources.push(name.to_owned());
    let mut recommended = true;
    for (k, val) in v.as_object().into_iter().flatten() {
        match k.as_str() {
            "$schema" => {}
            "linter" => {
                for (lk, lv) in val.as_object().into_iter().flatten() {
                    match lk.as_str() {
                        "enabled" if lv == &Json::Bool(false) => {
                            input.ignored.push((format!("{name}: linter.enabled = false"), "Biome's linter is off, so no rule is carried".into()));
                            return Ok(());
                        }
                        "enabled" => {}
                        "rules" => {
                            if lv.get("recommended") == Some(&Json::Bool(false)) {
                                recommended = false;
                            }
                            let r = biome_rules(name, lv, input);
                            input.rules.extend(r);
                        }
                        other => input.ignored.push((format!("{name}: linter.{other} = {lv}"), "codelaw has no setting for it".into())),
                    }
                }
            }
            "formatter" => biome_format(name, val, "formatter.", input),
            "javascript" => {
                for (jk, jv) in val.as_object().into_iter().flatten() {
                    if jk == "formatter" {
                        biome_format(name, jv, "javascript.formatter.", input);
                    } else {
                        input.ignored.push((format!("{name}: javascript.{jk} = {jv}"), "codelaw has no setting for it".into()));
                    }
                }
            }
            "overrides" => {
                for (i, o) in val.as_array().into_iter().flatten().enumerate() {
                    let files = strings(o.get("includes").or_else(|| o.get("include")));
                    let rules = o.pointer("/linter/rules").map(|r| biome_rules(&format!("{name} overrides[{i}]"), r, input)).unwrap_or_default();
                    if !files.is_empty() && !rules.is_empty() {
                        input.scopes.push(SourceScope { files, rules });
                    } else {
                        input.ignored.push((format!("{name}: overrides[{i}]"), "only an override's includes and linter.rules map".into()));
                    }
                }
            }
            "files" => input.ignored.push((format!("{name}: files = {val}"), "write ignored paths in codelaw's `exclude`".into())),
            other => input.ignored.push((format!("{name}: {other}"), "codelaw has no setting for it".into())),
        }
    }
    // Biome's recommended set is on unless the config turns it off.
    if recommended && !input.presets.iter().any(|p| p == "recommended") {
        input.presets.push("recommended".into());
    }
    Ok(())
}

// ---------------------------------------------------------------- Ruff

fn read_ruff(root: &Path, input: &mut Input) -> Result<(), String> {
    let (name, table) = if let Some((name, text)) = RUFF_FILES.iter().find_map(|n| read(root, n).map(|t| (*n, t))) {
        (name.to_owned(), toml::from_str::<toml::Table>(&text).map_err(|e| format!("{name}: {e}"))?)
    } else if let Some(text) = read(root, "pyproject.toml") {
        let t: toml::Table = toml::from_str(&text).map_err(|e| format!("pyproject.toml: {e}"))?;
        match t.get("tool").and_then(|t| t.get("ruff")).and_then(toml::Value::as_table) {
            Some(r) => ("pyproject.toml [tool.ruff]".to_owned(), r.clone()),
            None => return Err("no Ruff config here: ruff.toml, .ruff.toml or [tool.ruff] in pyproject.toml".into()),
        }
    } else {
        return Err("no Ruff config here: ruff.toml, .ruff.toml or [tool.ruff] in pyproject.toml".into());
    };
    input.sources.push(name.clone());
    // The formatter's keys, through the import `format --import-config` uses.
    let mut wrapped = toml::Table::new();
    let mut tool = toml::Table::new();
    tool.insert("ruff".into(), toml::Value::Table(table.clone()));
    wrapped.insert("tool".into(), toml::Value::Table(tool));
    let _ = input.format.pyproject(&toml::to_string(&wrapped).unwrap_or_default());
    input.format.skipped.retain(|s| !s.starts_with("pyproject.toml [tool.ruff.format]"));
    input.format.sources.clear();

    let lint = table.get("lint").and_then(toml::Value::as_table);
    let get = |k: &str| lint.and_then(|l| l.get(k)).or_else(|| table.get(k));
    let codes = |k: &str| -> Vec<String> { get(k).and_then(toml::Value::as_array).into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect() };
    let mut sel = RuffSelect::default();
    match get("select") {
        Some(_) => sel.select = codes("select"),
        None => {
            sel.select = RUFF_DEFAULT.iter().map(|s| s.to_string()).collect();
            sel.default = true;
        }
    }
    sel.select.extend(codes("extend-select"));
    sel.ignore = codes("ignore");
    sel.ignore.extend(codes("extend-ignore"));
    for key in ["per-file-ignores", "extend-per-file-ignores"] {
        for (glob, v) in get(key).and_then(toml::Value::as_table).into_iter().flatten() {
            let files = vec![if glob.contains('/') { glob.clone() } else { format!("**/{glob}") }];
            let rules = v.as_array().into_iter().flatten().filter_map(|c| c.as_str()).map(|c| SourceRule { id: c.to_owned(), severity: None, options: None }).collect();
            input.scopes.push(SourceScope { files, rules });
        }
    }
    let read_keys = ["select", "extend-select", "ignore", "extend-ignore", "per-file-ignores", "extend-per-file-ignores", "line-length", "indent-width", "format", "lint"];
    for (k, v) in &table {
        if !read_keys.contains(&k.as_str()) {
            input.ignored.push((format!("{name}: {k} = {v}"), "codelaw has no setting for it".into()));
        }
    }
    for (k, v) in lint.into_iter().flatten() {
        if !read_keys.contains(&k.as_str()) {
            let why = if v.is_table() { "a plugin's settings: set the codelaw rule's options by hand (`codelaw explain <id>`)" } else { "codelaw has no setting for it" };
            input.ignored.push((format!("{name}: lint.{k} = {v}"), why.into()));
        }
    }
    if let Some(f) = table.get("format").and_then(toml::Value::as_table) {
        for (k, v) in f {
            input.ignored.push((format!("{name}: format.{k} = {v}"), "codelaw formats Python with Ruff's defaults for it".into()));
        }
    }
    input.select = Some(sel);
    Ok(())
}

/// Whether Ruff selects `code`: the longest matching prefix decides, and
/// an ignore wins a tie, as Ruff resolves them.
fn ruff_selected(code: &str, sel: &RuffSelect) -> bool {
    let best = |list: &[String]| list.iter().filter(|p| *p == "ALL" || code.starts_with(p.as_str())).map(|p| if p == "ALL" { 0 } else { p.len() }).max();
    match (best(&sel.select), best(&sel.ignore)) {
        (Some(s), Some(i)) => s > i,
        (Some(_), None) => true,
        _ => false,
    }
}

// ---------------------------------------------------------------- stylelint

fn read_stylelint(root: &Path, input: &mut Input) -> Result<(), String> {
    let (name, v) = if let Some((name, text)) = STYLELINT_FILES.iter().find_map(|n| read(root, n).map(|t| (*n, t))) {
        (name.to_owned(), parse_data(name, &text)?)
    } else if let Some(name) = STYLELINT_PROGRAMS.iter().find(|n| root.join(n).is_file()) {
        return Err(program(name));
    } else if let Some(v) = package_json_key(root, "stylelint") {
        ("package.json \"stylelint\"".to_owned(), v)
    } else {
        return Err("no stylelint config here: .stylelintrc*, or package.json \"stylelint\"".into());
    };
    input.sources.push(name.clone());
    let default = match v.get("defaultSeverity").and_then(Json::as_str) {
        Some("warning") => "warn",
        _ => "error",
    };
    let rule = |id: &str, r: &Json| -> SourceRule {
        let (primary, secondary) = match r {
            Json::Array(a) if a.len() == 2 && a[1].is_object() => (a[0].clone(), Some(&a[1])),
            other => (other.clone(), None),
        };
        let severity = match (&primary, secondary.and_then(|s| s.get("severity")).and_then(Json::as_str)) {
            (Json::Null, _) | (Json::Bool(false), _) => None,
            (_, Some("warning")) => Some("warn".to_owned()),
            (_, Some(_)) => Some("error".to_owned()),
            (_, None) => Some(default.to_owned()),
        };
        let mut opts = serde_json::Map::new();
        if !matches!(primary, Json::Bool(true) | Json::Null) {
            opts.insert("primary".into(), primary);
        }
        for (k, v) in secondary.and_then(Json::as_object).into_iter().flatten() {
            if !matches!(k.as_str(), "severity" | "message" | "url" | "reportDisableComments" | "disableFix") {
                opts.insert(k.clone(), v.clone());
            }
        }
        SourceRule { id: id.to_owned(), severity, options: (!opts.is_empty()).then_some(Json::Object(opts)) }
    };
    for (k, val) in v.as_object().into_iter().flatten() {
        match k.as_str() {
            "extends" => input.presets.extend(strings(Some(val))),
            "rules" => {
                for (id, r) in val.as_object().into_iter().flatten() {
                    input.rules.push(rule(id, r));
                }
            }
            "overrides" => {
                for (i, o) in val.as_array().into_iter().flatten().enumerate() {
                    let files = strings(o.get("files"));
                    let rules: Vec<SourceRule> = o.get("rules").and_then(Json::as_object).into_iter().flatten().map(|(id, r)| rule(id, r)).collect();
                    if !files.is_empty() && !rules.is_empty() {
                        input.scopes.push(SourceScope { files, rules });
                    } else {
                        input.ignored.push((format!("{name}: overrides[{i}]"), "only an override's files and rules map".into()));
                    }
                }
            }
            "defaultSeverity" | "plugins" => {}
            "ignoreFiles" => input.ignored.push((format!("{name}: ignoreFiles = {val}"), "write them in codelaw's `exclude`".into())),
            other => input.ignored.push((format!("{name}: {other} = {val}"), "codelaw has no setting for it".into())),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- Prettier

fn read_prettier(root: &Path, input: &mut Input) -> Result<(), String> {
    for name in PRETTIER_FILES {
        if let Some(text) = read(root, name) {
            input.format.prettier(name, &text)?;
        }
    }
    if let Some(p) = package_json_key(root, "prettier") {
        input.format.prettier_object("package.json \"prettier\"", &p)?;
    }
    for name in PRETTIER_PROGRAMS {
        if root.join(name).is_file() {
            input.ignored.push(((*name).to_owned(), program(name)));
        }
    }
    if input.format.sources.is_empty() {
        return Err(match input.ignored.first() {
            Some((_, why)) => why.clone(),
            None => "no Prettier config here: .prettierrc*, or package.json \"prettier\"".into(),
        });
    }
    input.sources = std::mem::take(&mut input.format.sources);
    Ok(())
}

// ---------------------------------------------------------------- mapping

/// The rule's options a source rule's options carry: the keys the codelaw
/// rule declares, of its type. The rest is named in the report.
fn carry(rule: &RuleFile, options: &Json) -> (serde_json::Map<String, Json>, Vec<String>) {
    let mut kept = serde_json::Map::new();
    let mut dropped = Vec::new();
    // A positional option (ESLint's `"always"`, stylelint's primary `3`)
    // is the rule's one option when it declares exactly one of that type.
    let only = |v: &Json| -> Option<String> {
        let mut it = rule.options.iter();
        match (it.next(), it.next()) {
            (Some((name, spec)), None) if spec.accepts(v) => Some(name.clone()),
            _ => None,
        }
    };
    match options {
        Json::Object(o) => {
            for (k, v) in o {
                if k == "primary"
                    && let Some(name) = only(v)
                {
                    kept.insert(name, v.clone());
                    continue;
                }
                match rule.options.get(k) {
                    Some(spec) if spec.accepts(v) => {
                        kept.insert(k.clone(), v.clone());
                    }
                    _ => dropped.push(format!("{k} = {v}")),
                }
            }
        }
        other => match only(other) {
            Some(name) => {
                kept.insert(name, other.clone());
            }
            None => dropped.push(other.to_string()),
        },
    }
    (kept, dropped)
}

/// A `[rules]` value: a severity, or `{ severity, options }`.
fn setting(severity: &str, options: serde_json::Map<String, Json>) -> Json {
    if options.is_empty() { Json::String(severity.to_owned()) } else { json!({ "severity": severity, "options": options }) }
}

struct Plan {
    extends: Vec<String>,
    rules: BTreeMap<String, Json>,
    scopes: Vec<(String, Vec<String>, BTreeMap<String, Json>)>,
    preview: bool,
}

fn plan(tool: Tool, input: &Input, catalog: &Catalog, report: &mut Report) -> Plan {
    let presets: Json = serde_json::from_str(PRESETS).expect("data/migrate-presets.json parses");
    let table = presets.get(tool.name());
    let notes = presets.get("notes");
    let mut plan = Plan { extends: Vec::new(), rules: BTreeMap::new(), scopes: Vec::new(), preview: false };
    let mut packs: BTreeSet<&'static str> = BTreeSet::new();
    // The packs a mapped preset already loads: no extra preset is needed.
    let mut loaded: BTreeSet<&'static str> = BTreeSet::new();

    for p in &input.presets {
        let to = strings(table.and_then(|t| t.get(p)));
        let note = notes.and_then(|n| n.get(p)).and_then(Json::as_str).map(str::to_owned);
        if to.is_empty() {
            report.unmapped.push(Line { source: format!("preset {p}"), codelaw: vec![], note: "no codelaw preset ports it; its rules map only where the config lists them".into() });
            continue;
        }
        let missing: Vec<&String> = to.iter().filter(|s| !catalog.preset_rules(s, &mut BTreeSet::new())).collect();
        if !missing.is_empty() {
            report.unmapped.push(Line { source: format!("preset {p}"), codelaw: to.clone(), note: format!("{} needs a rule pack that is not installed (`codelaw init` installs it)", missing[0]) });
            continue;
        }
        for s in &to {
            if !plan.extends.contains(s) {
                plan.extends.push(s.clone());
            }
            if let Some(pack) = s.strip_prefix("codelaw:").and_then(|x| x.split_once('/')).and_then(|(ns, _)| namespace_pack(ns)) {
                loaded.insert(pack);
            }
        }
        let line = Line { source: format!("preset {p}"), codelaw: to, note: note.clone().unwrap_or_default() };
        if note.is_some() { report.changed.push(line) } else { report.mapped.push(line) }
    }

    // One source rule → its `[rules]` entries, or why it has none.
    let map_rule = |r: &SourceRule, files: Option<&[String]>, into: &mut BTreeMap<String, Json>, report: &mut Report, plan_preview: &mut bool, packs: &mut BTreeSet<&'static str>| {
        let top = files.is_none();
        let source = match files {
            None => format!("{}/{}", tool.name(), r.id),
            Some(f) => format!("{}/{} in {}", tool.name(), r.id, f.join(", ")),
        };
        let ids = catalog.lookup(tool, &r.id);
        if ids.is_empty() {
            let note = match tool {
                Tool::Biome if upstream_keys(Alias::Biome, &r.id).is_empty() => "Biome names no ESLint, typescript-eslint, React or stylelint source for it, and no codelaw rule ports it".to_owned(),
                _ => "no installed codelaw rule ports it".to_owned(),
            };
            report.unmapped.push(Line { source, codelaw: vec![], note });
            return;
        }
        let mut dropped_all = Vec::new();
        let mut preview_ids = Vec::new();
        for id in &ids {
            let rule = &catalog.rules[id];
            if let Some(p) = catalog.pack_of(id) {
                packs.insert(p);
            }
            match &r.severity {
                None => {
                    into.insert(id.clone(), Json::String("off".into()));
                }
                Some(sev) => {
                    let sev = if sev.is_empty() { rule.severity.name().to_owned() } else { sev.clone() };
                    let (kept, dropped) = r.options.as_ref().map(|o| carry(rule, o)).unwrap_or_default();
                    dropped_all.extend(dropped);
                    if rule.stability == Stability::Preview {
                        preview_ids.push(id.clone());
                        if top {
                            *plan_preview = true;
                        }
                    }
                    into.insert(id.clone(), setting(&sev, kept));
                }
            }
        }
        let mut sev = r.severity.clone().map(|s| if s.is_empty() { "on".to_owned() } else { s }).unwrap_or_else(|| "off".into());
        if !preview_ids.is_empty() {
            sev.push_str(&format!("; {} is preview", preview_ids.join(", ")));
        }
        if dropped_all.is_empty() {
            report.mapped.push(Line { source, codelaw: ids, note: sev });
        } else {
            let declared: Vec<String> = ids.iter().flat_map(|id| catalog.rules[id].options.keys().cloned()).collect();
            let have = if declared.is_empty() { "it takes none".to_owned() } else { format!("it takes {}", declared.join(", ")) };
            report.changed.push(Line { source, codelaw: ids, note: format!("{sev}; option {} not carried ({have}; `codelaw explain` says what each does)", dropped_all.join(", ")) });
        }
    };

    let mut top = BTreeMap::new();
    let mut preview = false;
    // Ruff: every code codelaw ports that the selection turns on.
    if let Some(sel) = &input.select {
        let ported: Vec<(String, Vec<String>)> = catalog.by_upstream.iter().filter_map(|(k, ids)| k.strip_prefix("ruff/").map(|c| (c.to_owned(), ids.clone()))).collect();
        for prefix in &sel.select {
            let hits: Vec<&(String, Vec<String>)> = ported.iter().filter(|(c, _)| (prefix == "ALL" || c.starts_with(prefix.as_str())) && ruff_selected(c, sel)).collect();
            let source = format!("ruff/{prefix}{}", if sel.default && RUFF_DEFAULT.contains(&prefix.as_str()) { " (Ruff's default)" } else { "" });
            if hits.is_empty() {
                report.unmapped.push(Line { source, codelaw: vec![], note: "no installed codelaw rule ports a code it selects".into() });
                continue;
            }
            let mut ids = Vec::new();
            for (c, _) in &hits {
                let r = SourceRule { id: c.clone(), severity: Some("error".into()), options: None };
                for id in catalog.lookup(tool, &r.id) {
                    if let Some(p) = catalog.pack_of(&id) {
                        packs.insert(p);
                    }
                    if catalog.rules[&id].stability == Stability::Preview {
                        preview = true;
                    }
                    top.insert(id.clone(), Json::String("error".into()));
                    if !ids.contains(&id) {
                        ids.push(id);
                    }
                }
            }
            let note = if hits.len() == 1 && hits[0].0 == *prefix {
                "error".to_owned()
            } else {
                format!("error; {} code(s) codelaw ports; Ruff's other codes under it have no codelaw rule", hits.len())
            };
            report.mapped.push(Line { source, codelaw: ids, note });
        }
        for code in &sel.ignore {
            if !sel.select.iter().any(|p| p == "ALL" || code.starts_with(p.as_str()) || p.starts_with(code.as_str())) {
                report.ignored.push(Line { source: format!("ruff/{code}"), codelaw: vec![], note: "ignored, and nothing selects it".into() });
            }
        }
    }
    for r in &input.rules {
        map_rule(r, None, &mut top, report, &mut preview, &mut packs);
    }
    for (i, s) in input.scopes.iter().enumerate() {
        let mut rules = BTreeMap::new();
        for r in &s.rules {
            map_rule(r, Some(&s.files), &mut rules, report, &mut preview, &mut packs);
        }
        if !rules.is_empty() {
            plan.scopes.push((format!("{}-{}", tool.name(), i + 1), s.files.clone(), rules));
        }
    }
    // A pack no mapped preset loads is loaded through its smallest preset,
    // and that preset's rules the source left off are off, so the rule set
    // is the source's.
    let mut on_by_presets: BTreeSet<String> = BTreeSet::new();
    for s in &plan.extends {
        catalog.preset_rules(s, &mut on_by_presets);
    }
    for pack in packs.iter().filter(|p| !loaded.contains(*p)) {
        let Some(p) = catalog.packs.get(pack) else { continue };
        // A `project` preset turns on a whole-repo `graph/` rule: never that.
        let smallest = p.presets.iter().filter(|(_, pr)| pr.extends.is_empty() && pr.scope.is_empty() && !pr.rules.keys().any(|id| id.starts_with("graph/"))).min_by_key(|(name, pr)| (pr.rules.len(), (*name).clone()));
        if let Some((name, _)) = smallest {
            let spec = format!("codelaw:{name}");
            catalog.preset_rules(&spec, &mut on_by_presets);
            plan.extends.push(spec);
        }
    }
    for id in &on_by_presets {
        if !top.contains_key(id) && !loaded.contains(catalog.pack_of(id).unwrap_or("")) {
            top.insert(id.clone(), Json::String("off".into()));
        }
    }
    plan.rules = top;
    plan.preview = preview;
    for (setting, why) in &input.ignored {
        report.ignored.push(Line { source: setting.clone(), codelaw: vec![], note: why.clone() });
    }
    for s in &input.format.skipped {
        report.ignored.push(Line { source: s.clone(), codelaw: vec![], note: "no [format] key holds it".into() });
    }
    for (k, v) in &input.format.keys {
        report.mapped.push(Line { source: format!("{} formatter", tool.name()), codelaw: vec![format!("[format] {k}")], note: v.to_string() });
    }
    plan
}

// ---------------------------------------------------------------- writing

fn toml_value(v: &Json) -> toml_edit::Value {
    match v {
        Json::String(s) => s.as_str().into(),
        Json::Bool(b) => (*b).into(),
        Json::Number(n) => match n.as_i64() {
            Some(i) => i.into(),
            None => n.as_f64().unwrap_or_default().into(),
        },
        Json::Array(a) => {
            let mut arr = toml_edit::Array::new();
            for x in a.iter().filter(|x| !x.is_null()) {
                arr.push(toml_value(x));
            }
            toml_edit::Value::Array(arr)
        }
        Json::Object(o) => {
            let mut t = toml_edit::InlineTable::new();
            for (k, x) in o.iter().filter(|(_, x)| !x.is_null()) {
                t.insert(k, toml_value(x));
            }
            toml_edit::Value::InlineTable(t)
        }
        Json::Null => "".into(),
    }
}

fn quoted(list: &[String]) -> String {
    list.iter().map(|s| toml_edit::Value::from(s.as_str()).to_string().trim().to_owned()).collect::<Vec<_>>().join(", ")
}

/// The config text, from `templates/migrate/` (L10).
fn render(tool: Tool, input: &Input, plan: &Plan) -> Result<String, String> {
    let mut doc = toml_edit::DocumentMut::new();
    if !plan.rules.is_empty() {
        let mut t = toml_edit::Table::new();
        for (id, v) in &plan.rules {
            t.insert(id, toml_edit::value(toml_value(v)));
        }
        doc.insert("rules", toml_edit::Item::Table(t));
    }
    if !plan.scopes.is_empty() {
        let mut scope = toml_edit::Table::new();
        scope.set_implicit(true);
        for (name, files, rules) in &plan.scopes {
            let mut t = toml_edit::Table::new();
            t.insert("files", toml_edit::value(toml_value(&json!(files))));
            let mut r = toml_edit::InlineTable::new();
            for (id, v) in rules {
                r.insert(id, toml_value(v));
            }
            t.insert("rules", toml_edit::value(r));
            scope.insert(name, toml_edit::Item::Table(t));
        }
        doc.insert("scope", toml_edit::Item::Table(scope));
    }
    if !input.format.keys.is_empty() {
        let mut f = toml_edit::Table::new();
        for (key, v) in &input.format.keys {
            let item = match v {
                toml::Value::Integer(n) => toml_edit::value(*n),
                toml::Value::String(s) => toml_edit::value(s.as_str()),
                toml::Value::Boolean(b) => toml_edit::value(*b),
                other => toml_edit::value(other.to_string()),
            };
            match key.split_once('.') {
                Some((sub, name)) => {
                    let e = f.entry(sub).or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()));
                    if let Some(t) = e.as_table_mut() {
                        t.insert(name, item);
                    }
                }
                None => {
                    f.insert(key, item);
                }
            }
        }
        f.set_implicit(true);
        doc.insert("format", toml_edit::Item::Table(f));
    }
    let mut tables = doc.to_string();
    if !tables.is_empty() {
        tables.insert(0, '\n');
    }
    let preview = if plan.preview { template::text("migrate/preview.toml").to_owned() } else { String::new() };
    let lint = match tool.alias() {
        Some(a) => template::render("migrate/lint.toml", &[("tool", tool.name()), ("aliases", &quoted(&[a.name().to_owned()]))])?,
        None => String::new(),
    };
    template::render(
        "migrate/codelaw.toml",
        &[("tool", tool.name()), ("sources", &input.sources.join(", ")), ("extends", &quoted(&plan.extends)), ("preview", &preview), ("lint", &lint), ("tables", &tables)],
    )
}

/// Add what a migration wrote to an existing `codelaw.toml`, keeping the
/// file as written: `extends` entries and alias tools it lacks, and each
/// rule, scope and `[format]` key it does not set; one it sets is kept and
/// named in `kept`. Running a second tool's migration (Prettier after
/// ESLint) so adds to the first's.
fn merge(existing: &str, new: &str, kept: &mut Vec<String>) -> Result<String, String> {
    let mut doc: toml_edit::DocumentMut = existing.parse().map_err(|e| format!("{e}"))?;
    let add: toml_edit::DocumentMut = new.parse().map_err(|e| format!("the migrated config: {e}"))?;
    let union = |doc: &mut toml_edit::Table, key: &str, from: Option<&toml_edit::Item>| -> Result<(), String> {
        let Some(from) = from.and_then(toml_edit::Item::as_array) else { return Ok(()) };
        let entry = doc.entry(key).or_insert_with(|| toml_edit::value(toml_edit::Array::new()));
        let arr = entry.as_array_mut().ok_or_else(|| format!("`{key}` is not an array"))?;
        for v in from.iter() {
            if !arr.iter().any(|x| x.as_str() == v.as_str()) {
                arr.push(v.clone());
            }
        }
        Ok(())
    };
    union(doc.as_table_mut(), "extends", add.get("extends"))?;
    if add.get("preview").and_then(toml_edit::Item::as_bool) == Some(true) && doc.get("preview").is_none() {
        doc.insert("preview", toml_edit::value(true));
    }
    for (section, depth) in [("lint", 0), ("rules", 1), ("scope", 1), ("format", 2)] {
        let Some(from) = add.get(section).and_then(toml_edit::Item::as_table) else { continue };
        let entry = doc.entry(section).or_insert_with(|| {
            let mut t = toml_edit::Table::new();
            t.set_implicit(section == "scope");
            toml_edit::Item::Table(t)
        });
        let to = entry.as_table_mut().ok_or_else(|| format!("`{section}` is not a table"))?;
        for (k, v) in from.iter() {
            if section == "lint" && k == "ignore_aliases" {
                union(to, k, Some(v))?;
            } else if depth == 2 && v.is_table() {
                let sub = to.entry(k).or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()));
                let sub = sub.as_table_mut().ok_or_else(|| format!("`{section}.{k}` is not a table"))?;
                for (k2, v2) in v.as_table().into_iter().flat_map(|t| t.iter()) {
                    if sub.contains_key(k2) {
                        kept.push(format!("[{section}.{k}] {k2}"));
                    } else {
                        sub.insert(k2, v2.clone());
                    }
                }
            } else if to.contains_key(k) {
                kept.push(format!("[{section}] {k}"));
            } else {
                to.insert(k, v.clone());
            }
        }
    }
    Ok(doc.to_string())
}

/// Read, map and render; with `write`, write `codelaw.toml`.
pub fn migrate_in(root: &Path, tool: Tool, write: bool) -> Result<Report, String> {
    let mut input = Input::default();
    match tool {
        Tool::Eslint => read_eslint(root, &mut input)?,
        Tool::Prettier => read_prettier(root, &mut input)?,
        Tool::Biome => read_biome(root, &mut input)?,
        Tool::Ruff => read_ruff(root, &mut input)?,
        Tool::Stylelint => read_stylelint(root, &mut input)?,
    }
    let catalog = Catalog::load(root);
    let mut report = Report { tool: tool.name(), sources: input.sources.clone(), not_installed: catalog.not_installed.clone(), ..Default::default() };
    let plan = plan(tool, &input, &catalog, &mut report);
    report.config = render(tool, &input, &plan)?;
    if write {
        let existing = ["codelaw.toml"].iter().map(|n| root.join(n)).find(|p| p.exists());
        let home = existing.map(|p| p.display().to_string()).or_else(|| {
            let py = read(root, "pyproject.toml").is_some_and(|t| t.parse::<toml::Table>().ok().is_some_and(|t| t.get("tool").and_then(|x| x.get("codelaw")).is_some()));
            let pkg = package_json_key(root, "codelaw").is_some();
            if py { Some("pyproject.toml [tool.codelaw]".into()) } else if pkg { Some("package.json \"codelaw\"".into()) } else { None }
        });
        let path = root.join("codelaw.toml");
        let text = match home {
            Some(h) if h != path.display().to_string() => {
                return Err(format!("{h} configures codelaw, and migrate writes TOML: move that config to codelaw.toml first, or run without --write and copy what you need"));
            }
            Some(_) => {
                let existing = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                report.merged = true;
                merge(&existing, &report.config, &mut report.kept).map_err(|e| format!("{}: {e}", path.display()))?
            }
            None => report.config.clone(),
        };
        std::fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
        report.written = Some(PathBuf::from("codelaw.toml"));
    }
    Ok(report)
}

fn lines_json(lines: &[Line]) -> Json {
    Json::Array(lines.iter().map(|l| json!({ "source": l.source, "codelaw": l.codelaw, "note": l.note })).collect())
}

pub fn to_json(r: &Report) -> Json {
    json!({
        "schema_version": 1,
        "codelaw_version": codelaw_core::VERSION,
        "tool": r.tool,
        "sources": r.sources,
        "mapped": lines_json(&r.mapped),
        "changed": lines_json(&r.changed),
        "unmapped": lines_json(&r.unmapped),
        "ignored": lines_json(&r.ignored),
        "not_installed": r.not_installed,
        "config": r.config,
        "written": r.written.as_ref().map(|p| p.display().to_string()),
        "kept": r.kept,
    })
}

pub fn to_text(r: &Report) -> String {
    let mut out = format!("migrate {}: read {}\n", r.tool, r.sources.join(", "));
    for (label, lines) in [("mapped  ", &r.mapped), ("changed ", &r.changed), ("unmapped", &r.unmapped), ("ignored ", &r.ignored)] {
        for l in lines {
            let to = if l.codelaw.is_empty() { String::new() } else { format!(" → {}", l.codelaw.join(", ")) };
            let note = if l.note.is_empty() { String::new() } else { format!(" ({})", l.note) };
            out.push_str(&format!("{label}  {}{to}{note}\n", l.source));
        }
    }
    out.push_str(&format!("{} mapped · {} changed · {} unmapped · {} ignored\n", r.mapped.len(), r.changed.len(), r.unmapped.len(), r.ignored.len()));
    if !r.not_installed.is_empty() {
        out.push_str(&format!("not searched, not installed: {}\n", r.not_installed.join(", ")));
    }
    for k in &r.kept {
        out.push_str(&format!("kept      {k}: codelaw.toml already sets it\n"));
    }
    match &r.written {
        Some(p) if !r.merged => out.push_str(&format!("wrote {}\n", p.display())),
        Some(p) => out.push_str(&format!("added to {}\n", p.display())),
        None => out.push_str(&format!("--- codelaw.toml (not written; --write writes it) ---\n{}", r.config)),
    }
    out
}

pub fn migrate(args: MigrateArgs) -> ExitCode {
    if !matches!(args.format.as_str(), "text" | "json") {
        return crate::fail(format!("`--format {}`: migrate prints text or json", args.format));
    }
    let root = args.dir.clone().unwrap_or_else(|| PathBuf::from("."));
    match migrate_in(&root, args.tool, args.write) {
        Ok(r) => {
            if args.format == "json" {
                println!("{}", serde_json::to_string_pretty(&to_json(&r)).unwrap_or_default());
            } else {
                print!("{}", to_text(&r));
            }
            ExitCode::SUCCESS
        }
        Err(e) => crate::fail(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruff_resolves_the_longest_prefix() {
        let sel = RuffSelect { select: vec!["E".into(), "F".into()], ignore: vec!["E501".into(), "F4".into()], default: false };
        assert!(ruff_selected("E711", &sel));
        assert!(!ruff_selected("E501", &sel));
        assert!(!ruff_selected("F401", &sel));
        assert!(ruff_selected("F811", &sel));
        assert!(!ruff_selected("B006", &sel));
        let all = RuffSelect { select: vec!["ALL".into()], ignore: vec![], default: false };
        assert!(ruff_selected("B006", &all));
    }

    #[test]
    fn eslint_severities() {
        assert_eq!(eslint_rule("a", &json!(2)).unwrap().severity.as_deref(), Some("error"));
        assert_eq!(eslint_rule("a", &json!("off")).unwrap().severity, None);
        let r = eslint_rule("a", &json!(["warn", { "allow": ["warn"] }])).unwrap();
        assert_eq!((r.severity.as_deref(), r.options), (Some("warn"), Some(json!({ "allow": ["warn"] }))));
        assert!(eslint_rule("a", &json!("loud")).is_none());
    }

    #[test]
    fn a_static_flat_config_is_read_and_a_computed_one_refused() {
        let mut i = Input::default();
        let text = "import js from \"@eslint/js\";\nimport tseslint from 'typescript-eslint';\nexport default [\n  js.configs.recommended,\n  ...tseslint.configs.recommended,\n  { rules: { \"no-console\": \"warn\", eqeqeq: [\"error\", \"always\"] } },\n  { files: [\"**/*.test.js\"], rules: { \"no-console\": \"off\" } },\n  { languageOptions: { globals: {} } },\n];\n";
        flat::read("eslint.config.js", text, &mut i).unwrap();
        assert_eq!(i.presets, ["@eslint/js:configs.recommended", "typescript-eslint:configs.recommended"]);
        assert_eq!(i.rules.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["no-console", "eqeqeq"]);
        assert_eq!(i.scopes[0].files, ["**/*.test.js"]);
        assert!(i.ignored.iter().any(|(s, _)| s.contains("languageOptions")));

        let mut i = Input::default();
        let wrapped = "const { defineConfig } = require('eslint/config');\nconst config = defineConfig([{ rules: { 'no-var': 2 } }]);\nmodule.exports = config;\n";
        flat::read("eslint.config.cjs", wrapped, &mut i).unwrap();
        assert_eq!(i.rules[0].id, "no-var");

        let mut i = Input::default();
        let computed = "export default makeConfig();\n";
        assert!(flat::read("eslint.config.js", computed, &mut i).unwrap_err().contains("convert it to JSON first, or list the rules by hand"));
        let mut i = Input::default();
        flat::read("eslint.config.js", "const r = {};\nexport default [{ rules: r }, { rules: { ...base, 'no-var': 'error' } }];\n", &mut i).unwrap();
        assert_eq!(i.rules[0].id, "no-var");
        assert_eq!(i.ignored.len(), 2, "{:?}", i.ignored);
    }
}
