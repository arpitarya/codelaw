//! Consumer-file templates (SR-LAW-10, L10): every file codelaw writes into a
//! consumer's repository is rendered from one template in `templates/` at
//! this repo's root. Code computes values and fills the template's named
//! slots; it holds no consumer text. Every template is embedded here at build
//! time **except `templates/agents/`**, which ships as the skills package
//! (`@codelaw/skills`, `codelaw-skills`; W-77, SR-LAW-10 decision 6) and is
//! read from it by `codelaw-agents`, through [`render_text`].
//!
//! A template is text and `{{name}}` slots, nothing else: no conditions, no
//! loops (an optional section is its own template, rendered into a slot).
//! `\{{` writes a literal `{{`. A slot the template names but the caller did
//! not fill, and a value for a slot the template lacks, are both errors.
//! Values are inserted as they are, never scanned again.

/// One embedded template: its path under `templates/`, and its text.
#[derive(Clone, Copy, Debug)]
pub struct Template {
    pub path: &'static str,
    pub text: &'static str,
}

macro_rules! t {
    ($path:literal) => {
        Template { path: $path, text: include_str!(concat!("../../../templates/", $path)) }
    };
}

/// Every embedded template, in path order: all of `templates/` but
/// `agents/`, the skills package's. `templates/README.md` is the registry of
/// what each is for; the test in this module keeps the three in step.
pub const TEMPLATES: &[Template] = &[
    t!("ci/bitbucket/bitbucket-pipelines.yml"),
    t!("ci/github/autofix-suggest-job.yml"),
    t!("ci/github/codelaw-autofix.yml"),
    t!("ci/github/codelaw.yml"),
    t!("ci/gitlab/autofix-suggest-job.yml"),
    t!("ci/gitlab/codelaw-autofix.yml"),
    t!("git-hooks/post-commit"),
    t!("git-hooks/pre-commit"),
    t!("init/.codelaw/rules/README.md"),
    t!("init/codelaw-agents.toml"),
    t!("init/codelaw.toml"),
    t!("init/gitignore"),
    t!("migrate/codelaw.toml"),
    t!("migrate/lint.toml"),
    t!("migrate/preview.toml"),
    t!("new-rule/example.md"),
    t!("new-rule/fix-line.md"),
    t!("new-rule/fix-reason.md"),
    t!("new-rule/how-to-fix.md"),
    t!("new-rule/matcher-file.md"),
    t!("new-rule/matcher-project.md"),
    t!("new-rule/rule.md"),
    t!("new-rule/scope-project.md"),
];

/// The template at `path` (relative to `templates/`), or `None`.
pub fn find(path: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.path == path)
}

/// The text of a template that is written as it stands. Panics on a path
/// that is not registered: a typo in code, caught by every test that runs it.
pub fn text(path: &str) -> &'static str {
    find(path).unwrap_or_else(|| panic!("no template `{path}` in templates/")).text
}

/// Render the template at `path`, filling `slots`.
pub fn render(path: &str, slots: &[(&str, &str)]) -> Result<String, String> {
    let template = find(path).ok_or_else(|| format!("no template `{path}` in templates/"))?;
    render_text(template.text, slots).map_err(|e| format!("templates/{path}: {e}"))
}

/// Render `text`, filling `slots`.
pub fn render_text(text: &str, slots: &[(&str, &str)]) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut used = vec![false; slots.len()];
    let mut rest = text;
    while let Some(at) = rest.find("{{") {
        if rest[..at].ends_with('\\') {
            out.push_str(&rest[..at - 1]);
            out.push_str("{{");
            rest = &rest[at + 2..];
            continue;
        }
        out.push_str(&rest[..at]);
        let after = &rest[at + 2..];
        let end = after.find("}}").ok_or("`{{` is never closed (write `\\{{` for a literal one)")?;
        let name = &after[..end];
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(format!("`{{{{{name}}}}}` is not a slot name (lowercase, digits, `_`; write `\\{{{{` for a literal one)"));
        }
        let Some(i) = slots.iter().position(|(n, _)| *n == name) else {
            return Err(format!("slot `{name}` is not filled"));
        };
        used[i] = true;
        out.push_str(slots[i].1);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    if let Some(i) = used.iter().position(|u| !u) {
        return Err(format!("a value was given for slot `{}`, which the template does not have", slots[i].0));
    }
    Ok(out)
}

/// The slot names a template has, in order of first use.
pub fn slot_names(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("{{") {
        let after = &rest[at + 2..];
        let end = after.find("}}").unwrap_or(0);
        if !rest[..at].ends_with('\\') && !out.iter().any(|n| n == &after[..end]) {
            out.push(after[..end].to_owned());
        }
        rest = &after[end..];
    }
    out
}

/// A value as the inside of a JSON string, for a slot between quotes in a
/// JSON template.
pub fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The slots each template that has any is rendered with in the test
    /// below; a template absent from here has none.
    const FIXTURES: &[(&str, &[(&str, &str)])] = &[
        ("agents/always-on.md", &[("cmd", "codelaw"), ("presets", "none")]),
        ("agents/block.md", &[("note", "n"), ("body", "b")]),
        ("agents/claude/hook-group.json", &[("cmd", "codelaw")]),
        ("agents/claude/pre-hook-group.json", &[("cmd", "codelaw")]),
        ("agents/codex/config.toml", &[("command", "npx"), ("args", "\"mcp\""), ("cmd", "codelaw")]),
        ("agents/copilot/hook.json", &[("cmd", "codelaw")]),
        ("agents/copilot/instructions.md", &[("glob", "**/*.py"), ("note", "n"), ("body", "b")]),
        ("agents/copilot/mcp-server.json", &[("command", "npx"), ("args", "[]")]),
        ("agents/cowork/hooks.json", &[("pre", "{}"), ("group", "{}")]),
        ("agents/cowork/mcp.json", &[("server", "{}")]),
        ("agents/cowork/plugin.json", &[("version", "0.0.0")]),
        ("agents/kiro/hook.json", &[("extensions", "js|py"), ("cmd", "codelaw")]),
        ("agents/kiro/steering-language.md", &[("glob", "**/*.py"), ("note", "n"), ("body", "b")]),
        ("agents/kiro/steering.md", &[("note", "n"), ("body", "b")]),
        ("agents/mcp-server.json", &[("command", "npx"), ("args", "[]")]),
        ("agents/path-scoped-rule.md", &[("id", "a/b"), ("title", "t"), ("fix", "auto")]),
        ("agents/path-scoped.md", &[("language", "Python"), ("cmd", "codelaw"), ("rules", "")]),
        ("ci/bitbucket/bitbucket-pipelines.yml", &[("version", "0.0.0")]),
        ("ci/github/autofix-suggest-job.yml", &[("version", "0.0.0")]),
        ("ci/github/codelaw-autofix.yml", &[("cron", "17 4 * * *"), ("version", "0.0.0"), ("suggest_job", "")]),
        ("ci/github/codelaw.yml", &[("version", "0.0.0")]),
        ("ci/gitlab/autofix-suggest-job.yml", &[("version", "0.0.0")]),
        ("ci/gitlab/codelaw-autofix.yml", &[("cron", "17 4 * * *"), ("version", "0.0.0"), ("suggest_job", "")]),
        ("init/codelaw-agents.toml", &[("targets", "\"claude\"")]),
        ("init/codelaw.toml", &[("extends", ""), ("agents", "")]),
        ("migrate/codelaw.toml", &[("tool", "eslint"), ("sources", ".eslintrc.json"), ("extends", ""), ("preview", ""), ("lint", ""), ("tables", "")]),
        ("migrate/lint.toml", &[("tool", "eslint"), ("aliases", "\"eslint\"")]),
        ("new-rule/example.md", &[("language", "Python"), ("ext", "py")]),
        ("new-rule/rule.md", &[("id", "a/b"), ("languages", "python"), ("scope", ""), ("fix", "auto"), ("fix_reason", ""), ("since", "0.0.0"), ("matcher", ""), ("fix_line", ""), ("how_to_fix", ""), ("examples", "")]),
    ];

    #[test]
    fn slots_fill_and_escape() {
        assert_eq!(render_text("a {{x}} b {{x}} \\{{y}}", &[("x", "1")]).unwrap(), "a 1 b 1 {{y}}");
        assert_eq!(render_text("{{x}}", &[("x", "{{x}}")]).unwrap(), "{{x}}", "values are not scanned again");
        assert!(render_text("{{x}}", &[]).unwrap_err().contains("not filled"));
        assert!(render_text("plain", &[("x", "1")]).unwrap_err().contains("does not have"));
        assert!(render_text("{{ x }}", &[]).unwrap_err().contains("not a slot name"));
        assert!(render_text("{{x", &[]).unwrap_err().contains("never closed"));
        assert_eq!(json_escape("a\"b\\c"), "a\\\"b\\\\c");
    }

    fn files(dir: &Path, root: &Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                files(&p, root, out);
            } else if e.file_name() != ".DS_Store" {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }

    /// L10 decision 7: the folder, the embedded list and the registry
    /// (`templates/README.md`) name the same files, and each renders. The
    /// folder is the embedded list plus `agents/`, which is the skills
    /// package's and is never embedded (W-77).
    #[test]
    fn the_registry_matches_the_folder() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates");
        let mut on_disk = Vec::new();
        files(&root, &root, &mut on_disk);
        on_disk.retain(|p| p != "README.md");
        on_disk.sort();
        let embedded: Vec<&str> = TEMPLATES.iter().map(|t| t.path).collect();
        assert!(embedded.iter().all(|p| !p.starts_with("agents/")), "templates/agents/ ships in the skills package, never in the binary (W-77)");
        let not_agents: Vec<&str> = on_disk.iter().map(String::as_str).filter(|p| !p.starts_with("agents/")).collect();
        assert_eq!(not_agents, embedded, "every file in templates/ outside agents/ is in TEMPLATES (crates/codelaw-core/src/template.rs) and the reverse");

        let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
        let mut rows: Vec<String> = readme
            .lines()
            .filter_map(|l| l.strip_prefix("| `"))
            .filter_map(|l| l.split_once("` |").map(|(p, _)| p.to_owned()))
            .collect();
        rows.sort();
        assert_eq!(rows, on_disk, "every template has one registry row in templates/README.md and the reverse");

        for path in &on_disk {
            let text = std::fs::read_to_string(root.join(path)).unwrap();
            if let Some(t) = find(path) {
                assert_eq!(t.text, text, "{path}: the embedded copy is the file");
            }
            let fixture = FIXTURES.iter().find(|(p, _)| p == path).map_or(&[][..], |(_, s)| *s);
            let mut names = slot_names(&text);
            names.sort();
            let mut given: Vec<&str> = fixture.iter().map(|(n, _)| *n).collect();
            given.sort();
            assert_eq!(names, given, "{path}: the fixture fills exactly the template's slots");
            let out = render_text(&text, fixture).unwrap_or_else(|e| panic!("{path}: {e}"));
            if fixture.is_empty() {
                assert_eq!(out, text, "{path}: a template without slots renders as it stands");
            }
        }
    }
}
