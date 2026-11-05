//! The changes of the demo board: pull requests that are open, merged and closed. A merge runs
//! Brock in the real verb, so the verb here records the merge through the board op instead
//! (see [`MERGED_PR`]).

use super::{Step, step};

/// The name the pull request that gets merged is bound to.
pub const MERGED_PR: &str = "pr_merged";

/// The merge commit recorded on it.
pub const MERGE_COMMIT: &str = "5e3c1a9f0b2d4c6e8a7f1b3d5c9e0a2b4d6f8c1e";

pub(super) const STEPS: &[Step] = &[
    step(
        "ui",
        "pr_open",
        &[
            "pr",
            "open",
            "Join button on the lobby list",
            "--head",
            "feature/join-button",
            "--commit",
            "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
            "--item",
            "{s3}",
            "--body",
            "Adds the join button and posts the request.",
        ],
    ),
    step(
        "api",
        "pr_merged",
        &[
            "pr",
            "open",
            "Seat table and host check",
            "--head",
            "feature/seat-table",
            "--commit",
            "0f1e2d3c4b5a69788796a5b4c3d2e1f001122334",
            "--item",
            "{t4}",
            "--body",
            "Adds the seat table.",
        ],
    ),
    step(
        "api",
        "pr_closed",
        &[
            "pr",
            "open",
            "Websocket relay spike",
            "--head",
            "spike/relay",
            "--commit",
            "9988776655443322110099887766554433221100",
            "--item",
            "{t5}",
        ],
    ),
    step("api", "", &["pr", "close", "{pr_closed}"]),
];
