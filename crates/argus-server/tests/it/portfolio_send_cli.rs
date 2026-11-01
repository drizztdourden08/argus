//! E2: the lead reads a 20000 character send of the Overlord page whole with the real Node
//! `coord inbox`, with no ellipsis. Needs `ARGUS_COORD_CLI`; with no CLI located the case ends at
//! once.
// test-tag: e2e

use crate::page_send::{brief, from_page};
use crate::wake_rig::rig;
use argus_testkit::CoordCli;
use serde_json::json;

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn the_lead_reads_it_whole_with_coord_inbox_no_ellipsis_nothing_cut() {
    let Some(cli) = CoordCli::locate() else {
        return;
    };
    let r = rig().await;
    let t = &r.t;
    let text = brief(20000);
    let sent = from_page(
        t,
        json!({ "kind": "followup", "project": "beta", "text": text }),
    )
    .await;
    assert_eq!(sent["status"], "delivered", "{sent}");
    let out = cli
        .run(
            t,
            &["inbox", "--project", "beta", "--as", "coordinator"],
            &t.coord_home,
            &[],
        )
        .await;
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert!(
        out.stdout.contains(&text),
        "all 20000 characters are in the output"
    );
    assert!(
        !out.stdout.contains('…'),
        "nothing is shortened with an ellipsis"
    );
}
