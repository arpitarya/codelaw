//! One rule file: the template of SR-RULE-FORMAT, parsed and checked.

use crate::LoadError;
use crate::markdown::{Piece, scan};
use codelaw_cache::{Key, hash_bytes};
use codelaw_core::{FixClass, Language, Severity};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Namespaces reserved for the first-party packs and the engine
/// (SR-RULE-FORMAT decision 1).
pub const RESERVED_NAMESPACES: &[&str] = &[
    "js", "ts", "react", "node", "stencil", "py", "json", "yaml", "toml", "md", "data", "css", "html", "graph", "format",
    "codelaw", "tanstack-query", "redux-toolkit", "sec-js", "sec-ts", "sec-html", "sec-py", "scss", "css-modules", "spell", "deps",
];

/// The namespaces of the four security packs: a rule in one must carry
/// `security:` (SR-RULE-FORMAT decision 2a).
pub const SECURITY_NAMESPACES: &[&str] = &["sec-js", "sec-ts", "sec-html", "sec-py"];

/// The OWASP Top 10:2025 categories, for `rules --security-coverage`
/// (SR-CLI-MCP decision 1).
pub const OWASP_TOP_10_2025: &[(&str, &str)] = &[
    ("A01:2025", "Broken Access Control"),
    ("A02:2025", "Security Misconfiguration"),
    ("A03:2025", "Software Supply Chain Failures"),
    ("A04:2025", "Cryptographic Failures"),
    ("A05:2025", "Injection"),
    ("A06:2025", "Insecure Design"),
    ("A07:2025", "Authentication Failures"),
    ("A08:2025", "Software or Data Integrity Failures"),
    ("A09:2025", "Security Logging and Alerting Failures"),
    ("A10:2025", "Mishandling of Exceptional Conditions"),
];

/// How bad a security finding is (SR-RULE-FORMAT decision 2a). SARIF's
/// `security-severity` is its score.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
}

impl SecuritySeverity {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "high" => Some(SecuritySeverity::High),
            "medium" => Some(SecuritySeverity::Medium),
            "low" => Some(SecuritySeverity::Low),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SecuritySeverity::High => "high",
            SecuritySeverity::Medium => "medium",
            SecuritySeverity::Low => "low",
        }
    }

    /// The score GitHub code scanning ranks by (`security-severity`): it
    /// reads 7.0–8.9 as high, 4.0–6.9 as medium and 0.1–3.9 as low.
    pub fn score(self) -> &'static str {
        match self {
            SecuritySeverity::High => "8.0",
            SecuritySeverity::Medium => "5.0",
            SecuritySeverity::Low => "2.0",
        }
    }
}

/// A rule's `security:` metadata: the CWE ids and OWASP Top 10 categories it
/// maps to, and how severe a finding is (SR-RULE-FORMAT decision 2a).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Security {
    pub cwe: Vec<u32>,
    /// `A01:2025` … `A10:2025`.
    pub owasp: Vec<String>,
    pub severity: SecuritySeverity,
}

impl Security {
    /// `(CWE-78)` or `(CWE-614, CWE-1004)`; empty without a CWE.
    pub fn cwe_label(&self) -> String {
        if self.cwe.is_empty() {
            return String::new();
        }
        format!("({})", self.cwe.iter().map(|c| format!("CWE-{c}")).collect::<Vec<_>>().join(", "))
    }

    /// The tags SARIF carries: `security`, `external/cwe/cwe-<n>` and
    /// `external/owasp/<category>`.
    pub fn sarif_tags(&self) -> Vec<String> {
        let mut out = vec!["security".to_owned()];
        out.extend(self.cwe.iter().map(|c| format!("external/cwe/cwe-{c}")));
        out.extend(self.owasp.iter().map(|o| format!("external/owasp/{o}")));
        out
    }

    /// `rules --tag`: `security`, `cwe:<n>`, or `owasp:<category>` (`A05`
    /// matches `A05:2025`).
    pub fn matches_tag(&self, tag: &str) -> bool {
        let tag = tag.trim();
        if tag.eq_ignore_ascii_case("security") {
            return true;
        }
        if let Some(n) = tag.strip_prefix("cwe:").or_else(|| tag.strip_prefix("CWE-")) {
            return n.trim_start_matches("CWE-").parse::<u32>().is_ok_and(|n| self.cwe.contains(&n));
        }
        if let Some(c) = tag.strip_prefix("owasp:") {
            return self.owasp.iter().any(|o| o.eq_ignore_ascii_case(c) || o.split(':').next().is_some_and(|x| x.eq_ignore_ascii_case(c)));
        }
        false
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityFm {
    #[serde(default)]
    cwe: Vec<u32>,
    #[serde(default)]
    owasp: Vec<String>,
    severity: Option<String>,
}

/// The skills a `fix_skill:` may name: those codelaw ships
/// (SR-AGENT-SURFACES; L4 decision 10).
pub const KNOWN_SKILLS: &[&str] = &[
    "codelaw-author-rule",
    "codelaw-ci-triage",
    "codelaw-debug-rule",
    "codelaw-explain",
    "codelaw-fix",
    "codelaw-migrate",
    "codelaw-pack",
    "codelaw-port-rule",
    "codelaw-review",
    "codelaw-rules-from-reviews",
    "codelaw-setup",
    "codelaw-suppress",
    "codelaw-upgrade",
    "codelaw-write",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Scope {
    #[default]
    File,
    Project,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Stability {
    Stable,
    #[default]
    Preview,
}

/// A typed rule option (SR-LINTER decision 5).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionSpec {
    #[serde(rename = "type")]
    pub kind: String,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
}

const OPTION_TYPES: &[&str] = &["string", "string[]", "number", "number[]", "integer", "boolean", "object", "call[]"];

impl OptionSpec {
    /// Whether `value` has this option's type.
    pub fn accepts(&self, value: &serde_json::Value) -> bool {
        use serde_json::Value as V;
        match (self.kind.as_str(), value) {
            ("string", V::String(_)) | ("boolean", V::Bool(_)) | ("number", V::Number(_)) | ("object", V::Object(_)) => true,
            ("integer", V::Number(n)) => n.is_i64() || n.is_u64(),
            ("string[]", V::Array(a)) => a.iter().all(|v| v.is_string()),
            ("number[]", V::Array(a)) => a.iter().all(|v| v.is_number()),
            ("call[]", v) => crate::calls::parse_call_items(v).is_ok(),
            (crate::calls::MODE_KIND, V::String(s)) => s == "extend" || s == "replace",
            _ => false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

/// The frontmatter as written. Unknown fields are a load error
/// (SR-RULE-FORMAT decision 2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    id: Option<String>,
    title: Option<String>,
    languages: Option<Vec<String>>,
    scope: Option<String>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    severity: Option<String>,
    fix: Option<String>,
    fix_reason: Option<String>,
    fix_skill: Option<String>,
    stability: Option<String>,
    host: Option<String>,
    invariant: Option<OneOrMany>,
    #[serde(default)]
    options: BTreeMap<String, OptionSpec>,
    #[serde(default)]
    ported_from: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    security: Option<SecurityFm>,
    since: Option<String>,
}

/// The four forms of a `codelaw` block (SR-RULE-FORMAT decision 3).
#[derive(Clone, Debug, PartialEq)]
pub enum BlockKind {
    /// An ast-grep rule, plus its optional `constraints`, `utils` and
    /// codelaw's semantic `where` predicates.
    Match {
        rule: serde_json::Value,
        constraints: Option<serde_json::Value>,
        utils: Option<serde_json::Value>,
        predicates: Option<serde_json::Value>,
        /// `type:`, the type questions (decision 3f; SR-TYPES); boxed, as
        /// few rules have one.
        types: Option<Box<serde_json::Value>>,
    },
    Graph(serde_json::Value),
    Target { target: serde_json::Value, assert: serde_json::Value },
    Native(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub kind: BlockKind,
    pub message: Option<String>,
    pub fix: Option<serde_json::Value>,
}

impl Block {
    /// An `external` fixer command, if the fix is one.
    pub fn external_fixer(&self) -> Option<&str> {
        self.fix.as_ref()?.get("external")?.as_str()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleKind {
    Dont,
    Do,
    Fixed,
}

/// One fenced example. The info string is `<tag> [path=<file>]
/// [options=<json>]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snippet {
    pub code: String,
    pub path: Option<String>,
    /// Options the example runs with, over the rule's defaults
    /// (`options={"max":3}`, compact JSON): a threshold rule's Don't needs
    /// a limit a snippet can exceed.
    pub options: BTreeMap<String, serde_json::Value>,
    /// 1-based line of the fence in the rule file.
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageExamples {
    pub language: Language,
    pub donts: Vec<Snippet>,
    pub dos: Vec<Snippet>,
    pub fixed: Vec<Snippet>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Example {
    pub language: Language,
    pub kind: ExampleKind,
    pub snippet: Snippet,
}

/// One edge case from `<name>.cases.md` (SR-RULE-FORMAT decision 6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    pub title: String,
    pub language: Language,
    pub input: Snippet,
    /// `line:col rule-id` lines; empty is `(none)`.
    pub expect: Vec<String>,
    pub fixed: Option<String>,
}

/// A parsed, checked rule file.
#[derive(Clone, Debug)]
pub struct RuleFile {
    pub path: PathBuf,
    pub id: String,
    pub title: String,
    pub languages: Vec<Language>,
    pub scope: Scope,
    pub files: Vec<String>,
    pub exclude: Vec<String>,
    pub severity: Severity,
    pub fix: FixClass,
    pub fix_reason: Option<String>,
    pub fix_skill: Option<String>,
    pub stability: Stability,
    pub host: Option<Language>,
    pub invariants: Vec<String>,
    pub options: BTreeMap<String, OptionSpec>,
    pub ported_from: Vec<String>,
    pub tags: Vec<String>,
    /// CWE, OWASP and severity (SR-RULE-FORMAT decision 2a).
    pub security: Option<Security>,
    pub since: Option<String>,
    pub summary: String,
    pub why: String,
    pub block: Block,
    pub steps: Vec<String>,
    pub examples: Vec<LanguageExamples>,
    pub cases: Vec<Case>,
    /// Hash of the rule file and its sidecar: part of the rule-set key.
    pub hash: Key,
}

impl RuleFile {
    pub fn namespace(&self) -> &str {
        self.id.split_once('/').map(|(ns, _)| ns).unwrap_or(&self.id)
    }

    pub fn native_id(&self) -> Option<&str> {
        match &self.block.kind {
            BlockKind::Native(id) => Some(id),
            _ => None,
        }
    }

    pub fn examples_for(&self, language: Language) -> Option<&LanguageExamples> {
        self.examples.iter().find(|e| e.language == language)
    }

    /// The options a rule runs with when config sets none.
    pub fn default_options(&self) -> BTreeMap<String, serde_json::Value> {
        self.options.iter().filter_map(|(k, v)| v.default.clone().map(|d| (k.clone(), d))).collect()
    }
}

/// Split `---` frontmatter from the body. Returns (frontmatter, body, the
/// body's 1-based first line).
fn split_frontmatter(text: &str) -> Option<(&str, &str, usize)> {
    let rest = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n"))?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            let fm = &rest[..offset];
            let body = &rest[offset + line.len()..];
            let body_line = 1 + text[..text.len() - body.len()].matches('\n').count() + 1;
            return Some((fm, body, body_line - 1));
        }
        offset += line.len();
    }
    None
}

fn line_of(body: &str, offset: usize, body_line: usize) -> usize {
    body_line + body[..offset.min(body.len())].matches('\n').count()
}

/// A fence's tag, its `path=` and its `options=`.
type FenceInfo = (String, Option<String>, BTreeMap<String, serde_json::Value>);

/// Parse the info string of an example fence: `<tag> [path=<file>]
/// [options=<json>]`. Options that are not a JSON object are an error.
fn fence_info(info: &str) -> Result<FenceInfo, String> {
    let mut parts = info.split_whitespace();
    let tag = parts.next().unwrap_or_default().to_owned();
    let rest: Vec<&str> = parts.collect();
    let path = rest.iter().find_map(|p| p.strip_prefix("path=").map(|s| s.trim_matches('"').to_owned()));
    let options = match rest.iter().find_map(|p| p.strip_prefix("options=")) {
        None => BTreeMap::new(),
        Some(json) => serde_json::from_str(json).map_err(|e| format!("`options=` must be a compact JSON object (no spaces): {e}"))?,
    };
    Ok((tag, path, options))
}

fn normalise_kind(heading: &str) -> Option<ExampleKind> {
    match heading.trim().replace('’', "'").to_ascii_lowercase().as_str() {
        "don't" | "dont" => Some(ExampleKind::Dont),
        "do" => Some(ExampleKind::Do),
        "fixed" => Some(ExampleKind::Fixed),
        _ => None,
    }
}

/// Rule files are YAML 1.2, as ast-grep reads them: `n`, `on` and `yes`
/// are strings, never booleans. A key written twice in one mapping is an
/// error: YAML would otherwise keep the last one silently.
fn yaml_options() -> serde_saphyr::Options {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.duplicate_keys = serde_saphyr::DuplicateKeyPolicy::Error;
    options.merge_keys = serde_saphyr::MergeKeyPolicy::AsOrdinary;
    options.reject_unsupported_tags = true;
    options
}

pub(crate) fn yaml<'de, T: Deserialize<'de>>(text: &'de str) -> Result<T, serde_saphyr::Error> {
    serde_saphyr::from_str_with_options(text, yaml_options())
}

fn yaml_value(text: &str) -> Result<serde_json::Value, String> {
    yaml::<serde_json::Value>(text).map_err(|e| e.to_string())
}

const BLOCK_KEYS: &[&str] = &["match", "constraints", "utils", "where", "type", "graph", "target", "assert", "native", "message", "fix"];

fn parse_block(code: &str) -> Result<Block, String> {
    let value = yaml_value(code)?;
    let serde_json::Value::Object(map) = value else {
        return Err("the codelaw block must be a YAML mapping".into());
    };
    for key in map.keys() {
        if !BLOCK_KEYS.contains(&key.as_str()) {
            let near = codelaw_config::did_you_mean(key, BLOCK_KEYS.iter().copied())
                .map(|n| format!("; did you mean `{n}`?"))
                .unwrap_or_default();
            return Err(format!("unknown key `{key}` in the codelaw block{near}"));
        }
    }
    let forms: Vec<&str> = ["match", "graph", "target", "native"].into_iter().filter(|k| map.contains_key(*k)).collect();
    if forms.len() != 1 {
        return Err(format!(
            "the codelaw block takes exactly one of `match:`, `graph:`, `target:` + `assert:` or `native:` (found {})",
            if forms.is_empty() { "none".to_owned() } else { forms.join(", ") }
        ));
    }
    let get = |k: &str| map.get(k).cloned();
    let kind = match forms[0] {
        "match" => BlockKind::Match {
            rule: get("match").unwrap(),
            constraints: get("constraints"),
            utils: get("utils"),
            predicates: get("where"),
            types: get("type").map(Box::new),
        },
        "graph" => BlockKind::Graph(get("graph").unwrap()),
        "target" => BlockKind::Target {
            target: get("target").unwrap(),
            assert: get("assert").ok_or("`target:` needs `assert:`")?,
        },
        _ => BlockKind::Native(
            get("native").and_then(|v| v.as_str().map(str::to_owned)).ok_or("`native:` takes the id of an engine rule")?,
        ),
    };
    if !matches!(kind, BlockKind::Match { .. }) {
        for k in ["constraints", "utils", "where", "type"] {
            if map.contains_key(k) {
                return Err(format!("`{k}:` only goes with `match:`"));
            }
        }
    }
    if !matches!(kind, BlockKind::Target { .. }) && map.contains_key("assert") {
        return Err("`assert:` only goes with `target:`".into());
    }
    let message = match get("message") {
        None => None,
        Some(serde_json::Value::String(s)) => Some(s),
        Some(_) => return Err("`message:` must be a string".into()),
    };
    if message.is_none() && !matches!(kind, BlockKind::Native(_)) {
        return Err("the codelaw block needs a `message:`".into());
    }
    Ok(Block { kind, message, fix: get("fix") })
}

/// Parse and check a rule file. Every problem is returned, so `codelaw
/// rules lint` can list them all at once.
pub fn parse_rule(path: &Path, text: &str, expected_id: Option<&str>) -> Result<RuleFile, Vec<LoadError>> {
    let mut errors = Vec::new();
    let err = |errors: &mut Vec<LoadError>, line: usize, msg: String| errors.push(LoadError::at(path, line, msg));

    let Some((fm_text, body, body_line)) = split_frontmatter(text) else {
        return Err(vec![LoadError::at(path, 1, "a rule file starts with `---` frontmatter")]);
    };
    let fm: Frontmatter = match yaml(fm_text) {
        Ok(fm) => fm,
        Err(e) => {
            let msg = e.to_string();
            let msg = if msg.contains("unknown field") {
                let valid = [
                    "id", "title", "languages", "scope", "files", "exclude", "severity", "fix", "fix_reason", "fix_skill",
                    "stability", "host", "invariant", "options", "ported_from", "tags", "security", "since",
                ];
                let unknown = msg.split('`').nth(1).unwrap_or_default();
                let near = codelaw_config::did_you_mean(unknown, valid).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
                format!("unknown frontmatter field `{unknown}`{near}")
            } else {
                format!("invalid frontmatter: {msg}")
            };
            return Err(vec![LoadError::at(path, 2, msg)]);
        }
    };

    // Identity.
    let id = fm.id.clone().unwrap_or_default();
    if id.is_empty() {
        err(&mut errors, 2, "`id:` is required".into());
    } else if id.split('/').count() != 2 || id.split('/').any(str::is_empty) {
        err(&mut errors, 2, format!("`id: {id}` must be `<namespace>/<name>`"));
    } else if let Some(expected) = expected_id
        && expected != id
    {
        err(&mut errors, 2, format!("`id: {id}` must match the file's path, `{expected}`"));
    }
    let title = fm.title.clone().unwrap_or_default();
    if title.trim().is_empty() {
        err(&mut errors, 2, "`title:` is required".into());
    }

    // Languages (L3: a name outside the nine is refused at load).
    let mut languages = Vec::new();
    match &fm.languages {
        None => err(&mut errors, 2, "`languages:` is required".into()),
        Some(names) if names.is_empty() => err(&mut errors, 2, "`languages:` names at least one language".into()),
        Some(names) => {
            for n in names {
                match Language::from_name(n) {
                    Some(l) if !languages.contains(&l) => languages.push(l),
                    Some(_) => {}
                    None => {
                        let near = codelaw_config::did_you_mean(n, Language::ALL.iter().map(|l| l.name()))
                            .map(|x| format!("; did you mean `{x}`?"))
                            .unwrap_or_default();
                        err(&mut errors, 2, format!("unknown language `{n}`{near}"));
                    }
                }
            }
        }
    }

    let scope = match fm.scope.as_deref() {
        None | Some("file") => Scope::File,
        Some("project") => Scope::Project,
        Some(s) => {
            err(&mut errors, 2, format!("`scope: {s}` must be `file` or `project`"));
            Scope::File
        }
    };
    let severity = match fm.severity.as_deref() {
        None => Severity::Error,
        Some(s) => Severity::parse(s).unwrap_or_else(|| {
            err(&mut errors, 2, format!("`severity: {s}` must be `error`, `warn` or `info`"));
            Severity::Error
        }),
    };

    // L4 decisions 1 and 2. A missing or unknown class is one error, not
    // a cascade of checks against a guessed class.
    let fix = match fm.fix.as_deref() {
        None => {
            err(&mut errors, 2, "`fix:` is required: `auto`, `suggest` or `manual` (L4 decision 1)".into());
            None
        }
        Some(s) => {
            let parsed = FixClass::parse(s);
            if parsed.is_none() {
                err(&mut errors, 2, format!("`fix: {s}` must be `auto`, `suggest` or `manual` (L4 decision 1)"));
            }
            parsed
        }
    };
    if let Some(class) = fix
        && class != FixClass::Auto
        && fm.fix_reason.as_deref().is_none_or(|r| r.trim().is_empty())
    {
        err(&mut errors, 2, format!("`fix: {class}` needs `fix_reason:`, one line on why it can't be automatic (L4 decision 2)"));
    }
    if let Some(skill) = &fm.fix_skill
        && !KNOWN_SKILLS.contains(&skill.as_str())
    {
        err(&mut errors, 2, format!("`fix_skill: {skill}` ships in no codelaw package (L4 decision 10)"));
    }

    let stability = match fm.stability.as_deref() {
        None | Some("preview") => Stability::Preview,
        Some("stable") => Stability::Stable,
        Some(s) => {
            err(&mut errors, 2, format!("`stability: {s}` must be `stable` or `preview`"));
            Stability::Preview
        }
    };
    let host = match fm.host.as_deref() {
        None => None,
        Some(h) => match Language::from_name(h) {
            Some(l @ (Language::Html | Language::Markdown)) => Some(l),
            _ => {
                err(&mut errors, 2, format!("`host: {h}` must be a host language: `html` or `markdown`"));
                None
            }
        },
    };
    for (name, spec) in &fm.options {
        if !OPTION_TYPES.contains(&spec.kind.as_str()) {
            err(&mut errors, 2, format!("option `{name}`: unknown type `{}` (one of {})", spec.kind, OPTION_TYPES.join(", ")));
        } else if let Some(d) = &spec.default
            && !spec.accepts(d)
        {
            let why = match crate::calls::parse_call_items(d) {
                Err(e) if spec.kind == "call[]" => format!(": {e}"),
                _ => String::new(),
            };
            err(&mut errors, 2, format!("option `{name}`: the default is not a `{}`{why}", spec.kind));
        }
    }
    // A `call[]` option declares its `<opt>_mode` (decision 2, W-73).
    let mut fm_options = fm.options.clone();
    for (name, spec) in &fm.options {
        if spec.kind == "call[]" {
            let mode = format!("{name}{}", crate::calls::MODE_SUFFIX);
            if fm.options.contains_key(&mode) {
                err(&mut errors, 2, format!("option `{mode}` is declared by the `call[]` option `{name}`; remove it"));
            }
            fm_options.insert(
                mode,
                OptionSpec {
                    kind: crate::calls::MODE_KIND.into(),
                    default: Some(serde_json::json!("extend")),
                    description: Some(format!("`extend` adds `{name}` to the rule's default list; `replace` uses it instead")),
                },
            );
        }
    }
    for p in &fm.ported_from {
        if p.split('/').count() < 2 {
            err(&mut errors, 2, format!("`ported_from: {p}` must be `<tool>/<rule>`"));
        }
    }
    let security = fm.security.as_ref().and_then(|sec| {
        let mut ok = true;
        let severity = match sec.severity.as_deref() {
            None => {
                err(&mut errors, 2, "`security:` needs `severity: high`, `medium` or `low`".into());
                ok = false;
                None
            }
            Some(v) => SecuritySeverity::parse(v).or_else(|| {
                err(&mut errors, 2, format!("`security:` `severity: {v}` must be `high`, `medium` or `low`"));
                ok = false;
                None
            }),
        };
        if sec.cwe.contains(&0) {
            err(&mut errors, 2, "`security:` `cwe:` lists CWE ids, which start at 1".into());
            ok = false;
        }
        for o in &sec.owasp {
            if !owasp_category(o) {
                err(&mut errors, 2, format!("`security:` `owasp: {o}` must be an OWASP Top 10 category with its edition, `A01:2025` … `A10:2025`"));
                ok = false;
            }
        }
        if sec.cwe.is_empty() && sec.owasp.is_empty() {
            err(&mut errors, 2, "`security:` names at least one `cwe` or `owasp` entry".into());
            ok = false;
        }
        ok.then(|| Security { cwe: sec.cwe.clone(), owasp: sec.owasp.clone(), severity: severity.expect("checked") })
    });
    if let Some((ns, _)) = id.split_once('/')
        && SECURITY_NAMESPACES.contains(&ns)
        && fm.security.is_none()
    {
        err(&mut errors, 2, format!("a `{ns}/` rule carries `security: {{ cwe, owasp, severity }}` (SR-RULE-FORMAT decision 2a)"));
    }
    let invariants = match fm.invariant {
        None => Vec::new(),
        Some(OneOrMany::One(s)) => vec![s],
        Some(OneOrMany::Many(v)) => v,
    };

    // The body, in the template's order (SR-RULE-FORMAT decision 4).
    let pieces = scan(body);
    let mut summary = None;
    let mut why = None;
    let mut block: Option<(usize, Block)> = None;
    let mut block_seen = false;
    let mut h2: Option<String> = None;
    let mut how_to_fix_at = None;
    let mut examples_at = None;
    let mut steps = Vec::new();
    let mut examples: Vec<LanguageExamples> = Vec::new();
    let mut lang: Option<Language> = None;
    let mut kind: Option<ExampleKind> = None;

    for located in &pieces {
        let line = line_of(body, located.offset, body_line);
        match &located.piece {
            Piece::Heading { level: 2, text } => {
                h2 = Some(text.clone());
                lang = None;
                kind = None;
                match text.as_str() {
                    "How to fix" => {
                        if examples_at.is_some() {
                            err(&mut errors, line, "`## How to fix` comes before `## Examples`".into());
                        }
                        how_to_fix_at = Some(line);
                    }
                    "Examples" => examples_at = Some(line),
                    _ => {
                        if examples_at.is_none() {
                            err(&mut errors, line, format!("`## {text}`: other sections go after `## Examples`"));
                        }
                    }
                }
            }
            Piece::Heading { level: 3, text } if h2.as_deref() == Some("Examples") => {
                kind = None;
                lang = Language::from_heading(text);
                match lang {
                    None => err(&mut errors, line, format!("`### {text}` is not a language")),
                    Some(l) if examples.iter().any(|e| e.language == l) => {
                        err(&mut errors, line, format!("`### {text}` appears twice"))
                    }
                    Some(l) => examples.push(LanguageExamples { language: l, donts: vec![], dos: vec![], fixed: vec![] }),
                }
            }
            Piece::Heading { level: 4, text } if h2.as_deref() == Some("Examples") => {
                kind = normalise_kind(text);
                if kind.is_none() {
                    err(&mut errors, line, format!("`#### {text}` must be `Don't`, `Do` or `Fixed`"));
                }
            }
            Piece::Fence { info, code } => {
                let (tag, fence_path, fence_options) = match fence_info(info) {
                    Ok(t) => t,
                    Err(e) => {
                        err(&mut errors, line, e);
                        (String::new(), None, BTreeMap::new())
                    }
                };
                if h2.is_none() && tag == "codelaw" {
                    if block_seen {
                        err(&mut errors, line, "a rule has one `codelaw` block".into());
                    }
                    block_seen = true;
                    match parse_block(code) {
                        Ok(b) => block = Some((line, b)),
                        Err(e) => err(&mut errors, line, e),
                    }
                } else if h2.as_deref() == Some("Examples")
                    && let (Some(l), Some(k)) = (lang, kind)
                {
                    let entry = examples.iter_mut().find(|e| e.language == l).expect("pushed at its heading");
                    let snippet = Snippet { code: code.clone(), path: fence_path, options: fence_options, line };
                    match k {
                        ExampleKind::Dont => entry.donts.push(snippet),
                        ExampleKind::Do => entry.dos.push(snippet),
                        ExampleKind::Fixed => entry.fixed.push(snippet),
                    }
                }
            }
            Piece::Paragraph { text, starts_strong } if h2.is_none() => {
                if starts_strong.as_deref().is_some_and(|s| s.trim() == "Why:") {
                    if block.is_some() {
                        err(&mut errors, line, "`**Why:**` comes before the codelaw block".into());
                    }
                    why = Some(text.trim_start_matches("Why:").trim().to_owned());
                } else if summary.is_none() && why.is_none() && block.is_none() {
                    summary = Some(text.clone());
                }
            }
            Piece::OrderedItem { text } if h2.as_deref() == Some("How to fix") => steps.push(text.clone()),
            _ => {}
        }
    }

    if summary.is_none() {
        err(&mut errors, body_line, "a rule opens with one summary paragraph".into());
    }
    if why.is_none() {
        err(&mut errors, body_line, "a rule needs a `**Why:**` paragraph".into());
    }
    let Some((block_line, block)) = block else {
        if !block_seen {
            errors.push(LoadError::at(path, body_line, "a rule needs a ```codelaw block"));
        }
        return Err(errors);
    };
    let _ = block_line;
    if scope == Scope::Project && !matches!(block.kind, BlockKind::Graph(_)) {
        err(&mut errors, 2, "`scope: project` goes with a `graph:` block".into());
    }
    if matches!(block.kind, BlockKind::Graph(_)) && scope != Scope::Project {
        err(&mut errors, 2, "a `graph:` block needs `scope: project`".into());
    }
    // Decision 3f: only TypeScript has types to ask about.
    if matches!(block.kind, BlockKind::Match { types: Some(_), .. })
        && let Some(other) = languages.iter().find(|l| !matches!(l, Language::Typescript | Language::Tsx))
    {
        err(&mut errors, 2, format!("a `type:` rule names only `typescript` and `tsx` in `languages:`, not `{}`", other.name()));
    }
    if block.fix.is_some() && fix == Some(FixClass::Manual) {
        err(&mut errors, block_line, "a `manual` rule has no `fix:` in its codelaw block; its fix is `## How to fix`".into());
    }
    if let Some(class) = fix.filter(|c| *c != FixClass::Manual)
        && block.fix.is_none()
        && !matches!(block.kind, BlockKind::Native(_))
    {
        err(&mut errors, block_line, format!("a `{class}` rule needs a `fix:` in its codelaw block (L4)"));
    }

    // L4 decision 3: manual rules carry their steps.
    if fix == Some(FixClass::Manual) && steps.is_empty() {
        let at = how_to_fix_at.unwrap_or(body_line);
        err(&mut errors, at, "a `manual` rule needs `## How to fix` with numbered steps (L4 decision 3)".into());
    }

    // SR-RULE-FORMAT decision 5: Don't and Do for every declared language.
    if let Some(at) = examples_at {
        for l in &languages {
            match examples.iter().find(|e| e.language == *l) {
                None => err(&mut errors, at, format!("no `### {}` examples for a declared language", l.display())),
                Some(e) => {
                    if e.donts.is_empty() {
                        err(&mut errors, at, format!("`### {}` needs a `#### Don't` example", l.display()));
                    }
                    if e.dos.is_empty() {
                        err(&mut errors, at, format!("`### {}` needs a `#### Do` example", l.display()));
                    }
                }
            }
        }
        for e in &examples {
            if !languages.contains(&e.language) {
                err(&mut errors, at, format!("`### {}` examples for a language the rule does not declare", e.language.display()));
            }
        }
    } else {
        err(&mut errors, body_line, "a rule needs `## Examples`".into());
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(RuleFile {
        path: path.to_owned(),
        id,
        title,
        languages,
        scope,
        files: fm.files,
        exclude: fm.exclude,
        severity,
        fix: fix.expect("checked above"),
        fix_reason: fm.fix_reason,
        fix_skill: fm.fix_skill,
        stability,
        host,
        invariants,
        options: fm_options,
        ported_from: fm.ported_from,
        tags: fm.tags,
        security,
        since: fm.since,
        summary: summary.unwrap_or_default(),
        why: why.unwrap_or_default(),
        block,
        steps,
        examples,
        cases: Vec::new(),
        hash: hash_bytes(text.as_bytes()),
    })
}

/// `A01:2025` … `A10:2025`: an OWASP Top 10 category and its edition's year.
fn owasp_category(s: &str) -> bool {
    let Some((cat, year)) = s.split_once(':') else { return false };
    let n = cat.strip_prefix('A').filter(|n| n.len() == 2).and_then(|n| n.parse::<u32>().ok());
    n.is_some_and(|n| (1..=10).contains(&n)) && year.len() == 4 && year.chars().all(|c| c.is_ascii_digit())
}

/// Parse a `<name>.cases.md` sidecar.
pub fn parse_cases(path: &Path, text: &str) -> Result<Vec<Case>, Vec<LoadError>> {
    let mut errors = Vec::new();
    let mut cases: Vec<Case> = Vec::new();
    struct Open {
        title: String,
        line: usize,
        input: Option<(Language, Snippet)>,
        expect: Option<Vec<String>>,
        fixed: Option<String>,
    }
    let mut open: Option<Open> = None;
    let finish = |open: Option<Open>, cases: &mut Vec<Case>, errors: &mut Vec<LoadError>| {
        let Some(o) = open else { return };
        match (o.input, o.expect) {
            (Some((language, input)), Some(expect)) => cases.push(Case { title: o.title, language, input, expect, fixed: o.fixed }),
            (None, _) => errors.push(LoadError::at(path, o.line, format!("case `{}` needs an input fence", o.title))),
            (_, None) => errors.push(LoadError::at(path, o.line, format!("case `{}` needs an `expect` fence", o.title))),
        }
    };
    for located in scan(text) {
        let line = line_of(text, located.offset, 1);
        match located.piece {
            Piece::Heading { level: 3, text: heading } => {
                finish(open.take(), &mut cases, &mut errors);
                match heading.strip_prefix("case:") {
                    Some(title) => open = Some(Open { title: title.trim().to_owned(), line, input: None, expect: None, fixed: None }),
                    None => errors.push(LoadError::at(path, line, format!("`### {heading}` must be `### case: <title>`"))),
                }
            }
            Piece::Fence { info, code } => {
                let Some(o) = open.as_mut() else { continue };
                let (tag, fence_path, fence_options) = match fence_info(&info) {
                    Ok(t) => t,
                    Err(e) => {
                        errors.push(LoadError::at(path, line, format!("case `{}`: {e}", o.title)));
                        continue;
                    }
                };
                match tag.as_str() {
                    "expect" => {
                        let lines: Vec<String> = code.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_owned).collect();
                        o.expect = Some(if lines == ["(none)"] { Vec::new() } else { lines });
                    }
                    "fixed" => o.fixed = Some(code),
                    _ => match Language::from_fence_tag(&tag) {
                        Some(l) if o.input.is_none() => o.input = Some((l, Snippet { code, path: fence_path, options: fence_options, line })),
                        Some(_) => errors.push(LoadError::at(path, line, format!("case `{}` has two input fences", o.title))),
                        None => errors.push(LoadError::at(path, line, format!("case `{}`: unknown fence `{tag}`", o.title))),
                    },
                }
            }
            _ => {}
        }
    }
    finish(open.take(), &mut cases, &mut errors);
    if errors.is_empty() { Ok(cases) } else { Err(errors) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_call_option_declares_its_mode_and_checks_its_items() {
        let spec = OptionSpec { kind: "call[]".into(), default: None, description: None };
        assert!(spec.accepts(&serde_json::json!(["a.b", { "name": "x", "module": ["m"] }])));
        assert!(!spec.accepts(&serde_json::json!([{ "name": "x" }])));
        assert!(!spec.accepts(&serde_json::json!("a.b")));
        let mode = OptionSpec { kind: crate::calls::MODE_KIND.into(), default: None, description: None };
        assert!(mode.accepts(&serde_json::json!("replace")) && !mode.accepts(&serde_json::json!("merge")));
    }

    #[test]
    fn a_fence_takes_options_as_compact_json() {
        let (tag, path, options) = fence_info("js path=src/a.js options={\"max\":3,\"skip_comments\":false}").unwrap();
        assert_eq!((tag.as_str(), path.as_deref()), ("js", Some("src/a.js")));
        assert_eq!(options.get("max"), Some(&serde_json::json!(3)));
        assert_eq!(options.get("skip_comments"), Some(&serde_json::json!(false)));
        assert!(fence_info("py").unwrap().2.is_empty());
        assert!(fence_info("py options=[1]").is_err());
        assert!(fence_info("py options={\"max\":").is_err());
    }

    const GOOD: &str = r#"---
id: acme/no-var
title: No var
languages: [javascript, typescript]
fix: auto
stability: stable
---

Use `let` or `const` instead of `var`.

**Why:** `var` is function-scoped and hoisted.

```codelaw
match:
  kind: variable_declaration
message: "Use let instead of var"
fix:
  - replace: "let"
```

## Examples

### JavaScript

#### Don't

```js
var a = 1;
```

#### Do

```js
let a = 1;
```

### TypeScript

#### Don't

```ts
var a: number = 1;
```

#### Do

```ts
let a: number = 1;
```
"#;

    #[test]
    fn a_good_rule_parses() {
        let r = parse_rule(Path::new("acme/no-var.md"), GOOD, Some("acme/no-var")).unwrap();
        assert_eq!(r.languages, vec![Language::Javascript, Language::Typescript]);
        assert_eq!(r.fix, FixClass::Auto);
        assert_eq!(r.summary, "Use let or const instead of var.");
        assert_eq!(r.why, "var is function-scoped and hoisted.");
        assert!(matches!(r.block.kind, BlockKind::Match { .. }));
        assert_eq!(r.examples[1].donts[0].code, "var a: number = 1;\n");
        assert_eq!(r.examples[1].donts[0].line, 41);
    }

    fn errors(text: &str) -> Vec<String> {
        parse_rule(Path::new("acme/no-var.md"), text, Some("acme/no-var")).unwrap_err().into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn l4_obligations_refuse_at_load() {
        let e = errors(&GOOD.replace("fix: auto\n", ""));
        assert_eq!(e, vec!["`fix:` is required: `auto`, `suggest` or `manual` (L4 decision 1)".to_string()], "one error, not a cascade");
        let e = errors(&GOOD.replace("fix: auto", "fix: suggest"));
        assert!(e.iter().any(|m| m.contains("needs `fix_reason:`")), "{e:?}");
        let e = errors(&GOOD.replace("fix: auto", "fix: manual\nfix_reason: x").replace("fix:\n  - replace: \"let\"\n", ""));
        assert!(e.iter().any(|m| m.contains("needs `## How to fix`")), "{e:?}");
        let e = errors(&GOOD.replace("fix: auto", "fix: auto\nfix_skill: codelaw-nope"));
        assert!(e.iter().any(|m| m.contains("ships in no codelaw package")), "{e:?}");
    }

    #[test]
    fn template_checks() {
        let e = errors(&GOOD.replace("id: acme/no-var", "id: acme/other"));
        assert!(e.iter().any(|m| m.contains("must match the file's path")), "{e:?}");
        let e = errors(&GOOD.replace("languages: [javascript, typescript]", "languages: [javascript, typescript, python]"));
        assert!(e.iter().any(|m| m.contains("no `### Python` examples")), "{e:?}");
        let e = errors(&GOOD.replace("title: No var", "title: No var\ncolour: red"));
        assert!(e.iter().any(|m| m.contains("unknown frontmatter field `colour`")), "{e:?}");
        let e = errors(&GOOD.replace("languages: [javascript, typescript]", "languages: [javascrpt]"));
        assert!(e.iter().any(|m| m.contains("did you mean `javascript`")), "{e:?}");
        let e = errors(&GOOD.replace("message:", "mesage:"));
        assert!(e.iter().any(|m| m.contains("did you mean `message`")), "{e:?}");
        let e = errors(&GOOD.replace("#### Do\n\n```ts", "#### Done\n\n```ts"));
        assert!(e.iter().any(|m| m.contains("must be `Don't`, `Do` or `Fixed`")), "{e:?}");
    }

    #[test]
    fn yaml_is_1_2_so_n_and_on_stay_strings() {
        let r = parse_rule(Path::new("acme/no-var.md"), &GOOD.replace("message: \"Use let instead of var\"", "message: n"), Some("acme/no-var")).unwrap();
        assert_eq!(r.block.message.as_deref(), Some("n"));
        let e = errors(&GOOD.replace("message: \"Use let instead of var\"", "message: [1]"));
        assert_eq!(e, vec!["`message:` must be a string".to_string()], "one error, not a second about a missing block");
    }

    #[test]
    fn a_type_clause_goes_with_match_and_typescript_only() {
        let typed = GOOD.replace("message: \"Use let", "type:\n  $$MATCH: any\nmessage: \"Use let");
        let e = errors(&typed);
        assert!(e.iter().any(|m| m.contains("names only `typescript` and `tsx`")), "{e:?}");
        let ts_only = typed.replace("languages: [javascript, typescript]", "languages: [typescript]");
        let ts_only = ts_only.replace("### JavaScript\n\n#### Don't\n\n```js\nvar a = 1;\n```\n\n#### Do\n\n```js\nlet a = 1;\n```\n\n", "");
        let r = parse_rule(Path::new("acme/no-var.md"), &ts_only, Some("acme/no-var")).unwrap();
        assert!(matches!(&r.block.kind, BlockKind::Match { types: Some(_), .. }));
        let native = "native: x\ntype:\n  $$MATCH: any\nmessage: m";
        assert_eq!(parse_block(native).unwrap_err(), "`type:` only goes with `match:`");
    }

    #[test]
    fn a_key_written_twice_is_refused() {
        let e = errors(&GOOD.replace("  kind: variable_declaration\n", "  kind: variable_declaration\n  kind: lexical_declaration\n"));
        assert!(e.iter().any(|m| m.to_lowercase().contains("duplicate")), "{e:?}");
    }

    #[test]
    fn manual_rule_with_steps() {
        let text = GOOD
            .replace("fix: auto", "fix: manual\nfix_reason: needs a human")
            .replace("fix:\n  - replace: \"let\"\n", "")
            .replace("## Examples", "## How to fix\n\n1. Change `var` to `let`.\n2. Check closures.\n\n## Examples");
        let r = parse_rule(Path::new("acme/no-var.md"), &text, Some("acme/no-var")).unwrap();
        assert_eq!(r.steps, vec!["Change var to let.", "Check closures."]);
    }

    #[test]
    fn security_metadata() {
        let text = GOOD.replace("stability: stable", "stability: stable\nsecurity: { cwe: [79, 1004], owasp: [\"A05:2025\"], severity: high }");
        let r = parse_rule(Path::new("acme/no-var.md"), &text, Some("acme/no-var")).unwrap();
        let s = r.security.unwrap();
        assert_eq!(s.cwe, [79, 1004]);
        assert_eq!(s.severity, SecuritySeverity::High);
        assert_eq!(s.cwe_label(), "(CWE-79, CWE-1004)");
        assert_eq!(s.sarif_tags(), ["security", "external/cwe/cwe-79", "external/cwe/cwe-1004", "external/owasp/A05:2025"]);
        for (tag, hit) in [("cwe:79", true), ("cwe:78", false), ("owasp:A05", true), ("owasp:a05:2025", true), ("owasp:A01", false), ("security", true), ("CWE-1004", true)] {
            assert_eq!(s.matches_tag(tag), hit, "{tag}");
        }
        let e = errors(&GOOD.replace("stability: stable", "stability: stable\nsecurity: { cwe: [79], severity: severe }"));
        assert!(e.iter().any(|m| m.contains("must be `high`, `medium` or `low`")), "{e:?}");
        let e = errors(&GOOD.replace("stability: stable", "stability: stable\nsecurity: { owasp: [A5], severity: low }"));
        assert!(e.iter().any(|m| m.contains("OWASP Top 10 category")), "{e:?}");
        let e = errors(&GOOD.replace("stability: stable", "stability: stable\nsecurity: { severity: low }"));
        assert!(e.iter().any(|m| m.contains("at least one")), "{e:?}");
        let e = errors(&GOOD.replace("stability: stable", "stability: stable\nsecurity: { cwe: [79], severity: low, cvss: 9 }"));
        assert!(!e.is_empty(), "an unknown `security:` key is refused");
        // A rule in a security pack's namespace must carry it.
        let sec = GOOD.replace("id: acme/no-var", "id: sec-js/no-var");
        let e: Vec<String> = parse_rule(Path::new("sec-js/no-var.md"), &sec, Some("sec-js/no-var")).unwrap_err().into_iter().map(|e| e.message).collect();
        assert!(e.iter().any(|m| m.contains("carries `security:")), "{e:?}");
    }

    #[test]
    fn cases_sidecar() {
        let text = "# cases\n\n### case: two vars\n\n```js\nvar a; var b;\n```\n\n```expect\n1:1 acme/no-var\n1:8 acme/no-var\n```\n\n### case: none\n\n```js\nlet a;\n```\n\n```expect\n(none)\n```\n";
        let cases = parse_cases(Path::new("x.cases.md"), text).unwrap();
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].expect, vec!["1:1 acme/no-var", "1:8 acme/no-var"]);
        assert!(cases[1].expect.is_empty());
    }
}
