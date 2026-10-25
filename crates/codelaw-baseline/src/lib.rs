//! Baselines and deprecations (SR-BASELINE): a committed, count-based
//! baseline that grandfathers a rule's existing findings until a date,
//! turns them into errors when their file is touched, and can only shrink;
//! and the deprecations registry that is its first user.
//!
//! Everything here is a function of the files, the as-of date and the
//! change set it is given (L5 decision 2): no clock, no git, no network.
//! The engine supplies those inputs.

pub mod apply;
pub mod date;
pub mod exception;
pub mod file;
pub mod ratchet;
pub mod registry;

pub use apply::{AddOptions, Context, Note, Occurrence, Outcome, Stale, add, apply, counts, prune};
pub use date::Date;
pub use exception::{AUTHOR_RULE, EXPIRED_RULE, EXPIRING_RULE, Exceptions, Expiry, WITHOUT_EXPIRY_RULE};
pub use file::{BaselineFile, Baselines, Entry, OnTouch};
pub use ratchet::{CODEOWNERS_PATHS, codeowners_covers, loosened};
pub use registry::{Deprecation, Policy, REGISTRY_REL, Registry, Rewrite};

/// The engine's own deprecation rule (SR-RULE-PACKS decision 1).
pub const DEPRECATED_RULE: &str = "graph/no-deprecated-usage";

/// The engine rule a stale entry is reported under.
pub const STALE_RULE: &str = "codelaw/stale-baseline";

#[cfg(test)]
mod tests;
