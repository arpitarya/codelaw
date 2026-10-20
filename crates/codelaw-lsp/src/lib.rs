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
//! Not yet here, and why: incremental re-parse (W-15; every change
//! re-checks the whole document), project rules and workspace diagnostics
//! (the graph is not built), pull diagnostics, the warm-process socket
//! (SR-ARCHITECTURE decision 21).

pub mod format;
pub mod position;
pub mod uri;

use codelaw_core::{Diagnostic, FixClass, Severity};
use codelaw_engine::{EngineOptions, FileEntry, Project};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument, LogMessage, Notification as _,
    PublishDiagnostics, ShowMessage,
};
use lsp_types::request::{CodeActionRequest, Formatting, HoverRequest, Request as _};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, DiagnosticRelatedInformation,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentFormattingParams, Hover, HoverContents, HoverParams, HoverProviderCapability, Location, LogMessageParams, MarkupContent, MarkupKind, MessageType,
    NumberOrString, OneOf, PositionEncodingKind, PublishDiagnosticsParams, Range, ServerCapabilities, ServerInfo, ShowMessageParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TextEdit, Uri, WorkspaceEdit,
};
use position::Utf16Index;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

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
            change: Some(TextDocumentSyncKind::FULL),
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
        ..Default::default()
    }
}

/// Run the server on a connection: the handshake, then the message loop.
pub fn serve(connection: &Connection) -> Result<(), Error> {
    let (id, params) = connection.initialize_start()?;
    let params: lsp_types::InitializeParams = serde_json::from_value(params)?;
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
}

impl<'c> Server<'c> {
    fn new(connection: &'c Connection, roots: Vec<PathBuf>) -> Self {
        Server { connection, roots, docs: HashMap::new(), projects: HashMap::new() }
    }

    fn send(&self, msg: impl Into<Message>) -> Result<(), Error> {
        self.connection.sender.send(msg.into()).map_err(|e| Error::from(e.to_string()))
    }

    fn notify<N: lsp_types::notification::Notification>(&self, params: N::Params) -> Result<(), Error> {
        self.send(Notification::new(N::METHOD.to_owned(), params))
    }

    fn log(&self, typ: MessageType, message: String) -> Result<(), Error> {
        self.notify::<LogMessage>(LogMessageParams { typ, message })
    }

    // ---- notifications -------------------------------------------------

    fn notification(&mut self, n: Notification) -> Result<(), Error> {
        match n.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let p: DidOpenTextDocumentParams = serde_json::from_value(n.params)?;
                let d = p.text_document;
                let doc = Doc { uri: d.uri, version: d.version, text: d.text, target: None, diagnostics: vec![] };
                let key = doc.uri.as_str().to_owned();
                self.docs.insert(key.clone(), doc);
                self.check(&key)?;
            }
            DidChangeTextDocument::METHOD => {
                let p: DidChangeTextDocumentParams = serde_json::from_value(n.params)?;
                let key = p.text_document.uri.as_str().to_owned();
                // Full sync: the last change is the whole text.
                if let (Some(doc), Some(change)) = (self.docs.get_mut(&key), p.content_changes.into_iter().last()) {
                    doc.text = change.text;
                    doc.version = p.text_document.version;
                    self.check(&key)?;
                }
            }
            DidSaveTextDocument::METHOD => {
                let p: DidSaveTextDocumentParams = serde_json::from_value(n.params)?;
                let path = uri::to_path(p.text_document.uri.as_str());
                if path.as_deref().is_some_and(reloads) {
                    self.reload()?;
                } else {
                    self.check(p.text_document.uri.as_str())?;
                }
            }
            DidCloseTextDocument::METHOD => {
                let p: DidCloseTextDocumentParams = serde_json::from_value(n.params)?;
                if let Some(doc) = self.docs.remove(p.text_document.uri.as_str()) {
                    self.notify::<PublishDiagnostics>(PublishDiagnosticsParams { uri: doc.uri, diagnostics: vec![], version: None })?;
                }
            }
            // Config, rules or baseline changed on disk: reload live
            // (SR-EDITOR decision 1).
            DidChangeWatchedFiles::METHOD => self.reload()?,
            _ => {}
        }
        Ok(())
    }

    /// Drop every loaded project and re-check every open document.
    fn reload(&mut self) -> Result<(), Error> {
        self.projects.clear();
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
                self.log(MessageType::ERROR, format!("codelaw: {e}"))?;
                return Ok(None);
            }
        };
        let key = config.config_dir().to_path_buf();
        if !self.projects.contains_key(&key) {
            // The editor reads no cache and writes none.
            let options = EngineOptions { no_cache: true, ..EngineOptions::from_env() };
            let loaded = Project::from_config(config, options).map_err(|e| e.to_string());
            match &loaded {
                Ok(p) => {
                    for w in &p.warnings {
                        self.log(MessageType::WARNING, format!("codelaw: {w}"))?;
                    }
                }
                Err(e) => {
                    self.notify::<ShowMessage>(ShowMessageParams { typ: MessageType::ERROR, message: format!("codelaw: {e}") })?;
                }
            }
            self.projects.insert(key.clone(), loaded);
        }
        Ok(matches!(self.projects.get(&key), Some(Ok(_))).then_some(key))
    }

    /// Re-check one open document and publish what it finds.
    fn check(&mut self, key: &str) -> Result<(), Error> {
        let Some(doc) = self.docs.get(key) else { return Ok(()) };
        let target = match uri::to_path(doc.uri.as_str()) {
            Some(path) => {
                let path = canonical(&path);
                match self.project_for(&path)? {
                    Some(pk) => match &self.projects[&pk] {
                        Ok(project) => project.entry_for(&path).map(|e| (pk, e)),
                        Err(_) => None,
                    },
                    None => None,
                }
            }
            None => None,
        };
        let doc = self.docs.get(key).expect("an open document");
        let project = target.as_ref().and_then(|(pk, _)| self.projects[pk].as_ref().ok());
        let diagnostics = match (&target, project) {
            (Some((_, entry)), Some(project)) => project.check_document(entry, doc.body().0),
            _ => vec![],
        };
        let lsp = to_lsp(doc, target.as_ref().map(|t| &t.1), project.map(Project::root), &diagnostics);
        let params = PublishDiagnosticsParams { uri: doc.uri.clone(), diagnostics: lsp, version: Some(doc.version) };
        let doc = self.docs.get_mut(key).expect("an open document");
        doc.target = target;
        doc.diagnostics = diagnostics;
        self.notify::<PublishDiagnostics>(params)
    }

    // ---- requests ------------------------------------------------------

    fn request(&mut self, req: Request) -> Result<(), Error> {
        let id = req.id.clone();
        let result = match req.method.as_str() {
            CodeActionRequest::METHOD => serde_json::from_value(req.params).map(|p| serde_json::to_value(self.code_actions(p))),
            HoverRequest::METHOD => serde_json::from_value(req.params).map(|p| serde_json::to_value(self.hover(p))),
            Formatting::METHOD if format::AVAILABLE => serde_json::from_value(req.params).map(|p| serde_json::to_value(self.formatting(p))),
            m => return self.send(Response::new_err(id, ErrorCode::MethodNotFound as i32, format!("codelaw lsp does not handle `{m}`"))),
        };
        match result {
            Ok(Ok(v)) => self.send(Response::new_ok(id, v)),
            Ok(Err(e)) | Err(e) => self.send(Response::new_err(id, ErrorCode::InvalidParams as i32, e.to_string())),
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
        let own = |d: &&Diagnostic| doc.target.as_ref().is_some_and(|(_, e)| e.rel == d.file);
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

/// Saving these changes what every file's verdict may be.
fn reloads(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    matches!(name, "codelaw.toml" | "pyproject.toml" | "package.json")
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
