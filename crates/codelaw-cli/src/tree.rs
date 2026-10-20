//! `codelaw tree`: the syntax tree the engine sees (SR-RULE-AUTHORING
//! decision 3), named nodes with their fields and positions.

use crate::fail;
use clap::Args;
use codelaw_core::Language;
use codelaw_report::Format;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub struct TreeArgs {
    /// The file to show.
    file: Option<PathBuf>,
    /// The language of `-e` code (or to override the file's).
    #[arg(long)]
    lang: Option<String>,
    /// Code to parse instead of a file.
    #[arg(short = 'e', long = "expr")]
    expr: Option<String>,
    /// Only the nodes on this line.
    #[arg(long)]
    line: Option<u32>,
    /// text or json.
    #[arg(long, default_value = "text")]
    format: String,
}

pub fn tree(args: TreeArgs) -> ExitCode {
    let lang = match args.lang.as_deref().map(|l| Language::from_name(l).or_else(|| Language::from_fence_tag(l))) {
        Some(None) => return fail(format!("unknown language `{}`", args.lang.unwrap_or_default())),
        Some(Some(l)) => Some(l),
        None => None,
    };
    let (text, language) = match (&args.expr, &args.file) {
        (Some(code), _) => match lang {
            Some(l) => (code.clone(), l),
            None => return fail("`-e` needs `--lang`"),
        },
        (None, Some(file)) => {
            let Some(language) = lang.or_else(|| Language::from_path(file)) else {
                return fail("unknown file type; pass --lang");
            };
            match std::fs::read_to_string(file) {
                Ok(t) => (t, language),
                Err(e) => return fail(format!("{}: {e}", file.display())),
            }
        }
        (None, None) => return fail("give a file, or `--lang <l> -e \"<code>\"`"),
    };
    match crate::format(&args.format) {
        Ok(Format::Json) => println!("{}", serde_json::to_string_pretty(&codelaw_report::tree_json(language, &text, args.line)).expect("json")),
        Ok(_) => print!("{}", codelaw_report::tree_text(language, &text, args.line)),
        Err(e) => return fail(e),
    }
    ExitCode::SUCCESS
}
