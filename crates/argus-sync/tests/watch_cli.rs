//! The watcher against the real coord clients: a `coord item add` from another process (the Node
//! CLI, then `argus coord`, same homes) reaches the page stream within a second, and a board made
//! by `coord board init` after the server started is mirrored. Ported from the CLI cases of
//! `sync.watch.test.mjs`. Each case runs once
//! per client that is found and says why it skips the others.
//! Keep rule: E2.
// test-tag: e2e

use std::time::Duration;

mod support;

use argus_testkit::{CoordCli, TestServer};
use support::{item_arrival, mirrored_meta, seeded};

async fn run_ok(cli: &CoordCli, t: &TestServer, args: &[&str]) {
    let out = cli.run(t, args, &t.coord_home, &[]).await;
    assert_eq!(out.status, 0, "{} {args:?}: {}", cli.name(), out.stderr);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_cli_change_reaches_the_stream_within_a_second() {
    for cli in CoordCli::locate_each() {
        let t = seeded().await;
        let add = [
            "item",
            "add",
            "smoke",
            "--for",
            "ui",
            "--project",
            "alpha",
            "--as",
            "lead",
        ];
        let took = item_arrival(&t, "smoke", async || run_ok(&cli, &t, &add).await).await;
        let took = took.unwrap_or_else(|| panic!("{}: the new item did not arrive", cli.name()));
        assert!(
            took < Duration::from_secs(1),
            "{}: arrived after {took:?}",
            cli.name()
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_board_made_by_the_cli_after_start_is_mirrored_by_the_poll_or_the_watcher() {
    for cli in CoordCli::locate_each() {
        let t = seeded().await;
        let init = ["board", "init", "--name", "Gamma", "--project", "gamma"];
        run_ok(&cli, &t, &init).await;
        let meta = mirrored_meta(&t, "gamma").await;
        assert_eq!(meta["data"]["name"], "Gamma", "{}", cli.name());
    }
}
