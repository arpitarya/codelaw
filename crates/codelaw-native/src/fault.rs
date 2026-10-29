//! Rules for tests only: `test/panic` fails on purpose, for the tests of
//! SR-ARCHITECTURE decision 25 (a rule failure is isolated); `test/node-version`
//! and `test/elsewhere-outside-list` report a finding in a plain-text config
//! file, for decision 15d and SR-LAW-3 decision 10. Compiled only with the
//! `fault-injection` feature, which no shipped build enables: the engine's
//! and the CLI's tests turn it on as a dev-dependency feature.

use crate::{Elsewhere, NativeContext, NativeFinding, NativeRule, resolve_path};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;

/// `test/panic`: panics on every JavaScript program.
pub struct Panic;

impl NativeRule for Panic {
    fn id(&self) -> &'static str {
        "test/panic"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Javascript]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn check(&self, _node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, _out: &mut Vec<NativeFinding>) {
        panic!("test/panic always panics");
    }
}

/// `test/node-version`: a `package.json` whose `engines.node` does not
/// start with the major version its folder's `.nvmrc` names; the finding is
/// on the `engines.node` value and on the `.nvmrc` line. A stand-in for
/// W-129's runtime rule, read crudely from the text.
pub struct NodeVersion;

/// `.nvmrc`'s path beside `package.json`, its text, and the major version
/// on its first line.
fn nvmrc(ctx: &NativeContext<'_>) -> Option<(String, String, String)> {
    let path = resolve_path(ctx.file.path(), ".nvmrc")?;
    let text = ctx.file.read(&path)?;
    let major: String = text.lines().next()?.trim().trim_start_matches('v').chars().take_while(char::is_ascii_digit).collect();
    (!major.is_empty()).then_some((path, text, major))
}

impl NativeRule for NodeVersion {
    fn id(&self) -> &'static str {
        "test/node-version"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Json]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, _node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        if !ctx.file.path().ends_with("package.json") {
            return;
        }
        let Some((_, _, major)) = nvmrc(ctx) else { return };
        let src = ctx.source;
        let Some(engines) = src.find("\"engines\"") else { return };
        let Some(key) = src[engines..].find("\"node\"").map(|i| engines + i + 6) else { return };
        let Some(open) = src[key..].find('"').map(|i| key + i) else { return };
        let Some(close) = src[open + 1..].find('"').map(|i| open + 1 + i) else { return };
        let range = &src[open + 1..close];
        let wanted: String = range.trim_start_matches(['>', '=', '^', '~', ' ']).chars().take_while(char::is_ascii_digit).collect();
        if wanted != major {
            out.push(NativeFinding {
                start: open,
                end: close + 1,
                message: format!("`engines.node` is {range}, but `.nvmrc` says {major}"),
                edits: Vec::new(),
            });
        }
    }
    fn elsewhere(&self, _finding: &NativeFinding, ctx: &NativeContext<'_>) -> Vec<Elsewhere> {
        nvmrc(ctx).and_then(|(path, text, _)| Elsewhere::line(path, &text, 1)).into_iter().collect()
    }
}

/// `test/elsewhere-outside-list`: one finding per JSON document, said to be
/// also at `notes.txt`, which is not a text config file: the engine refuses
/// it as the rule's failure.
pub struct OutsideList;

impl NativeRule for OutsideList {
    fn id(&self) -> &'static str {
        "test/elsewhere-outside-list"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Json]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, _node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        out.push(NativeFinding { start: 0, end: 1, message: "always".into(), edits: Vec::new() });
    }
    fn elsewhere(&self, _finding: &NativeFinding, _ctx: &NativeContext<'_>) -> Vec<Elsewhere> {
        vec![Elsewhere::new("notes.txt", 0, 1)]
    }
}

static PANIC: Panic = Panic;
static NODE_VERSION: NodeVersion = NodeVersion;
static OUTSIDE_LIST: OutsideList = OutsideList;

pub fn lookup(id: &str) -> Option<&'static dyn NativeRule> {
    [&PANIC as &dyn NativeRule, &NODE_VERSION, &OUTSIDE_LIST].into_iter().find(|r| r.id() == id)
}
