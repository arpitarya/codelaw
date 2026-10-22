//! The deprecations registry, `.codelaw/deprecations.toml`, and the
//! `[deprecation]` defaults of `codelaw.toml` (SR-BASELINE decision 6).

use crate::date::{Date, from_toml};
use crate::file::OnTouch;
use std::path::{Path, PathBuf};

/// A deprecation's policy: its deadline, its warning window and what a
/// touch does.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Policy {
    pub until: Option<Date>,
    pub warn_from: Option<Date>,
    pub on_touch: Option<OnTouch>,
}

/// An optional `rewrite = { from, to }`: the shape of a `suggest` fix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rewrite {
    pub from: String,
    pub to: String,
}

/// One `[[deprecated]]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deprecation {
    /// `<package or path>#<export>` (JS/TS) or `<module>:<qualname>` (Python).
    pub symbol: String,
    pub replacement: Option<String>,
    pub policy: Policy,
    pub reason: Option<String>,
    pub rewrite: Option<Rewrite>,
}

/// The registry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Registry {
    pub deprecated: Vec<Deprecation>,
}

/// `<root>/.codelaw/deprecations.toml`.
pub fn registry_path(root: &Path) -> PathBuf {
    root.join(".codelaw").join("deprecations.toml")
}

pub const REGISTRY_REL: &str = ".codelaw/deprecations.toml";

const KEYS: &[&str] = &["symbol", "replacement", "until", "warn_from", "on_touch", "reason", "rewrite"];

fn policy(t: &toml::Table, at: &dyn Fn(String) -> String) -> Result<Policy, String> {
    let date = |k: &str| t.get(k).map(from_toml).transpose().map_err(|e| at(format!("`{k}`: {e}")));
    let on_touch = match t.get("on_touch") {
        None => None,
        Some(toml::Value::String(s)) => Some(OnTouch::parse(s).map_err(at)?),
        Some(_) => return Err(at("`on_touch` must be a string".into())),
    };
    let p = Policy { until: date("until")?, warn_from: date("warn_from")?, on_touch };
    if let (Some(w), Some(u)) = (p.warn_from, p.until)
        && w > u
    {
        return Err(at(format!("`warn_from` ({w}) is after `until` ({u})")));
    }
    Ok(p)
}

impl Registry {
    pub fn parse(text: &str) -> Result<Registry, String> {
        let table: toml::Table = text.parse().map_err(|e: toml::de::Error| e.message().to_owned())?;
        for k in table.keys() {
            if k != "deprecated" {
                return Err(format!("unknown key `{k}` (the registry holds only `[[deprecated]]` entries)"));
            }
        }
        let list = match table.get("deprecated") {
            None => &Vec::new(),
            Some(toml::Value::Array(a)) => a,
            Some(_) => return Err("`deprecated` must be an array of tables (`[[deprecated]]`)".into()),
        };
        let mut out = Registry::default();
        for (i, d) in list.iter().enumerate() {
            let at = |m: String| format!("[[deprecated]] #{}: {m}", i + 1);
            let toml::Value::Table(t) = d else { return Err(at("not a table".into())) };
            for k in t.keys() {
                if !KEYS.contains(&k.as_str()) {
                    return Err(at(format!("unknown key `{k}` (one of {})", KEYS.join(", "))));
                }
            }
            let string = |k: &str| -> Result<Option<String>, String> {
                match t.get(k) {
                    None => Ok(None),
                    Some(toml::Value::String(s)) => Ok(Some(s.clone())),
                    Some(_) => Err(at(format!("`{k}` must be a string"))),
                }
            };
            let symbol = string("symbol")?.filter(|s| !s.is_empty()).ok_or_else(|| at("`symbol` is required".into()))?;
            if !symbol.contains('#') && !symbol.contains(':') {
                return Err(at(format!("`symbol = \"{symbol}\"`: name it as `<package or path>#<export>` or `<module>:<name>`")));
            }
            let rewrite = match t.get("rewrite") {
                None => None,
                Some(toml::Value::Table(r)) => match (r.get("from").and_then(|v| v.as_str()), r.get("to").and_then(|v| v.as_str()), r.len()) {
                    (Some(from), Some(to), 2) => Some(Rewrite { from: from.to_owned(), to: to.to_owned() }),
                    _ => return Err(at("`rewrite` is `{ from = \"…\", to = \"…\" }`".into())),
                },
                Some(_) => return Err(at("`rewrite` is `{ from = \"…\", to = \"…\" }`".into())),
            };
            if out.deprecated.iter().any(|d: &Deprecation| d.symbol == symbol) {
                return Err(at(format!("`{symbol}` is deprecated twice")));
            }
            out.deprecated.push(Deprecation { symbol, replacement: string("replacement")?, policy: policy(t, &at)?, reason: string("reason")?, rewrite });
        }
        Ok(out)
    }

    /// Read the registry; `None` when there is none.
    pub fn load(root: &Path) -> Result<Option<Registry>, String> {
        match std::fs::read_to_string(registry_path(root)) {
            Ok(text) => Registry::parse(&text).map(Some).map_err(|e| format!("{REGISTRY_REL}: {e}")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{REGISTRY_REL}: {e}")),
        }
    }

    pub fn find(&self, symbol: &str) -> Option<&Deprecation> {
        self.deprecated.iter().find(|d| d.symbol == symbol)
    }
}

/// The `[deprecation]` table of `codelaw.toml`: the policy of a symbol
/// marked deprecated in code (JSDoc `@deprecated`, PEP 702) that the
/// registry does not list.
/// The loader has already refused an unknown key (SR-CONFIG decision 5);
/// this checks the values.
pub fn defaults(until: Option<&str>, warn_from: Option<&str>, on_touch: Option<&str>) -> Result<Policy, String> {
    let mut t = toml::Table::new();
    for (k, v) in [("until", until), ("warn_from", warn_from), ("on_touch", on_touch)] {
        if let Some(v) = v {
            t.insert(k.to_owned(), toml::Value::String(v.to_owned()));
        }
    }
    policy(&t, &|m| format!("[deprecation]: {m}"))
}

/// The text `codelaw deprecate` appends to the registry: the file stays
/// the user's, so what is there is kept byte for byte.
pub fn entry_text(d: &Deprecation) -> String {
    let q = |s: &str| toml::Value::String(s.to_owned()).to_string();
    let mut out = format!("[[deprecated]]\nsymbol = {}\n", q(&d.symbol));
    if let Some(r) = &d.replacement {
        out.push_str(&format!("replacement = {}\n", q(r)));
    }
    if let Some(u) = d.policy.until {
        out.push_str(&format!("until = \"{u}\"\n"));
    }
    if let Some(w) = d.policy.warn_from {
        out.push_str(&format!("warn_from = \"{w}\"\n"));
    }
    if let Some(o) = d.policy.on_touch {
        out.push_str(&format!("on_touch = \"{}\"\n", o.name()));
    }
    if let Some(r) = &d.reason {
        out.push_str(&format!("reason = {}\n", q(r)));
    }
    out
}
