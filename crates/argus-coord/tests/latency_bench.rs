//! The latency bench of the coord port (docs/coord-port.md, section 3): cold start p50 and p95
//! of `coord --help`, a read verb and the hook verbs, Node against `argus coord`, on a copy of a
//! board tree. Run it on purpose, with a release binary:
//!
//! ```text
//! cargo build --release -p argus
//! ARGUS_EXE=target/release/argus.exe cargo test -p argus-coord --test latency_bench \
//!     -- --ignored --nocapture
//! ```
//!
//! `COORD_BENCH_SEED` names the board tree to copy (default: `~/.coord`, read only; the fixture
//! when that is absent), `COORD_BENCH_PROJECT` the project of the read verb (default `argus`),
//! `COORD_BENCH_RUNS` the runs per case (default 200). A verb the Argus side still answers
//! with exit 120 is timed but never gated. A served hook fails the bench above its target; a
//! served verb that is not a hook fails when its p50 is above Node's.

#[path = "it/support/mod.rs"]
mod support;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use support::differ::engines::Engines;
use support::differ::exec::Which;
use support::differ::fixture::standard_boards;
use support::differ::homes::{Home, Seed, Twin};
use support::differ::scenario::{LEAD_SESSION, WORKER_SESSION};
use support::differ::step::Step;
use support::latency::{Timing, time_step};

const DEFAULT_RUNS: usize = 200;

/// A p50 and p95 target.
type Gate = (Duration, Duration);

/// One thing to time, on both engines.
struct Case {
    label: &'static str,
    node: Step,
    argus: Step,
    gate: Option<Gate>,
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn hook(home: &Home, event: &str) -> Step {
    let stdin = serde_json::json!({
        "session_id": env::var("COORD_BENCH_SESSION").unwrap_or_else(|_| LEAD_SESSION.to_owned()),
        "cwd": home.work().to_string_lossy(),
        "hook_event_name": event,
    });
    Step::new(&["hook", event]).stdin(&stdin.to_string())
}

/// The cases of the docs: help, a read verb, the four hooks that run on every prompt and tool.
fn cases(twin: &Twin, project: &str) -> Vec<Case> {
    let hook_gate = Some((ms(15), ms(40)));
    let start_gate = Some((ms(25), ms(50)));
    let both = |label, args: &[&str], gate| Case {
        label,
        node: Step::new(args),
        argus: Step::new(args),
        gate,
    };
    let hooked = |label, event: &str, gate| Case {
        label,
        node: hook(&twin.node, event),
        argus: hook(&twin.rust, event),
        gate,
    };
    vec![
        both("coord --help", &["--help"], None),
        both(
            "coord item list (read)",
            &["item", "list", "--project", project],
            None,
        ),
        hooked("hook session-start", "session-start", start_gate),
        hooked("hook prompt", "prompt", hook_gate),
        hooked("hook tool", "tool", hook_gate),
        hooked("hook stop", "stop", hook_gate),
    ]
}

/// The seed, the project of the read verb, and a label for the report.
fn bench_seed() -> (Seed, String, String) {
    let project = env::var("COORD_BENCH_PROJECT").unwrap_or_else(|_| "argus".to_owned());
    let named = env::var_os("COORD_BENCH_SEED").map(PathBuf::from);
    let real = env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(|h| PathBuf::from(h).join(".coord"));
    match named.or(real).filter(|p| p.is_dir()) {
        Some(dir) => (
            Seed::CopyOf(dir.clone()),
            project,
            format!("copy of {}", dir.display()),
        ),
        None => (
            Seed::Fixture(standard_boards(LEAD_SESSION, WORKER_SESSION)),
            "alpha".to_owned(),
            "the standard fixture".to_owned(),
        ),
    }
}

/// Tells each home which project it is, as a rendered checkout does.
fn wire_project(twin: &Twin, project: &str) {
    for home in [&twin.node, &twin.rust] {
        let wired = format!("{{ \"projectKey\": \"{project}\" }}\n");
        fs::write(home.work().join(".ai-config.json"), wired).expect("wire the work folder");
    }
}

fn show(label: &str, which: &str, t: &Timing) -> String {
    let note = if t.served() {
        ""
    } else {
        "  (answers 120: not ported, not a measurement)"
    };
    format!(
        "{label:<26} {which:<6} p50 {:>6.1} ms  p95 {:>6.1} ms  runs {}{note}",
        ms_of(t.p50),
        ms_of(t.p95),
        t.runs
    )
}

fn ms_of(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// The reason a served case is over its gate, if it is.
fn over(case: &Case, node: &Timing, argus: &Timing) -> Option<String> {
    if !argus.served() {
        return None;
    }
    match case.gate {
        Some((p50, p95)) if argus.p50 > p50 || argus.p95 > p95 => Some(format!(
            "{}: p50 {:.1} ms (target {} ms), p95 {:.1} ms (target {} ms)",
            case.label,
            ms_of(argus.p50),
            p50.as_millis(),
            ms_of(argus.p95),
            p95.as_millis()
        )),
        None if argus.p50 > node.p50 => Some(format!("{}: argus p50 is above node's", case.label)),
        _ => None,
    }
}

#[test]
#[ignore = "bench: see the module documentation"]
fn cold_start_latency_against_node() {
    let Some(engines) = Engines::or_skip("cold_start_latency_against_node") else {
        return;
    };
    let runs = env::var("COORD_BENCH_RUNS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(DEFAULT_RUNS);
    let (seed, project, label) = bench_seed();
    let twin = Twin::seeded(&seed).expect("seed the twin homes");
    wire_project(&twin, &project);
    println!("seed: {label}, project {project}, {runs} runs per case");
    let mut failures = Vec::new();
    for case in cases(&twin, &project) {
        let port = twin.closed_port;
        let node =
            time_step(&engines, Which::Node, &twin.node, port, &case.node, runs).expect("node");
        let argus =
            time_step(&engines, Which::Argus, &twin.rust, port, &case.argus, runs).expect("argus");
        println!(
            "{}\n{}",
            show(case.label, "node", &node),
            show(case.label, "argus", &argus)
        );
        failures.extend(over(&case, &node, &argus));
    }
    assert!(
        failures.is_empty(),
        "over the latency gate:\n{}",
        failures.join("\n")
    );
}
