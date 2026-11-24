//! S2: coord run refusals against an in-process Argus test server, and the exit 3 of a dashboard
//! that cannot be reached.

// test-tag: standard

use std::fs;

use crate::common::{Home, Ran, coord};
use crate::remote::{Rig, env_plus, ok, refused};
use crate::runs_support::{define, run, run_argv, seed};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn start_stop_and_status_refuse_a_bad_name_id_or_agent() {
    let rig = Rig::start(json!({}), seed).await;
    define(&rig, "hello", "console.log(\"hi\")");
    let cases: [(&[&str], &str); 5] = [
        (&["stop", "R-99"], "no run R-99"),
        (&["stop", "nope"], "run id"),
        (&["start", "ghost"], "no launch \"ghost\""),
        (&["start", "hello", "--visible"], "--visible is only for"),
        (&["start", "hello", "--agent", "bob"], "--agent"),
    ];
    for (words, text) in cases {
        refused(&run(&rig, words, &[]), 1, text);
    }
    rig.stop().await;
}

#[test]
fn with_the_server_down_and_no_way_to_start_it_the_plain_message_and_exit_3() {
    let home = Home::new();
    seed(&home.coord_home(), home.path());
    let missing = home
        .path()
        .join("no-install")
        .to_string_lossy()
        .into_owned();
    let env = env_plus(
        &home.env(),
        &[
            ("COORD_DASHBOARD_WAIT_MS", "700"),
            ("COORD_DASHBOARD_INSTALL", &missing),
        ],
    );
    let call = |words: &[&str]| -> Ran { coord(&run_argv(words, &[]), env.clone(), home.path()) };
    ok(&call(&[
        "define", "quick", "--cmd", "echo hi", "--as", "lead",
    ]));
    refused(
        &call(&["start", "quick"]),
        3,
        "coord: the dashboard is not running (dashboard start)",
    );
    fs::write(
        home.dashboard_home().join("config.json"),
        json!({ "port": 1, "token": "a".repeat(64), "installDir": home.path().join("missing") })
            .to_string(),
    )
    .expect("config");
    refused(&call(&["list"]), 3, "the dashboard is not running");
    assert_eq!(call(&["defs"]).code, 0, "defs never needs it");
}
