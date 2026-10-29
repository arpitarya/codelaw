//! The skill catalogue. The texts are not in the binary: they ship in the
//! skills package (`@codelaw/skills`, `codelaw-skills`; W-77), whose files
//! are `templates/agents/` of the same release (SR-LAW-10), and are read
//! from it through [`AgentFiles`].

use crate::package::AgentFiles;

/// One skill: its folder name and its `SKILL.md`.
pub struct Skill<'a> {
    pub name: &'static str,
    pub text: &'a str,
}

/// The catalogue, P1 first (SR-AGENT-SURFACES §The skill catalogue).
pub const NAMES: [&str; 14] = [
    "codelaw-write",
    "codelaw-fix",
    "codelaw-explain",
    "codelaw-author-rule",
    "codelaw-setup",
    "codelaw-migrate",
    "codelaw-debug-rule",
    "codelaw-ci-triage",
    "codelaw-review",
    "codelaw-suppress",
    "codelaw-pack",
    "codelaw-upgrade",
    "codelaw-rules-from-reviews",
    "codelaw-port-rule",
];

/// Every skill of the catalogue, from the package.
pub fn skills(files: &AgentFiles) -> Result<Vec<Skill<'_>>, String> {
    NAMES.iter().map(|name| Ok(Skill { name, text: files.text(&format!("skills/{name}/SKILL.md"))? })).collect()
}

/// The custom agents (Claude Code and Copilot), each a context-isolated
/// twin of a skill: `codelaw-rule-author` of `codelaw-author-rule`,
/// `codelaw-rule-porter` of `codelaw-port-rule` (W-140).
pub const CUSTOM_AGENTS: [&str; 2] = ["codelaw-rule-author", "codelaw-rule-porter"];

/// One custom agent's text, from the package.
pub fn custom_agent<'a>(files: &'a AgentFiles, name: &str) -> Result<&'a str, String> {
    files.text(&format!("agents/{name}.md"))
}

/// The trigger evals of every skill (decision 5).
pub fn trigger_evals(files: &AgentFiles) -> Result<&str, String> {
    files.text("evals/triggers.json")
}

/// Split `---` frontmatter from the body.
pub fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some((&rest[..end], &rest[end + 5..]))
}

/// A frontmatter field on one line (`key: value`).
pub fn field<'a>(frontmatter: &'a str, key: &str) -> Option<&'a str> {
    frontmatter.lines().find_map(|l| l.strip_prefix(key).and_then(|r| r.strip_prefix(':')).map(str::trim))
}

/// A source with the "generated" note after its frontmatter, so a reader
/// of the copy knows where to edit (decision 4).
pub fn with_generated_note(files: &AgentFiles, text: &str) -> Result<String, String> {
    let note = files.text("note-skill.md")?;
    match split(text) {
        Some((fm, body)) => Ok(format!("---\n{fm}\n---\n{note}{body}")),
        None => Ok(format!("{note}{text}")),
    }
}

/// The repo's own `templates/agents/`, for the tests: what the package holds.
#[cfg(test)]
pub(crate) fn source() -> AgentFiles {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/agents");
    AgentFiles::read_tree(&dir, env!("CARGO_PKG_VERSION").to_owned()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_skill_follows_the_agent_skills_format() {
        let files = source();
        for s in skills(&files).unwrap() {
            let (fm, body) = split(s.text).unwrap_or_else(|| panic!("{}: no frontmatter", s.name));
            assert_eq!(field(fm, "name"), Some(s.name), "{}: `name` must equal the folder", s.name);
            assert!(s.name.len() <= 64 && s.name.chars().all(|c| c.is_ascii_lowercase() || c == '-'), "{}", s.name);
            let description = field(fm, "description").unwrap_or_default();
            assert!(!description.is_empty() && description.len() <= 1024, "{}: description is {} chars", s.name, description.len());
            assert!(description.contains("Use "), "{}: the description says when to use it", s.name);
            // Under 5,000 tokens: about four characters a token.
            assert!(body.len() < 20_000, "{}: body is {} bytes", s.name, body.len());
            // Decision 4: every skill states the safety rules that apply to it.
            assert!(body.contains("## Never") || body.contains("## Refuse"), "{}: no safety section", s.name);
        }
    }

    #[test]
    fn the_catalogue_is_what_fix_skill_may_name() {
        // L4 decision 10: a `fix_skill:` resolves at load only to a skill
        // codelaw ships; both lists must be the same.
        let mut shipped: Vec<&str> = NAMES.to_vec();
        shipped.sort();
        let mut known = codelaw_rules::KNOWN_SKILLS.to_vec();
        known.sort();
        assert_eq!(shipped, known);
    }

    #[test]
    fn every_skill_has_trigger_evals() {
        let files = source();
        let v: serde_json::Value = serde_json::from_str(trigger_evals(&files).unwrap()).unwrap();
        for s in skills(&files).unwrap() {
            let e = &v["skills"][s.name];
            assert!(e["should"].as_array().is_some_and(|a| a.len() >= 3), "{}: needs prompts that fire it", s.name);
            assert!(e["should_not"].as_array().is_some_and(|a| a.len() >= 3), "{}: needs near-misses", s.name);
        }
        assert_eq!(v["skills"].as_object().unwrap().len(), NAMES.len());
    }

    #[test]
    fn every_custom_agent_names_itself() {
        let files = source();
        for name in CUSTOM_AGENTS {
            let (fm, _) = split(custom_agent(&files, name).unwrap()).unwrap_or_else(|| panic!("{name}: no frontmatter"));
            assert_eq!(field(fm, "name"), Some(name));
            assert!(field(fm, "description").is_some_and(|d| d.contains("Use ")), "{name}: the description says when to use it");
        }
    }

    #[test]
    fn the_generated_note_keeps_frontmatter_first() {
        let files = source();
        let out = with_generated_note(&files, skills(&files).unwrap()[0].text).unwrap();
        assert!(out.starts_with("---\nname: codelaw-write\n"));
        assert!(out.contains("---\n<!-- Generated by `codelaw sync-agents`"));
    }
}
