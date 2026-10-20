//! Fixes: the edit primitives, the apply loop and the verifier.

mod apply;
mod external;
mod primitives;

pub use apply::{Candidate, Failure, LoopEnd, LoopOutcome, LoopTrace, MAX_PASSES, TraceEvent, apply_edits, fix_loop, fix_loop_traced, overlaps};
pub use external::{ExternalCall, request_json, run_external};
pub use primitives::{FixContext, FixSpec, Place, Primitive, Target, edits_for, group_edits, parse_fix};

#[cfg(test)]
mod tests;
