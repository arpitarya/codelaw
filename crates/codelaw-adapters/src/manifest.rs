//! The manifests (SR-ARCHITECTURE decision 19): one TOML file per adapter
//! in `adapters/`, compiled into the binary so nothing is read at run time.

use codelaw_core::Language;
use semver::VersionReq;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Every manifest, by file. Adding an adapter is adding a line here and a
/// file beside the others; its licence must pass L2 (`tests/manifests.rs`).
const SOURCES: &[(&str, &str)] = &[
    ("oxlint.toml", include_str!("../adapters/oxlint.toml")),
    ("ruff.toml", include_str!("../adapters/ruff.toml")),
    ("stylelint.toml", include_str!("../adapters/stylelint.toml")),
    ("html-validate.toml", include_str!("../adapters/html-validate.toml")),
];

/// Which parser reads an adapter's output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Oxlint,
    Ruff,
    Stylelint,
    HtmlValidate,
}

/// One adapter, as its manifest describes it.
#[derive(Clone, Debug)]
pub struct Manifest {
    /// The name in `[adapters] enabled` and in `adapter/<name>/<rule>`.
    pub name: String,
    /// The program looked up in `node_modules/.bin`, `.venv/bin`, `PATH`.
    pub binary: String,
    /// The tool's SPDX licence; L2 holds it to the permissive list.
    pub licence: String,
    /// On when installed and `[adapters] enabled` is not set (SR-LINTER
    /// decision 2).
    pub default: bool,
    pub languages: Vec<Language>,
    pub version_args: Vec<String>,
    /// The command line for JSON output; the files follow it.
    pub args: Vec<String>,
    /// The version range the fixtures were captured in.
    pub tested: VersionReq,
    pub output: Output,
    /// What turns a tool rule into the id a pack rule's `ported_from:`
    /// names: `ruff/` + `F401`.
    pub ported_prefix: String,
    /// A tool plugin whose rules `ported_from:` names under another
    /// plugin: oxlint's `typescript/<rule>` is `typescript-eslint/<rule>`.
    pub ported_plugins: BTreeMap<String, String>,
    /// The rule docs, `{rule}` replaced, when the output carries no link.
    pub docs: Option<String>,
    /// The tool's own config files: their content enters the cache key.
    pub config_files: Vec<String>,
    /// A key in those config files that makes the tool read the whole
    /// program (oxlint's `typeAware`, W-107): when their text names it, a
    /// file's findings depend on files outside its cache key, so the
    /// adapter's results are not cached.
    pub uncached_when: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    name: String,
    binary: String,
    licence: String,
    default: bool,
    languages: Vec<String>,
    version_args: Vec<String>,
    args: Vec<String>,
    tested: String,
    output: String,
    ported_prefix: String,
    #[serde(default)]
    ported_plugins: BTreeMap<String, String>,
    docs: Option<String>,
    #[serde(default)]
    config_files: Vec<String>,
    uncached_when: Option<String>,
}

/// Parse one manifest.
pub fn parse_manifest(file: &str, text: &str) -> Result<Manifest, String> {
    let raw: RawManifest = toml::from_str(text).map_err(|e| format!("{file}: {e}"))?;
    let languages = raw
        .languages
        .iter()
        .map(|l| Language::from_name(l).ok_or_else(|| format!("{file}: unknown language `{l}`")))
        .collect::<Result<Vec<_>, _>>()?;
    let tested = VersionReq::parse(&raw.tested).map_err(|e| format!("{file}: tested: {e}"))?;
    let output = match raw.output.as_str() {
        "oxlint" => Output::Oxlint,
        "ruff" => Output::Ruff,
        "stylelint" => Output::Stylelint,
        "html-validate" => Output::HtmlValidate,
        other => return Err(format!("{file}: unknown output `{other}`")),
    };
    Ok(Manifest {
        name: raw.name,
        binary: raw.binary,
        licence: raw.licence,
        default: raw.default,
        languages,
        version_args: raw.version_args,
        args: raw.args,
        tested,
        output,
        ported_prefix: raw.ported_prefix,
        ported_plugins: raw.ported_plugins,
        docs: raw.docs,
        config_files: raw.config_files,
        uncached_when: raw.uncached_when,
    })
}

/// Every adapter codelaw ships a manifest for, in name order.
pub fn manifests() -> &'static [Manifest] {
    static ALL: OnceLock<Vec<Manifest>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut all: Vec<Manifest> = SOURCES.iter().map(|(f, t)| parse_manifest(f, t).unwrap_or_else(|e| panic!("{e}"))).collect();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    })
}

/// The manifest named `name`.
pub fn manifest(name: &str) -> Option<&'static Manifest> {
    manifests().iter().find(|m| m.name == name)
}

impl Manifest {
    /// The id a pack rule's `ported_from:` would give this tool rule.
    pub fn ported_id(&self, rule: &str) -> String {
        match rule.split_once('/').and_then(|(plugin, name)| Some((self.ported_plugins.get(plugin)?, name))) {
            Some((plugin, name)) => format!("{}{plugin}/{name}", self.ported_prefix),
            None => format!("{}{rule}", self.ported_prefix),
        }
    }

    /// The codelaw id of a tool rule (SR-ARCHITECTURE decision 19).
    pub fn rule_id(&self, rule: &str) -> String {
        format!("adapter/{}/{rule}", self.name)
    }

    /// Whether the config text `config` (the content of `config_files`
    /// that governs a file) turns on whole-program checking.
    pub fn uncached(&self, config: &[u8]) -> bool {
        self.uncached_when.as_deref().is_some_and(|k| config.windows(k.len()).any(|w| w == k.as_bytes()))
    }

    pub fn handles(&self, language: Language) -> bool {
        self.languages.contains(&language)
    }
}
