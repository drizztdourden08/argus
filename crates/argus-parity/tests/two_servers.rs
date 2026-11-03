//! `argus parity` against two real test servers on ephemeral ports, started from the same fixture
//! boards: no route differs; after one doc is changed on one side, exactly that doc's routes
//! are reported; a server that cannot be reached is an error; and the comparison writes nothing.
//! Keep rule: E3.
// test-tag: e2e

use std::path::Path;

use argus_core::{Coll, DocChange};
use argus_parity::{Options, Report, Verdict, compare};
use argus_testkit::{
    TestOptions, TestServer, alpha_board, beta_board, portfolio_board, write_board,
};
use serde_json::{Value, json};
use tempfile::TempDir;

struct Pair {
    left: TestServer,
    right: TestServer,
    _homes: Vec<TempDir>,
}

/// The fixture boards, written once and copied to every home so no timestamp differs.
fn boards(shared: &Path) -> [(&'static str, Value); 3] {
    [
        ("alpha", alpha_board(shared)),
        ("beta", beta_board()),
        ("_portfolio", portfolio_board()),
    ]
}

async fn server(shared: &Path, homes: &mut Vec<TempDir>) -> TestServer {
    let coord = tempfile::tempdir().expect("coord home");
    for (key, board) in boards(shared) {
        write_board(coord.path(), key, &board);
    }
    let dash = tempfile::tempdir().expect("dashboard home");
    let t = TestServer::start(TestOptions {
        coord_home: Some(coord.path().to_path_buf()),
        dashboard_home: Some(dash.path().to_path_buf()),
        ..TestOptions::default()
    })
    .await;
    homes.extend([coord, dash]);
    t
}

async fn pair() -> Pair {
    let shared = tempfile::tempdir().expect("shared path");
    let mut homes = vec![];
    let left = server(shared.path(), &mut homes).await;
    let right = server(shared.path(), &mut homes).await;
    homes.push(shared);
    Pair {
        left,
        right,
        _homes: homes,
    }
}

fn options(p: &Pair) -> Options {
    let mut o = Options::new(&p.left.url, &p.right.url);
    o.tolerance_ms = 60_000;
    o.stream_secs = 3;
    o
}

fn differing(report: &Report) -> Vec<&str> {
    report
        .routes
        .iter()
        .filter(|r| r.verdict != Verdict::Equal)
        .map(|r| r.label.as_str())
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn two_servers_on_the_same_boards_have_no_difference() {
    let p = pair().await;
    let report = compare(&options(&p)).await.expect("a report");
    assert!(report.is_equal(), "{}", report.text(20));
    assert_eq!(report.exit_code(), 0);
    let labels: Vec<&str> = report.routes.iter().map(|r| r.label.as_str()).collect();
    for want in [
        "GET /api/health",
        "GET /api/projects",
        "GET /api/p/alpha/db/items",
        "GET /api/p/beta/db/meta",
        "GET /api/p/_portfolio/db/cards",
        "GET /api/p/alpha/launches",
        "POST /mcp tools/list",
        "STREAM /api/p/alpha/stream snapshot items",
        "STREAM /api/p/alpha/stream ready",
    ] {
        assert!(labels.contains(&want), "{want} is compared");
    }
}

fn change_title(t: &TestServer, title: &str) {
    let docs = t.svc.db.docs();
    let row = docs
        .get("alpha", Coll::Items, "I-1")
        .expect("read")
        .expect("the fixture item");
    let mut data = row.data;
    data["title"] = json!(title);
    docs.apply("alpha", &[DocChange::set(Coll::Items, "I-1", data)])
        .expect("apply");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_changed_doc_is_reported_with_its_path_and_nothing_else_is() {
    let p = pair().await;
    change_title(&p.right, "A title only the right side has");
    let report = compare(&options(&p)).await.expect("a report");
    assert_eq!(report.exit_code(), 1);
    assert_eq!(
        differing(&report),
        [
            "GET /api/p/alpha/db/items",
            "STREAM /api/p/alpha/stream snapshot items"
        ]
    );
    let route = &report
        .routes
        .iter()
        .find(|r| r.label.ends_with("alpha/db/items"))
        .expect("route");
    let Verdict::Differs(changes) = &route.verdict else {
        panic!("expected a difference");
    };
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "docs[I-1].data.title");
    assert!(
        report
            .text(8)
            .contains("DIFFERS  GET /api/p/alpha/db/items")
    );
    assert_eq!(report.json(8)["summary"]["differs"], 2);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn asking_for_a_project_compares_only_its_scope() {
    let p = pair().await;
    let mut o = options(&p);
    o.projects = vec!["beta".to_owned()];
    o.stream_secs = 0;
    let report = compare(&o).await.expect("a report");
    assert!(report.is_equal(), "{}", report.text(20));
    let has = |part: &str| report.routes.iter().any(|r| r.label.contains(part));
    assert!(has("/api/p/beta/db/items"));
    assert!(!has("/api/p/alpha/"));
    assert!(!has("/api/p/_portfolio/"));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn the_comparison_writes_nothing() {
    let p = pair().await;
    let seqs = |p: &Pair| {
        (
            p.left.svc.db.docs().seq().expect("seq"),
            p.right.svc.db.docs().seq().expect("seq"),
        )
    };
    let before = seqs(&p);
    let _ = compare(&options(&p)).await.expect("a report");
    assert_eq!(seqs(&p), before);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_server_that_cannot_be_reached_is_an_error() {
    let p = pair().await;
    let closed = std::net::TcpListener::bind(("127.0.0.1", 0))
        .and_then(|l| l.local_addr())
        .expect("a free port")
        .port();
    let mut o = options(&p);
    o.left = format!("http://127.0.0.1:{closed}");
    let err = compare(&o).await.expect_err("an error");
    assert!(err.contains("cannot reach left"), "{err}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn skipped_routes_are_listed_and_not_compared() {
    let p = pair().await;
    let mut o = options(&p);
    o.skip = vec!["wakes".to_owned()];
    o.stream_secs = 0;
    let report = compare(&o).await.expect("a report");
    assert!(report.skipped.iter().any(|s| s.contains("/wakes")));
    assert!(report.routes.iter().all(|r| !r.label.contains("wakes")));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_board_file_of_nul_bytes_is_its_own_finding_and_fails_the_run() {
    let p = pair().await;
    let coord = tempfile::tempdir().expect("coord home");
    std::fs::create_dir_all(coord.path().join("tessera")).expect("folder");
    std::fs::write(coord.path().join("tessera").join("board.json"), [0u8; 128]).expect("board");
    let mut o = options(&p);
    o.coord_homes = vec![coord.path().to_path_buf()];
    let report = compare(&o).await.expect("a report");
    assert_eq!(report.findings.len(), 1, "{}", report.text(8));
    assert!(report.findings[0].contains("board \"tessera\" is damaged"));
    assert!(report.findings[0].contains("128 NUL bytes"));
    assert_eq!(
        differing(&report),
        Vec::<&str>::new(),
        "the routes are equal"
    );
    assert_eq!(report.exit_code(), 1);
    assert!(report.text(8).contains("FINDING  board \"tessera\""));
}
