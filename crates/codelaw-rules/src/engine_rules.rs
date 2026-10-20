//! The engine's own diagnostics (SR-RULE-PACKS decision 1). They are not
//! rule files, but every one still carries its fix class and steps (L4).

use codelaw_core::{FixClass, Severity};

pub struct EngineRule {
    pub id: &'static str,
    pub title: &'static str,
    pub severity: Severity,
    pub fix: FixClass,
    pub steps: &'static [&'static str],
}

pub const ENGINE_RULES: &[EngineRule] = &[
    EngineRule {
        id: "codelaw/no-rules",
        title: "No rule pack and no local rule is configured",
        severity: Severity::Info,
        fix: FixClass::Manual,
        steps: &["Run `codelaw init` to install the rule packs for the languages in this repo, or add a rule under `.codelaw/rules/`."],
    },
    EngineRule {
        id: "codelaw/not-utf8",
        title: "File is not valid UTF-8 and was skipped",
        severity: Severity::Info,
        fix: FixClass::Manual,
        steps: &["Re-encode the file as UTF-8, or exclude it in `codelaw.toml`."],
    },
    EngineRule {
        id: "codelaw/file-too-large",
        title: "File is larger than `[files] max_file_size` and was skipped",
        severity: Severity::Info,
        fix: FixClass::Manual,
        steps: &["Exclude the file, or raise `[files] max_file_size` in `codelaw.toml`."],
    },
    EngineRule {
        id: "codelaw/generated",
        title: "File is marked generated and was skipped",
        severity: Severity::Info,
        fix: FixClass::Manual,
        steps: &["Fix the generator instead, or set `[files] lint_generated = true`."],
    },
    EngineRule {
        id: "codelaw/internal-error",
        title: "A rule failed on this file",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Report the rule and file to the rule's maintainer; the rest of the run is unaffected."],
    },
    EngineRule {
        id: "codelaw/file-changed-during-run",
        title: "File changed on disk during the run, so the fix was not written",
        severity: Severity::Warn,
        fix: FixClass::Manual,
        steps: &["Run `codelaw fix` again once the file is no longer being edited."],
    },
    EngineRule {
        id: "codelaw/write-failed",
        title: "The fixed file could not be written, so it is unchanged",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Make the file writable (its permissions, a read-only disk, a full disk), then run `codelaw fix` again."],
    },
    EngineRule {
        id: "codelaw/suppression-needs-reason",
        title: "A `codelaw-ignore` comment has no reason",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Write the reason after a colon: `codelaw-ignore <rule>: <reason>`."],
    },
    EngineRule {
        id: "codelaw/unused-suppression",
        title: "A `codelaw-ignore` comment suppresses nothing",
        severity: Severity::Info,
        fix: FixClass::Auto,
        steps: &[],
    },
    EngineRule {
        id: "codelaw/unknown-suppression",
        title: "A `codelaw-ignore` comment names a rule that does not exist",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Correct the rule id, or remove the comment."],
    },
];

pub fn engine_rule(id: &str) -> Option<&'static EngineRule> {
    ENGINE_RULES.iter().find(|r| r.id == id)
}
