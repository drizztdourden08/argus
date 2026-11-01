//! K2: coord wake-fault says so when the dashboard is down.

// test-tag: standard

use crate::common::coord;
use crate::remote::{env_plus, refused};

#[test]
fn a_dashboard_that_is_down_is_an_error_for_list_and_clear_not_a_silent_success() {
    let home = crate::common::Home::new();
    let env = env_plus(&home.env(), &[("CLAUDE_CODE_SESSION_ID", "")]);
    let listed = coord(&["wake-fault", "list"], env, home.path());
    refused(&listed, 1, "the dashboard is not running");
    let unknown = coord(&["wake-fault", "frobnicate"], home.env(), home.path());
    refused(
        &unknown,
        1,
        "unknown wake-fault command \"frobnicate\" (list, clear).",
    );
}
