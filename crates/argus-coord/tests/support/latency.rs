//! Cold start timing of one step: a new process every run, so the number is what a hook or a
//! typed command costs. Used by the latency bench; the same spawn code as the differential
//! harness, so the two measure the same environment.

use std::io;
use std::time::{Duration, Instant};

use super::differ::engines::Engines;
use super::differ::exec::{Which, run_step};
use super::differ::homes::Home;
use super::differ::step::Step;
use super::differ::verdict::NOT_PORTED;

/// Runs that are timed but dropped, to warm the file cache the way a second hook finds it.
const WARMUP: usize = 3;

/// p50 and p95 of a set of runs, and what the last run answered.
#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub p50: Duration,
    pub p95: Duration,
    pub runs: usize,
    /// The exit code of the last run.
    pub code: Option<i32>,
}

impl Timing {
    /// Whether the verb is answered by this engine (an exit 120 is a quick refusal, not a
    /// measurement of the verb).
    #[must_use]
    pub fn served(&self) -> bool {
        self.code != Some(NOT_PORTED)
    }
}

/// The nearest-rank percentile (`p` in 0 to 100) of sorted samples.
#[must_use]
pub fn percentile(sorted: &[Duration], p: usize) -> Duration {
    let rank = (p * sorted.len()).div_ceil(100).max(1);
    sorted[rank - 1]
}

/// Times `runs` cold starts of `step` on `which` in `home`, after a short warm-up.
pub fn time_step(
    engines: &Engines,
    which: Which,
    home: &Home,
    port: u16,
    step: &Step,
    runs: usize,
) -> io::Result<Timing> {
    let mut code = None;
    for _ in 0..WARMUP {
        run_step(engines, which, home, port, step)?;
    }
    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        let observed = run_step(engines, which, home, port, step)?;
        samples.push(start.elapsed());
        code = observed.code;
    }
    samples.sort();
    Ok(Timing {
        p50: percentile(&samples, 50),
        p95: percentile(&samples, 95),
        runs,
        code,
    })
}
