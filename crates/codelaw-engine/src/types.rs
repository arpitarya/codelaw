//! Type-aware rules in the pipeline (SR-TYPES): which TypeScript answers a
//! file, the helper that asks it, and what the report says about both.
//!
//! - **Cold** (`check`, CI): the first file a type rule needs starts one
//!   helper per TypeScript copy, which loads the program once for the run.
//! - **Warm** (`daemon`, `lsp`, `mcp`, `--watch`): helpers outlive the run in
//!   this process; a later run reuses one after it re-reads what changed on
//!   disk (decision 8).
//! - **Hooks** (`codelaw hook`, `hook --pre`) never start one: they use a
//!   warm helper or skip the type rules with a note (L8 decision 8).

use crate::{EngineOptions, Project};
use crate::discover::FileEntry;
use codelaw_types::Helper;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// A running helper, shared by the files of a run and the runs of a warm
/// process.
type Shared = Arc<Mutex<Helper>>;

/// A helper this process keeps for later runs (decision 8).
struct Kept {
    typescript: PathBuf,
    root: PathBuf,
    helper: Shared,
}

/// Every helper this process has started, most recent last.
static KEPT: Mutex<Vec<Kept>> = Mutex::new(Vec::new());

/// How many helpers a warm process keeps: one per project and TypeScript.
const KEEP: usize = 4;

/// A run's type state.
#[derive(Default)]
pub(crate) struct Types {
    /// The `typescript` package for each folder, looked up once.
    found: Mutex<HashMap<PathBuf, Option<PathBuf>>>,
    node: OnceLock<Option<PathBuf>>,
    /// The helper for each TypeScript this run used, or why there is none.
    sessions: Mutex<BTreeMap<PathBuf, Result<Shared, String>>>,
    /// The TypeScript versions that answered (decision 12).
    used: Mutex<BTreeSet<String>>,
    /// Why type rules were skipped, or ran outside the tested range.
    notes: Mutex<BTreeSet<String>>,
    /// Whether any file had a type rule to run.
    asked: std::sync::atomic::AtomicBool,
}

/// What the report says about the type rules (SR-TYPES decisions 6 and 12).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypesReport {
    /// The helper's protocol version.
    pub protocol: u32,
    /// The TypeScript versions that answered.
    pub typescript: Vec<String>,
    /// Why the type rules were skipped, or ran with a warning.
    pub notes: Vec<String>,
}

/// A note's wording for a version too old for the API (decision 6).
fn too_old(version: &str) -> String {
    format!("the type rules were skipped: TypeScript {version} has no type API; they need TypeScript 7.1 or later")
}

impl Types {
    fn note(&self, n: String) {
        self.notes.lock().unwrap_or_else(|e| e.into_inner()).insert(n);
    }

    /// The report's `types` part, when a type rule had a file to check.
    pub(crate) fn report(&self) -> Option<TypesReport> {
        if !self.asked.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        Some(TypesReport {
            protocol: codelaw_types::PROTOCOL,
            typescript: self.used.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect(),
            notes: self.notes.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect(),
        })
    }
}

/// The `typescript` package that governs `dir`: from it up to `root`, else
/// the one `CODELAW_TYPESCRIPT` names (decision 5).
fn typescript_in(dir: &Path, root: &Path, env: Option<&str>) -> Option<PathBuf> {
    codelaw_types::find_typescript(dir, root).or_else(|| env.map(PathBuf::from).filter(|p| p.join("package.json").is_file()))
}

/// What `codelaw --version --json` names about the type rules from a
/// folder (SR-TYPES decision 12, L5 decision 2): the protocol, the tested
/// range, and the TypeScript and Node a `check` there would use. Nothing is
/// started: the TypeScript's version is its `package.json`'s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypesVersion {
    pub protocol: u32,
    /// The TypeScript versions the fixtures pass on (decision 7).
    pub tested: &'static str,
    pub typescript: Option<FoundTypescript>,
    pub node: Option<PathBuf>,
}

/// The project's TypeScript, as `--version --json` names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundTypescript {
    pub version: String,
    /// The `typescript` package folder.
    pub path: PathBuf,
    /// 7.1 or later: the type rules can run (decision 6).
    pub api: bool,
    /// Inside [`codelaw_types::TESTED`] (decision 7).
    pub tested: bool,
}

/// [`TypesVersion`] for the project that governs `start`: its TypeScript
/// found from `start` up to the project root, as for a file there.
pub fn types_version(start: &Path, options: &EngineOptions) -> TypesVersion {
    let start = std::fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    let root = codelaw_config::find_config(&start).map(|c| c.root_dir).unwrap_or_else(|_| start.clone());
    let typescript = typescript_in(&start, &root, options.typescript_env.as_deref()).map(|path| {
        let version = codelaw_types::package_version(&path).unwrap_or_default();
        FoundTypescript { api: codelaw_types::has_api(&version), tested: codelaw_types::tested(&version), version, path }
    });
    TypesVersion {
        protocol: codelaw_types::PROTOCOL,
        tested: codelaw_types::TESTED,
        typescript,
        node: codelaw_adapters::locate_program("node", &root, options.path_env.as_deref()),
    }
}

impl Project {
    /// The `typescript` package that governs a file: from its folder up to
    /// the project root, else `CODELAW_TYPESCRIPT` (decision 5).
    fn typescript_for(&self, entry: &FileEntry) -> Option<PathBuf> {
        let dir = entry.abs.parent().unwrap_or(self.root()).to_path_buf();
        let mut found = self.types.found.lock().unwrap_or_else(|e| e.into_inner());
        found.entry(dir.clone()).or_insert_with(|| typescript_in(&dir, self.root(), self.options.typescript_env.as_deref())).clone()
    }

    fn node(&self) -> Option<&PathBuf> {
        self.types.node.get_or_init(|| codelaw_adapters::locate_program("node", self.root(), self.options.path_env.as_deref())).as_ref()
    }

    /// Whether a type program can answer for `entry`, without starting
    /// one: the TypeScript to ask, or why the type rules are skipped. A
    /// skip is noted for the report.
    pub(crate) fn types_available(&self, entry: &FileEntry) -> Result<PathBuf, String> {
        self.types.asked.store(true, std::sync::atomic::Ordering::Relaxed);
        let out = self.availability(entry);
        if let Err(why) = &out {
            self.types.note(why.clone());
        }
        out
    }

    fn availability(&self, entry: &FileEntry) -> Result<PathBuf, String> {
        let Some(ts) = self.typescript_for(entry) else {
            return Err("the type rules were skipped: no `typescript` package in `node_modules` from the file's folder up to the project root".into());
        };
        let version = codelaw_types::package_version(&ts).unwrap_or_default();
        if !codelaw_types::has_api(&version) {
            return Err(too_old(&version));
        }
        if self.node().is_none() {
            return Err("the type rules were skipped: Node was not found on `PATH`".into());
        }
        if self.options.no_cold_types && kept(&ts, self.root()).is_none() {
            return Err("the type rules were skipped: hooks use only a warm type program (`codelaw daemon start`), never a cold one".into());
        }
        Ok(ts)
    }

    /// The helper that answers for `entry`: this run's, a warm one this
    /// process kept, or a new one (never in a hook).
    pub(crate) fn types_helper(&self, entry: &FileEntry) -> Result<Shared, String> {
        let ts = self.types_available(entry)?;
        let mut sessions = self.types.sessions.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = sessions.get(&ts) {
            return s.clone();
        }
        let session = self.session(&ts);
        match &session {
            Ok(h) => {
                let version = h.lock().unwrap_or_else(|e| e.into_inner()).typescript.clone();
                if !codelaw_types::tested(&version) {
                    self.types.note(format!(
                        "TypeScript {version} is outside the tested range ({}); the type rules ran (SR-TYPES decision 7)",
                        codelaw_types::TESTED
                    ));
                }
                self.types.used.lock().unwrap_or_else(|e| e.into_inner()).insert(version);
            }
            Err(why) => self.types.note(why.clone()),
        }
        sessions.insert(ts, session.clone());
        session
    }

    fn session(&self, ts: &Path) -> Result<Shared, String> {
        // A warm helper re-reads what changed since its last run.
        if let Some(h) = kept(ts, self.root()) {
            if h.lock().unwrap_or_else(|e| e.into_inner()).refresh().is_ok() {
                return Ok(h);
            }
            forget(&h);
            if self.options.no_cold_types {
                return Err("the type rules were skipped: the warm type program stopped, and hooks never load a cold one".into());
            }
        }
        let node = self.node().ok_or("the type rules were skipped: Node was not found on `PATH`")?;
        let cwd = if self.root().is_dir() { self.root().to_path_buf() } else { std::env::temp_dir() };
        let helper = Helper::start(node, ts, &cwd).map_err(|e| format!("the type rules were skipped: the type helper did not start: {e}"))?;
        if !codelaw_types::has_api(&helper.typescript) {
            return Err(too_old(&helper.typescript));
        }
        let helper = Arc::new(Mutex::new(helper));
        let mut all = KEPT.lock().unwrap_or_else(|e| e.into_inner());
        all.push(Kept { typescript: ts.to_path_buf(), root: self.root().to_path_buf(), helper: helper.clone() });
        if all.len() > KEEP {
            all.remove(0);
        }
        Ok(helper)
    }

    /// Why no type program would answer for a file at `rel`, or `None`
    /// when one would (the rule tester asks before it checks a type rule's
    /// examples).
    pub fn types_unavailable(&self, rel: &str, language: codelaw_core::Language) -> Option<String> {
        let entry = FileEntry { abs: self.root().join(rel), rel: rel.to_owned(), language };
        self.availability(&entry).err()
    }

    /// Tell every helper this run uses what changed on disk (`--watch`).
    pub(crate) fn types_changed(&self, changed: &[PathBuf]) {
        let sessions = self.types.sessions.lock().unwrap_or_else(|e| e.into_inner());
        for h in sessions.values().flatten() {
            let _ = h.lock().unwrap_or_else(|e| e.into_inner()).changed(changed, &[], &[]);
        }
    }
}

fn kept(ts: &Path, root: &Path) -> Option<Shared> {
    KEPT.lock().unwrap_or_else(|e| e.into_inner()).iter().rev().find(|k| k.typescript == ts && k.root == root).map(|k| k.helper.clone())
}

fn forget(h: &Shared) {
    KEPT.lock().unwrap_or_else(|e| e.into_inner()).retain(|k| !Arc::ptr_eq(&k.helper, h));
}
