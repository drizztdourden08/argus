//! E3: the differential harness reads the fixture board through the real Node CLI, and a save
//! leaves the twin homes alike. Skips when node or the Node coord CLI is absent.

// test-tag: e2e

use crate::support::differ::exec::{Which, run_step};
use crate::support::differ::fixture::standard_boards;
use crate::support::differ::snapshot::snapshot;
use crate::support::differ::{Engines, Normaliser, Seed, Step, Twin};
use crate::support::differ::{LEAD_SESSION, WORKER_SESSION};

#[test]
#[ignore = "e2e"]
fn node_reads_the_fixture_board_and_a_save_leaves_the_homes_alike() {
    let Ok(node_cli) = Engines::locate_node_cli() else {
        eprintln!("skipped: node or the Node coord CLI is absent");
        return;
    };
    let engines = Engines {
        node_cli,
        argus: std::path::PathBuf::new(),
    };
    let seed = Seed::Fixture(standard_boards(LEAD_SESSION, WORKER_SESSION));
    let twin = Twin::seeded(&seed).expect("twin");
    let list = Step::new(&["item", "list", "--project", "alpha"]).session(LEAD_SESSION);
    let ran = run_step(&engines, Which::Node, &twin.node, twin.closed_port, &list).expect("run");
    let out = String::from_utf8_lossy(&ran.stdout).into_owned();
    assert_eq!(
        ran.code,
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&ran.stderr)
    );
    assert!(
        out.contains("Plan the release") && out.contains("Port the parser"),
        "{out}"
    );
    let add = Step::new(&["item", "add", "Check", "--project", "alpha"]).session(WORKER_SESSION);
    for home in [&twin.node, &twin.rust] {
        let ran = run_step(&engines, Which::Node, home, twin.closed_port, &add).expect("add");
        assert_eq!(ran.code, Some(0));
    }
    let norm = Normaliser::new(
        &[twin.node.root(), twin.rust.root()],
        vec![LEAD_SESSION.into(), WORKER_SESSION.into()],
    );
    let (a, b) = (
        snapshot(twin.node.root(), &norm).unwrap(),
        snapshot(twin.rust.root(), &norm).unwrap(),
    );
    assert_eq!(
        a, b,
        "two Node runs in twin homes must normalise to the same files"
    );
}
