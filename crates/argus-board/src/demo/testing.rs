//! The test cases (AI and manual), the test plans that hold them, and the builds (launchers)
//! the runs can name.

use super::{Step, step};

pub(super) const STEPS: &[Step] = &[
    step(
        "ui",
        "ta1",
        &[
            "test",
            "add",
            "{s1}",
            "The lobby list shows open lobbies",
            "--mode",
            "ai",
        ],
    ),
    step(
        "ui",
        "",
        &["test", "step", "{ta1}", "add", "Open the lobby page"],
    ),
    step(
        "ui",
        "",
        &[
            "test",
            "expect",
            "{ta1}",
            "add",
            "Every open lobby is listed",
        ],
    ),
    step(
        "ui",
        "ta2",
        &[
            "test",
            "add",
            "{s1}",
            "A join request reaches the host",
            "--mode",
            "manual",
        ],
    ),
    step(
        "ui",
        "",
        &["test", "step", "{ta2}", "add", "Press Join on a lobby"],
    ),
    step(
        "ui",
        "",
        &[
            "test",
            "expect",
            "{ta2}",
            "add",
            "The host sees the request",
        ],
    ),
    step(
        "api",
        "ta3",
        &[
            "test",
            "add",
            "{s3}",
            "A leaving player frees the seat",
            "--mode",
            "ai",
        ],
    ),
    step(
        "api",
        "ta4",
        &[
            "test",
            "add",
            "{s3}",
            "The seat count looks right on screen",
            "--mode",
            "manual",
        ],
    ),
];
