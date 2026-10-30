//! `deps/vulnerable` and `deps/incomplete-scan` (W-128; SR-RULE-PACKS
//! decision 15): the advisories the committed snapshot holds, against the
//! versions a project installs. Both read only files (L5 decision 3).
//!
//! - A **direct** dependency's finding is on its manifest line, and its fix
//!   moves the range to the target, keeping the operator.
//! - A **transitive** one's is on its lockfile entry where the lockfile is
//!   an L3 file (`package-lock.json`, `pnpm-lock.yaml`), naming the
//!   override to add; where it is not (`yarn.lock`, `uv.lock`,
//!   `poetry.lock`) it is on the manifest beside the lockfile, with the
//!   override as its edit (yarn `resolutions`, uv
//!   `override-dependencies`; Poetry has none).

use super::advisory::{Target, TargetRules, Ver, affects, graded, severity_rank, target};
use super::lock::{Locked, all_locked, lockfile_for, package_lock_name};
use super::npm::Range;
use super::pep440::{Pep440, normalize, requirement};
use super::rules::{MANIFESTS, Options, as_of_days, ignored, ignored_advisory, project_manifest, settings, severity};
use super::snapshot::{Advisory, Package, SNAPSHOT_PATH, Snapshot, days};
use super::{Dep, Ecosystem, folder_of, lock, manifest_deps};
use crate::data_json::{indent_of, pairs, top_object};
use crate::{Elsewhere, NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, Severity, TextEdit};
use codelaw_lang::SyntaxNode;
use serde_json::{Value, json};

use std::collections::BTreeMap;

const LANGUAGES: &[Language] = &[Language::Json, Language::Toml, Language::Yaml];

/// What a vulnerable version is, and where it can go.
#[derive(Clone, Debug)]
pub struct Assessment<'s> {
    /// The advisories that affect it and are not ignored, worst first.
    pub advisories: Vec<&'s Advisory>,
    pub target: Target,
}

impl Assessment<'_> {
    /// The worst severity among its advisories.
    pub fn severity(&self) -> &str {
        self.advisories.first().map(|a| graded(a)).unwrap_or("low")
    }
}

/// The settings a target obeys, from `[deps]` and the snapshot's day.
fn target_rules(options: &Options, fetched: i64) -> TargetRules {
    let s = settings(options);
    TargetRules { fetched, min_release_age: s.min_release_age, prerelease: s.prerelease }
}

/// The advisories affecting `version` of `key` and the version to move to;
/// `None` when nothing (not ignored) affects it.
pub fn assess<'s>(eco: Ecosystem, key: &str, pkg: &'s Package, version: &str, options: &Options, fetched: i64, as_of: &dyn Fn() -> Option<i64>) -> Option<Assessment<'s>> {
    let v = Ver::parse(eco, version)?;
    let mut hits: Vec<&Advisory> = pkg
        .advisories
        .iter()
        .filter(|a| affects(eco, a, &v))
        .filter(|a| {
            let ids: Vec<&str> = std::iter::once(a.id.as_str()).chain(a.aliases.iter().map(String::as_str)).collect();
            !ignored_advisory(options, key, &ids, as_of)
        })
        .collect();
    if hits.is_empty() {
        return None;
    }
    hits.sort_by(|a, b| severity_rank(graded(a)).cmp(&severity_rank(graded(b))).then(a.id.cmp(&b.id)));
    // The target is clean against every advisory of the package, ignored ones included.
    let all: Vec<&Advisory> = pkg.advisories.iter().collect();
    let t = target(eco, pkg, &all, &v, &target_rules(options, fetched));
    Some(Assessment { advisories: hits, target: t })
}

/// `GHSA-x (high, CVE-y) and 2 more`.
fn advisory_text(a: &Assessment<'_>) -> String {
    let first = a.advisories[0];
    let cve = first.aliases.iter().find(|x| x.starts_with("CVE-")).map(|c| format!(", {c}")).unwrap_or_default();
    let graded_note = if first.severity.is_empty() { " — no source grades it" } else { "" };
    let more = match a.advisories.len() {
        1 => String::new(),
        n => format!(" and {} more", n - 1),
    };
    format!("{} ({}{cve}{graded_note}){more}", first.id, graded(first))
}

/// ` (snapshot 2026-10-25)`, or with `; incomplete: osv unavailable`.
fn snapshot_note(snap: &Snapshot, options: &Options) -> String {
    let missing = unavailable(snap, options);
    if missing.is_empty() {
        format!(" (snapshot {})", snap.fetched)
    } else {
        format!(" (snapshot {}; incomplete: {} unavailable)", snap.fetched, missing.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "))
    }
}

/// The enabled advisory sources the snapshot says were unavailable.
fn unavailable<'s>(snap: &'s Snapshot, options: &Options) -> Vec<(&'s String, &'s super::snapshot::SourceState)> {
    let enabled = |name: &str| match options.get("advisory_sources").and_then(Value::as_array) {
        Some(list) => list.iter().any(|s| s.get("name").and_then(Value::as_str) == Some(name) && s.get("enabled").and_then(Value::as_bool) != Some(false)),
        None => true,
    };
    snap.sources.iter().filter(|(n, s)| s.unavailable() && enabled(n)).collect()
}

/// The command that brings the lockfile up to date.
fn install_command(lockfile: Option<&str>, eco: Ecosystem) -> &'static str {
    match lockfile.map(|l| l.rsplit('/').next().unwrap_or(l)) {
        Some("pnpm-lock.yaml") => "pnpm install",
        Some("yarn.lock") => "yarn install",
        Some("uv.lock") => "uv lock",
        Some("poetry.lock") => "poetry lock",
        _ if eco == Ecosystem::Pypi => "your install",
        _ => "npm install",
    }
}

/// What the message says about the target.
fn target_text(t: &Target) -> String {
    match t {
        Target::Clean { version, major: false } => format!("upgrade to {version}"),
        Target::Clean { version, major: true } => format!("upgrade to {version}, a new major version: read its changelog"),
        Target::Unverified { fixed } => format!("fixed in {fixed}, not verified clean: no advisory source covered it, so no version is offered"),
        Target::None => "no fixed version is known: replace it, remove it, or add a dated `[deps] ignore` for the advisory".into(),
    }
}

/// The edit moving a direct dependency's range to `t`, keeping its
/// operator; `None` when the range is not one that can move.
fn direct_edit(dep: &Dep, t: &str) -> Option<TextEdit> {
    match dep.ecosystem {
        Ecosystem::Npm => {
            let r = Range::parse(&dep.spec)?;
            matches!(r.op.as_str(), "^" | "~" | ">=" | "=" | "").then(|| TextEdit::new(dep.start + r.start, dep.start + r.end, t.to_owned()))
        }
        Ecosystem::Pypi => {
            let req = requirement(&dep.spec)?;
            let target = Pep440::parse(t)?;
            let i = req.clauses.iter().position(|c| matches!(c.op.as_str(), "==" | "===" | ">=" | "~=") && !c.wildcard)?;
            let mut moved = req.clauses.clone();
            moved[i].version = t.to_owned();
            if !moved.iter().all(|c| c.allows(&target)) {
                return None;
            }
            let c = &req.clauses[i];
            Some(TextEdit::new(dep.start + c.start, dep.start + c.end, t.to_owned()))
        }
    }
}

/// Insert `"name": "value"` into the top-level object `key` of a JSON
/// document, creating it after the last member when missing.
fn json_member_edit(doc: tree_sitter::Node<'_>, src: &str, key: &str, name: &str, value: &str) -> Option<TextEdit> {
    let obj = top_object(doc)?;
    let members = pairs(obj, src);
    let pair = format!("\"{name}\": \"{value}\"");
    if let Some((_, _, Some(v))) = members.iter().find(|(k, _, _)| k == key) {
        if v.kind() != "object" {
            return None;
        }
        let inner = pairs(*v, src);
        return Some(match inner.last() {
            Some((_, k, _)) => {
                let end = k.parent().map(|p| p.end_byte()).unwrap_or(k.end_byte());
                TextEdit::new(end, end, format!(",\n{}{pair}", indent_of(src, k.start_byte())))
            }
            None => TextEdit::new(v.start_byte(), v.end_byte(), format!("{{ {pair} }}")),
        });
    }
    let (_, k, _) = members.last()?;
    let end = k.parent().map(|p| p.end_byte()).unwrap_or(k.end_byte());
    let ind = indent_of(src, k.start_byte()).to_owned();
    Some(TextEdit::new(end, end, format!(",\n{ind}\"{key}\": {{\n{ind}{ind}{pair}\n{ind}}}")))
}

/// Add `requirement` to `[tool.uv] override-dependencies`: into the array
/// when there is one, as the table's first key when the table has none, or
/// as a new table at the end.
fn uv_override_edit(src: &str, requirement: &str) -> TextEdit {
    let quoted = format!("\"{requirement}\"");
    let header = |t: &str| t.starts_with('[') && t.ends_with(']') && !t.contains('=') && !t.contains('"');
    let mut offset = 0;
    let mut table: Option<usize> = None;
    for line in src.split_inclusive('\n') {
        let t = line.trim();
        if table.is_some() {
            if header(t) {
                break;
            }
            if t.starts_with("override-dependencies")
                && let Some(open) = line.find('[')
            {
                let at = offset + open + 1;
                let text = if src[at..].trim_start().starts_with(']') { quoted } else { format!("{quoted}, ") };
                return TextEdit::new(at, at, text);
            }
        } else if t == "[tool.uv]" {
            table = Some(offset + line.len());
        }
        offset += line.len();
    }
    match table {
        Some(at) => {
            let lead = if src[..at].ends_with('\n') { "" } else { "\n" };
            TextEdit::new(at, at, format!("{lead}override-dependencies = [{quoted}]\n"))
        }
        None => {
            let lead = if src.is_empty() || src.ends_with("\n\n") { "" } else if src.ends_with('\n') { "\n" } else { "\n\n" };
            TextEdit::new(src.len(), src.len(), format!("{lead}[tool.uv]\noverride-dependencies = [{quoted}]\n"))
        }
    }
}

pub struct Vulnerable;

/// Shared by every finding of one file.
struct Run<'a> {
    snap: Snapshot,
    fetched: i64,
    options: &'a Options,
    note: String,
}

impl Vulnerable {
    fn message(&self, who: &str, a: &Assessment<'_>, fix: &str, note: &str) -> String {
        format!("{who} is vulnerable — {}; {}{fix}{note}", advisory_text(a), target_text(&a.target))
    }

    /// The direct dependencies of a manifest.
    fn direct(&self, ctx: &NativeContext<'_>, run: &Run<'_>, deps: &[Dep], out: &mut Vec<NativeFinding>) {
        let dir = folder_of(ctx.file.path());
        let read = |p: &str| ctx.file.read(p);
        let as_of_cell = std::cell::OnceCell::new();
        let as_of = || *as_of_cell.get_or_init(|| as_of_days(ctx));
        for eco in [Ecosystem::Npm, Ecosystem::Pypi] {
            let mine: Vec<&Dep> = deps.iter().filter(|d| d.ecosystem == eco).collect();
            if mine.is_empty() {
                continue;
            }
            let specs: BTreeMap<String, String> = mine.iter().map(|d| (lock_name(d), d.spec.clone())).collect();
            let installed = lock::installed(eco, dir, &specs, &read);
            let lockfile = lockfile_for(eco, dir, &read).map(|(p, _)| p);
            for dep in mine {
                let Some(pkg) = run.snap.packages.get(&dep.key) else { continue };
                if ignored(run.options, &dep.key, &as_of) {
                    continue;
                }
                let Some(version) = installed.get(&lock_name(dep)).cloned().or_else(|| written_version(dep)) else { continue };
                let Some(a) = assess(eco, &dep.key, pkg, &version, run.options, run.fetched, &as_of) else { continue };
                if severity(run.options, a.severity()).is_none() {
                    continue;
                }
                let (fix, edits) = match &a.target {
                    Target::Clean { version: t, .. } => match direct_edit(dep, t) {
                        Some(e) => (format!(", then run `{}`", install_command(lockfile.as_deref(), eco)), vec![e]),
                        None => (format!(": change the requirement by hand, then run `{}`", install_command(lockfile.as_deref(), eco)), Vec::new()),
                    },
                    _ => (String::new(), Vec::new()),
                };
                out.push(NativeFinding { start: dep.start, end: dep.end, message: self.message(&format!("`{}` {version}", dep.name), &a, &fix, &run.note), edits });
            }
        }
    }

    /// The transitive packages of a lockfile: each vulnerable version it
    /// installs that no manifest names directly, with where it sits.
    fn transitive<'s>(&self, ctx: &NativeContext<'_>, run: &'s Run<'_>, eco: Ecosystem, lockfile: &str, text: &str) -> Vec<(Locked, Assessment<'s>)> {
        let as_of_cell = std::cell::OnceCell::new();
        let as_of = || *as_of_cell.get_or_init(|| as_of_days(ctx));
        let mut out = Vec::new();
        for l in all_locked(lockfile, text).into_iter().filter(|l| !l.direct) {
            let key = format!("{}:{}", eco.prefix(), l.name);
            let Some(pkg) = run.snap.packages.get(&key) else { continue };
            if ignored(run.options, &key, &as_of) {
                continue;
            }
            if let Some(a) = assess(eco, &key, pkg, &l.version, run.options, run.fetched, &as_of).filter(|a| severity(run.options, a.severity()).is_some()) {
                out.push((l, a));
            }
        }
        out
    }

    /// The override text and edit a transitive finding offers.
    fn override_fix(&self, lockfile: &str, name: &str, t: &str) -> String {
        let manifest_note = " — an override changes the install of the whole tree";
        match lockfile.rsplit('/').next().unwrap_or(lockfile) {
            "package-lock.json" => format!(": add `\"overrides\": {{ \"{name}\": \"{t}\" }}` to package.json, then run `npm install`{manifest_note}"),
            "pnpm-lock.yaml" => format!(": add `\"pnpm\": {{ \"overrides\": {{ \"{name}\": \"{t}\" }} }}` to package.json, then run `pnpm install`{manifest_note}"),
            "yarn.lock" => format!(" through `resolutions` in package.json, then run `yarn install`{manifest_note}"),
            "uv.lock" => format!(" through `[tool.uv] override-dependencies`, then run `uv lock`{manifest_note}"),
            _ => format!(": Poetry has no override, so add `{name}>={t}` to the project's dependencies, then run `poetry lock`"),
        }
    }
}

/// The name `lock::installed` keys a dependency by.
fn lock_name(dep: &Dep) -> String {
    match dep.ecosystem {
        Ecosystem::Npm => dep.name.clone(),
        Ecosystem::Pypi => normalize(&dep.name),
    }
}

/// The version a requirement starts from, when no lockfile says.
fn written_version(dep: &Dep) -> Option<String> {
    match dep.ecosystem {
        Ecosystem::Npm => Range::parse(&dep.spec).map(|r| r.base().to_string()),
        Ecosystem::Pypi => {
            let req = requirement(&dep.spec)?;
            req.clauses.iter().find(|c| matches!(c.op.as_str(), "==" | "===" | ">=" | "~=") && !c.wildcard).map(|c| c.version.clone())
        }
    }
}

/// Each `packages` entry of a `package-lock.json` with its `version`
/// string's span: `(key, version, start, end)`.
fn package_lock_spans(doc: tree_sitter::Node<'_>, src: &str) -> Vec<(String, String, usize, usize)> {
    let mut out = Vec::new();
    let Some(top) = top_object(doc) else { return out };
    for (k, _, v) in pairs(top, src) {
        if k != "packages" {
            continue;
        }
        let Some(obj) = v.filter(|v| v.kind() == "object") else { continue };
        for (key, _, entry) in pairs(obj, src) {
            let Some(entry) = entry.filter(|e| e.kind() == "object") else { continue };
            let Some(name) = package_lock_name(&key) else { continue };
            let own = pairs(entry, src);
            let name = own.iter().find(|(k, _, _)| k == "name").and_then(|(_, _, v)| *v).map(|v| crate::data_json::string_value(v, src)).unwrap_or_else(|| name.to_owned());
            if let Some((_, _, Some(ver))) = own.iter().find(|(k, _, _)| k == "version") {
                out.push((name, crate::data_json::string_value(*ver, src), ver.start_byte(), ver.end_byte()));
            }
        }
    }
    out
}

/// Each `packages:` key of a `pnpm-lock.yaml`, with its span.
fn pnpm_spans(src: &str) -> Vec<(String, String, usize, usize)> {
    let mut out = Vec::new();
    let mut offset = 0;
    let mut inside = false;
    for line in src.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        if !body.starts_with(' ') && !body.is_empty() {
            inside = body == "packages:";
        } else if inside && body.starts_with("  ") && !body[2..].starts_with(' ') && body.ends_with(':') {
            let raw = &body[2..body.len() - 1];
            let key = raw.trim_matches(['\'', '"']);
            if let Some((name, version)) = super::lock::pnpm_key(key) {
                out.push((name, version, offset + 2, offset + 2 + raw.len()));
            }
        }
        offset += line.len();
    }
    out
}

impl NativeRule for Vulnerable {
    fn id(&self) -> &'static str {
        "deps/vulnerable"
    }
    fn languages(&self) -> &'static [Language] {
        LANGUAGES
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("deps")
    }
    fn severity_of(&self, finding: &NativeFinding, options: &Options) -> Option<Severity> {
        let rest = finding.message.split_once(" is vulnerable — ")?.1;
        let word = rest.split_once(" (")?.1.split([',', ')', ' ']).next()?;
        severity(options, word)
    }
    fn data_of(&self, finding: &NativeFinding, _options: &Options) -> Option<Value> {
        // The structured facts ride in the message's own words; the JSON
        // report's `data` names the package, version and target.
        let (who, rest) = finding.message.split_once(" is vulnerable — ")?;
        let mut words = who.split_whitespace();
        let name = words.next()?.trim_matches('`');
        let version = words.next()?;
        let id = rest.split_once(" (")?.0;
        let target = rest.split_once("; upgrade to ").map(|(_, t)| t.split([',', ' ']).next().unwrap_or(t).to_owned());
        Some(json!({ "package": name, "version": version, "advisory": id, "target": target, "direct": !who.contains("installed through") }))
    }
    /// A transitive finding from yarn classic's `yarn.lock` (a plain-text
    /// file) has a copy on the lockfile's entry (W-136; SR-LAW-3 decision
    /// 10); the finding and its `resolutions` edit stay on `package.json`.
    fn elsewhere(&self, finding: &NativeFinding, ctx: &NativeContext<'_>) -> Vec<Elsewhere> {
        let Some((who, _)) = finding.message.split_once(" (installed through another package, in yarn.lock) is vulnerable") else { return Vec::new() };
        let mut words = who.split_whitespace();
        let (Some(name), Some(version)) = (words.next().map(|n| n.trim_matches('`')), words.next()) else { return Vec::new() };
        let dir = folder_of(ctx.file.path());
        let path = if dir.is_empty() { "yarn.lock".to_owned() } else { format!("{dir}/yarn.lock") };
        let Some(text) = ctx.file.read(&path) else { return Vec::new() };
        yarn_entry_line(&text, name, version).and_then(|n| Elsewhere::line(path, &text, n)).into_iter().collect()
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let path = ctx.file.path();
        let file = path.rsplit('/').next().unwrap_or(path);
        let lockfile_here = matches!((ctx.language, file), (Language::Json, "package-lock.json") | (Language::Yaml, "pnpm-lock.yaml"));
        let manifest = matches!((ctx.language, file), (Language::Json, "package.json") | (Language::Toml, "pyproject.toml"));
        if !lockfile_here && !manifest {
            return;
        }
        let Some(snap) = ctx.file.read(SNAPSHOT_PATH).and_then(|t| Snapshot::parse(&t)) else { return };
        let Some(fetched) = days(&snap.fetched) else { return };
        if !snap.packages.values().any(|p| !p.advisories.is_empty()) {
            return;
        }
        let note = snapshot_note(&snap, ctx.options);
        let run = Run { snap, fetched, options: ctx.options, note };
        let doc = node.get_inner_node();
        if lockfile_here {
            let (eco, spans) = if file == "package-lock.json" { (Ecosystem::Npm, package_lock_spans(doc, ctx.source)) } else { (Ecosystem::Npm, pnpm_spans(ctx.source)) };
            for (l, a) in self.transitive(ctx, &run, eco, file, ctx.source) {
                let fix = match &a.target {
                    Target::Clean { version, .. } => self.override_fix(file, &l.name, version),
                    _ => String::new(),
                };
                let who = format!("`{}` {} (installed through another package)", l.name, l.version);
                let message = self.message(&who, &a, &fix, &run.note);
                for (_, _, start, end) in spans.iter().filter(|(n, v, _, _)| *n == l.name && *v == l.version) {
                    out.push(NativeFinding { start: *start, end: *end, message: message.clone(), edits: Vec::new() });
                }
            }
            return;
        }
        let deps = manifest_deps(ctx.language, path, doc, ctx.source);
        self.direct(ctx, &run, &deps, out);
        // A lockfile that is not an L3 file, in this manifest's own folder:
        // its transitive findings sit here.
        let dir = folder_of(path);
        let eco = if file == "package.json" { Ecosystem::Npm } else { Ecosystem::Pypi };
        let read = |p: &str| ctx.file.read(p);
        let Some((lockfile, text)) = lockfile_for(eco, dir, &read) else { return };
        let lock_file = lockfile.rsplit('/').next().unwrap_or(&lockfile);
        if folder_of(&lockfile) != dir || matches!(lock_file, "package-lock.json" | "pnpm-lock.yaml") {
            return;
        }
        let line_end = ctx.source.find('\n').unwrap_or(ctx.source.len());
        for (l, a) in self.transitive(ctx, &run, eco, lock_file, &text) {
            let (fix, edits) = match &a.target {
                Target::Clean { version, .. } => {
                    let edit = match lock_file {
                        "yarn.lock" => json_member_edit(doc, ctx.source, "resolutions", &l.name, version),
                        "uv.lock" => Some(uv_override_edit(ctx.source, &format!("{}=={version}", l.name))),
                        _ => None,
                    };
                    (self.override_fix(lock_file, &l.name, version), edit.into_iter().collect())
                }
                _ => (String::new(), Vec::new()),
            };
            let who = format!("`{}` {} (installed through another package, in {lock_file})", l.name, l.version);
            out.push(NativeFinding { start: 0, end: line_end, message: self.message(&who, &a, &fix, &run.note), edits });
        }
    }
}

/// The 1-based line of the `yarn.lock` entry that installs `name` at
/// `version`: its descriptor line.
fn yarn_entry_line(text: &str, name: &str, version: &str) -> Option<usize> {
    let mut header: Option<(usize, bool)> = None;
    for (i, line) in text.lines().enumerate() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            let first = line.trim_end_matches(':').split(", ").next().unwrap_or("").trim().trim_matches('"');
            let named = first.strip_prefix(name).is_some_and(|r| r.starts_with('@'));
            header = Some((i + 1, named));
            continue;
        }
        let t = line.trim();
        if let Some(v) = t.strip_prefix("version:").or_else(|| t.strip_prefix("version "))
            && let Some((n, true)) = header
            && v.trim().trim_matches('"') == version
        {
            return Some(n);
        }
    }
    None
}

pub struct IncompleteScan;

impl NativeRule for IncompleteScan {
    fn id(&self) -> &'static str {
        "deps/incomplete-scan"
    }
    fn languages(&self) -> &'static [Language] {
        MANIFESTS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("deps")
    }
    fn check(&self, _node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        if !project_manifest(ctx) {
            return;
        }
        let Some(snap) = ctx.file.read(SNAPSHOT_PATH).and_then(|t| Snapshot::parse(&t)) else { return };
        let end = ctx.source.find('\n').unwrap_or(ctx.source.len());
        if snap.sources.is_empty() {
            if !snap.packages.is_empty() {
                out.push(NativeFinding {
                    start: 0,
                    end,
                    message: format!("The dependency snapshot ({}) holds no advisories: no advisory source was read, so no vulnerable dependency can be reported; run `codelaw deps refresh`", snap.fetched),
                    edits: Vec::new(),
                });
            }
            return;
        }
        let missing = unavailable(&snap, ctx.options);
        if missing.is_empty() {
            return;
        }
        let names: Vec<String> = missing
            .iter()
            .map(|(n, s)| match &s.error {
                Some(e) => format!("`{n}` ({e}, on {})", s.on),
                None => format!("`{n}` (on {})", s.on),
            })
            .collect();
        out.push(NativeFinding {
            start: 0,
            end,
            message: format!(
                "The vulnerability scan is incomplete: the advisory source{} {} could not be read, so what only {} knows is not reported; run `codelaw deps refresh` once it is reachable, or set `enabled = false` on it in `[deps] advisory_sources`",
                if names.len() == 1 { "" } else { "s" },
                names.join(", "),
                if names.len() == 1 { "it" } else { "they" }
            ),
            edits: Vec::new(),
        });
    }
}

/// `codelaw deps check` and MCP `check_dependency` (SR-CLI-MCP decision
/// 5): what the snapshot says of one package — offline.
pub fn check_dependency(snap: Option<&Snapshot>, options: &Options, eco: Ecosystem, name: &str, version: Option<&str>, as_of: Option<i64>) -> Value {
    let key = match eco {
        Ecosystem::Npm => format!("npm:{name}"),
        Ecosystem::Pypi => format!("pypi:{}", normalize(name)),
    };
    let Some(snap) = snap else {
        return json!({ "ecosystem": eco.prefix(), "package": name, "known": false, "snapshot": Value::Null,
            "next": format!("no `{SNAPSHOT_PATH}`: run `codelaw deps refresh` first") });
    };
    let complete = unavailable(snap, options).is_empty() && !snap.sources.is_empty();
    let unavailable_sources: Vec<&String> = unavailable(snap, options).into_iter().map(|(n, _)| n).collect();
    let Some(pkg) = snap.packages.get(&key) else {
        return json!({ "ecosystem": eco.prefix(), "package": name, "known": false, "snapshot": snap.fetched, "complete": complete,
            "unavailable_sources": unavailable_sources,
            "next": "the snapshot does not hold this package: add it to a manifest and run `codelaw deps refresh`" });
    };
    let fetched = days(&snap.fetched).unwrap_or(0);
    let version = version.map(str::to_owned).or_else(|| pkg.latest.clone());
    let as_of_f = || as_of;
    let a = version.as_deref().and_then(|v| assess(eco, &key, pkg, v, options, fetched, &as_of_f));
    let (vulnerable, advisories, target, major, unverified) = match &a {
        Some(a) => (
            true,
            a.advisories.iter().map(|x| json!({ "id": x.id, "aliases": x.aliases, "severity": graded(x), "url": x.url, "fixed": x.fixed, "sources": x.sources })).collect::<Vec<_>>(),
            match &a.target { Target::Clean { version, .. } => json!(version), _ => Value::Null },
            matches!(a.target, Target::Clean { major: true, .. }),
            match &a.target { Target::Unverified { fixed } => json!(fixed), _ => Value::Null },
        ),
        None => (false, Vec::new(), Value::Null, false, Value::Null),
    };
    let next = match (&a, version.as_deref()) {
        (Some(a), Some(v)) => format!("{name} {v} is vulnerable ({}, {}): {}", a.advisories[0].id, a.severity(), target_text(&a.target)),
        (None, Some(v)) => format!("{name} {v} has no known advisory in the snapshot{}", if complete { "" } else { " (the scan was incomplete)" }),
        (None, None) => format!("no version given and the snapshot has no latest for {name}"),
        (Some(_), None) => String::new(),
    };
    json!({
        "ecosystem": eco.prefix(),
        "package": name,
        "known": true,
        "version": version,
        "vulnerable": vulnerable,
        "advisories": advisories,
        "target": target,
        "major": major,
        "unverified_fix": unverified,
        "latest": pkg.latest,
        "licence": version.as_deref().and_then(|v| pkg.licence.get(v)),
        "snapshot": snap.fetched,
        "complete": complete,
        "unavailable_sources": unavailable_sources,
        "next": next,
    })
}

#[cfg(test)]
#[path = "vulnerable_tests.rs"]
mod tests;
