//! `.codelaw/baseline/<rule-id>.toml` (SR-BASELINE decision 1): one file
//! per rule, one `[[allow]]` per file (and symbol), counts not lines.

use crate::date::{Date, from_toml};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What a touched file's grandfathered occurrences become (decision 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OnTouch {
    /// Every occurrence in the file is an error: migrate it now.
    #[default]
    Error,
    /// Every occurrence in the file is a warning.
    Warn,
    /// Touching the file changes nothing.
    Keep,
}

impl OnTouch {
    pub fn parse(s: &str) -> Result<OnTouch, String> {
        match s {
            "error" => Ok(OnTouch::Error),
            "warn" => Ok(OnTouch::Warn),
            "keep" => Ok(OnTouch::Keep),
            other => Err(format!("`on_touch = \"{other}\"` is not one of error, warn, keep")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            OnTouch::Error => "error",
            OnTouch::Warn => "warn",
            OnTouch::Keep => "keep",
        }
    }

    /// How strict: `error` > `warn` > `keep`. The ratchet refuses a
    /// policy that got less strict.
    pub fn strictness(self) -> u8 {
        match self {
            OnTouch::Error => 2,
            OnTouch::Warn => 1,
            OnTouch::Keep => 0,
        }
    }
}

/// One `[[allow]]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Relative to the project root, `/`-separated.
    pub file: String,
    /// For symbol-based rules (`graph/no-deprecated-usage`).
    pub symbol: Option<String>,
    pub count: usize,
    /// Inherited from the deprecation when unset.
    pub until: Option<Date>,
    pub warn_from: Option<Date>,
    /// Inherited from the deprecation, else `error`, when unset.
    pub on_touch: Option<OnTouch>,
    pub reason: Option<String>,
}

impl Entry {
    pub fn key(&self) -> (&str, Option<&str>) {
        (&self.file, self.symbol.as_deref())
    }
}

/// One rule's baseline.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BaselineFile {
    pub rule: String,
    /// Sorted by file, then symbol (L5: deterministic output).
    pub entries: Vec<Entry>,
}

/// The file name of a rule's baseline: `/` becomes `--`
/// (`graph/no-deprecated-usage` → `graph--no-deprecated-usage.toml`).
pub fn file_name(rule: &str) -> String {
    format!("{}.toml", rule.replace('/', "--"))
}

/// The rule a baseline file name is for.
pub fn rule_of(file_name: &str) -> Option<String> {
    file_name.strip_suffix(".toml").filter(|s| !s.is_empty()).map(|s| s.replace("--", "/"))
}

/// `<root>/.codelaw/baseline`: one baseline per project root
/// (SR-ARCHITECTURE decision 17).
pub fn dir(root: &Path) -> PathBuf {
    root.join(".codelaw").join("baseline")
}

/// The project-relative path of a rule's baseline file.
pub fn rel_path(rule: &str) -> String {
    format!(".codelaw/baseline/{}", file_name(rule))
}

const ALLOW_KEYS: &[&str] = &["file", "symbol", "count", "until", "warn_from", "on_touch", "reason"];

impl BaselineFile {
    /// Parse one baseline file. Every key is checked: an unknown one is
    /// an error, as in the config (SR-CONFIG decision 5).
    pub fn parse(rule: &str, text: &str) -> Result<BaselineFile, String> {
        let table: toml::Table = text.parse().map_err(|e: toml::de::Error| e.message().to_owned())?;
        for k in table.keys() {
            if k != "allow" {
                return Err(format!("unknown key `{k}` (a baseline holds only `[[allow]]` entries)"));
            }
        }
        let mut entries = Vec::new();
        let allows = match table.get("allow") {
            None => &Vec::new(),
            Some(toml::Value::Array(a)) => a,
            Some(_) => return Err("`allow` must be an array of tables (`[[allow]]`)".into()),
        };
        for (i, a) in allows.iter().enumerate() {
            let at = |m: String| format!("[[allow]] #{}: {m}", i + 1);
            let toml::Value::Table(t) = a else { return Err(at("not a table".into())) };
            for k in t.keys() {
                if !ALLOW_KEYS.contains(&k.as_str()) {
                    return Err(at(format!("unknown key `{k}` (one of {})", ALLOW_KEYS.join(", "))));
                }
            }
            let string = |k: &str| -> Result<Option<String>, String> {
                match t.get(k) {
                    None => Ok(None),
                    Some(toml::Value::String(s)) => Ok(Some(s.clone())),
                    Some(_) => Err(at(format!("`{k}` must be a string"))),
                }
            };
            let date = |k: &str| t.get(k).map(from_toml).transpose().map_err(|e| at(format!("`{k}`: {e}")));
            let file = string("file")?.filter(|f| !f.is_empty()).ok_or_else(|| at("`file` is required".into()))?;
            let count = match t.get("count") {
                Some(toml::Value::Integer(n)) if *n >= 0 => *n as usize,
                Some(_) => return Err(at("`count` must be a whole number, 0 or more".into())),
                None => return Err(at("`count` is required".into())),
            };
            let on_touch = string("on_touch")?.map(|s| OnTouch::parse(&s)).transpose().map_err(at)?;
            entries.push(Entry {
                file: file.replace('\\', "/"),
                symbol: string("symbol")?,
                count,
                until: date("until")?,
                warn_from: date("warn_from")?,
                on_touch,
                reason: string("reason")?,
            });
        }
        let mut b = BaselineFile { rule: rule.to_owned(), entries };
        b.sort();
        for w in b.entries.windows(2) {
            if w[0].key() == w[1].key() {
                return Err(format!("two `[[allow]]` entries for `{}`{}", w[0].file, w[0].symbol.as_deref().map(|s| format!(" and `{s}`")).unwrap_or_default()));
            }
        }
        Ok(b)
    }

    pub fn sort(&mut self) {
        self.entries.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.symbol.cmp(&b.symbol)));
    }

    pub fn find(&self, file: &str, symbol: Option<&str>) -> Option<&Entry> {
        self.entries.iter().find(|e| e.key() == (file, symbol))
    }

    /// The file's text: sorted, one key per line, always the same bytes
    /// for the same entries.
    pub fn write(&self) -> String {
        let q = |s: &str| toml::Value::String(s.to_owned()).to_string();
        let mut out = format!(
            "# The baseline of `{}` (SR-BASELINE). Written by `codelaw baseline`;\n# review every change: a count may only go down.\n",
            self.rule
        );
        for e in &self.entries {
            out.push_str("\n[[allow]]\n");
            out.push_str(&format!("file = {}\n", q(&e.file)));
            if let Some(s) = &e.symbol {
                out.push_str(&format!("symbol = {}\n", q(s)));
            }
            out.push_str(&format!("count = {}\n", e.count));
            if let Some(d) = e.until {
                out.push_str(&format!("until = \"{d}\"\n"));
            }
            if let Some(d) = e.warn_from {
                out.push_str(&format!("warn_from = \"{d}\"\n"));
            }
            if let Some(o) = e.on_touch {
                out.push_str(&format!("on_touch = \"{}\"\n", o.name()));
            }
            if let Some(r) = &e.reason {
                out.push_str(&format!("reason = {}\n", q(r)));
            }
        }
        out
    }
}

/// Every rule's baseline in a project.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Baselines {
    pub rules: BTreeMap<String, BaselineFile>,
}

impl Baselines {
    /// Read `<root>/.codelaw/baseline/*.toml`. No folder is no baseline.
    pub fn load(root: &Path) -> Result<Baselines, String> {
        let dir = dir(root);
        let mut out = Baselines::default();
        let Ok(read) = std::fs::read_dir(&dir) else { return Ok(out) };
        let mut names: Vec<String> = read.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|n| n.ends_with(".toml")).collect();
        names.sort();
        for name in names {
            let Some(rule) = rule_of(&name) else { continue };
            let path = dir.join(&name);
            let text = std::fs::read_to_string(&path).map_err(|e| format!(".codelaw/baseline/{name}: {e}"))?;
            out.insert(BaselineFile::parse(&rule, &text).map_err(|e| format!(".codelaw/baseline/{name}: {e}"))?);
        }
        Ok(out)
    }

    /// Parse baseline files given as `(file name, text)` pairs (another
    /// revision's, read through `git show`).
    pub fn from_texts(files: &[(String, String)]) -> Result<Baselines, String> {
        let mut out = Baselines::default();
        for (name, text) in files {
            let Some(rule) = rule_of(name) else { continue };
            out.insert(BaselineFile::parse(&rule, text).map_err(|e| format!(".codelaw/baseline/{name}: {e}"))?);
        }
        Ok(out)
    }

    pub fn insert(&mut self, b: BaselineFile) {
        self.rules.insert(b.rule.clone(), b);
    }

    pub fn is_empty(&self) -> bool {
        self.rules.values().all(|b| b.entries.is_empty())
    }

    /// Write one rule's file; an empty baseline removes it.
    pub fn save(root: &Path, b: &BaselineFile) -> std::io::Result<()> {
        let path = dir(root).join(file_name(&b.rule));
        if b.entries.is_empty() {
            return match std::fs::remove_file(&path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            };
        }
        std::fs::create_dir_all(dir(root))?;
        std::fs::write(path, b.write())
    }
}
