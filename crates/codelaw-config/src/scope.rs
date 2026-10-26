//! `[scope.<name>]` and `[profile.<name>.scope.<name>]` (SR-CONFIG
//! decisions 3, 4 and 10): the named groups of files a config gives their
//! own rule settings, built from the presets' scopes first and then the
//! project's, in the order written.

use crate::raw::{RawConfig, RawProfile, RawScope, RuleSetting};
use crate::{ConfigError, did_you_mean};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// A scope a preset ships, resolved from `extends` (SR-RULE-PACKS).
#[derive(Clone, Debug, PartialEq)]
pub struct PresetScope {
    /// The `extends` entry that brought it, e.g. `codelaw:js/recommended`.
    pub preset: String,
    pub name: String,
    pub scope: RawScope,
}

/// One scope after merging, ready to match files.
#[derive(Clone, Debug)]
pub(crate) struct Scope {
    pub name: String,
    pub files: Vec<String>,
    pub exclude: Vec<String>,
    files_set: GlobSet,
    exclude_set: GlobSet,
    pub rules: BTreeMap<String, RuleSetting>,
    pub remap: Option<BTreeMap<String, String>>,
    /// The profile it belongs to.
    pub profile: Option<String>,
    /// Where the project wrote it, as a path of keys below the config's
    /// table; `None` for a scope only a preset defines.
    pub toml_path: Option<Vec<String>>,
    /// The presets that contributed to it.
    pub presets: Vec<String>,
    /// Rules whose setting is still the preset's, by preset.
    pub rule_from_preset: BTreeMap<String, String>,
}

impl Scope {
    pub fn matches(&self, rel: &Path) -> bool {
        self.files_set.is_match(rel) && !self.exclude_set.is_match(rel)
    }

    fn compile(&mut self, file: &Path) -> Result<(), ConfigError> {
        self.files_set = glob_set(&self.files, file)?;
        self.exclude_set = glob_set(&self.exclude, file)?;
        Ok(())
    }

    fn apply(&mut self, raw: &RawScope, preset: Option<&str>) {
        if let Some(f) = &raw.files {
            self.files = f.clone();
        }
        if let Some(e) = &raw.exclude {
            self.exclude = e.clone();
        }
        for (id, s) in &raw.rules {
            let merged = match self.rules.get(id) {
                Some(old) => merge_setting(old, s),
                None => s.clone(),
            };
            self.rules.insert(id.clone(), merged);
            match preset {
                Some(p) => {
                    self.rule_from_preset.insert(id.clone(), p.to_owned());
                }
                None => {
                    self.rule_from_preset.remove(id);
                }
            }
        }
        for (from, to) in raw.remap.iter().flatten() {
            self.remap.get_or_insert_with(Default::default).insert(from.clone(), to.clone());
        }
        if let Some(p) = preset
            && !self.presets.iter().any(|x| x == p)
        {
            self.presets.push(p.to_owned());
        }
    }

    fn new(name: &str, profile: Option<&str>, toml_path: Option<Vec<String>>) -> Scope {
        Scope {
            name: name.to_owned(),
            files: Vec::new(),
            exclude: Vec::new(),
            files_set: GlobSet::empty(),
            exclude_set: GlobSet::empty(),
            rules: BTreeMap::new(),
            remap: None,
            profile: profile.map(str::to_owned),
            toml_path,
            presets: Vec::new(),
            rule_from_preset: BTreeMap::new(),
        }
    }
}

/// The project's setting laid over the preset's: its severity and each
/// option it names win; what it leaves out is the preset's.
fn merge_setting(base: &RuleSetting, over: &RuleSetting) -> RuleSetting {
    let severity = over.severity().or(base.severity()).map(str::to_owned);
    let mut options = base.options().cloned().unwrap_or_default();
    if let Some(o) = over.options() {
        options.extend(o.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    match severity {
        Some(s) if options.is_empty() => RuleSetting::Severity(s),
        severity => RuleSetting::Full { severity, options },
    }
}

pub(crate) fn glob_set(patterns: &[String], file: &Path) -> Result<GlobSet, ConfigError> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        let glob = GlobBuilder::new(p.trim_start_matches("./"))
            .literal_separator(true)
            .build()
            .map_err(|e| ConfigError::new(Some(file.to_owned()), format!("invalid glob `{p}`: {e}")))?;
        b.add(glob);
    }
    b.build().map_err(|e| ConfigError::new(Some(file.to_owned()), e.to_string()))
}

/// A scope name is `[a-z0-9_-]+`, and `default` is reserved.
pub fn check_scope_name(name: &str) -> Result<(), String> {
    let ok = !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if !ok {
        return Err(format!("scope name `{name}` is not valid; a scope name is lowercase letters, digits, `_` and `-` (for example `[scope.utils]`)"));
    }
    if name == "default" {
        return Err("scope name `default` is reserved; pick another name (for example `[scope.utils]`)".into());
    }
    Ok(())
}

pub(crate) fn check_name(file: &Path, name: &str) -> Result<(), ConfigError> {
    check_scope_name(name).map_err(|e| ConfigError::new(Some(file.to_owned()), e))
}

fn no_files(file: &Path, what: &str, name: &str, known: &[&str]) -> ConfigError {
    let near = did_you_mean(name, known.iter().copied()).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
    ConfigError::new(
        Some(file.to_owned()),
        format!("{what} `{name}` has no `files`, and no preset or project scope of that name defines them{near}"),
    )
}

pub(crate) struct Built {
    pub scopes: Vec<Scope>,
    pub profile_scopes: BTreeMap<String, Vec<Scope>>,
}

/// `strict`: a scope with no `files` is an error; a root config that
/// `extends` presets is not strict until the presets are in.
/// Preset scopes in order, each project scope merged into the one of its
/// name or appended, then the `[[overrides]]` alias as `override-<n>`;
/// each profile's scopes the same way, inheriting `files` from the scope
/// of the same name.
pub(crate) fn build(
    path: &Path,
    raw: &RawConfig,
    profiles: &BTreeMap<String, RawProfile>,
    alias_profiles: &BTreeSet<String>,
    presets: &[PresetScope],
    strict: bool,
) -> Result<Built, ConfigError> {
    let mut scopes: Vec<Scope> = Vec::new();
    for p in presets {
        let at = match scopes.iter().position(|s| s.name == p.name) {
            Some(i) => i,
            None => {
                scopes.push(Scope::new(&p.name, None, None));
                scopes.len() - 1
            }
        };
        scopes[at].apply(&p.scope, Some(&p.preset));
    }
    for (name, rs) in &raw.scope.0 {
        check_name(path, name)?;
        let at = match scopes.iter().position(|s| s.name == *name) {
            Some(i) => i,
            None => {
                scopes.push(Scope::new(name, None, None));
                scopes.len() - 1
            }
        };
        scopes[at].toml_path = Some(vec!["scope".into(), name.clone()]);
        scopes[at].apply(rs, None);
    }
    for (i, o) in raw.overrides.iter().enumerate() {
        let name = format!("override-{}", i + 1);
        let mut s = Scope::new(&name, None, Some(vec!["overrides".into(), i.to_string()]));
        s.apply(&RawScope { files: Some(o.files.clone()), exclude: Some(o.exclude.clone()), rules: o.rules.clone(), remap: o.remap.clone() }, None);
        scopes.push(s);
    }
    let known: Vec<String> = scopes.iter().map(|s| s.name.clone()).collect();
    let known: Vec<&str> = known.iter().map(String::as_str).collect();
    for s in &mut scopes {
        if strict && s.files.is_empty() {
            return Err(no_files(path, "scope", &s.name, &known));
        }
        s.compile(path)?;
    }

    let mut profile_scopes = BTreeMap::new();
    for (pname, p) in profiles {
        let key = if alias_profiles.contains(pname) { "profiles" } else { "profile" };
        let mut list: Vec<Scope> = Vec::new();
        for (name, rs) in &p.scope.0 {
            check_name(path, name)?;
            let mut s = Scope::new(name, Some(pname), Some(vec![key.into(), pname.clone(), "scope".into(), name.clone()]));
            if rs.files.is_none()
                && let Some(base) = scopes.iter().find(|b| b.name == *name)
            {
                s.files = base.files.clone();
                s.exclude = base.exclude.clone();
            }
            s.apply(rs, None);
            list.push(s);
        }
        for (i, o) in p.overrides.iter().enumerate() {
            let mut s = Scope::new(&format!("override-{}", i + 1), Some(pname), Some(vec![key.into(), pname.clone(), "overrides".into(), i.to_string()]));
            s.apply(&RawScope { files: Some(o.files.clone()), exclude: Some(o.exclude.clone()), rules: o.rules.clone(), remap: o.remap.clone() }, None);
            list.push(s);
        }
        for s in &mut list {
            if strict && s.files.is_empty() {
                return Err(no_files(path, &format!("profile `{pname}` scope"), &s.name, &known));
            }
            s.compile(path)?;
        }
        profile_scopes.insert(pname.clone(), list);
    }
    Ok(Built { scopes, profile_scopes })
}
