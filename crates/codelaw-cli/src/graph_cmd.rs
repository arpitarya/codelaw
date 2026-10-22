//! `codelaw graph refs` and `codelaw graph why-unused` (SR-PROJECT-GRAPH
//! decision 6): the project-rule debug commands.

use crate::{Common, engine_options, fail, load};
use clap::{Args, Subcommand};
use codelaw_graph::DeclKind;
use std::process::ExitCode;

#[derive(Args)]
pub struct GraphArgs {
    #[command(subcommand)]
    action: GraphAction,
}

#[derive(Subcommand)]
enum GraphAction {
    /// Every use of a symbol and the entry points that keep its module.
    Refs {
        /// `<package or path>#<export>` or `<module>:<name>`.
        symbol: String,
        #[command(flatten)]
        common: Common,
    },
    /// Each clause of `graph/no-unused-exports` for a symbol, and why it held or failed.
    WhyUnused {
        symbol: String,
        #[command(flatten)]
        common: Common,
    },
}

pub fn graph(args: GraphArgs) -> ExitCode {
    let (symbol, common, why) = match args.action {
        GraphAction::Refs { symbol, common } => (symbol, common, false),
        GraphAction::WhyUnused { symbol, common } => (symbol, common, true),
    };
    let options = match engine_options(&common) {
        Ok(o) => o,
        Err(e) => return fail(e),
    };
    let mut project = match load(&[], options) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let graph = match project.graph() {
        Ok(g) => g,
        Err(e) => return fail(e),
    };
    let out = if why {
        let kinds = [DeclKind::Function, DeclKind::Class, DeclKind::Variable, DeclKind::Type, DeclKind::Interface, DeclKind::Enum];
        codelaw_graph::why_unused(&graph, &symbol, &kinds, &["public".to_owned(), "keep".to_owned()])
    } else {
        codelaw_graph::refs(&graph, &symbol)
    };
    match out {
        Ok(s) => {
            print!("{s}");
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}
