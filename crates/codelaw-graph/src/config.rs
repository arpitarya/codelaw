//! The `[graph]` section (SR-PROJECT-GRAPH decisions 3 and 4): its keys are
//! this record's, validated strictly (SR-CONFIG decision 5).

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;

/// Decorators that register a function with a framework: the decorated
/// symbol is used even with no import (decision 3). `*` matches any one
/// dotted part (`*.get` is `app.get`, `router.get`, …).
pub const DEFAULT_REGISTRARS: &[&str] = &[
    "*.get",
    "*.post",
    "*.put",
    "*.patch",
    "*.delete",
    "*.head",
    "*.options",
    "*.route",
    "*.websocket",
    "*.api_route",
    "*.middleware",
    "*.exception_handler",
    "*.on_event",
    // Flask's request and app hooks (the W-11 corpus run, `flask`).
    "*.before_request",
    "*.before_app_request",
    "*.after_request",
    "*.after_app_request",
    "*.teardown_request",
    "*.teardown_app_request",
    "*.teardown_appcontext",
    "*.errorhandler",
    "*.app_errorhandler",
    "*.context_processor",
    "*.app_context_processor",
    "*.shell_context_processor",
    "*.template_filter",
    "*.app_template_filter",
    "*.template_global",
    "*.app_template_global",
    "*.template_test",
    "*.app_template_test",
    "*.url_value_preprocessor",
    "*.url_defaults",
    "*.listens_for",
    "*.command",
    "*.group",
    "*.callback",
    "*.task",
    "*.shared_task",
    "shared_task",
    "*.fixture",
    "pytest.fixture",
    "fixture",
    "*.hookimpl",
    "*.register",
    "register",
    "receiver",
    "*.receiver",
    "*.listener",
    "*.event",
    "*.tool",
    "*.resource",
    "*.prompt",
    "*.cli",
    "click.command",
    "click.group",
    "Component",
    "Injectable",
    "NgModule",
    "Directive",
    "Pipe",
    "Controller",
    "Module",
    "customElement",
];

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphConfig {
    /// More entry files (globs from the project root).
    #[serde(default)]
    pub entry: Vec<String>,
    /// Files whose exports are kept, whatever uses them.
    #[serde(default)]
    pub keep: Vec<String>,
    /// More registrar decorators, added to the defaults.
    #[serde(default)]
    pub registrars: Vec<String>,
}

fn globs(patterns: &[String]) -> GlobSet {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        if let Ok(g) = GlobBuilder::new(p.trim_start_matches("./")).literal_separator(false).build() {
            b.add(g);
        }
    }
    b.build().unwrap_or_else(|_| GlobSet::empty())
}

impl GraphConfig {
    /// Parse `[graph]`; an unknown key or a bad glob is an error.
    pub fn from_table(table: Option<&toml::Table>) -> Result<GraphConfig, String> {
        let Some(t) = table else { return Ok(GraphConfig::default()) };
        let c: GraphConfig = toml::Value::Table(t.clone()).try_into().map_err(|e: toml::de::Error| format!("`[graph]`: {}", e.message()))?;
        for p in c.entry.iter().chain(&c.keep) {
            GlobBuilder::new(p).build().map_err(|e| format!("`[graph]`: invalid glob `{p}`: {e}"))?;
        }
        Ok(c)
    }

    pub fn entry_set(&self) -> GlobSet {
        globs(&self.entry)
    }

    pub fn keep_set(&self) -> GlobSet {
        globs(&self.keep)
    }

    /// Whether a decorator name registers the symbol it decorates.
    pub fn is_registrar(&self, decorator: &str) -> bool {
        DEFAULT_REGISTRARS.iter().copied().chain(self.registrars.iter().map(String::as_str)).any(|p| registrar_match(p, decorator))
    }
}

fn registrar_match(pattern: &str, name: &str) -> bool {
    let pp: Vec<&str> = pattern.split('.').collect();
    let np: Vec<&str> = name.split('.').collect();
    if pp.first() == Some(&"*") {
        // `*.get`: the last parts match, with at least one part before.
        let rest = &pp[1..];
        np.len() > rest.len() && np[np.len() - rest.len()..] == *rest
    } else {
        pp == np
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registrars() {
        let c = GraphConfig::default();
        assert!(c.is_registrar("app.get"));
        assert!(c.is_registrar("api.router.post"));
        assert!(c.is_registrar("pytest.fixture"));
        assert!(c.is_registrar("bp.before_app_request"));
        assert!(!c.is_registrar("get"));
        assert!(!c.is_registrar("functools.lru_cache"));
    }

    #[test]
    fn strict_keys() {
        let t: toml::Table = toml::from_str("entri = []").unwrap();
        assert!(GraphConfig::from_table(Some(&t)).unwrap_err().contains("entri"));
    }
}
