//! Rules that fail on purpose, for the tests of SR-ARCHITECTURE decision 25
//! (a rule failure is isolated). Compiled only with the `fault-injection`
//! feature, which no shipped build enables: the engine's and the CLI's
//! tests turn it on as a dev-dependency feature.

use crate::{NativeContext, NativeFinding, NativeRule};
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

static PANIC: Panic = Panic;

pub fn lookup(id: &str) -> Option<&'static dyn NativeRule> {
    (id == PANIC.id()).then_some(&PANIC as &dyn NativeRule)
}
