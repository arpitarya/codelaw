//! The plain-text config files a finding may sit in ([SR-LAW-3] decision
//! 10): a closed list, by file name. They are not a tenth language: nothing
//! lints, formats or parses them, no rule targets them, and discovery never
//! hands one to a rule. A native rule that read one reports its finding on
//! the L3 file it concerns and, through `NativeRule::elsewhere`, on the
//! text file's line too (SR-ARCHITECTURE decision 15d).
//!
//! Adding a name is a law change: the decider's ruling, then this list.
//!
//! [SR-LAW-3]: ../../../records/0005_LAW-3-languages.md

/// The file names, matched exactly against a path's last segment, in any
/// folder: `CODEOWNERS` is GitHub's and GitLab's wherever it lives.
pub const TEXT_CONFIG_FILES: [&str; 7] = [".nvmrc", ".node-version", ".python-version", ".tool-versions", "Dockerfile", "CODEOWNERS", "yarn.lock"];

/// Whether `path` (`/`-separated, project-relative or absolute) names one of
/// [`TEXT_CONFIG_FILES`].
pub fn is_text_config(path: &str) -> bool {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    TEXT_CONFIG_FILES.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_closed_and_by_name() {
        for ok in [".nvmrc", "web/.node-version", ".python-version", ".tool-versions", "Dockerfile", ".github/CODEOWNERS", "docs/CODEOWNERS", "yarn.lock", "a\\yarn.lock"] {
            assert!(is_text_config(ok), "{ok}");
        }
        for no in ["package.json", "Dockerfile.dev", "dockerfile", "requirements.txt", "CODEOWNERS.md", "yarn.lock.bak", ".nvmrc/x", "uv.lock"] {
            assert!(!is_text_config(no), "{no}");
        }
    }

    #[test]
    fn no_listed_file_is_an_l3_language() {
        for name in TEXT_CONFIG_FILES {
            assert_eq!(crate::Language::from_path(std::path::Path::new(name)), None, "{name}");
        }
    }
}
