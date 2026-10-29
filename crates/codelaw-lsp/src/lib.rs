//! `codelaw lsp`: the language server (SR-EDITOR). It only translates
//! between the protocol and `codelaw-engine` (SR-ARCHITECTURE decision 2);
//! every verdict is the engine's, so the editor never shows a result that
//! `codelaw check` on the same file wouldn't (SR-EDITOR decision 3).
//!
//! Two boundaries are kept here and nowhere else:
//! - **Positions** are converted from bytes to 0-based UTF-16 only in
//!   [`position`] (SR-ARCHITECTURE decision 13).
//! - **No writes.** Fixes and formatting come back as `WorkspaceEdit`s and
//!   `TextEdit`s; the editor applies them (SR-ARCHITECTURE decision 12).
//!   Nothing in this crate opens a file for writing.
//!
//! Edits arrive as incremental changes, and each open document keeps its
//! syntax tree between them: an edit re-parses only what it touched and
//! re-runs that file's rules (SR-EDITOR decision 2, SR-ARCHITECTURE
//! decision 23). The tree is the one a fresh parse gives, so the verdict is
//! the same. A closed document's tree goes to its project's [`TreeCache`]
//! of the `[lsp] tree_cache` most recently touched closed files, and a
//! reopened file whose text is unchanged gets it back (decision 24).
//!
//! Not yet here, and why: project rules and workspace diagnostics (the
//! graph is not built), pull diagnostics. The warm-process socket of
//! SR-ARCHITECTURE decision 21 is registered by the `codelaw lsp` command,
//! not here: this crate never writes a file.

pub mod format;
pub mod position;
pub mod uri;

use codelaw_core::{Diagnostic, FixClass, Severity};
use codelaw_engine::{DocumentTree, EngineOptions, FileEntry, Project, TreeCache};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::notification::{
    DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument, LogMessage, Notification as _,
    PublishDiagnostics, ShowMessage,
};
use lsp_types::request::{CodeActionRequest, Formatting, HoverRequest, RangeFormatting, Request as _};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, DiagnosticRelatedInformation,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentFormattingParams, DocumentRangeFormattingParams, Hover, HoverContents, HoverParams, HoverProviderCapability, Location, LogMessageParams, MarkupContent, MarkupKind, MessageType,
    NumberOrString, OneOf, PositionEncodingKind, PublishDiagnosticsParams, Range, ServerCapabilities, ServerInfo, ShowMessageParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TextEdit, Uri, WorkspaceEdit,
};
use position::Utf16Index;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// How much the server says through `window/logMessage` (SR-EDITOR
/// decision 7). The extension passes it as `initializationOptions.logLevel`
/// and changes it with `workspace/didChangeConfiguration`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    /// `error`, `warn`, `info` or `debug`; anything else is `None`.
    pub fn parse(s: &str) -> Option<LogLevel> {
        match s {
            "error" => Some(LogLevel::Error),
            "warn" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            _ => None,
        }
    }

    fn of(typ: MessageType) -> LogLevel {
        match typ {
            MessageType::ERROR => LogLevel::Error,
            MessageType::WARNING => LogLevel::Warn,
            MessageType::INFO => LogLevel::Info,
            _ => LogLevel::Debug,
        }
    }
}

/// The code-action kind of "fix everything safe in this file".
pub const FIX_ALL: &str = "source.fixAll.codelaw";

/// Serve LSP over stdin and stdout until the client says `exit`.
pub fn run_stdio() -> Result<(), Error> {
    let (connection, io) = Connection::stdio();
    serve(&connection)?;
    drop(connection);
    io.join()?;
    Ok(())
}

/// What the server offers.
pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF16),
        text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::INCREMENTAL),
            save: Some(TextDocumentSyncSaveOptions::Supported(true)),
            ..Default::default()
        })),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICKFIX, CodeActionKind::from(FIX_ALL)]),
            resolve_provider: Some(false),
            ..Default::default()
        })),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        document_formatting_provider: format::AVAILABLE.then_some(OneOf::Left(true)),
        document_range_formatting_provider: format::AVAILABLE.then_some(OneOf::Left(true)),
        ..Default::default()
    }
}

/// Run the server on a connection: the handshake, then the message loop.
pub fn serve(connection: &Connection) -> Result<(), Error> {
    let (id, params) = connection.initialize_start()?;
    let params: lsp_types::InitializeParams = serde_json::from_value(params)?;
    let initial_level = params.initialization_options.as_ref().and_then(level_of);
    #[allow(deprecated)] // `rootUri`, for clients without workspace folders
    let mut roots: Vec<PathBuf> = params
        .workspace_folders
        .unwrap_or_default()
        .iter()
        .map(|f| f.uri.as_str().to_owned())
        .chain(params.root_uri.map(|u| u.as_str().to_owned()))
        .filter_map(|u| uri::to_path(&u))
        .map(|p| canonical(&p))
        .collect();
    roots.sort();
    roots.dedup();
    let result = lsp_types::InitializeResult {
        capabilities: capabilities(),
        server_info: Some(ServerInfo { name: "codelaw".into(), version: Some(env!("CARGO_PKG_VERSION").into()) }),
    };
    connection.initialize_finish(id, serde_json::to_value(result)?)?;
    let mut server = Server::new(connection, roots);
    if let Some(level) = initial_level {
        server.level = level;
    }
    server.log_at(
        LogLevel::Info,
        format!("codelaw lsp {} started: {} workspace folder(s): {}", env!("CARGO_PKG_VERSION"), server.roots.len(), server.paths(&server.roots)),
    )?;
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                server.request(req)?;
            }
            Message::Notification(n) if n.method == "exit" => return Ok(()),
            Message::Notification(n) => server.notification(n)?,
            Message::Response(_) => {}
        }
    }
    Ok(())
}

/// One open document and what was last found in it.
struct Doc {
    uri: Uri,
    version: i32,
    text: String,
    /// The project key and the entry `check` would give the file; `None`
    /// when `check` would not check it.
    target: Option<(PathBuf, FileEntry)>,
    /// The engine's diagnostics for `text`, in byte offsets.
    diagnostics: Vec<Diagnostic>,
    /// The syntax tree of the body, kept between edits; `None` until the
    /// document is first checked, or after an edit it cannot follow.
    tree: Option<DocumentTree>,
    /// For a plain-text config file (`.nvmrc`, `CODEOWNERS`, …: SR-LAW-3
    /// decision 10), its project-relative path: `diagnostics` are then the
    /// copies of findings its L3 files reported into it.
    text_config: Option<String>,
}

impl Doc {
    /// The text the engine sees (no byte-order mark) and how far its
    /// offsets are from the editor's.
    fn body(&self) -> (&str, usize) {
        match self.text.strip_prefix('\u{feff}') {
            Some(t) => (t, '\u{feff}'.len_utf8()),
            None => (&self.text, 0),
        }
    }

    /// Apply one content change. A ranged change edits the text and the
    /// kept tree incrementally; a whole-text change, or an edit that moves
    /// the byte-order mark, drops the tree so the next check parses afresh.
    fn change(&mut self, change: lsp_types::TextDocumentContentChangeEvent) {
        let Some(range) = change.range else {
            self.text = change.text;
            self.tree = None;
            return;
        };
        let idx = Utf16Index::new(&self.text);
        let (a, b) = (idx.offset(range.start), idx.offset(range.end));
        let (start, end) = (a.min(b), a.max(b));
        let shift = self.body().1;
        self.text.replace_range(start..end, &change.text);
        let follows = start >= shift && self.body().1 == shift;
        let kept = follows && self.tree.as_mut().is_some_and(|tree| tree.edit(start - shift, end - shift, &change.text).is_ok());
        if !kept {
            self.tree = None;
        }
    }
}

struct Server<'c> {
    connection: &'c Connection,
    /// The workspace folders: where a file without a config of its own
    /// finds its project.
    roots: Vec<PathBuf>,
    docs: HashMap<String, Doc>,
    /// Loaded projects by their config folder; an `Err` is a config or
    /// rule that does not load, reported once until something changes.
    projects: HashMap<PathBuf, Result<Project, String>>,
    /// The trees of closed documents, by project key, each bounded by its
    /// project's `[lsp] tree_cache` (SR-ARCHITECTURE decision 24).
    closed: HashMap<PathBuf, TreeCache>,
    /// What `window/logMessage` carries (decision 7).
    level: LogLevel,
    /// Log lines already said since the last reload, so a keystroke never
    /// repeats one: `(file or project, line)`.
    said: HashSet<(String, String)>,
    /// What is published for each plain-text config file, by the document
    /// whose check found it: an open L3 file's copies, or the text file's
    /// own `check_text_config` (SR-EDITOR decision 2, SR-ARCHITECTURE
    /// decision 15d). A text file's diagnostics are all of these, at once.
    copies: HashMap<String, BTreeMap<String, Vec<lsp_types::Diagnostic>>>,
}

impl<'c> Server<'c> {
    fn new(connection: &'c Connection, roots: Vec<PathBuf>) -> Self {
        Server {
            connection,
            roots,
            docs: HashMap::new(),
            projects: HashMap::new(),
            closed: HashMap::new(),
            level: LogLevel::Info,
            said: HashSet::new(),
            copies: HashMap::new(),
        }
    }

    fn send(&self, msg: impl Into<Message>) -> Result<(), Error> {
        self.connection.sender.send(msg.into()).map_err(|e| Error::from(e.to_string()))
    }

    fn notify<N: lsp_types::notification::Notification>(&self, params: N::Params) -> Result<(), Error> {
        self.send(Notification::new(N::METHOD.to_owned(), params))
    }

    fn log(&self, typ: MessageType, message: String) -> Result<(), Error> {
        if LogLevel::of(typ) > self.level {
            return Ok(());
        }
        self.notify::<LogMessage>(LogMessageParams { typ, message })
    }

    /// Log at `level`. Debug lines go as `LOG`, which the extension shows
    /// at its debug level; they hold paths, rule ids, counts and timings,
    /// never source text (decision 7). Nothing is printed: stdout is the
    /// transport.
    fn log_at(&self, level: LogLevel, message: String) -> Result<(), Error> {
        let typ = match level {
            LogLevel::Error => MessageType::ERROR,
            LogLevel::Warn => MessageType::WARNING,
            LogLevel::Info => MessageType::INFO,
            LogLevel::Debug => MessageType::LOG,
        };
        self.log(typ, message)
    }

    /// `log_at`, but once per `subject` and line until the next reload.
    fn log_once(&mut self, level: LogLevel, subject: &str, message: String) -> Result<(), Error> {
        if self.said.insert((subject.to_owned(), message.clone())) {
            self.log_at(level, message)?;
        }
        Ok(())
    }

    fn paths(&self, paths: &[PathBuf]) -> String {
        paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
    }

    // ---- notifications -------------------------------------------------

    fn notification(&mut self, n: Notification) -> Result<(), Error> {
        match n.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let p: DidOpenTextDocumentParams = serde_json::from_value(n.params)?;
                let d = p.text_document;
                let doc = Doc { uri: d.uri, version: d.version, text: d.text, target: None, diagnostics: vec![], tree: None, text_config: None };
                let key = doc.uri.as_str().to_owned();
                self.docs.insert(key.clone(), doc);
                self.check(&key)?;
            }
            DidChangeTextDocument::METHOD => {
                let p: DidChangeTextDocumentParams = serde_json::from_value(n.params)?;
                let key = p.text_document.uri.as_str().to_owned();
                // Incremental sync: the changes apply in order.
                if let Some(doc) = self.docs.get_mut(&key) {
                    for change in p.content_changes {
                        doc.change(change);
                    }
                    doc.version = p.text_document.version;
                    // A text config file's copies come from its L3 files on
                    // disk: it is checked when saved, not per keystroke.
                    if doc.text_config.is_none() {
                        self.check(&key)?;
                    }
                }
            }
            DidSaveTextDocument::METHOD => {
                let p: DidSaveTextDocumentParams = serde_json::from_value(n.params)?;
                let path = uri::to_path(p.text_document.uri.as_str());
                if path.as_deref().is_some_and(reloads) {
                    let name = path.as_deref().map(|p| p.display().to_string()).unwrap_or_default();
                    self.reload(&format!("saved {name}"))?;
                } else {
                    self.check(p.text_document.uri.as_str())?;
                }
            }
            DidCloseTextDocument::METHOD => {
                let p: DidCloseTextDocumentParams = serde_json::from_value(n.params)?;
                if let Some(doc) = self.docs.remove(p.text_document.uri.as_str()) {
                    if let (Some((pk, _)), Some(tree)) = (&doc.target, doc.tree)
                        && let Some(cache) = self.closed.get_mut(pk)
                    {
                        cache.put(doc.uri.as_str().to_owned(), tree);
                    }
                    let key = doc.uri.as_str().to_owned();
                    // What this document found in text config files goes
                    // with it; a text file keeps what open L3 files found.
                    self.set_copies(&key, BTreeMap::new())?;
                    if self.copies.contains_key(&key) {
                        self.publish_copies(&key)?;
                    } else {
                        self.notify::<PublishDiagnostics>(PublishDiagnosticsParams { uri: doc.uri, diagnostics: vec![], version: None })?;
                    }
                }
            }
            // Config, rules or baseline changed on disk: reload live
            // (SR-EDITOR decision 1).
            DidChangeWatchedFiles::METHOD => {
                let p: lsp_types::DidChangeWatchedFilesParams = serde_json::from_value(n.params)?;
                let files: Vec<String> = p.changes.iter().filter_map(|c| uri::to_path(c.uri.as_str())).map(|f| f.display().to_string()).collect();
                self.reload(&format!("watched file changed: {}", files.join(", ")))?
            }
            // `{ "settings": { "codelaw": { "logLevel": "debug" } } }`
            DidChangeConfiguration::METHOD => {
                let p: lsp_types::DidChangeConfigurationParams = serde_json::from_value(n.params)?;
                if let Some(level) = p.settings.get("codelaw").and_then(level_of).filter(|l| *l != self.level) {
                    self.level = level;
                    self.log_at(LogLevel::Info, format!("log level is now {level:?}").to_lowercase())?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Drop every loaded project and re-check every open document.
    fn reload(&mut self, why: &str) -> Result<(), Error> {
        self.log_at(LogLevel::Info, format!("reloading projects: {why}"))?;
        self.projects.clear();
        self.said.clear();
        let mut keys: Vec<String> = self.docs.keys().cloned().collect();
        keys.sort();
        for k in keys {
            self.check(&k)?;
        }
        Ok(())
    }

    /// The project that governs `path`, loaded once.
    fn project_for(&mut self, path: &Path) -> Result<Option<PathBuf>, Error> {
        let dir = path.parent().unwrap_or(path);
        // With no config above the file, the workspace folder is the
        // project, as `codelaw check` run there would see it.
        let found =
            codelaw_config::find_config(dir).and_then(|c| match self.roots.iter().filter(|r| path.starts_with(r)).max_by_key(|r| r.components().count()) {
                Some(root) if c.files.is_empty() => codelaw_config::find_config(root),
                _ => Ok(c),
            });
        let config = match found {
            Ok(c) => c,
            Err(e) => {
                let subject = path.display().to_string();
                if self.said.insert((subject.clone(), format!("show {e}"))) {
                    self.notify::<ShowMessage>(ShowMessageParams { typ: MessageType::ERROR, message: format!("codelaw: {e}") })?;
                }
                self.log_once(LogLevel::Error, &subject, format!("codelaw: no project for {subject}: {e}"))?;
                return Ok(None);
            }
        };
        let key = config.config_dir().to_path_buf();
        if !self.projects.contains_key(&key) {
            // The editor reads no cache and writes none.
            // A person's editor: the `local` profile unless CODELAW_PROFILE says
            // otherwise (SR-CONFIG decision 10).
            let options = EngineOptions { no_cache: true, profile_surface: Some(("local", "LSP")), ..EngineOptions::from_env() };
            let loaded = Project::from_config(config, options).map_err(|e| e.to_string());
            match &loaded {
                Ok(p) => {
                    // A reload keeps the closed trees, under the bound the
                    // config gives now.
                    let bound = p.config.lsp_tree_cache();
                    self.closed.entry(key.clone()).or_insert_with(|| TreeCache::new(bound)).set_capacity(bound);
                    for w in &p.warnings {
                        self.log(MessageType::WARNING, format!("codelaw: {w}"))?;
                    }
                }
                Err(e) => {
                    self.notify::<ShowMessage>(ShowMessageParams { typ: MessageType::ERROR, message: format!("codelaw: {e}") })?;
                }
            }
            match &loaded {
                Ok(p) => self.log_at(
                    LogLevel::Info,
                    format!("project {} loaded: config {}, {} rule(s) compiled", key.display(), p.config.config_dir().display(), p.compiled.len()),
                )?,
                Err(e) => self.log_at(LogLevel::Error, format!("project {} failed to load: {e}", key.display()))?,
            }
            self.projects.insert(key.clone(), loaded);
        }
        Ok(matches!(self.projects.get(&key), Some(Ok(_))).then_some(key))
    }

    /// Re-check one open document and publish what it finds.
    fn check(&mut self, key: &str) -> Result<(), Error> {
        let started = Instant::now();
        let Some(doc) = self.docs.get(key) else { return Ok(()) };
        // Why a file gets nothing, when it does: said once per file per
        // reload, never per keystroke (decision 7).
        let mut why_not: Option<String> = None;
        let target = match uri::to_path(doc.uri.as_str()) {
            Some(path) => {
                let path = canonical(&path);
                match self.project_for(&path)? {
                    Some(pk) => match &self.projects[&pk] {
                        Ok(project) => {
                            let entry = project.entry_for(&path);
                            if entry.is_none() {
                                why_not = Some(if codelaw_core::Language::from_path(&path).is_none() {
                                    "its language is not one codelaw covers".to_owned()
                                } else if !project.config.is_included(&path) {
                                    "it is outside the project's `include` or matched by `exclude`".to_owned()
                                } else {
                                    "its language is not enabled for it in `languages`".to_owned()
                                });
                            } else {
                                self.log_once(LogLevel::Info, &pk.display().to_string(), format!("{} is checked by project {}", path.display(), pk.display()))?;
                            }
                            entry.map(|e| (pk, e))
                        }
                        Err(e) => {
                            why_not = Some(format!("its project {} failed to load: {e}", pk.display()));
                            None
                        }
                    },
                    None => {
                        why_not = Some("its config does not load (see the error above)".to_owned());
                        None
                    }
                }
            }
            None => {
                why_not = Some("it is not a file: URI".to_owned());
                None
            }
        };
        if target.is_none()
            && let Some(path) = uri::to_path(key).map(|p| canonical(&p))
            && codelaw_core::is_text_config(&path.to_string_lossy())
        {
            return self.check_text_config(key, &path);
        }
        if let Some(reason) = why_not {
            let name = uri::to_path(key).map_or_else(|| key.to_owned(), |p| p.display().to_string());
            self.log_once(LogLevel::Info, &name, format!("no diagnostics for {name}: {reason}"))?;
        }
        let doc = self.docs.get_mut(key).expect("an open document");
        let project = target.as_ref().and_then(|(pk, _)| self.projects[pk].as_ref().ok());
        let diagnostics = match (&target, project) {
            (Some((pk, entry)), Some(project)) => {
                // The kept tree when it is of this text and language; the
                // engine checks that too, so a stale tree never shows. A
                // document just reopened takes its tree from the closed
                // files' cache.
                let kept = doc.tree.take().or_else(|| self.closed.get_mut(pk).and_then(|c| c.take(key)));
                let body = doc.body().0;
                let tree = match kept {
                    Some(t) if t.language() == entry.language && t.text() == body => t,
                    _ => DocumentTree::new(entry.language, body),
                };
                let d = project.check_document_tree(entry, &tree);
                doc.tree = Some(tree);
                d
            }
            _ => {
                doc.tree = None;
                vec![]
            }
        };
        // A finding's copy in a text config file is published on that file
        // (SR-ARCHITECTURE decision 15d), never here.
        let (copies, diagnostics): (Vec<Diagnostic>, Vec<Diagnostic>) = diagnostics
            .into_iter()
            .partition(|d| target.as_ref().is_some_and(|(_, e)| e.rel != d.file) && codelaw_core::is_text_config(&d.file));
        let copies = project.map(|p| copies_by_uri(p.root(), &copies)).unwrap_or_default();
        let lsp = to_lsp(doc, target.as_ref().map(|t| &t.1), project.map(Project::root), &diagnostics);
        let params = PublishDiagnosticsParams { uri: doc.uri.clone(), diagnostics: lsp, version: Some(doc.version) };
        let summary = format!(
            "check {}: {} rule(s), {} diagnostic(s), {} ms",
            uri::to_path(key).map_or_else(|| key.to_owned(), |p| p.display().to_string()),
            project.map_or(0, |p| p.compiled.len()),
            diagnostics.len(),
            started.elapsed().as_millis()
        );
        let lang = target.as_ref().map_or("-".to_owned(), |t| format!("{:?}", t.1.language));
        doc.target = target;
        doc.diagnostics = diagnostics;
        self.notify::<PublishDiagnostics>(params)?;
        self.set_copies(key, copies)?;
        self.log_at(LogLevel::Debug, format!("{summary} ({lang})"))
    }

    /// Check an open plain-text config file: the copies its project's L3
    /// files report into it, as they are on disk (`check_text_config`).
    fn check_text_config(&mut self, key: &str, path: &Path) -> Result<(), Error> {
        let pk = self.project_for(path)?;
        let project = pk.as_ref().and_then(|pk| self.projects[pk].as_ref().ok());
        let found = project.map(|p| p.check_text_config(path)).unwrap_or_default();
        let lsp = project.map(|p| copies_by_uri(p.root(), &found)).unwrap_or_default().into_values().flatten().collect();
        let rel = project.map(|p| codelaw_core::normalize_path(path.strip_prefix(p.root()).unwrap_or(path)));
        self.log_at(LogLevel::Debug, format!("check {}: {} finding(s) copied in from its L3 files", path.display(), found.len()))?;
        if let Some(doc) = self.docs.get_mut(key) {
            doc.target = None;
            doc.tree = None;
            doc.text_config = rel;
            doc.diagnostics = found;
        }
        let mine = self.copies.entry(key.to_owned()).or_default();
        mine.insert(key.to_owned(), lsp);
        self.publish_copies(key)
    }

    /// Replace what `from` found in text config files with `found`, by the
    /// text file's URI, and republish every text file that changes.
    fn set_copies(&mut self, from: &str, found: BTreeMap<String, Vec<lsp_types::Diagnostic>>) -> Result<(), Error> {
        let mut touched: Vec<String> = Vec::new();
        for (uri, by) in self.copies.iter_mut() {
            if uri != from && !found.contains_key(uri) && by.remove(from).is_some() {
                touched.push(uri.clone());
            }
        }
        for (uri, d) in found {
            let by = self.copies.entry(uri.clone()).or_default();
            if by.get(from) != Some(&d) {
                by.insert(from.to_owned(), d);
                touched.push(uri);
            }
        }
        touched.sort();
        touched.dedup();
        for uri in touched {
            self.publish_copies(&uri)?;
        }
        self.copies.retain(|_, by| !by.is_empty());
        Ok(())
    }

    /// Publish everything found for one text config file, once each.
    fn publish_copies(&self, uri_key: &str) -> Result<(), Error> {
        let Ok(uri) = Uri::from_str(uri_key) else { return Ok(()) };
        let mut all: Vec<lsp_types::Diagnostic> = Vec::new();
        for d in self.copies.get(uri_key).into_iter().flat_map(|by| by.values().flatten()) {
            if !all.iter().any(|a| a.range == d.range && a.code == d.code && a.message == d.message) {
                all.push(d.clone());
            }
        }
        let version = self.docs.get(uri_key).map(|d| d.version);
        self.notify::<PublishDiagnostics>(PublishDiagnosticsParams { uri, diagnostics: all, version })
    }

    // ---- requests ------------------------------------------------------

    fn request(&mut self, req: Request) -> Result<(), Error> {
        let id = req.id.clone();
        let method = req.method.clone();
        let file = req.params.pointer("/textDocument/uri").and_then(|u| u.as_str()).and_then(uri::to_path).map(|p| p.display().to_string());
        let result = match req.method.as_str() {
            CodeActionRequest::METHOD => serde_json::from_value(req.params).map(|p| serde_json::to_value(self.code_actions(p))),
            HoverRequest::METHOD => serde_json::from_value(req.params).map(|p| serde_json::to_value(self.hover(p))),
            Formatting::METHOD if format::AVAILABLE => serde_json::from_value(req.params).map(|p| serde_json::to_value(self.formatting(p))),
            RangeFormatting::METHOD if format::AVAILABLE => serde_json::from_value(req.params).map(|p| serde_json::to_value(self.range_formatting(p))),
            m => {
                self.log_at(LogLevel::Warn, format!("request `{m}` is not handled by codelaw lsp"))?;
                return self.send(Response::new_err(id, ErrorCode::MethodNotFound as i32, format!("codelaw lsp does not handle `{m}`")));
            }
        };
        match result {
            Ok(Ok(v)) => {
                if v.is_null() || v.as_array().is_some_and(Vec::is_empty) {
                    self.log_at(LogLevel::Debug, format!("{method} {}: nothing to return", file.as_deref().unwrap_or("-")))?;
                }
                self.send(Response::new_ok(id, v))
            }
            Ok(Err(e)) | Err(e) => {
                self.log_at(LogLevel::Error, format!("request `{method}` for {} failed: {e}", file.as_deref().unwrap_or("-")))?;
                self.send(Response::new_err(id, ErrorCode::InvalidParams as i32, e.to_string()))
            }
        }
    }

    fn project_of<'a>(&'a self, doc: &'a Doc) -> Option<(&'a Project, &'a FileEntry)> {
        let (pk, entry) = doc.target.as_ref()?;
        self.projects.get(pk)?.as_ref().ok().map(|p| (p, entry))
    }

    /// Quick fixes for the `auto` and `suggest` diagnostics under the
    /// range, and "fix all" for the file. Every fix is a `WorkspaceEdit`;
    /// the server writes nothing (SR-ARCHITECTURE decision 12).
    fn code_actions(&self, p: CodeActionParams) -> Vec<CodeActionOrCommand> {
        let Some(doc) = self.docs.get(p.text_document.uri.as_str()) else { return vec![] };
        let wanted =
            |kind: &str| p.context.only.as_ref().is_none_or(|only| only.iter().any(|o| kind == o.as_str() || kind.starts_with(&format!("{}.", o.as_str()))));
        let idx = Utf16Index::new(&doc.text);
        let shift = doc.body().1;
        let (start, end) = (idx.offset(p.range.start).saturating_sub(shift), idx.offset(p.range.end).saturating_sub(shift));
        let own = |d: &&Diagnostic| match &doc.target {
            Some((_, e)) => e.rel == d.file,
            None => doc.text_config.as_deref() == Some(d.file.as_str()),
        };
        let mut out = Vec::new();
        if wanted(CodeActionKind::QUICKFIX.as_str()) {
            for d in doc.diagnostics.iter().filter(own) {
                let fixable = matches!(d.fix, FixClass::Auto | FixClass::Suggest) && !d.edits.is_empty();
                if !fixable || d.start_byte > end || d.end_byte < start {
                    continue;
                }
                let edits = d.edits.iter().map(|e| TextEdit { range: idx.range(e.start + shift, e.end + shift), new_text: e.text.clone() }).collect();
                let unsafe_note = if d.fix == FixClass::Suggest { " (suggested; check the result)" } else { "" };
                out.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Fix {}{unsafe_note}", d.rule),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![lsp_diagnostic(d, idx.range(d.start_byte + shift, d.end_byte + shift))]),
                    edit: Some(workspace_edit(&doc.uri, edits)),
                    is_preferred: Some(d.fix == FixClass::Auto),
                    ..Default::default()
                }));
            }
        }
        // "Fix all" only when it was asked for, or when the file has an
        // `auto` fix to offer.
        let any_auto = doc.diagnostics.iter().filter(own).any(|d| d.fix == FixClass::Auto && !d.edits.is_empty());
        let asked = p.context.only.is_some() && wanted(FIX_ALL);
        if (asked || (p.context.only.is_none() && any_auto))
            && let Some((project, entry)) = self.project_of(doc)
        {
            let body = doc.body().0;
            let fixed = project.fix_source(&entry.rel, entry.language, body, false);
            if fixed.changed() {
                let edit = TextEdit { range: idx.range(shift, doc.text.len()), new_text: fixed.fixed };
                out.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Fix all auto-fixable codelaw problems".into(),
                    kind: Some(CodeActionKind::from(FIX_ALL)),
                    edit: Some(workspace_edit(&doc.uri, vec![edit])),
                    ..Default::default()
                }));
            }
        }
        out
    }

    /// What the rule of each diagnostic under the cursor requires.
    fn hover(&self, p: HoverParams) -> Option<Hover> {
        let at = &p.text_document_position_params;
        let doc = self.docs.get(at.text_document.uri.as_str())?;
        let idx = Utf16Index::new(&doc.text);
        let offset = idx.offset(at.position).saturating_sub(doc.body().1);
        let project = self.project_of(doc).map(|(p, _)| p);
        let mut seen = BTreeMap::new();
        for d in doc.diagnostics.iter().filter(|d| doc.target.as_ref().is_some_and(|(_, e)| e.rel == d.file)) {
            if d.start_byte <= offset && offset <= d.end_byte {
                seen.entry(d.rule.clone()).or_insert_with(|| explain(project, d));
            }
        }
        if seen.is_empty() {
            return None;
        }
        let value = seen.into_values().collect::<Vec<_>>().join("\n\n---\n\n");
        Some(Hover { contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value }), range: None })
    }

    /// The whole document, formatted, as one edit.
    fn formatting(&self, p: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let doc = self.docs.get(p.text_document.uri.as_str())?;
        let (project, entry) = self.project_of(doc)?;
        let (body, shift) = doc.body();
        let formatted = format::format_document(project, entry, body)?;
        let idx = Utf16Index::new(&doc.text);
        Some(if formatted == body { vec![] } else { vec![TextEdit { range: idx.range(shift, doc.text.len()), new_text: formatted }] })
    }

    /// The selection's lines, formatted: the edits of the whole document's
    /// formatting that touch them (SR-FORMATTER decision 8).
    fn range_formatting(&self, p: DocumentRangeFormattingParams) -> Option<Vec<TextEdit>> {
        let doc = self.docs.get(p.text_document.uri.as_str())?;
        let (project, entry) = self.project_of(doc)?;
        let (body, shift) = doc.body();
        let formatted = format::format_document(project, entry, body)?;
        let idx = Utf16Index::new(&doc.text);
        let (start, end) = (idx.offset(p.range.start).saturating_sub(shift).min(body.len()), idx.offset(p.range.end).saturating_sub(shift).min(body.len()));
        let first = body[..start].matches('\n').count();
        let mut last = body[..end].matches('\n').count();
        // A selection that ends at the start of a line does not take it.
        if end > start && body[..end].ends_with('\n') {
            last -= 1;
        }
        let edits = format::range_edits(body, &formatted, first, last.max(first));
        Some(edits.into_iter().map(|(a, b, new_text)| TextEdit { range: idx.range(a + shift, b + shift), new_text }).collect())
    }
}

/// The engine's diagnostics as LSP's. A diagnostic of another file
/// (`codelaw/no-rules` names the config) is shown at the top of this
/// one and points at where it belongs.
fn to_lsp(doc: &Doc, entry: Option<&FileEntry>, root: Option<&Path>, diagnostics: &[Diagnostic]) -> Vec<lsp_types::Diagnostic> {
    let idx = Utf16Index::new(&doc.text);
    let shift = doc.body().1;
    diagnostics
        .iter()
        .map(|d| {
            let own = entry.is_some_and(|e| e.rel == d.file);
            let mut out = lsp_diagnostic(d, if own { idx.range(d.start_byte + shift, d.end_byte + shift) } else { Range::default() });
            if !own && let Some(root) = root {
                let path = root.join(&d.file);
                let line = d.start.line.saturating_sub(1);
                let at = lsp_types::Position { line, character: d.start.column.saturating_sub(1) };
                if let Ok(uri) = Uri::from_str(&uri::from_path(&path)) {
                    out.related_information =
                        Some(vec![DiagnosticRelatedInformation { location: Location { uri, range: Range { start: at, end: at } }, message: d.file.clone() }]);
                }
            }
            out
        })
        .collect()
}

/// `logLevel` in an options object, when it is one of the four.
fn level_of(v: &serde_json::Value) -> Option<LogLevel> {
    v.get("logLevel").and_then(|l| l.as_str()).and_then(LogLevel::parse)
}

/// Saving these changes what every file's verdict may be.
fn reloads(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    matches!(name, "codelaw.toml" | "pyproject.toml" | "package.json" | ".editorconfig")
        || path.components().any(|c| c.as_os_str() == ".codelaw")
        || (name.ends_with(".md") && path.components().any(|c| c.as_os_str() == "rules"))
}

/// The path as discovery sees it (symlinks resolved), for a file that may
/// not exist yet.
fn canonical(path: &Path) -> PathBuf {
    if let Ok(p) = std::fs::canonicalize(path) {
        return p;
    }
    match (path.parent().and_then(|d| std::fs::canonicalize(d).ok()), path.file_name()) {
        (Some(dir), Some(name)) => dir.join(name),
        _ => path.to_path_buf(),
    }
}

/// The copies of findings in text config files, as LSP diagnostics by the
/// text file's URI, their ranges taken from the file on disk, the text the
/// engine read (SR-ARCHITECTURE decision 15d).
fn copies_by_uri(root: &Path, copies: &[Diagnostic]) -> BTreeMap<String, Vec<lsp_types::Diagnostic>> {
    let mut out: BTreeMap<String, Vec<lsp_types::Diagnostic>> = BTreeMap::new();
    for d in copies {
        let path = root.join(&d.file);
        let range = match std::fs::read_to_string(&path) {
            Ok(text) => Utf16Index::new(&text).range(d.start_byte, d.end_byte),
            Err(_) => Range {
                start: lsp_types::Position { line: d.start.line.saturating_sub(1), character: d.start.column.saturating_sub(1) },
                end: lsp_types::Position { line: d.end.line.saturating_sub(1), character: d.end.column.saturating_sub(1) },
            },
        };
        out.entry(uri::from_path(&canonical(&path))).or_default().push(lsp_diagnostic(d, range));
    }
    out
}

fn lsp_diagnostic(d: &Diagnostic, range: Range) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range,
        severity: Some(match d.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warn => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(d.rule.clone())),
        source: Some("codelaw".into()),
        message: d.message.clone(),
        data: Some(serde_json::json!({ "fix": d.fix.name(), "source": d.source })),
        ..Default::default()
    }
}

fn workspace_edit(uri: &Uri, edits: Vec<TextEdit>) -> WorkspaceEdit {
    WorkspaceEdit { changes: Some(HashMap::from([(uri.clone(), edits)])), ..Default::default() }
}

/// A rule's hover text: its title, summary, Why and fix (SR-EDITOR
/// decision 1, from the rule file of SR-RULE-FORMAT).
fn explain(project: Option<&Project>, d: &Diagnostic) -> String {
    let mut s = format!("**{}**", d.rule);
    if let Some(r) = project.and_then(|p| p.rules.get(&d.rule)) {
        let r = &r.rule;
        s.push_str(&format!(" — {}\n\n{}", r.title, r.summary));
        if !r.why.is_empty() {
            s.push_str(&format!("\n\n**Why:** {}", r.why));
        }
    } else if let Some(e) = codelaw_rules::engine_rule(&d.rule) {
        s.push_str(&format!(" — {}", e.title));
    }
    s.push_str(&format!("\n\nfix: `{}`", d.fix));
    if d.fix == FixClass::Manual && !d.steps.is_empty() {
        for (i, step) in d.steps.iter().enumerate() {
            s.push_str(&format!("\n{}. {step}", i + 1));
        }
    }
    s
}

// The server's own unit tests write a fixture repo, so they live outside
// `src/`, which `tests/server.rs` checks has no write path.
#[cfg(test)]
#[path = "../tests/unit/tree_cache.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/unit/logging.rs"]
mod log_tests;
