//! `--timings` (SR-CLI-MCP decision 2; SR-RULE-AUTHORING decision 3):
//! where a run spent its time — each phase, with how often it ran, and each
//! rule. Off by default; when off nothing is measured.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// The phases of a run (SR-ARCHITECTURE decision 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Compile,
    Discover,
    Read,
    Parse,
    Semantic,
    Rules,
    Suppress,
}

impl Phase {
    pub const ALL: [Phase; 7] = [Phase::Compile, Phase::Discover, Phase::Read, Phase::Parse, Phase::Semantic, Phase::Rules, Phase::Suppress];

    pub fn name(self) -> &'static str {
        match self {
            Phase::Compile => "compile",
            Phase::Discover => "discover",
            Phase::Read => "read",
            Phase::Parse => "parse",
            Phase::Semantic => "semantic",
            Phase::Rules => "rules",
            Phase::Suppress => "suppress",
        }
    }
}

#[derive(Default)]
struct Clock {
    nanos: AtomicU64,
    count: AtomicU64,
}

impl Clock {
    fn add(&self, nanos: u64) {
        self.nanos.fetch_add(nanos, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

/// The counters of one run, shared by every worker thread.
pub struct Timings {
    started: Instant,
    phases: [Clock; 7],
    rules: Vec<Clock>,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

/// Adds the time until it is dropped to one phase.
pub(crate) struct Guard<'a> {
    clock: &'a Clock,
    start: Instant,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.clock.add(self.start.elapsed().as_nanos() as u64);
    }
}

impl Timings {
    pub(crate) fn new(rules: usize) -> Timings {
        Timings {
            started: Instant::now(),
            phases: Default::default(),
            rules: (0..rules).map(|_| Clock::default()).collect(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    pub(crate) fn guard(&self, phase: Phase) -> Guard<'_> {
        Guard { clock: &self.phases[phase as usize], start: Instant::now() }
    }

    pub(crate) fn add(&self, phase: Phase, since: Instant) {
        self.phases[phase as usize].add(since.elapsed().as_nanos() as u64);
    }

    pub(crate) fn rule(&self, index: usize, since: Instant) {
        if let Some(c) = self.rules.get(index) {
            c.add(since.elapsed().as_nanos() as u64);
        }
    }

    pub(crate) fn cache(&self, hit: bool) {
        if hit { &self.cache_hits } else { &self.cache_misses }.fetch_add(1, Ordering::Relaxed);
    }
}

/// One line of the phase table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseTiming {
    pub phase: &'static str,
    /// Summed over worker threads.
    pub nanos: u64,
    pub count: u64,
}

/// One rule's time, summed over files and threads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleTiming {
    pub rule: String,
    pub nanos: u64,
    /// Node visits (and fix computations) the rule was timed for.
    pub calls: u64,
}

/// What `--timings` prints.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimingReport {
    /// From loading the project to the report.
    pub wall_nanos: u64,
    pub threads: usize,
    pub phases: Vec<PhaseTiming>,
    /// Slowest first; rules that never ran are left out.
    pub rules: Vec<RuleTiming>,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl crate::Project {
    /// The timings so far, when the project was loaded with `timings` on.
    pub fn timings(&self) -> Option<TimingReport> {
        let t = self.timings.as_ref()?;
        let phases = Phase::ALL
            .iter()
            .map(|p| {
                let c = &t.phases[*p as usize];
                PhaseTiming { phase: p.name(), nanos: c.nanos.load(Ordering::Relaxed), count: c.count.load(Ordering::Relaxed) }
            })
            .collect();
        let mut rules: Vec<RuleTiming> = self
            .compiled
            .iter()
            .zip(&t.rules)
            .map(|(r, c)| RuleTiming { rule: r.id().to_owned(), nanos: c.nanos.load(Ordering::Relaxed), calls: c.count.load(Ordering::Relaxed) })
            .filter(|r| r.calls > 0)
            .collect();
        rules.sort_by(|a, b| b.nanos.cmp(&a.nanos).then_with(|| a.rule.cmp(&b.rule)));
        Some(TimingReport {
            wall_nanos: t.started.elapsed().as_nanos() as u64,
            threads: self.pool.current_num_threads(),
            phases,
            rules,
            cache_hits: t.cache_hits.load(Ordering::Relaxed),
            cache_misses: t.cache_misses.load(Ordering::Relaxed),
        })
    }
}
