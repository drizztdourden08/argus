//! The threads of the demo board: a lead and two workers.

use super::{Step, step};

pub(super) const STEPS: &[Step] = &[
    step("", "", &["board", "init", "--name", "Argus Demo"]),
    step(
        "",
        "",
        &[
            "thread",
            "register",
            "lead",
            "--role",
            "coordinator",
            "--root",
            "{root}",
            "--session",
            "Demo lead",
        ],
    ),
    step(
        "",
        "",
        &[
            "thread",
            "register",
            "ui",
            "--root",
            "{root}",
            "--session",
            "Demo UI",
        ],
    ),
    step(
        "",
        "",
        &[
            "thread",
            "register",
            "api",
            "--root",
            "{root}",
            "--session",
            "Demo API",
        ],
    ),
];
