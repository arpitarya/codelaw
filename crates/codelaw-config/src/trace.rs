//! `codelaw config --print [--for <file>]`: every resolved value and where
//! it came from — default, file:line, override #n, env or flag
//! (SR-CONFIG decision 7).

use crate::raw::{RawConfig, RuleSetting};
use crate::resolve::{Config, ConfigFile, Home};
use codelaw_core::{Language, LineIndex};
use serde_json::{Value, json};
use std::path::Path;

/// One resolved value and its source.
#[derive(Clone, Debug, PartialEq)]
pub struct Traced {
    /// Dotted: `extends`, `cache.max_size`, `rules."py/no-print"`.
    pub key: String,
    pub value: Value,
    /// `default`, `codelaw.toml:3`, `codelaw.toml:9 override #1`,
    /// `env CODELAW_CACHE_DIR` or `flag --no-cache`.
    pub source: String,
}

/// What a run takes from outside the config files (decision 4, steps 6–7).
#[derive(Clone, Debug, Default)]
pub struct TraceEnv {
    /// `CODELAW_CACHE_DIR`.
    pub cache_dir: Option<String>,
    /// `--no-cache`.
    pub no_cache: bool,
}

impl ConfigFile {
    /// The 1-based line of a key, as a path of table keys and array indexes
    /// below this home's codelaw table (`["rules", "py/no-print"]`,
    /// `["overrides", "0", "rules", "a/b"]`).
    pub fn line_of(&self, path: &[&str]) -> Option<u32> {
        let offset = match self.home {
            Home::CodelawToml => toml_offset(&self.text, path),
            Home::Pyproject => {
                let mut full = vec!["tool", "codelaw"];
                full.extend_from_slice(path);
                toml_offset(&self.text, &full)
            }
            Home::PackageJson => json_offset(&self.text, path),
        }?;
        Some(LineIndex::new(&self.text).position(offset).line)
    }
}

fn toml_offset(text: &str, path: &[&str]) -> Option<usize> {
    use toml::de::{DeTable, DeValue};
    let doc = DeTable::parse(text).ok()?;
    let mut table = doc.get_ref();
    let mut offset = None;
    let mut i = 0;
    while i < path.len() {
        let (key, value) = table.iter().find(|(k, _)| k.get_ref().as_ref() == path[i])?;
        offset = Some(key.span().start);
        let mut value = value;
        // An array element: `[[overrides]]` or an inline array of tables.
        if let (DeValue::Array(a), Some(n)) = (value.get_ref(), path.get(i + 1).and_then(|s| s.parse::<usize>().ok())) {
            value = a.get(n)?;
            offset = Some(value.span().start);
            i += 1;
        }
        i += 1;
        if i < path.len() {
            match value.get_ref() {
                DeValue::Table(t) => table = t,
                _ => return None,
            }
        }
    }
    offset
}

/// JSON homes: the first `"key"` after the previous one, below `"codelaw"`.
/// Approximate, but enough to name a line.
fn json_offset(text: &str, path: &[&str]) -> Option<usize> {
    let mut at = text.find("\"codelaw\"")?;
    for seg in path {
        if seg.parse::<usize>().is_ok() {
            continue;
        }
        let needle = serde_json::to_string(seg).ok()?;
        at += text[at..].find(&needle)?;
    }
    Some(at)
}

fn rel(config: &Config, file: &ConfigFile) -> String {
    codelaw_core::normalize_path(file.path.strip_prefix(&config.root_dir).unwrap_or(&file.path))
}

fn size(n: u64) -> Value {
    json!(n)
}

fn duration(d: std::time::Duration) -> Value {
    let ms = d.as_millis();
    if ms.is_multiple_of(1000) { json!(format!("{}s", ms / 1000)) } else { json!(format!("{ms}ms")) }
}

impl Config {
    fn source(&self, file: &ConfigFile, path: &[&str]) -> String {
        match file.line_of(path) {
            Some(line) => format!("{}:{line}", rel(self, file)),
            None => rel(self, file),
        }
    }

    /// The folder or file a trace is for: the root when none is given.
    fn target(&self, file: Option<&Path>) -> std::path::PathBuf {
        file.map(Path::to_path_buf).unwrap_or_else(|| self.root_dir.clone())
    }

    /// The configs that govern `target`, outermost first.
    fn governing(&self, target: &Path) -> Vec<&ConfigFile> {
        self.files.iter().filter(|f| target.starts_with(&f.dir)).collect()
    }

    /// Every resolved value of the configuration for `file` (the project
    /// root when `None`), with its source, in schema order. Rule severities
    /// need the rule set and are traced by the engine
    /// ([`Config::rule_setting_sources`]).
    pub fn trace(&self, file: Option<&Path>, env: &TraceEnv) -> Vec<Traced> {
        let target = self.target(file);
        let chain = self.governing(&target);
        let root = self.files.first();
        let mut out = Vec::new();
        let mut push = |key: &str, value: Value, source: String| out.push(Traced { key: key.to_owned(), value, source });

        // A root-only key: the root config's value, or the default.
        let root_key = |path: &[&str], set: fn(&RawConfig) -> Option<Value>, default: Value| -> (Value, String) {
            match root.and_then(|f| set(&f.raw).map(|v| (v, f))) {
                Some((v, f)) => (v, self.source(f, path)),
                None => (default, "default".to_owned()),
            }
        };

        let (v, s) = root_key(&["extends"], |r| (!r.extends.is_empty()).then(|| json!(r.extends)), json!([]));
        push("extends", v, s);

        match chain.iter().rev().find(|f| f.raw.languages.is_some()) {
            Some(f) => push("languages", json!(f.raw.languages), self.source(f, &["languages"])),
            None => push("languages", json!(Language::ALL.iter().map(|l| l.name()).collect::<Vec<_>>()), "default".into()),
        }
        for key in ["include", "exclude"] {
            let mut any = false;
            for f in &chain {
                let globs = if key == "include" { &f.raw.include } else { &f.raw.exclude };
                if !globs.is_empty() {
                    any = true;
                    push(key, json!(globs), self.source(f, &[key]));
                }
            }
            if !any {
                push(key, json!([]), "default".into());
            }
        }
        let (v, s) = root_key(&["rules_dir"], |r| r.rules_dir.as_ref().map(|d| json!(d)), json!(".codelaw/rules"));
        push("rules_dir", v, s);
        match chain.iter().rev().find(|f| f.raw.root) {
            Some(f) => push("root", json!(true), self.source(f, &["root"])),
            None => push("root", json!(false), "default".into()),
        }
        let mut any = false;
        for f in chain.iter().filter(|f| f.raw.requires.is_some()) {
            any = true;
            push("requires", json!(f.raw.requires), self.source(f, &["requires"]));
        }
        if !any {
            push("requires", Value::Null, "default".into());
        }
        let (v, s) = root_key(&["preview"], |r| r.preview.map(|p| json!(p)), json!(false));
        push("preview", v, s);
        let (v, s) = root_key(&["allow_pack_fixers"], |r| (!r.allow_pack_fixers.is_empty()).then(|| json!(r.allow_pack_fixers)), json!([]));
        push("allow_pack_fixers", v, s);

        let (v, s) = root_key(&["files", "follow_symlinks"], |r| r.files.follow_symlinks.map(|x| json!(x)), json!(false));
        push("files.follow_symlinks", v, s);
        let (_, s) = root_key(&["files", "generated_markers"], |r| r.files.generated_markers.as_ref().map(|x| json!(x)), Value::Null);
        push("files.generated_markers", json!(self.generated_markers()), if s == "default" { s } else { format!("default + {s}") });
        let (v, s) = root_key(&["files", "lint_generated"], |r| r.files.lint_generated.map(|x| json!(x)), json!(false));
        push("files.lint_generated", v, s);
        let (_, s) = root_key(&["files", "max_file_size"], |r| r.files.max_file_size.map(|x| json!(x.0)), Value::Null);
        push("files.max_file_size", size(self.max_file_size()), s);

        if env.no_cache {
            push("cache.enabled", json!(false), "flag --no-cache".into());
        } else {
            let (v, s) = root_key(&["cache", "enabled"], |r| r.cache.enabled.map(|x| json!(x)), json!(true));
            push("cache.enabled", v, s);
        }
        match env.cache_dir.as_deref().filter(|d| !d.is_empty()) {
            Some(d) => push("cache.dir", json!(d), "env CODELAW_CACHE_DIR".into()),
            None => {
                let (v, s) = root_key(&["cache", "dir"], |r| r.cache.dir.as_ref().map(|x| json!(x)), json!(".codelaw/cache"));
                push("cache.dir", v, s);
            }
        }
        let (_, s) = root_key(&["cache", "max_size"], |r| r.cache.max_size.map(|x| json!(x.0)), Value::Null);
        push("cache.max_size", size(self.cache_max_size()), s);
        let (_, s) = root_key(&["fix", "external_timeout"], |r| r.fix.external_timeout.map(|_| Value::Null), Value::Null);
        push("fix.external_timeout", duration(self.external_timeout()), s);
        let (v, s) = root_key(&["adapters", "enabled"], |r| r.adapters.enabled.as_ref().map(|x| json!(x)), Value::Null);
        push("adapters.enabled", v, s);
        let (_, s) = root_key(&["adapters", "timeout"], |r| r.adapters.timeout.map(|_| Value::Null), Value::Null);
        push("adapters.timeout", duration(self.adapter_timeout()), s);
        let (v, s) = root_key(&["lint", "tagged_templates"], |r| r.lint.tagged_templates.map(|x| json!(x)), json!(false));
        push("lint.tagged_templates", v, s);
        let (v, s) = root_key(&["markdown", "lint_code_blocks"], |r| r.markdown.lint_code_blocks.map(|x| json!(x)), json!(false));
        push("markdown.lint_code_blocks", v, s);

        // The sections not validated yet: shown as written.
        for (name, get) in [
            ("format", (|r: &RawConfig| r.format.clone()) as fn(&RawConfig) -> Option<toml::Table>),
            ("graph", |r| r.graph.clone()),
            ("baseline", |r| r.baseline.clone()),
            ("agents", |r| r.agents.clone()),
            ("daemon", |r| r.daemon.clone()),
            ("lsp", |r| r.lsp.clone()),
        ] {
            for f in &chain {
                if let Some(t) = get(&f.raw) {
                    push(name, serde_json::to_value(&t).unwrap_or(Value::Null), format!("{} (not validated yet)", self.source(f, &[name])));
                }
            }
        }
        out
    }

    /// Every setting of rule `id` that applies to `file` (the root when
    /// `None`), lowest precedence first, with its source: each config's
    /// `[rules]`, then the matching `[[overrides]]` (decision 4).
    pub fn rule_setting_sources(&self, file: Option<&Path>, id: &str) -> Vec<(String, &RuleSetting)> {
        let target = self.target(file);
        let mut out = Vec::new();
        for f in self.governing(&target) {
            if let Some(s) = f.raw.rules.get(id) {
                out.push((self.source(f, &["rules", id]), s));
            }
        }
        for f in self.governing(&target) {
            let Some(relative) = f.relative(&target) else { continue };
            for (i, (o, (files, exclude))) in f.raw.overrides.iter().zip(&f.overrides).enumerate() {
                if files.is_match(relative)
                    && !exclude.is_match(relative)
                    && let Some(s) = o.rules.get(id)
                {
                    let n = i.to_string();
                    out.push((format!("{} override #{}", self.source(f, &["overrides", &n, "rules", id]), i + 1), s));
                }
            }
        }
        out
    }

    /// The config files that govern `file` (the root when `None`),
    /// outermost first, relative to the project root.
    pub fn governing_files(&self, file: Option<&Path>) -> Vec<String> {
        let target = self.target(file);
        self.governing(&target).into_iter().map(|f| rel(self, f)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("codelaw-trace-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        fs::canonicalize(d).unwrap()
    }

    fn get<'a>(t: &'a [Traced], key: &str) -> &'a Traced {
        t.iter().find(|x| x.key == key).unwrap_or_else(|| panic!("no {key} in {t:?}"))
    }

    #[test]
    fn every_value_names_its_source() {
        let d = temp("sources");
        fs::write(
            d.join("codelaw.toml"),
            "extends = [\"codelaw:js/recommended\"]\n\n[cache]\nmax_size = \"1 MiB\"\n\n[rules]\n\"a/x\" = \"warn\"\n\n[[overrides]]\nfiles = [\"pkg/**\"]\nrules = { \"a/x\" = \"off\" }\n",
        )
        .unwrap();
        fs::create_dir_all(d.join("pkg")).unwrap();
        fs::write(d.join("pkg/codelaw.toml"), "languages = [\"typescript\"]\n").unwrap();
        let c = crate::find_config(&d.join("pkg")).unwrap();
        let t = c.trace(None, &TraceEnv { cache_dir: Some("/tmp/x".into()), no_cache: false });
        assert_eq!(get(&t, "extends").source, "codelaw.toml:1");
        assert_eq!(get(&t, "cache.max_size").value, json!(1 << 20));
        assert_eq!(get(&t, "cache.max_size").source, "codelaw.toml:4");
        assert_eq!(get(&t, "cache.dir").source, "env CODELAW_CACHE_DIR");
        assert_eq!(get(&t, "preview").source, "default");
        assert_eq!(get(&t, "languages").source, "default");
        let file = d.join("pkg/a.ts");
        let t = c.trace(Some(&file), &TraceEnv { no_cache: true, ..Default::default() });
        assert_eq!(get(&t, "languages").source, "pkg/codelaw.toml:1");
        assert_eq!(get(&t, "cache.enabled").source, "flag --no-cache");
        let s: Vec<String> = c.rule_setting_sources(Some(&file), "a/x").into_iter().map(|(s, _)| s).collect();
        assert_eq!(s, ["codelaw.toml:7", "codelaw.toml:11 override #1"]);
        assert_eq!(c.rule_setting_sources(None, "a/x").len(), 1);
    }

    #[test]
    fn lines_in_pyproject_and_package_json() {
        let d = temp("homes");
        fs::write(d.join("pyproject.toml"), "[project]\nname = \"x\"\n\n[tool.codelaw]\npreview = true\n").unwrap();
        let c = crate::find_config(&d).unwrap();
        assert_eq!(get(&c.trace(None, &TraceEnv::default()), "preview").source, "pyproject.toml:5");
        fs::remove_file(d.join("pyproject.toml")).unwrap();
        fs::write(d.join("package.json"), "{\n  \"name\": \"x\",\n  \"codelaw\": {\n    \"preview\": true\n  }\n}\n").unwrap();
        let c = crate::find_config(&d).unwrap();
        assert_eq!(get(&c.trace(None, &TraceEnv::default()), "preview").source, "package.json:4");
    }
}
