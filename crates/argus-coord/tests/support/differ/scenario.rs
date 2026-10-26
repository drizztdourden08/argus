//! The differential run: the same steps, in order, on the Node engine in one home and the Argus
//! engine in the other, with a verdict per step.

use std::fmt;
use std::io;

use super::engines::Engines;
use super::exec::{Observed, Which, run_step};
use super::fixture::{FixtureBoard, standard_boards};
use super::homes::{Seed, Twin};
use super::normalise::Normaliser;
use super::snapshot::snapshot;
use super::step::Step;
use super::verdict::{NOT_PORTED, Seen, Verdict, judge};

/// The session id of the coordinator thread of the standard fixture.
pub const LEAD_SESSION: &str = "11111111-1111-4111-8111-111111111111";

/// The session id of the worker thread of the standard fixture.
pub const WORKER_SESSION: &str = "22222222-2222-4222-8222-222222222222";

/// A differential run over twin homes.
#[derive(Debug)]
pub struct Differ {
    engines: Engines,
    twin: Twin,
    stable_ids: Vec<String>,
}

/// One step and what came of it.
#[derive(Debug)]
pub struct StepReport {
    /// The command, as `coord <args>`.
    pub line: String,
    /// Same, not ported, or a mismatch.
    pub verdict: Verdict,
}

/// The verdicts of a run, in step order.
#[derive(Debug)]
pub struct Report {
    pub steps: Vec<StepReport>,
}

impl Differ {
    /// Twin homes seeded with `seed`. `stable_ids` are session ids that are the same on both
    /// sides by construction (those of the steps are added when they run).
    pub fn new(engines: Engines, seed: &Seed, stable_ids: &[&str]) -> io::Result<Differ> {
        Ok(Differ {
            engines,
            twin: Twin::seeded(seed)?,
            stable_ids: stable_ids.iter().map(|id| (*id).to_owned()).collect(),
        })
    }

    /// Twin homes seeded with the standard fixture: `alpha` (coordinator `lead` bound to
    /// [`LEAD_SESSION`], thread `worker` bound to [`WORKER_SESSION`], three items, one waiting
    /// message) and `beta`.
    pub fn standard(engines: Engines) -> io::Result<Differ> {
        let boards: Vec<FixtureBoard> = standard_boards(LEAD_SESSION, WORKER_SESSION);
        Differ::new(
            engines,
            &Seed::Fixture(boards),
            &[LEAD_SESSION, WORKER_SESSION],
        )
    }

    /// The twin homes, for a test that wants to look at a file.
    pub fn twin(&self) -> &Twin {
        &self.twin
    }

    /// Runs every step on both engines and judges each one. A step whose Argus side answers
    /// 120 is run on Node in the Argus home too, as the shim does, so later steps start alike.
    pub fn run(&self, steps: &[Step]) -> io::Result<Report> {
        let mut stable = self.stable_ids.clone();
        stable.extend(steps.iter().filter_map(|s| s.session.clone()));
        let roots = [self.twin.node.root(), self.twin.rust.root()];
        let norm = Normaliser::new(&roots, stable);
        let mut report = Report { steps: Vec::new() };
        for step in steps {
            let verdict = self.one(step, &norm)?;
            report.steps.push(StepReport {
                line: step.line(),
                verdict,
            });
        }
        Ok(report)
    }

    fn one(&self, step: &Step, norm: &Normaliser) -> io::Result<Verdict> {
        let port = self.twin.closed_port;
        let node = run_step(&self.engines, Which::Node, &self.twin.node, port, step)?;
        let rust = run_step(&self.engines, Which::Argus, &self.twin.rust, port, step)?;
        if rust.code == Some(NOT_PORTED) {
            run_step(&self.engines, Which::Node, &self.twin.rust, port, step)?;
        }
        let node_seen = seen(&node, &self.twin.node, norm)?;
        let rust_seen = seen(&rust, &self.twin.rust, norm)?;
        Ok(judge(&node_seen, &rust_seen))
    }
}

fn seen(observed: &Observed, home: &super::homes::Home, norm: &Normaliser) -> io::Result<Seen> {
    Ok(Seen {
        code: observed.code,
        stdout: norm.bytes(&observed.stdout),
        stderr: norm.bytes(&observed.stderr),
        files: snapshot(home.root(), norm)?,
    })
}

impl Report {
    /// The steps that differ.
    #[must_use]
    pub fn mismatches(&self) -> Vec<&StepReport> {
        self.steps
            .iter()
            .filter(|s| matches!(s.verdict, Verdict::Mismatch(_)))
            .collect()
    }

    /// How many steps the Argus side does not serve yet.
    #[must_use]
    pub fn not_ported(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.verdict == Verdict::NotPorted)
            .count()
    }

    /// Panics with the whole report when any step is a mismatch. A step that is not ported is
    /// not a failure: it turns into a real comparison the day the verb is ported.
    pub fn assert_no_mismatch(&self) {
        assert!(self.mismatches().is_empty(), "{self}");
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for step in &self.steps {
            writeln!(f, "{} -> {}", step.line, step.verdict)?;
        }
        Ok(())
    }
}
