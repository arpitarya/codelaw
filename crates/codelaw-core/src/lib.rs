//! Foundation types shared by every codelaw crate.
//!
//! Offsets are bytes internally; output positions are 1-based line and
//! column in characters (SR-ARCHITECTURE decision 13).

mod diagnostic;
mod language;
pub mod process;
pub mod similar;
pub mod template;
mod text;
pub mod text_config;

pub use diagnostic::{Diagnostic, FixClass, Severity, Skipped, TextEdit};
pub use language::Language;
pub use text::{LineIndex, Position, normalize_path};
pub use text_config::{TEXT_CONFIG_FILES, is_text_config};

/// The engine version. One version for everything (L6 decision 9).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
