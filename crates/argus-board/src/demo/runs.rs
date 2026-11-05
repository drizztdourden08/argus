//! The test runs: passed, reported (with a failure), running with a build, and a draft and a
//! ready one. A plan holds one open run at a time, so each stage has a plan of its own.

use super::{Step, step};

pub(super) const STEPS: &[Step] = &[
    step("lead", "", &["test", "plan", "run", "{p1}"]),
    step(
        "lead",
        "",
        &[
            "test",
            "result",
            "{ta1}",
            "pass",
            "The list shows both lobbies",
            "--run",
            "{p1}",
        ],
    ),
    step(
        "lead",
        "",
        &[
            "test",
            "result",
            "{ta2}",
            "pass",
            "The host got the request",
            "--run",
            "{p1}",
        ],
    ),
    step("lead", "", &["test", "plan", "end", "{p1}"]),
    step("lead", "", &["test", "plan", "run", "{p2}"]),
    step(
        "lead",
        "",
        &[
            "test",
            "result",
            "{ta3}",
            "pass",
            "The seat came free",
            "--run",
            "{p2}",
        ],
    ),
    step(
        "lead",
        "",
        &[
            "test",
            "result",
            "{ta4}",
            "fail",
            "The count stays at four",
            "--run",
            "{p2}",
        ],
    ),
    step("lead", "", &["test", "plan", "end", "{p2}"]),
    step(
        "lead",
        "",
        &["test", "plan", "run", "{p3}", "--build", "demo-app"],
    ),
    step(
        "lead",
        "",
        &[
            "test",
            "result",
            "{ta1}",
            "pass",
            "Opens on the demo build",
            "--run",
            "{p3}",
        ],
    ),
    step(
        "lead",
        "run_draft",
        &[
            "item",
            "add",
            "Run 1: release candidate, draft",
            "--type",
            "test-run",
            "--stage",
            "draft",
            "--field",
            "plan={p4}",
        ],
    ),
    step(
        "lead",
        "run_ready",
        &[
            "item",
            "add",
            "Run 2: release candidate, ready",
            "--type",
            "test-run",
            "--stage",
            "ready",
            "--field",
            "plan={p4}",
            "--field",
            "build=demo-app",
        ],
    ),
    step("lead", "", &["test", "run", "hand", "{run_ready}"]),
];
