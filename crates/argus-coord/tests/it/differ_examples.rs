//! E3: the router answers of the Node CLI and of argus coord, compared.

// test-tag: e2e

use crate::support::differ::{Differ, Engines, Report, Step};

fn run(test: &str, steps: &[Step]) -> Option<Report> {
    let engines = Engines::or_skip(test)?;
    let differ = Differ::standard(engines).expect("twin homes");
    let report = differ.run(steps).expect("run the steps");
    println!("{report}");
    Some(report)
}

#[test]
#[ignore = "e2e"]
fn router_answers_that_exist_on_both_sides_are_compared_for_real() {
    let steps = [
        Step::new(&["--help"]),
        Step::new(&["help"]),
        Step::new(&["frobnicate"]),
    ];
    let Some(report) = run("router_answers_compared_for_real", &steps) else {
        return;
    };
    report.assert_no_mismatch();
    assert_eq!(report.not_ported(), 0, "{report}");
}
