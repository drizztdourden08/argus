//! I-158: `coord wake-fault clear` in a home with a boards table finds the Overlord's board by
//! its role: the folder is an id, `portfolio` only an alias.

// test-tag: critical

use std::fs;
use std::path::Path;

use crate::common::{ThreadSpec, board_with};
use crate::remote::{HELPER, MAIN, UI, make_checkout, ok, refused, seed_standard};
use crate::wake_faults_cli::{NOTE, as_session, clear, faulted_with};
use serde_json::json;

const OVERLORD: &str = "bovl001";

/// The standard home, with the Overlord's board moved to the folder `bovl001` and a `boards.json`
/// that gives it the role and the alias `portfolio`.
fn seed_table(coord_home: &Path, base: &Path) {
    seed_standard(coord_home, base);
    fs::remove_dir_all(coord_home.join("portfolio")).expect("old folder");
    let root = make_checkout(base, "portfolio");
    let threads: [ThreadSpec<'_>; 2] = [
        (
            "coordinator",
            "coordinator",
            "active",
            Some(MAIN),
            Some(&root),
        ),
        ("helper", "thread", "active", Some(HELPER), None),
    ];
    argus_testkit::write_board(coord_home, OVERLORD, &board_with(OVERLORD, &threads));
    let at = "2026-11-01T00:00:00.000Z";
    let table = json!({
        "version": 1,
        "state": "active",
        "boards": {
            "alpha": { "name": "Alpha", "aliases": [], "createdAt": at },
            OVERLORD: {
                "name": "Overlord",
                "aliases": ["portfolio"],
                "role": "overlord",
                "createdAt": at
            }
        }
    });
    fs::write(coord_home.join("boards.json"), table.to_string()).expect("table");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_overlord_on_a_board_id_clears_a_wake_fault_and_others_are_refused() {
    let rig = faulted_with(seed_table).await;
    let with_note = ["--note", NOTE];
    let said = "the coordinator thread of project bovl001) can clear a wake fault";
    let from_alpha = clear(&rig, &with_note, Some(UI), "checkout-alpha");
    refused(&from_alpha, 1, said);
    refused(&from_alpha, 1, "this session belongs to project alpha");
    let helper = clear(&rig, &with_note, Some(HELPER), "checkout-portfolio");
    refused(&helper, 1, said);
    refused(&helper, 1, "thread helper is not the coordinator");
    let cleared = clear(&rig, &with_note, Some(MAIN), "checkout-portfolio");
    assert!(
        ok(&cleared).starts_with("cleared the wake fault of alpha/api (since "),
        "{}",
        cleared.out
    );
    let args = ["wake-fault", "list"];
    let listed = as_session(&rig, &args, Some(MAIN), "checkout-portfolio");
    assert!(!ok(&listed).contains("alpha/api"), "{}", listed.out);
    rig.stop().await;
}
