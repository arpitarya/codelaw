//! The skills package (W-77; SR-AGENT-SURFACES decision 8): every agent file
//! codelaw writes — skills, steering, instructions, the custom agent, hook and
//! MCP registration, trigger evals — ships as its own package,
//! `@codelaw/skills` on npm and `codelaw-skills` on PyPI, released with the
//! engine at the same version. The binary embeds none of it; `sync-agents` and
//! `init --agents` read the installed package from disk.
//!
//! Finding it is a filesystem lookup only (L5: no network), in this order:
//! `CODELAW_SKILLS`; `node_modules/@codelaw/skills` from the project root
//! upwards; `codelaw_skills/` in the site-packages the rule packs are found in
//! (`CODELAW_SITE_PACKAGES`, else `$VIRTUAL_ENV`, else `.venv/`); then the
//! environment the binary itself is installed in (a `pipx` or `uvx` codelaw).
//! A package whose version is not this binary's is never used.

use codelaw_core::template;
use codelaw_rules::PackSearch;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The manifest at the package's root, the same bytes on both registries.
pub const MANIFEST: &str = "codelaw-skills.json";
pub const NPM_NAME: &str = "@codelaw/skills";
pub const PYPI_NAME: &str = "codelaw-skills";
pub const PY_MODULE: &str = "codelaw_skills";
/// A folder holding the package, ahead of every other place.
pub const ENV: &str = "CODELAW_SKILLS";

/// The files a package carries beside the agent files, never read as one.
const NOT_AGENT_FILES: &[&str] = &[MANIFEST, "LICENSE", "README.md", "package.json"];

/// The agent files of one installed skills package: `templates/agents/` of
/// the release it came from, byte for byte, keyed by path inside it.
#[derive(Clone, Debug)]
pub struct AgentFiles {
    pub root: PathBuf,
    pub version: String,
    files: BTreeMap<String, String>,
}

impl AgentFiles {
    /// Read a package folder: its manifest's version and every agent file.
    pub fn load(dir: &Path) -> Result<AgentFiles, String> {
        let manifest = dir.join(MANIFEST);
        let text = std::fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: not valid JSON ({e})", manifest.display()))?;
        let version = v["version"].as_str().ok_or_else(|| format!("{}: no `version`", manifest.display()))?.to_owned();
        AgentFiles::read_tree(dir, version)
    }

    /// Every file under `dir` but the package's own (manifest, licence,
    /// readme, npm manifest) and hidden or cache folders.
    pub fn read_tree(dir: &Path, version: String) -> Result<AgentFiles, String> {
        fn walk(dir: &Path, rel: &str, out: &mut BTreeMap<String, String>) -> Result<(), String> {
            let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') || name == "__pycache__" {
                    continue;
                }
                let path = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
                if e.path().is_dir() {
                    walk(&e.path(), &path, out)?;
                } else if !(rel.is_empty() && NOT_AGENT_FILES.contains(&name.as_str())) {
                    let text = std::fs::read_to_string(e.path()).map_err(|err| format!("{}: {err}", e.path().display()))?;
                    out.insert(path, text);
                }
            }
            Ok(())
        }
        let mut files = BTreeMap::new();
        walk(dir, "", &mut files)?;
        Ok(AgentFiles { root: dir.to_owned(), version, files })
    }

    /// The file at `path` inside the package (`skills/codelaw-fix/SKILL.md`).
    pub fn text(&self, path: &str) -> Result<&str, String> {
        self.files.get(path).map(String::as_str).ok_or_else(|| {
            format!("the skills package at {} has no `{path}`; reinstall {NPM_NAME} or {PYPI_NAME} at {}", self.root.display(), env!("CARGO_PKG_VERSION"))
        })
    }

    /// Render the template at `path`, filling `slots` (SR-LAW-10 decision 4).
    pub fn render(&self, path: &str, slots: &[(&str, &str)]) -> Result<String, String> {
        template::render_text(self.text(path)?, slots).map_err(|e| format!("{}/{path}: {e}", self.root.display()))
    }

    /// Every path the package holds, in order.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(String::as_str)
    }
}

/// Cargo's `0.2.0-beta.1` as PyPI writes it, `0.2.0b1`.
pub fn pep440(version: &str) -> String {
    let Some((core, pre)) = version.split_once('-') else { return version.to_owned() };
    let Some((kind, n)) = pre.split_once('.') else { return version.to_owned() };
    let tag = match kind {
        "alpha" => "a",
        "beta" => "b",
        "rc" => "rc",
        _ => return version.to_owned(),
    };
    format!("{core}{tag}{n}")
}

/// The two install lines, at this binary's version.
pub fn install_lines() -> String {
    let v = env!("CARGO_PKG_VERSION");
    format!("  npm i -D {NPM_NAME}@{v}\n  pip install {PYPI_NAME}=={}", pep440(v))
}

/// Where the package may be, in the order it is looked for.
pub fn candidates(project_root: &Path, var: &dyn Fn(&str) -> Option<String>, exe: Option<&Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(dir) = var(ENV).filter(|d| !d.is_empty()) {
        out.push(PathBuf::from(dir));
    }
    for d in project_root.ancestors() {
        out.push(d.join("node_modules").join("@codelaw").join("skills"));
    }
    let site = PackSearch::new(project_root, project_root, var("CODELAW_SITE_PACKAGES").as_deref(), var("VIRTUAL_ENV").as_deref());
    out.extend(site.site_packages.iter().map(|sp| sp.join(PY_MODULE)));
    // The binary's own environment: `<prefix>/bin/codelaw` (`Scripts\` on
    // Windows) beside `<prefix>/lib/python3.*/site-packages`, and an npm
    // install's `node_modules/@codelaw/cli-<key>/codelaw`.
    if let Some(exe) = exe {
        if let Some(prefix) = exe.parent().and_then(Path::parent) {
            let own = PackSearch::new(prefix, prefix, None, prefix.to_str());
            out.extend(own.site_packages.iter().map(|sp| sp.join(PY_MODULE)));
        }
        for d in exe.ancestors().skip(1) {
            out.push(d.join("node_modules").join("@codelaw").join("skills"));
        }
    }
    let mut seen = Vec::new();
    out.retain(|p| {
        let new = !seen.contains(p);
        seen.push(p.clone());
        new
    });
    out
}

/// The installed skills package at this binary's version, or the error
/// that says where it looked and how to install it (exit 2 at the caller).
pub fn locate(project_root: &Path) -> Result<AgentFiles, String> {
    let exe = std::env::current_exe().ok().and_then(|e| std::fs::canonicalize(e).ok());
    let var = |k: &str| std::env::var(k).ok();
    find(project_root, &var, exe.as_deref(), env!("CARGO_PKG_VERSION"))
}

/// `locate`, with the environment, the binary and the version given.
pub fn find(project_root: &Path, var: &dyn Fn(&str) -> Option<String>, exe: Option<&Path>, version: &str) -> Result<AgentFiles, String> {
    let places = candidates(project_root, var, exe);
    let mut mismatched: Vec<(PathBuf, String)> = Vec::new();
    for dir in &places {
        if !dir.join(MANIFEST).is_file() {
            continue;
        }
        let files = AgentFiles::load(dir)?;
        if files.version == version {
            return Ok(files);
        }
        mismatched.push((dir.clone(), files.version));
    }
    let install = install_lines();
    if let Some((dir, found)) = mismatched.first() {
        return Err(format!(
            "the skills package at {} is version {found}, and this codelaw is {version}; they are released together and must be the same version. Install the matching one:\n{install}",
            dir.display()
        ));
    }
    let npm = Path::new("node_modules").join("@codelaw").join("skills");
    let mut looked = vec![format!(
        "  {} in {} and every folder above it{}",
        npm.display(),
        project_root.display(),
        exe.map(|e| format!(", and above the binary {}", e.display())).unwrap_or_default()
    )];
    looked.extend(places.iter().filter(|p| !p.ends_with(&npm)).map(|p| format!("  {}", p.display())));
    looked.push(format!(
        "  {PY_MODULE}/ in the site-packages of CODELAW_SITE_PACKAGES, else $VIRTUAL_ENV, else {}, and of the binary's own Python environment",
        project_root.join(".venv").display()
    ));
    Err(format!(
        "the agent files come from the skills package, {NPM_NAME} on npm or {PYPI_NAME} on PyPI, and it is not installed. Install it beside codelaw, at the same version:\n{install}\nlooked in ({ENV} names another folder):\n{}",
        looked.join("\n")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("codelaw-skills-pkg-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn package(dir: &Path, version: &str) {
        std::fs::create_dir_all(dir.join("skills/codelaw-fix")).unwrap();
        std::fs::write(dir.join(MANIFEST), format!("{{\"name\": \"codelaw-skills\", \"version\": \"{version}\"}}\n")).unwrap();
        std::fs::write(dir.join("skills/codelaw-fix/SKILL.md"), "fix").unwrap();
        std::fs::write(dir.join("LICENSE"), "MIT").unwrap();
    }

    #[test]
    fn pep440_matches_package_py() {
        assert_eq!(pep440("0.2.0"), "0.2.0");
        assert_eq!(pep440("0.2.0-beta.1"), "0.2.0b1");
        assert_eq!(pep440("1.0.0-rc.2"), "1.0.0rc2");
        assert_eq!(pep440("1.0.0-alpha.3"), "1.0.0a3");
    }

    #[test]
    fn the_lookup_order_and_the_refusals() {
        let root = temp("order");
        let project = root.join("repo");
        std::fs::create_dir_all(&project).unwrap();
        let none = |_: &str| None;
        let err = find(&project, &none, None, "9.9.9").unwrap_err();
        assert!(err.contains("npm i -D @codelaw/skills@") && err.contains("pip install codelaw-skills=="), "{err}");
        assert!(err.contains(&format!("in {} and every folder above it", project.display())), "it says where it looked: {err}");

        // A Python project: .venv's site-packages.
        let venv = project.join(".venv/lib/python3.12/site-packages/codelaw_skills");
        package(&venv, "9.9.9");
        let found = find(&project, &none, None, "9.9.9").unwrap();
        assert_eq!(found.root, venv);
        assert_eq!(found.text("skills/codelaw-fix/SKILL.md").unwrap(), "fix");
        assert!(found.paths().all(|p| p != "LICENSE" && p != MANIFEST), "the package's own files are not agent files");

        // npm wins over the venv, as for the rule packs.
        let npm = root.join("node_modules/@codelaw/skills");
        package(&npm, "9.9.9");
        assert_eq!(find(&project, &none, None, "9.9.9").unwrap().root, npm);

        // A mismatched copy is passed over for a matching one, and named when none matches.
        package(&npm, "1.0.0");
        assert_eq!(find(&project, &none, None, "9.9.9").unwrap().root, venv);
        package(&venv, "1.0.0");
        let err = find(&project, &none, None, "9.9.9").unwrap_err();
        assert!(err.contains("is version 1.0.0, and this codelaw is 9.9.9"), "{err}");

        // CODELAW_SKILLS comes first.
        let own = root.join("own");
        package(&own, "9.9.9");
        let o = own.to_string_lossy().into_owned();
        let var = move |k: &str| (k == ENV).then(|| o.clone());
        assert_eq!(find(&project, &var, None, "9.9.9").unwrap().root, own);

        // A pipx codelaw: the binary's own environment.
        let pipx = root.join("pipx/venvs/codelaw");
        package(&pipx.join("lib/python3.13/site-packages/codelaw_skills"), "9.9.9");
        let exe = pipx.join("bin/codelaw");
        assert_eq!(find(&project, &none, Some(&exe), "9.9.9").unwrap().root, pipx.join("lib/python3.13/site-packages/codelaw_skills"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
