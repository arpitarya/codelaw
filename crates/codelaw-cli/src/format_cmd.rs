//! `codelaw format [--check] [--diff] [--format text|json|github|gitlab|junit|checkstyle] [--import-config] <paths>`
//! (SR-FORMATTER decision 1; SR-CLI-MCP decisions 3 and 4).

use crate::{Common, Ctx, Output, engine_options_in, json_line, load, timings, with_timings};
use codelaw_report::Format;
use clap::Args;
use codelaw_config::Home;
use codelaw_core::{Diagnostic, FixClass, Position, Severity, TextEdit};
use codelaw_engine::{FormatReport, FormatRunOptions};
use codelaw_format::import::{Imported, PRETTIER_FILES, PRETTIER_PROGRAMS};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct FormatArgs {
    /// Files and folders; the project root when none.
    pub paths: Vec<PathBuf>,
    #[command(flatten)]
    common: Common,
    /// Change nothing; exit 1 if any file is not formatted.
    #[arg(long)]
    check: bool,
    /// Change nothing; print what formatting would change, as a unified diff.
    #[arg(long)]
    diff: bool,
    /// text or json (the shape of the npm and Python `format`), or a CI format: github, gitlab, junit or checkstyle.
    #[arg(long, default_value = "text")]
    format: String,
    /// Read `.prettierrc*`, `[tool.ruff]`/`[tool.ruff.format]` and
    /// `[tool.black]` once and write the equivalent `[format]` keys to the
    /// config; formats nothing.
    #[arg(long, conflicts_with_all = ["check", "diff"])]
    import_config: bool,
}

/// What `--import-config` wrote, kept and skipped.
pub struct ImportReport {
    pub config: PathBuf,
    pub imported: Imported,
    /// The keys written.
    pub written: Vec<String>,
    /// The keys the config already set, so left as they are.
    pub kept: Vec<String>,
}

/// `--import-config` in `root`: read the formatter configs there and add
/// each `[format]` key the config does not already set. Running it again
/// writes nothing, because every key is then set.
pub fn import_config(root: &Path, config: Option<(&Path, Home)>) -> Result<ImportReport, String> {
    let mut imported = Imported::default();
    for name in PRETTIER_FILES {
        if let Ok(text) = std::fs::read_to_string(root.join(name)) {
            imported.prettier(name, &text)?;
        }
    }
    for name in PRETTIER_PROGRAMS {
        if root.join(name).is_file() {
            imported.skipped.push(format!("{name}: a program, which codelaw does not run; copy its settings by hand"));
        }
    }
    if let Ok(text) = std::fs::read_to_string(root.join("package.json"))
        && let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&text)
        && let Some(p) = pkg.get("prettier")
    {
        imported.prettier_object("package.json \"prettier\"", p)?;
    }
    if let Ok(text) = std::fs::read_to_string(root.join("pyproject.toml")) {
        imported.pyproject(&text)?;
    }
    let (path, home) = match config {
        Some((p, h)) => (p.to_path_buf(), h),
        None => (root.join("codelaw.toml"), Home::CodelawToml),
    };
    let mut report = ImportReport { config: path.clone(), imported, written: Vec::new(), kept: Vec::new() };
    if report.imported.keys.is_empty() {
        return Ok(report);
    }
    let prefix: &[&str] = match home {
        Home::CodelawToml => &[],
        Home::Pyproject => &["tool", "codelaw"],
        Home::PackageJson => return Err(format!("the config is in {}; --import-config writes TOML, so move it to codelaw.toml first", path.display())),
    };
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut table = doc.as_table_mut();
    for part in prefix.iter().chain(&["format"]) {
        let entry = table.entry(part).or_insert_with(|| {
            let mut t = toml_edit::Table::new();
            t.set_implicit(true);
            toml_edit::Item::Table(t)
        });
        table = entry.as_table_mut().ok_or_else(|| format!("{}: `{part}` is not a table", path.display()))?;
    }
    table.set_implicit(false);
    for (key, value) in &report.imported.keys {
        let (sub, name) = match key.split_once('.') {
            Some((sub, name)) => (Some(sub), name),
            None => (None, key.as_str()),
        };
        let target = match sub {
            None => &mut *table,
            Some(sub) => {
                let t = table.entry(sub).or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()));
                t.as_table_mut().ok_or_else(|| format!("{}: `format.{sub}` is not a table", path.display()))?
            }
        };
        if target.contains_key(name) {
            report.kept.push(key.clone());
            continue;
        }
        let v = match value {
            toml::Value::Integer(n) => toml_edit::value(*n),
            toml::Value::String(s) => toml_edit::value(s.as_str()),
            toml::Value::Boolean(b) => toml_edit::value(*b),
            other => return Err(format!("{key}: cannot write {other}")),
        };
        target.insert(name, v);
        report.written.push(key.clone());
    }
    if !report.written.is_empty() {
        std::fs::write(&path, doc.to_string()).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(report)
}

fn print_import(r: &ImportReport) -> Output {
    let mut out = Output::default();
    if r.imported.sources.is_empty() {
        out.stderr.push_str("no .prettierrc*, [tool.ruff] or [tool.black] settings found\n");
    } else {
        out.stderr.push_str(&format!("read {}\n", r.imported.sources.join(", ")));
    }
    for k in &r.written {
        out.stdout.push_str(&format!("wrote [format] {k}\n"));
    }
    for k in &r.kept {
        out.stdout.push_str(&format!("kept [format] {k}: {} already sets it\n", r.config.display()));
    }
    for s in &r.imported.skipped {
        out.stdout.push_str(&format!("not imported: {s}\n"));
    }
    if !r.written.is_empty() {
        out.stderr.push_str(&format!("{} [format] keys written to {}; from now on it is the only source\n", r.written.len(), r.config.display()));
    }
    out
}

/// A unified diff of one file's formatting.
pub fn unified_diff(file: &str, before: &str, after: &str) -> String {
    similar::TextDiff::from_lines(before, after).unified_diff().context_radius(3).header(&format!("a/{file}"), &format!("b/{file}")).to_string()
}

/// 0: every file is formatted (or was just formatted); 1: `--check` or
/// `--diff` found an unformatted file, or a file could not be formatted or
/// written; 2: the run could not start.
pub fn exit_code(report: &FormatReport, check: bool) -> u8 {
    if report.check.internal_errors() > 0 {
        2
    } else if (check && !report.files.is_empty()) || !report.check.diagnostics.is_empty() {
        1
    } else {
        0
    }
}

/// What the CI formats report for `format`: the run's diagnostics, and under
/// `--check` a `format/unformatted` finding for each file that would change
/// (the finding `check` reports for it).
fn ci_report(report: &FormatReport, check: bool) -> codelaw_engine::CheckReport {
    let mut r = report.check.clone();
    if check {
        for f in &report.files {
            r.diagnostics.push(Diagnostic {
                rule: "format/unformatted".into(),
                severity: Severity::Warn,
                file: f.file.clone(),
                start_byte: 0,
                end_byte: 0,
                start: Position { line: 1, column: 1 },
                end: Position { line: 1, column: 1 },
                message: "The file is not formatted; `codelaw format` formats it".into(),
                fix: FixClass::Auto,
                edits: vec![TextEdit::new(0, f.original.len(), &f.formatted)],
                steps: vec![],
                source: "engine".into(),
                note: None,
                data: None,
            });
        }
        r.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    }
    r
}

impl FormatArgs {
    pub fn prefer_warm(&self) -> bool {
        self.common.prefer_warm
    }
}

pub fn format(args: FormatArgs, ctx: &Ctx) -> Output {
    let fmt = match crate::format(&args.format) {
        Ok(f @ (Format::Text | Format::Json | Format::Github | Format::Gitlab | Format::Junit | Format::Checkstyle)) => f,
        Ok(_) => return Output::fail(format!("`format` takes --format text, json, github, gitlab, junit or checkstyle, not `{}`", args.format)),
        Err(e) => return Output::fail(e),
    };
    if args.diff && !matches!(fmt, Format::Text) {
        return Output::fail(format!("--diff prints a diff; it takes no --format {}", args.format));
    }
    let json = fmt == Format::Json;
    let options = match engine_options_in(&args.common, ctx.env.clone()) {
        Ok(o) => o,
        Err(e) => return Output::fail(e),
    };
    let mut project = match load(&args.paths, options) {
        Ok(p) => p,
        Err(e) => return Output::fail(e),
    };
    if args.import_config {
        let config = project.config.files.first().map(|f| (f.path.as_path(), f.home));
        return match import_config(&project.config.root_dir, config) {
            Ok(r) => print_import(&r),
            Err(e) => Output::fail(format!("--import-config: {e}")),
        };
    }
    let check = args.check || args.diff;
    let report = match project.format(&args.paths, &FormatRunOptions { check }) {
        Ok(r) => r,
        Err(e) => return Output::fail(e),
    };
    let mut out = Output::default();
    if json {
        out.stdout = json_line(&with_timings(codelaw_report::format_json(&report, check), &project));
        out.code = exit_code(&report, check);
        return out;
    }
    if !matches!(fmt, Format::Text | Format::Json) {
        out.stdout = crate::ci_output(fmt, &ci_report(&report, check));
        out.code = exit_code(&report, check);
        return out;
    }
    for f in &report.files {
        if args.diff {
            out.stdout.push_str(&unified_diff(&f.file, &f.original, &f.formatted));
        } else if args.check {
            out.stdout.push_str(&format!("would format {}\n", f.file));
        } else if f.written {
            out.stdout.push_str(&format!("formatted {}\n", f.file));
        }
    }
    if !report.check.diagnostics.is_empty() {
        out.stdout.push_str(&codelaw_report::check_text(&report.check, ctx.color));
    }
    let n = report.files.len();
    let files = report.check.files_checked;
    out.stderr.push_str(&match (check, n) {
        (_, 0) => format!("{files} files already formatted\n"),
        (true, _) => format!("{n} of {files} files would be formatted\n"),
        (false, _) => format!("formatted {n} of {files} files\n"),
    });
    timings(&project, Format::Text, &mut out);
    out.code = exit_code(&report, check);
    out
}
