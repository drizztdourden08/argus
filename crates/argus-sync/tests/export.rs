//! Board to docs: every page collection, meta/board, launches without env values, proof urls
//! served by the dashboard, deletes when a doc leaves the board, no event when nothing changed,
//! portfolio cards (with running runs and the master launcher), launchers and a hard resync
//! (port of `sync.export.test.mjs`).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use argus_board::sync::{PAGE_COLLECTIONS, export_docs};
use argus_core::log::{LOG_MAX_BYTES, Logger};
use argus_core::redact::Redactor;
use argus_core::{
    ApiError, Bus, Coll, DocChange, DocsEvent, ErrorCode, Paths, ResetEvent, Subscription,
};
use argus_store::Db;
use argus_sync::{Proof, ProofSource, RunsView, ScopeQueue, SyncOptions, SyncService};
use argus_testkit::{alpha_board, make_home, write_board};
use serde_json::{Map, Value, json};
use tempfile::TempDir;

const PNG_HEX: &str = "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000154a24f5d0000000049454e44ae426082";
const AT: &str = "2026-10-20T10:00:00.000Z";

fn png() -> Vec<u8> {
    (0..PNG_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&PNG_HEX[i..i + 2], 16).expect("hex"))
        .collect()
}

fn last_run() -> Value {
    json!({ "id": "R-7", "name": "launcher-master", "status": "exited", "exitCode": 0, "signal": null, "error": null, "stdin": false, "queuedAt": "q", "startedAt": "s", "endedAt": "e", "lastLine": "bye" })
}

/// Alpha has one run going; its master launcher's Play last ran as R-7.
struct StubRuns;

impl RunsView for StubRuns {
    fn running(&self, key: &str) -> usize {
        usize::from(key == "alpha")
    }

    fn last_run(&self, key: &str, name: &str) -> Option<Value> {
        (key == "alpha" && name == "launcher-master").then(last_run)
    }
}

/// A sync service on a real database without the HTTP server.
struct Harness {
    home: TempDir,
    _dashboard: TempDir,
    db: Arc<Db>,
    bus: Bus,
    sync: SyncService,
    events: Arc<Mutex<Vec<DocsEvent>>>,
    _sub: Subscription,
}

impl Harness {
    async fn start() -> Harness {
        let home = tempfile::tempdir().expect("coord home");
        let dashboard = tempfile::tempdir().expect("dashboard home");
        make_home(home.path());
        let paths = Paths::new(home.path(), Some(dashboard.path().to_path_buf()));
        let bus = Bus::new();
        let db =
            Arc::new(Db::open(&dashboard.path().join("dashboard.db"), bus.clone()).expect("db"));
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let sub = bus.on_docs(move |e| sink.lock().unwrap().push(e.clone()));
        let config = argus_core::config::load_config(&paths, None).expect("config");
        let log = Logger::new(
            dashboard.path().join("server.log"),
            Redactor::default(),
            LOG_MAX_BYTES,
        );
        let sync = SyncService::new(SyncOptions {
            paths,
            db: Arc::clone(&db),
            config: Arc::new(RwLock::new(config)),
            log: Arc::new(log),
            runs: Some(Arc::new(StubRuns)),
            queue: ScopeQueue::new(),
        });
        sync.start().await;
        Harness {
            home,
            _dashboard: dashboard,
            db,
            bus,
            sync,
            events,
            _sub: sub,
        }
    }

    fn home(&self) -> &Path {
        self.home.path()
    }

    fn board_file(&self, key: &str) -> PathBuf {
        self.home().join(key).join("board.json")
    }

    fn read(&self, key: &str) -> Value {
        let text = std::fs::read_to_string(self.board_file(key)).expect("board file");
        serde_json::from_str(&text).expect("board json")
    }

    fn write(&self, key: &str, board: &Value) {
        write_board(self.home(), key, board);
    }

    fn docs(&self, scope: &str, coll: Coll) -> Map<String, Value> {
        let rows = self.db.docs().list(scope, coll).expect("list");
        rows.into_iter().map(|r| (r.id, r.data)).collect()
    }

    fn keys(&self, scope: &str, coll: Coll) -> Vec<String> {
        let mut keys: Vec<String> = self.docs(scope, coll).keys().cloned().collect();
        keys.sort();
        keys
    }

    fn seq(&self) -> i64 {
        self.db.docs().seq().expect("seq")
    }

    fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    fn events_since(&self, n: usize) -> Vec<DocsEvent> {
        self.events.lock().unwrap()[n..].to_vec()
    }

    fn launch_defs(&self, scope: &str) -> Vec<String> {
        self.db.with_conn(|c| {
            let mut st = c
                .prepare("SELECT name FROM launch_defs WHERE scope = ? ORDER BY name")
                .expect("prepare");
            st.query_map([scope], |r| r.get::<_, String>(0))
                .expect("query")
                .map(|r| r.expect("row"))
                .collect()
        })
    }

    async fn resync(&self, project: &str) {
        self.sync
            .resync(Some(project), false)
            .await
            .expect("resync");
    }

    async fn resync_with_cards(&self, project: &str) {
        self.resync(project).await;
        self.resync("_portfolio").await;
    }

    async fn proof(
        &self,
        test: &str,
        bytes: Vec<u8>,
        mime: &str,
        caption: &str,
    ) -> Result<Proof, ApiError> {
        let src = ProofSource::Bytes {
            bytes,
            mime: Some(mime.to_owned()),
        };
        self.sync
            .attach_proof("alpha", test, src, caption, "human")
            .await
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.sync.stop();
    }
}

fn pick(v: &Value, fields: &[&str]) -> Value {
    Value::Object(
        fields
            .iter()
            .map(|f| ((*f).to_owned(), v[*f].clone()))
            .collect(),
    )
}

fn doc_data(docs: &[DocChange], coll: Coll, id: &str) -> Value {
    let found = docs.iter().find(|d| d.coll == coll && d.id == id);
    found
        .and_then(|d| d.data.clone())
        .expect("the doc is exported")
}

fn proof_error(r: Result<Proof, ApiError>) -> ErrorCode {
    r.expect_err("the proof is refused").code
}

#[test]
fn export_docs_covers_every_page_collection_meta_and_launches_without_env_values() {
    let mut board = alpha_board(Path::new("."));
    board["runDefs"]["hello"]["env"] = json!({ "API_TOKEN": "secret-value" });
    let docs = export_docs(&board, &|_| "/x".to_owned());
    let has = |c: Coll| docs.iter().any(|d| d.coll == c);
    for c in [
        Coll::Threads,
        Coll::Items,
        Coll::Tests,
        Coll::Entries,
        Coll::Locks,
        Coll::Actions,
        Coll::Agents,
        Coll::Log,
        Coll::Meta,
        Coll::Launches,
    ] {
        assert!(has(c), "has {c}");
    }
    assert!(PAGE_COLLECTIONS.iter().all(|c| *c != Coll::Meta));
    let meta = doc_data(&docs, Coll::Meta, "board");
    let mut keys: Vec<&String> = meta.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "counters",
            "handedAt",
            "handedBuild",
            "name",
            "project",
            "updatedAt"
        ]
    );
    assert_eq!(meta["handedBuild"], "b1");
    assert_eq!(
        doc_data(&docs, Coll::Launches, "hello")["env"],
        json!({ "API_TOKEN": "" })
    );
    assert!(
        !serde_json::to_string(&docs)
            .unwrap()
            .contains("secret-value")
    );
}

#[test]
fn an_agent_record_carries_model_why_and_one_without_it_stays_as_it_is() {
    let mut board = alpha_board(Path::new("."));
    let why = "design: new document model touches every module";
    board["agents"]["A-1"]["model"] = json!("opus");
    board["agents"]["A-1"]["modelWhy"] = json!(why);
    let docs = export_docs(&board, &|_| "/x".to_owned());
    assert_eq!(doc_data(&docs, Coll::Agents, "A-1")["modelWhy"], why);
    assert!(
        doc_data(&docs, Coll::Agents, "A-2")
            .get("modelWhy")
            .is_none()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn start_mirrors_every_board_and_the_portfolio_cards() {
    let h = Harness::start().await;
    assert_eq!(h.keys("alpha", Coll::Items), ["I-1", "I-2"]);
    assert_eq!(h.docs("alpha", Coll::Meta)["board"]["project"], "alpha");
    let defs = ["app", "fail", "hello", "slow"];
    assert_eq!(h.keys("alpha", Coll::Launches), defs);
    assert_eq!(h.docs("beta", Coll::Entries).len(), 1);
    let cards = h.docs("_portfolio", Coll::Cards);
    assert_eq!(h.keys("_portfolio", Coll::Cards), ["alpha", "beta"]);
    assert_eq!(cards["alpha"]["url"], "/p/alpha/");
    assert_eq!(cards["alpha"]["runs"], json!({ "running": 1 }));
    assert_eq!(cards["beta"]["runs"]["running"], 0);
    assert!(cards["alpha"].get("board").is_none());
    assert!(cards["alpha"].get("pageVersion").is_none());
    assert_eq!(h.launch_defs("alpha"), defs);
    let mut run_defs: Vec<String> = h
        .sync
        .run_defs("alpha")
        .iter()
        .map(|d| d["name"].as_str().unwrap().to_owned())
        .collect();
    run_defs.sort();
    assert_eq!(run_defs, defs);
    assert_eq!(h.sync.board("alpha").unwrap()["name"], "Alpha");
    let rows: Vec<(String, bool, String)> = h
        .sync
        .projects()
        .into_iter()
        .map(|p| (p.key, p.has_board, p.url))
        .collect();
    let want = |k: &str| (k.to_owned(), true, format!("/p/{k}/"));
    assert_eq!(rows, [want("alpha"), want("beta")]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_soft_resync_of_an_unchanged_board_writes_and_emits_nothing() {
    let h = Harness::start().await;
    let (events, seq) = (h.event_count(), h.seq());
    let r = h.sync.resync(Some("alpha"), false).await.expect("resync");
    assert_eq!(r.scopes, ["alpha"]);
    assert!(r.docs > 10);
    assert_eq!(h.seq(), seq);
    assert_eq!(h.event_count(), events);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_doc_removed_from_the_board_is_deleted_and_a_changed_one_is_set() {
    let h = Harness::start().await;
    let mut board = h.read("alpha");
    board["items"].as_object_mut().unwrap().remove("I-2");
    board["items"]["I-1"]["title"] = json!("Login screen v2");
    h.write("alpha", &board);
    let before = h.event_count();
    h.resync("alpha").await;
    let events = h.events_since(before);
    let e = events
        .iter()
        .find(|e| e.scope == "alpha")
        .expect("one docs event for alpha");
    let ops: Vec<String> = e
        .changes
        .iter()
        .map(|c| format!("{}:{}", c.change.op.as_str(), c.change.key()))
        .collect();
    assert!(ops.contains(&"delete:items/I-2".to_owned()), "{ops:?}");
    assert!(ops.contains(&"set:items/I-1".to_owned()), "{ops:?}");
    assert_eq!(
        h.docs("alpha", Coll::Items)["I-1"]["title"],
        "Login screen v2"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_run_definition_removed_from_the_board_leaves_launches_and_launch_defs() {
    let h = Harness::start().await;
    let mut board = h.read("alpha");
    board["runDefs"].as_object_mut().unwrap().remove("fail");
    h.write("alpha", &board);
    h.resync("alpha").await;
    assert!(!h.docs("alpha", Coll::Launches).contains_key("fail"));
    assert_eq!(h.launch_defs("alpha"), ["app", "hello", "slow"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_proof_is_written_under_proofs_stamped_and_its_url_points_at_the_dashboard() {
    let h = Harness::start().await;
    let url = "/files/alpha/proofs/T-2-1.png";
    let r = h
        .proof("T-2", png(), "image/png", "the error")
        .await
        .expect("kept");
    assert_eq!(
        r,
        Proof {
            n: 1,
            url: url.to_owned()
        }
    );
    let test = &h.docs("alpha", Coll::Tests)["T-2"];
    let proofs = test["proofs"].as_array().unwrap();
    assert_eq!(proofs.len(), 1);
    assert_eq!(
        pick(&proofs[0], &["n", "caption", "url"]),
        json!({ "n": 1, "caption": "the error", "url": url })
    );
    assert!(proofs[0].get("file").is_none());
    assert_eq!(test["updatedBy"], "human");
    assert_eq!(
        std::fs::read(h.home().join("alpha/proofs/T-2-1.png")).unwrap(),
        png()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_proof_that_is_not_what_it_says_or_has_no_test_or_no_root_is_refused() {
    let h = Harness::start().await;
    let text = b"not an image".to_vec();
    assert_eq!(
        proof_error(h.proof("T-2", text, "image/png", "x").await),
        ErrorCode::UnsupportedMedia
    );
    assert_eq!(
        proof_error(h.proof("T-2", png(), "image/jpeg", "x").await),
        ErrorCode::UnsupportedMedia
    );
    assert_eq!(
        proof_error(h.proof("T-9", png(), "image/png", "x").await),
        ErrorCode::NotFound
    );
    let outside = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let r = h
        .sync
        .attach_proof("alpha", "T-2", ProofSource::Path(outside), "x", "")
        .await;
    assert!(matches!(
        proof_error(r),
        ErrorCode::Forbidden | ErrorCode::UnsupportedMedia
    ));
    assert_eq!(h.docs("alpha", Coll::Tests)["T-2"]["proofs"], json!([]));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_proof_from_a_path_inside_an_allowed_root_is_kept() {
    let h = Harness::start().await;
    let file = h.home().join("shot.png");
    std::fs::write(&file, png()).unwrap();
    let r = h
        .sync
        .attach_proof("alpha", "T-1", ProofSource::Path(file), "", "")
        .await
        .expect("kept");
    assert_eq!(r.url, "/files/alpha/proofs/T-1-1.png");
    assert_eq!(
        h.docs("alpha", Coll::Tests)["T-1"]["proofs"][0]["caption"],
        "T-1 proof"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn portfolio_cards_follow_a_project_change() {
    let h = Harness::start().await;
    let mut board = h.read("beta");
    board["tests"]["T-1"]["status"] = json!("fail");
    h.write("beta", &board);
    h.resync_with_cards("beta").await;
    let card = &h.docs("_portfolio", Coll::Cards)["beta"];
    assert_eq!(card["counts"]["testsFailing"], 1);
    let attention = card["attention"].as_array().unwrap();
    assert!(
        attention
            .iter()
            .any(|a| a.as_str().is_some_and(|s| s.contains("T-1 failing"))),
        "{attention:?}"
    );
    assert_eq!(h.sync.cards().len(), 2);
}

fn launchers(at: &str, retired: bool) -> Value {
    let master = json!({ "name": "master", "title": "Main", "cmd": "game run", "cwd": null, "kind": "app", "branch": "main", "commit": null, "note": null, "current": true, "retiredAt": retired.then_some(at), "retiredBy": null, "retireNote": null, "createdAt": at, "createdBy": "coordinator", "updatedAt": at, "by": "coordinator" });
    let old = json!({ "name": "old-1", "title": null, "cmd": "game run --old", "cwd": null, "kind": "dev", "branch": null, "commit": null, "note": null, "current": false, "retiredAt": at, "retiredBy": "coordinator", "retireNote": "merged", "createdAt": at, "createdBy": "coordinator", "updatedAt": at, "by": "coordinator" });
    json!({ "master": master, "old-1": old })
}

async fn install_launchers(h: &Harness, retired: bool) {
    let mut board = h.read("alpha");
    board["launchers"] = launchers(AT, retired);
    let cwd = h.home().display().to_string();
    board["runDefs"]["launcher-master"] = json!({ "name": "launcher-master", "label": "Play master", "launcher": "master", "cmd": "game run", "cwd": cwd, "kind": "launch", "env": {}, "hand": true, "visible": false, "window": false, "timeoutSec": 0, "parallel": 1, "definedBy": "coordinator", "definedAt": AT, "rev": 1 });
    h.write("alpha", &board);
    h.resync_with_cards("alpha").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn launchers_are_mirrored_and_a_project_card_carries_its_master_with_its_last_run() {
    let h = Harness::start().await;
    let before = h.docs("_portfolio", Coll::Cards);
    assert_eq!(
        before["alpha"]["master"],
        Value::Null,
        "no master launcher yet"
    );
    assert_eq!(
        before["alpha"]["masterMissing"], true,
        "the thread must set one"
    );
    install_launchers(&h, false).await;
    let docs = h.docs("alpha", Coll::Launchers);
    assert_eq!(h.keys("alpha", Coll::Launchers), ["master", "old-1"]);
    assert_eq!(
        pick(&docs["master"], &["current", "window", "bundle"]),
        json!({ "current": true, "window": false, "bundle": null })
    );
    assert_eq!(docs["old-1"]["retireNote"], "merged");
    assert_eq!(
        h.docs("alpha", Coll::Launches)["launcher-master"]["launcher"],
        "master"
    );
    let card = &h.docs("_portfolio", Coll::Cards)["alpha"];
    let want = json!({ "name": "master", "title": "Main", "cmd": "game run", "current": true, "play": "launcher-master", "window": false });
    assert_eq!(
        pick(
            &card["master"],
            &["name", "title", "cmd", "current", "play", "window"]
        ),
        want
    );
    assert!(
        card.get("masterMissing").is_none(),
        "a project with a master has no flag"
    );
    assert_eq!(
        card["master"]["lastRun"],
        last_run(),
        "no command, no folder"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_retired_master_has_no_play_on_the_card() {
    let h = Harness::start().await;
    install_launchers(&h, true).await;
    let card = &h.docs("_portfolio", Coll::Cards)["alpha"];
    assert_eq!(card["master"]["play"], Value::Null);
    assert_eq!(card["masterMissing"], true, "a retired master is no master");
}

#[tokio::test(flavor = "multi_thread")]
async fn launchers_gone_from_the_board_are_gone_from_the_mirror() {
    let h = Harness::start().await;
    install_launchers(&h, false).await;
    let mut board = h.read("alpha");
    board.as_object_mut().unwrap().remove("launchers");
    board["runDefs"]
        .as_object_mut()
        .unwrap()
        .remove("launcher-master");
    h.write("alpha", &board);
    h.resync_with_cards("alpha").await;
    assert!(h.docs("alpha", Coll::Launchers).is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_hard_resync_clears_the_scope_re_exports_it_and_tells_the_streams_to_reset() {
    let h = Harness::start().await;
    let resets = Arc::new(Mutex::new(Vec::<ResetEvent>::new()));
    let sink = Arc::clone(&resets);
    let sub = h
        .bus
        .on_reset(move |e| sink.lock().unwrap().push(e.clone()));
    let r = h
        .sync
        .resync(Some("beta"), true)
        .await
        .expect("hard resync");
    drop(sub);
    let want = ResetEvent {
        scope: "beta".into(),
        reason: "resync".into(),
    };
    assert_eq!(*resets.lock().unwrap(), [want]);
    assert!(r.docs > 3);
    assert_eq!(h.docs("beta", Coll::Items).len(), 1);
    let missing = h
        .sync
        .resync(Some("nope"), false)
        .await
        .expect_err("no such project");
    assert_eq!(missing.code, ErrorCode::NotFound);
}

#[tokio::test(flavor = "multi_thread")]
async fn health_reports_the_watcher() {
    let h = Harness::start().await;
    let s = h.sync.health();
    assert!(s.ok, "the watcher is running");
    assert_eq!(s.errors, 0);
}
