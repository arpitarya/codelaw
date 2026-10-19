//! `codelaw tree`: the syntax tree the engine sees (SR-RULE-AUTHORING
//! decision 3), named nodes with their fields and positions.

use crate::fail;
use clap::Args;
use codelaw_core::{Language, LineIndex};
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
}

fn render(node: tree_sitter::Node<'_>, field: Option<&str>, depth: usize, text: &str, index: &LineIndex<'_>, line: Option<u32>, out: &mut String) {
    let start = index.position(node.start_byte());
    let end = index.position(node.end_byte());
    let on_line = line.is_none_or(|l| start.line <= l && l <= end.line);
    if !on_line {
        return;
    }
    if node.is_named() || node.is_missing() {
        let label = field.map(|f| format!("{f}: ")).unwrap_or_default();
        let leaf = node.named_child_count() == 0;
        let snippet = if leaf {
            let t = &text[node.byte_range()];
            let t: String = t.chars().take(40).collect();
            format!("  {:?}", t)
        } else {
            String::new()
        };
        let flag = if node.is_error() { " ERROR" } else if node.is_missing() { " MISSING" } else { "" };
        out.push_str(&format!(
            "{}{label}{}{flag}  [{}:{}-{}:{}]{snippet}\n",
            "  ".repeat(depth),
            node.kind(),
            start.line,
            start.column,
            end.line,
            end.column
        ));
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let next_depth = if node.is_named() { depth + 1 } else { depth };
            render(cursor.node(), cursor.field_name(), next_depth, text, index, line, out);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
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
    let root = codelaw_lang::parse(language, &text);
    let index = LineIndex::new(&text);
    let mut out = String::new();
    render(root.root().get_inner_node(), None, 0, &text, &index, args.line, &mut out);
    print!("{out}");
    ExitCode::SUCCESS
}
