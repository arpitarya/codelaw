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
        id: "format/unformatted",
        title: "File is not formatted",
        severity: Severity::Warn,
        fix: FixClass::Auto,
        steps: &["Run `codelaw format` (or `codelaw fix`) on the file."],
    },
    EngineRule {
        id: "format/failed",
        title: "The formatter left the file as it was",
        severity: Severity::Info,
        fix: FixClass::Manual,
        steps: &[
            "If the file does not parse, correct the syntax error and run `codelaw format` again.",
            "Otherwise the formatted text failed verification; add the file to `[format] exclude` and report it with the file attached.",
        ],
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
        id: "graph/no-deprecated-usage",
        title: "A deprecated symbol is used",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &[
            "Replace the use with the replacement `.codelaw/deprecations.toml` names (or the deprecation marker's message gives).",
            "An existing use may stay until its deadline only through `.codelaw/baseline/graph--no-deprecated-usage.toml`, which a person adds with `codelaw baseline add`.",
        ],
    },
    EngineRule {
        id: "codelaw/stale-baseline",
        title: "A baseline entry allows more findings than its file has",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Run `codelaw baseline prune` (or `codelaw fix`, which prunes too) to lower the entry; lowering is always safe."],
    },
    EngineRule {
        id: "codelaw/expiring-exception",
        title: "An exception (a baseline entry, a deprecation, a `codelaw-ignore`) expires within its warning window",
        severity: Severity::Warn,
        fix: FixClass::Manual,
        steps: &[
            "Fix the findings the exception covers before its `expires` date: on that day they are reported again and the exception is an error.",
            "To keep it longer, a person moves `expires` later with `codelaw baseline add --expires …` (a baseline entry) or edits the comment; `codelaw baseline check` flags an extended deadline for review.",
        ],
    },
    EngineRule {
        id: "codelaw/expired-exception",
        title: "An exception reached its `expires` date and no longer covers anything",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &[
            "Fix the findings it covered; they are reported again at their rule's own severity.",
            "Then remove the exception: `codelaw baseline prune` (or `codelaw fix`) lowers a baseline entry, and the `codelaw-ignore` comment is deleted by hand.",
        ],
    },
    EngineRule {
        id: "codelaw/exception-without-expiry",
        title: "An exception has no `expires` date while `[exceptions] require_expiry` is on",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Add `expires = \"YYYY-MM-DD\"` (the first day it no longer holds): `codelaw baseline add --expires …` rewrites a baseline entry; a `codelaw-ignore` takes `expires=YYYY-MM-DD` before its colon."],
    },
    EngineRule {
        id: "codelaw/exception-author",
        title: "An exception's `added_by` is not an email address, or not the git author of the change",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Set `added_by` (`by=` in a `codelaw-ignore`) to your own git email, `git config user.email`; `codelaw baseline add` writes it for you."],
    },
    EngineRule {
        id: "codelaw/unknown-suppression",
        title: "A `codelaw-ignore` comment names a rule that does not exist",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &["Correct the rule id, or remove the comment."],
    },
    EngineRule {
        id: "graph/untested-function",
        title: "A new or edited function is reached by no test file",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &[
            "Write a unit test that imports the function (or its module, for a function the module does not export) and calls it: a test needs judgment, so codelaw does not write one.",
            "If the function should never need its own test (a thin wrapper, generated code), exempt it in `.codelawignore` with `forever: <reason>`; to defer the test, give the line `expires=YYYY-MM-DD` and `on_touch=…` and a reason.",
        ],
    },
    EngineRule {
        id: "graph/function-coverage",
        title: "A new or edited function's line coverage is under `[coverage] min`",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &[
            "Add test cases that run the lines the coverage report shows as missed, regenerate the lcov report in the same job, and run `codelaw check --coverage <report>` again: a test needs judgment, so codelaw does not write one.",
            "If the function cannot reasonably be covered, exempt it in `.codelawignore` with a reason, `expires=` or `forever`, and `on_touch=`.",
        ],
    },
    EngineRule {
        id: "codelaw/stale-coverage",
        title: "The coverage report does not match this file, so its coverage is not judged",
        severity: Severity::Warn,
        fix: FixClass::Manual,
        steps: &["Regenerate the lcov report from the code being checked — in the same CI job, after the tests and before `codelaw check` — and pass it with `--coverage` or `[coverage] report`."],
    },
    EngineRule {
        id: "codelaw/stale-ignore",
        title: "A `.codelawignore` line matches no file or function",
        severity: Severity::Warn,
        fix: FixClass::Auto,
        steps: &[],
    },
    EngineRule {
        id: "codelaw/incomplete-ignore",
        title: "A `.codelawignore` line is missing a field or contradicts itself, so it exempts nothing",
        severity: Severity::Error,
        fix: FixClass::Manual,
        steps: &[
            "Give the line what the message names: a reason after a colon (always, with `expires=` or `forever`), `expires=YYYY-MM-DD` or `forever` (`[coverage.ignore] require_expiry`), and `on_touch=error|warn|keep` unless it is `forever` (`[coverage.ignore] require_on_touch`).",
            "`forever` takes no `expires=`, `warn_days=` or `on_touch=`; a glob takes no `on_touch_scope=function`.",
        ],
    },
];

/// The opt-in preset that turns on the test rules of new code (W-71):
/// `extends = ["codelaw:tests"]`. It is the engine's, not a pack's, so it
/// has no namespace and no rule pack resolves it.
pub const TESTS_PRESET: &str = "codelaw:tests";

/// The engine rules `codelaw:tests` turns on.
pub const TEST_RULES: &[&str] = &["graph/untested-function", "graph/function-coverage"];

pub fn engine_rule(id: &str) -> Option<&'static EngineRule> {
    ENGINE_RULES.iter().find(|r| r.id == id)
}
