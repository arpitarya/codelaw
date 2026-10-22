//! `codelaw mcp`: the MCP server (SR-CLI-MCP decision 5).
//!
//! - **stdio only**: newline-delimited JSON-RPC 2.0 on stdin and stdout,
//!   nothing listening on a port (L5 decision 3). Nothing but protocol
//!   messages is ever written to stdout.
//! - **No logic here** (decision 6): every tool calls the engine function
//!   the matching CLI command calls, and returns the same JSON object that
//!   command prints with `--format json` — byte-equal, as the tool result's
//!   text and as its `structuredContent`.
//! - A small hand-written loop rather than an SDK: the protocol surface a
//!   tools-only server needs is `initialize`, `ping`, `tools/list` and
//!   `tools/call`, and no dependency is added (L2).

use codelaw_engine::{EngineError, EngineOptions, FixOptions, Project};
use serde_json::{Map, Value, json};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// The MCP revisions this server speaks, newest first. `initialize`
/// answers with the client's revision when it is one of these.
pub const PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "codelaw lints and fixes JS, TS, Python, JSON, YAML, TOML, Markdown, CSS and HTML against the rules this repo configures. \
Before you write or edit a file, call `rules_for_path` and follow those rules. After you write, call `check_paths` on the files you changed \
and handle each finding by its fix class: `auto` — call `fix_paths`; `suggest` — show the diff and apply it only once accepted; \
`manual` — follow the steps exactly. Never disable or suppress a rule to make a check pass. Every verdict comes from these tools, never from judgment.";

/// One tool: its name, when to call it, and its arguments.
struct Tool {
    name: &'static str,
    description: &'static str,
    schema: fn() -> Value,
}

fn string(desc: &str) -> Value {
    json!({ "type": "string", "description": desc })
}

fn strings(desc: &str) -> Value {
    json!({ "type": "array", "items": { "type": "string" }, "description": desc })
}

fn boolean(desc: &str) -> Value {
    json!({ "type": "boolean", "description": desc })
}

fn object(props: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": props, "required": required, "additionalProperties": false })
}

const PATHS: &str = "Files and folders, relative to the folder the server runs in; the project root when empty.";
const RULE: &str = "Only these rule ids (as `--rule`).";
const LANG: &str = "Only these languages (as `--lang`), e.g. `typescript`, `python`.";
const AS_OF: &str = "Judge baseline and deprecation deadlines on this date, YYYY-MM-DD (as `--as-of`); `HEAD`'s commit date by default.";
const CHANGED_SINCE: &str = "Files changed since the merge base with this git ref count as touched (as `--changed-since`).";
const STAGED: &str = "Files in the git index count as touched (as `--staged`).";

/// SR-CLI-MCP decision 5's tools, each described by when to call it
/// (decision 7).
const TOOLS: &[Tool] = &[
    Tool {
        name: "rules_for_path",
        description: "Call BEFORE you create or edit a file: the rules that apply to that path, one short card each (id, severity, fix class, title). \
Write code that complies; ask `explain_rule` for any rule you need detail on. Same JSON as `codelaw rules --for <path> --format json`.",
        schema: || object(json!({ "path": string("The file you are about to write; it need not exist yet.") }), &["path"]),
    },
    Tool {
        name: "explain_rule",
        description: "Call when you need to know what a rule requires, why, and how to fix it: summary, reason, fix class, the steps of a manual fix, \
the skill that carries it, and the Do/Don't examples. Same JSON as `codelaw explain <id> --format json`.",
        schema: || object(json!({ "id": string("The rule id, e.g. `js/eqeqeq`.") }), &["id"]),
    },
    Tool {
        name: "list_rules",
        description: "Call to see every rule this project loads, with its default severity, fix class and source pack. \
Prefer `rules_for_path` when you are about to write one file. Same JSON as `codelaw rules --format json`.",
        schema: || object(json!({}), &[]),
    },
    Tool {
        name: "check_snippet",
        description: "Call to check code you have not written to disk yet, as if it were the text of `path` (which decides the language and the config). \
Returns the same JSON as `codelaw check <path> --format json` would for a file holding that code.",
        schema: || object(json!({ "path": string("The path the code will live at."), "code": string("The full text of the file.") }), &["path", "code"]),
    },
    Tool {
        name: "check_paths",
        description: "Call AFTER you write: checks files and folders and returns every finding with its fix — `auto` edits, `suggest` edits to show as a diff, \
or `manual` steps. Same JSON as `codelaw check --format json`.",
        schema: || {
            object(
                json!({
                    "paths": strings(PATHS),
                    "rule": strings(RULE),
                    "lang": strings(LANG),
                    "as_of": string(AS_OF),
                    "changed_since": string(CHANGED_SINCE),
                    "staged": boolean(STAGED),
                }),
                &[],
            )
        },
    },
    Tool {
        name: "fix_paths",
        description: "Call when `check_paths` reports `auto` fixes: applies them to the files and returns what is left. `suggest` fixes come back as edits \
to show the user as a diff; apply them only once accepted, by calling again with `unsafe: true`. Same JSON as `codelaw fix --format json`.",
        schema: || {
            object(
                json!({
                    "paths": strings(PATHS),
                    "unsafe": boolean("Apply `suggest` fixes too (as `--unsafe`); only after the user accepted the diff."),
                    "dry_run": boolean("Compute the fixes but write nothing (as `--dry-run`)."),
                    "rule": strings(RULE),
                    "lang": strings(LANG),
                    "as_of": string(AS_OF),
                    "changed_since": string(CHANGED_SINCE),
                    "staged": boolean(STAGED),
                }),
                &[],
            )
        },
    },
    Tool {
        name: "format_snippet",
        description: "Call to get code as `codelaw fix` would write it, without touching disk: the snippet with every `auto` fix applied \
(and formatted, as `fix` formats), plus what is left. `code` in the result is the exact text to write to `path`.",
        schema: || object(json!({ "path": string("The path the code will live at."), "code": string("The full text of the file.") }), &["path", "code"]),
    },
    Tool {
        name: "dump_tree",
        description: "Call when writing or debugging a rule's matcher: the named syntax tree the engine sees for a file, or for `code` in `lang`, \
optionally only the nodes on one line. Same JSON as `codelaw tree --format json`.",
        schema: || {
            object(
                json!({
                    "path": string("A file to parse."),
                    "code": string("Code to parse instead of a file; needs `lang`."),
                    "lang": string("The language of `code`, or to override the file's."),
                    "line": { "type": "integer", "minimum": 1, "description": "Only the nodes on this line." },
                }),
                &[],
            )
        },
    },
    Tool {
        name: "trace_rule",
        description: "Call when a rule fired and you don't see why, or didn't fire when you expected: where the rule comes from, what enabled it, \
which config layer sets it for `path`, whether it applies there, and what it reports in that file. Same JSON as \
`codelaw rules --why <id> --for <path> --format json`.",
        schema: || object(json!({ "id": string("The rule id."), "path": string("The file to trace it on.") }), &["id"]),
    },
];

/// The tool names, in SR-CLI-MCP decision 5's order.
pub fn tool_names() -> Vec<&'static str> {
    TOOLS.iter().map(|t| t.name).collect()
}

fn tools_list() -> Value {
    let tools: Vec<Value> = TOOLS.iter().map(|t| json!({ "name": t.name, "description": t.description, "inputSchema": (t.schema)() })).collect();
    json!({ "tools": tools })
}

/// A tool's arguments, read the way the CLI reads its flags.
struct ArgsView<'a>(&'a Map<String, Value>);

impl ArgsView<'_> {
    fn str(&self, key: &str) -> Result<Option<String>, String> {
        match self.0.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(format!("`{key}` must be a string")),
        }
    }
    fn required(&self, key: &str) -> Result<String, String> {
        self.str(key)?.ok_or_else(|| format!("`{key}` is required"))
    }
    fn list(&self, key: &str) -> Result<Vec<String>, String> {
        match self.0.get(key) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::String(s)) => Ok(vec![s.clone()]),
            Some(Value::Array(a)) => a.iter().map(|v| v.as_str().map(str::to_owned).ok_or_else(|| format!("`{key}` must be a list of strings"))).collect(),
            Some(_) => Err(format!("`{key}` must be a list of strings")),
        }
    }
    fn flag(&self, key: &str) -> Result<bool, String> {
        match self.0.get(key) {
            None | Some(Value::Null) => Ok(false),
            Some(Value::Bool(b)) => Ok(*b),
            Some(_) => Err(format!("`{key}` must be true or false")),
        }
    }
    fn line(&self) -> Result<Option<u32>, String> {
        match self.0.get("line") {
            None | Some(Value::Null) => Ok(None),
            Some(v) => v.as_u64().and_then(|n| u32::try_from(n).ok()).filter(|n| *n > 0).map(Some).ok_or_else(|| "`line` must be a positive integer".into()),
        }
    }
    fn paths(&self) -> Result<Vec<PathBuf>, String> {
        Ok(self.list("paths")?.into_iter().map(PathBuf::from).collect())
    }
    /// `check_paths` and `fix_paths`: `--lang`, `--rule`, and the time
    /// and change-set inputs `--as-of`, `--changed-since`, `--staged`.
    fn run_options(&self) -> Result<EngineOptions, String> {
        let mut o = EngineOptions::with_filters(&self.list("lang")?, &self.list("rule")?)?;
        o.as_of = self.str("as_of")?.or(o.as_of);
        o.changed_since = self.str("changed_since")?;
        o.staged = self.flag("staged")?;
        Ok(o)
    }
}

fn err(e: EngineError) -> String {
    e.to_string()
}

fn project(paths: &[PathBuf], options: EngineOptions) -> Result<Project, String> {
    Project::load_for(paths, options).map_err(err)
}

/// Run one tool: the JSON object its CLI twin prints, or the error its CLI
/// twin would print after `error: `.
pub fn call_tool(name: &str, args: &Map<String, Value>) -> Result<Value, String> {
    let a = ArgsView(args);
    match name {
        "rules_for_path" => {
            let path = PathBuf::from(a.required("path")?);
            let p = project(std::slice::from_ref(&path), EngineOptions::from_env())?;
            let rows = p.rules_for(&path).map_err(err)?;
            Ok(codelaw_report::rules_json(&rows, Some(&codelaw_core::normalize_path(&path))))
        }
        "explain_rule" => {
            let id = a.required("id")?;
            if let Some(e) = codelaw_rules::engine_rule(&id) {
                return Ok(codelaw_report::explain_engine_json(e));
            }
            let p = project(&[], EngineOptions::from_env())?;
            Ok(codelaw_report::explain_json(p.find_rule(&id).map_err(err)?))
        }
        "list_rules" => Ok(codelaw_report::rules_json(&project(&[], EngineOptions::from_env())?.list_rules(), None)),
        "check_snippet" => {
            let path = PathBuf::from(a.required("path")?);
            let code = a.required("code")?;
            let p = project(std::slice::from_ref(&path), EngineOptions::from_env())?;
            Ok(codelaw_report::check_json(&p.check_snippet(&path, &code).map_err(err)?))
        }
        "check_paths" => {
            let paths = a.paths()?;
            let mut p = project(&paths, a.run_options()?)?;
            Ok(codelaw_report::check_json(&p.check(&paths).map_err(err)?))
        }
        "fix_paths" => {
            let paths = a.paths()?;
            let mut p = project(&paths, a.run_options()?)?;
            let options = FixOptions { unsafe_fixes: a.flag("unsafe")?, dry_run: a.flag("dry_run")?, trace: false };
            Ok(codelaw_report::fix_json(&p.fix(&paths, &options).map_err(err)?))
        }
        "format_snippet" => {
            let path = PathBuf::from(a.required("path")?);
            let code = a.required("code")?;
            let p = project(std::slice::from_ref(&path), EngineOptions::from_env())?;
            Ok(codelaw_report::snippet_fix_json(&p.fix_snippet(&path, &code).map_err(err)?))
        }
        "dump_tree" => {
            let lang = match a.str("lang")? {
                Some(l) => Some(
                    codelaw_core::Language::from_name(&l)
                        .or_else(|| codelaw_core::Language::from_fence_tag(&l))
                        .ok_or_else(|| format!("unknown language `{l}`"))?,
                ),
                None => None,
            };
            let (text, language) = match (a.str("code")?, a.str("path")?) {
                (Some(code), _) => (code, lang.ok_or("`code` needs `lang`")?),
                (None, Some(path)) => {
                    let language = lang.or_else(|| codelaw_core::Language::from_path(Path::new(&path))).ok_or("unknown file type; pass `lang`")?;
                    (std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?, language)
                }
                (None, None) => return Err("give `path`, or `lang` and `code`".into()),
            };
            Ok(codelaw_report::tree_json(language, &text, a.line()?))
        }
        "trace_rule" => {
            let id = a.required("id")?;
            let path = a.str("path")?.map(PathBuf::from);
            let mut p = project(path.as_slice(), EngineOptions::from_env())?;
            Ok(codelaw_report::why_json(&p.rule_why(&id, path.as_deref()).map_err(err)?))
        }
        _ => Err(format!("unknown tool `{name}`{}", nearest_tool(name))),
    }
}

fn nearest_tool(name: &str) -> String {
    let best = TOOLS.iter().map(|t| t.name).min_by_key(|t| edit_distance(t, name)).filter(|t| edit_distance(t, name) <= 3);
    best.map(|t| format!("; did you mean `{t}`?")).unwrap_or_default()
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = if ca == *cb { prev } else { 1 + prev.min(row[j]).min(row[j + 1]) };
            prev = cur;
        }
    }
    row[b.len()]
}

/// A `tools/call` result: the JSON as pretty text — the bytes the CLI
/// prints, without its final newline — and as structured content.
fn tool_result(outcome: Result<Value, String>) -> Value {
    match outcome {
        Ok(v) => json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).expect("json") }],
            "structuredContent": v,
            "isError": false,
        }),
        Err(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
    }
}

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;

fn error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message.into() } })
}

fn reply(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn initialize(params: &Value) -> Value {
    let asked = params.get("protocolVersion").and_then(Value::as_str);
    let version = asked.filter(|v| PROTOCOL_VERSIONS.contains(v)).unwrap_or(PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "codelaw", "version": codelaw_core::VERSION },
        "instructions": INSTRUCTIONS,
    })
}

/// Answer one message; `None` for a notification or a response.
pub fn handle(message: &str) -> Option<Value> {
    let msg: Value = match serde_json::from_str(message) {
        Ok(v) => v,
        Err(e) => return Some(error(Value::Null, PARSE_ERROR, format!("not JSON: {e}"))),
    };
    let Some(obj) = msg.as_object() else {
        return Some(error(Value::Null, INVALID_REQUEST, "a message is one JSON-RPC object (batches are not part of MCP)"));
    };
    let Some(method) = obj.get("method").and_then(Value::as_str) else {
        // A response to a request we never send, or garbage with an id.
        return obj.get("id").filter(|_| !obj.contains_key("result") && !obj.contains_key("error")).map(|id| error(id.clone(), INVALID_REQUEST, "no `method`"));
    };
    // Notifications (no id) get no answer: `notifications/initialized`,
    // `notifications/cancelled` and any other.
    let id = obj.get("id")?.clone();
    let params = obj.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => reply(id, initialize(&params)),
        "ping" => reply(id, json!({})),
        "tools/list" => reply(id, tools_list()),
        "tools/call" => {
            let Some(name) = params.get("name").and_then(Value::as_str) else {
                return Some(error(id, INVALID_PARAMS, "`tools/call` needs `name`"));
            };
            if !TOOLS.iter().any(|t| t.name == name) {
                return Some(error(id, INVALID_PARAMS, format!("unknown tool `{name}`{}", nearest_tool(name))));
            }
            let empty = Map::new();
            let args = match params.get("arguments") {
                None | Some(Value::Null) => &empty,
                Some(Value::Object(m)) => m,
                Some(_) => return Some(error(id, INVALID_PARAMS, "`arguments` must be an object")),
            };
            // A rule that panics must not take the session down; the CLI
            // reports the same failure as exit 2.
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call_tool(name, args))) {
                Ok(outcome) => reply(id, tool_result(outcome)),
                Err(_) => error(id, INTERNAL_ERROR, format!("`{name}` failed inside the engine; run the matching `codelaw` command to reproduce it")),
            }
        }
        // A tools-only server: empty lists for the other primitives a
        // client may probe.
        "resources/list" => reply(id, json!({ "resources": [] })),
        "prompts/list" => reply(id, json!({ "prompts": [] })),
        _ => error(id, METHOD_NOT_FOUND, format!("codelaw's MCP server does not implement `{method}`")),
    })
}

/// Serve until stdin closes: one JSON-RPC message per line in, one per
/// line out.
pub fn serve(input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(answer) = handle(&line) {
            serde_json::to_writer(&mut output, &answer)?;
            output.write_all(b"\n")?;
            output.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(msg: Value) -> Value {
        handle(&msg.to_string()).expect("an answer")
    }

    #[test]
    fn initialize_echoes_a_known_revision_and_offers_tools() {
        let r = call(
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": { "name": "t", "version": "0" } } }),
        );
        assert_eq!(r["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(r["result"]["serverInfo"]["name"], "codelaw");
        assert!(r["result"]["capabilities"]["tools"].is_object());
        let r = call(json!({ "jsonrpc": "2.0", "id": 2, "method": "initialize", "params": { "protocolVersion": "1999-01-01" } }));
        assert_eq!(r["result"]["protocolVersion"], PROTOCOL_VERSIONS[0]);
    }

    #[test]
    fn notifications_get_no_answer() {
        assert!(handle(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
    }

    #[test]
    fn every_decision_5_tool_is_listed_with_a_schema_and_a_when() {
        let r = call(json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }));
        let names: Vec<&str> = r["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            ["rules_for_path", "explain_rule", "list_rules", "check_snippet", "check_paths", "fix_paths", "format_snippet", "dump_tree", "trace_rule"]
        );
        for t in r["result"]["tools"].as_array().unwrap() {
            assert_eq!(t["inputSchema"]["type"], "object", "{t}");
            assert!(t["description"].as_str().unwrap().starts_with("Call "), "decision 7: say when to call it: {t}");
        }
    }

    #[test]
    fn errors_name_the_fix() {
        let r = call(json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "check_snipet", "arguments": {} } }));
        assert_eq!(r["error"]["code"], INVALID_PARAMS);
        assert!(r["error"]["message"].as_str().unwrap().contains("did you mean `check_snippet`?"), "{r}");
        let r = call(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": { "name": "explain_rule", "arguments": {} } }));
        assert_eq!(r["result"]["isError"], true);
        assert_eq!(r["result"]["content"][0]["text"], "`id` is required");
        let r = call(json!({ "jsonrpc": "2.0", "id": 3, "method": "nope" }));
        assert_eq!(r["error"]["code"], METHOD_NOT_FOUND);
        assert_eq!(handle("{").unwrap()["error"]["code"], PARSE_ERROR);
    }

    #[test]
    fn dump_tree_parses_code() {
        let v = call_tool("dump_tree", json!({ "code": "x = 1\n", "lang": "python" }).as_object().unwrap()).unwrap();
        assert_eq!(v["language"], "python");
        assert_eq!(v["nodes"][0]["kind"], "module");
    }

    #[test]
    fn serve_answers_line_by_line() {
        let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\n{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n";
        let mut out = Vec::new();
        serve(&input[..], &mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n");
    }
}
