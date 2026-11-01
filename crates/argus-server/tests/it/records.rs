//! The local remote over HTTP, on temp homes, a temp release store, a temp check folder and a
//! temp localpkg home: the switch and the replay counts, the release API, the update feed
//! (ranges, types, refusals, the feed host), the check runs with numbers that survive a
//! restart, their logs, and the registry. Never the real `~/.coord`, never port 80 or 47615.
//! K2, K3, K4: the local remote, the release feed, the check runs and the registry over HTTP; the Node status script is in `records_node.rs`.
// test-tag: standard

use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::wait::until;
use argus_core::fixture::{check_run_record, put_file as put};
use argus_server::{ConsumerSource, ConsumerStatus};
use argus_testkit::{
    Reply, Request, TestOptions, TestServer, alpha_board, make_home, request_at, write_board,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tempfile::TempDir;

const LATEST: &str = "version: VER\nfiles:\n  - url: App-Setup-VER.exe\n    sha512: abc==\n    size: 10\npath: App-Setup-VER.exe\nsha512: abc==\n";
const EXE: &[u8] = b"0123456789";

fn release(home: &Path, channel: &str, version: &str, replay: &str) {
    let base = format!("releases/brock/{channel}/{version}");
    let record = json!({
        "kind": "release", "project": "brock", "version": version, "tag": format!("v{version}"),
        "commit": "abc1234", "title": format!("Brock {version}"), "notes": "notes.md",
        "packages": [{ "name": "@d/core", "version": version, "tarball": "core.tgz", "sha512": "x", "size": 3 }],
        "assets": [{ "name": "App-Setup.exe", "path": format!("App-Setup-{version}.exe"), "size": 10, "kind": "installer" }],
        "publishedAt": "2026-10-29T10:00:00Z", "mode": "local", "replay": { "state": replay },
    });
    put(home, &format!("{base}/release.json"), record.to_string());
    put(home, &format!("{base}/notes.md"), "# notes");
    put(home, &format!("{base}/core.tgz"), "tgz");
    put(home, &format!("{base}/App-Setup-{version}.exe"), EXE);
    put(
        home,
        &format!("{base}/App-Setup-{version}.exe.blockmap"),
        "map",
    );
    put(
        home,
        &format!("{base}/latest.yml"),
        LATEST.replace("VER", version),
    );
}

fn check_run(home: &Path, project: &str, id: &str, started: &str, commit: &str, pr: &str) {
    let record = check_run_record(project, id, started, commit, pr);
    put(home, &format!("ci/{project}/{id}.json"), record.to_string());
    put(home, &format!("ci/{project}/{id}/lint.log"), "lint ok\n");
}

struct Rig {
    t: TestServer,
    dirs: Vec<TempDir>,
}

impl Rig {
    fn home(&self) -> &Path {
        &self.t.coord_home
    }
}

async fn start_with(
    home: PathBuf,
    dashboard: Option<PathBuf>,
    tweak: impl Fn(&mut argus_server::ServeOptions) + Send + Sync + 'static,
) -> TestServer {
    TestServer::start(TestOptions {
        coord_home: Some(home),
        dashboard_home: dashboard,
        tweak: Some(Arc::new(tweak)),
        ..TestOptions::default()
    })
    .await
}

async fn rig(prepare: impl FnOnce(&Path)) -> Rig {
    let home = tempfile::tempdir().expect("coord home");
    let pkg = tempfile::tempdir().expect("localpkg home");
    make_home(home.path());
    prepare(home.path());
    let pkg_home = pkg.path().to_path_buf();
    let t = start_with(home.path().to_path_buf(), None, move |o| {
        o.records.localpkg_home = Some(pkg_home.clone());
        o.localhost_door_port = Some(0);
    })
    .await;
    Rig {
        t,
        dirs: vec![home, pkg],
    }
}

fn door(t: &TestServer) -> SocketAddr {
    let port = t.svc.localhost_door().expect("the localhost door is bound");
    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

async fn feed(t: &TestServer, req: Request) -> Reply {
    t.send(req).await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_switch_and_the_replay_counts_are_reported() {
    let r = rig(|home| {
        release(home, "latest", "1.0.0", "pending");
        release(home, "latest", "1.1.0", "done");
        check_run(
            home,
            "brock",
            "run-a",
            "2026-10-29T10:00:00Z",
            "aaaaaaa1",
            "",
        );
        let mut alpha = alpha_board(home);
        alpha["prs"] = json!({
            "PR-1": { "id": "PR-1", "status": "merged", "replay": { "state": "pending" } },
            "PR-2": { "id": "PR-2", "status": "open", "remote": { "state": "pending" } },
            "PR-3": { "id": "PR-3", "status": "merged", "replay": { "state": "done" } },
        });
        write_board(home, "alpha", &alpha);
    })
    .await;
    let body = r.t.get("/api/remote").await.json();
    assert_eq!(body["mode"], "local", "no switch file means local");
    assert_eq!(
        body["replay"],
        json!({ "prs": 2, "releases": 1, "packages": 0, "checks": 1 })
    );
    put(r.home(), "remote.json", r#"{ "mode": "remote" }"#);
    assert_eq!(r.t.get("/api/remote").await.json()["mode"], "remote");
    assert_eq!(body["ciRoots"], json!({}));
    put(
        r.home(),
        "remote.json",
        r#"{ "mode": "local", "ciRoots": { "brock": "X:/brock", "argus": "X:/argus" } }"#,
    );
    let roots = r.t.get("/api/remote").await.json()["ciRoots"].clone();
    assert_eq!(roots, json!({ "brock": "X:/brock", "argus": "X:/argus" }));
    assert_eq!(body["syncs"], json!({}), "no record yet");
    put(
        r.home(),
        "remote.json",
        r#"{ "mode": "local", "syncs": { "alpha": { "at": "2026-11-01T10:00:00.000Z", "by": "alpha", "note": "pushed 2 PRs" }, "beta": { "note": "no time" } } }"#,
    );
    let synced = r.t.get("/api/remote").await.json()["syncs"].clone();
    assert_eq!(
        synced,
        json!({ "alpha": { "at": "2026-11-01T10:00:00.000Z", "by": "alpha", "note": "pushed 2 PRs" } }),
        "an entry without a time is left out"
    );
    put(r.home(), "remote.json", "{ torn");
    assert_eq!(r.t.get("/api/remote").await.json()["mode"], "local");
    let mirrored = r.t.svc.db.docs().list("alpha", argus_core::Coll::Prs);
    assert_eq!(
        mirrored.expect("prs docs").len(),
        3,
        "prs are in the board stream"
    );
    r.t.stop(false).await;
    drop(r.dirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_release_api_lists_versions_newest_first_and_a_release_is_indexed_when_it_appears() {
    let r = rig(|home| release(home, "latest", "1.9.0", "done")).await;
    let all = r.t.get("/api/releases").await.json();
    let channel = &all["projects"][0]["channels"][0];
    assert_eq!(all["projects"][0]["project"], "brock");
    assert_eq!(
        (channel["channel"].as_str(), channel["latest"].as_str()),
        (Some("latest"), Some("1.9.0"))
    );
    release(r.home(), "latest", "1.10.0", "pending");
    let t = &r.t;
    let one = until("the new release in the index", || async move {
        let body = t.get("/api/p/brock/releases").await.json();
        (body["channels"][0]["versions"].as_array()?.len() == 2).then_some(body)
    })
    .await;
    let versions = &one["channels"][0]["versions"];
    assert_eq!(versions[0]["version"], "1.10.0");
    assert_eq!(versions[0]["replay"]["state"], "pending");
    assert_eq!(
        versions[0]["notesUrl"],
        "/releases/brock/latest/1.10.0/notes.md"
    );
    assert_eq!(
        versions[0]["assets"][0]["url"],
        "/releases/brock/latest/1.10.0/App-Setup-1.10.0.exe"
    );
    assert_eq!(versions[0]["packages"][0]["size"], 3);
    assert_eq!(one["channels"][0]["latest"], "1.10.0");
    assert_eq!(r.t.get("/api/p/Bad_Key/releases").await.status, 400);
    r.t.stop(false).await;
    drop(r.dirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_feed_serves_files_with_types_ranges_and_head() {
    let r = rig(|home| release(home, "latest", "1.0.0", "done")).await;
    let exe = "/releases/brock/latest/1.0.0/App-Setup-1.0.0.exe";
    let whole = feed(&r.t, Request::new("GET", exe)).await;
    assert_eq!((whole.status, whole.body.as_slice()), (200, EXE));
    assert_eq!(whole.header("accept-ranges"), Some("bytes"));
    assert_eq!(
        whole.header("content-type"),
        Some("application/octet-stream")
    );
    let part = feed(&r.t, Request::new("GET", exe).header("Range", "bytes=2-5")).await;
    assert_eq!((part.status, part.body.as_slice()), (206, &EXE[2..6]));
    assert_eq!(part.header("content-range"), Some("bytes 2-5/10"));
    assert_eq!(part.header("content-length"), Some("4"));
    let tail = feed(&r.t, Request::new("GET", exe).header("Range", "bytes=-3")).await;
    assert_eq!((tail.status, tail.body.as_slice()), (206, &EXE[7..]));
    let beyond = feed(
        &r.t,
        Request::new("GET", exe).header("Range", "bytes=10-20"),
    )
    .await;
    assert_eq!(
        (beyond.status, beyond.header("content-range")),
        (416, Some("bytes */10"))
    );
    let head = feed(&r.t, Request::new("HEAD", exe)).await;
    assert_eq!(
        (head.status, head.header("content-length"), head.body.len()),
        (200, Some("10"), 0)
    );
    let tag = whole.header("etag").expect("etag").to_owned();
    let same = Request::new("GET", exe)
        .header("Range", "bytes=0-1")
        .header("If-Range", &tag);
    assert_eq!(feed(&r.t, same).await.status, 206);
    let other = Request::new("GET", exe)
        .header("Range", "bytes=0-1")
        .header("If-Range", "\"x\"");
    assert_eq!(
        feed(&r.t, other).await.body,
        EXE,
        "another version: the whole file"
    );
    let map = feed(&r.t, Request::new("GET", &format!("{exe}.blockmap"))).await;
    assert_eq!(map.header("content-type"), Some("application/octet-stream"));
    let tgz = feed(
        &r.t,
        Request::new("GET", "/releases/brock/latest/1.0.0/core.tgz"),
    )
    .await;
    assert_eq!(tgz.header("content-type"), Some("application/gzip"));
    r.t.stop(false).await;
    drop(r.dirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_channel_manifest_is_made_from_the_newest_version_and_a_file_on_disk_wins() {
    let r = rig(|home| {
        release(home, "latest", "1.9.0", "done");
        release(home, "latest", "1.10.0", "done");
    })
    .await;
    let made = feed(
        &r.t,
        Request::new("GET", "/releases/brock/latest/latest.yml"),
    )
    .await;
    assert_eq!(made.status, 200);
    assert_eq!(
        made.header("content-type"),
        Some("text/yaml; charset=utf-8")
    );
    let text = made.text();
    assert!(text.starts_with("version: 1.10.0\n"), "{text}");
    assert!(text.contains("  - url: 1.10.0/App-Setup-1.10.0.exe\n"));
    assert!(text.contains("\npath: 1.10.0/App-Setup-1.10.0.exe\n"));
    let missing = feed(
        &r.t,
        Request::new("GET", "/releases/brock/latest/latest-mac.yml"),
    )
    .await;
    assert_eq!(missing.status, 404);
    put(
        r.home(),
        "releases/brock/latest/latest.yml",
        "version: 1.9.0\npath: 1.9.0/x.exe\n",
    );
    let on_disk = feed(
        &r.t,
        Request::new("GET", "/releases/brock/latest/latest.yml"),
    )
    .await;
    assert!(on_disk.text().starts_with("version: 1.9.0"));
    r.t.stop(false).await;
    drop(r.dirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_feed_refuses_what_leaves_the_store_what_lists_it_and_every_write() {
    let r = rig(|home| {
        release(home, "latest", "1.0.0", "done");
        put(home, "secret.txt", "no");
        put(
            home,
            "releases/brock/latest/1.0.0/App-Setup-1.0.0.exe.tmp",
            "half",
        );
    })
    .await;
    for bad in [
        "/releases/",
        "/releases/brock",
        "/releases/brock/latest/1.0.0/",
        "/releases/brock/latest/1.0.0",
        "/releases/..%2fsecret.txt",
        "/releases/brock/..%2f..%2fsecret.txt",
        "/releases/brock/latest/1.0.0/..%2f..%2f..%2f..%2fsecret.txt",
        "/releases/brock/latest/1.0.0/%2e%2e%2f%2e%2e%2f%2e%2e%2f%2e%2e%2fsecret.txt",
        "/releases/brock/latest/1.0.0/..%5c..%5c..%5c..%5csecret.txt",
        "/releases/%2fetc%2fpasswd",
        "/releases/C%3a%2fWindows%2fwin.ini",
        "/releases/brock/latest/1.0.0/App-Setup-1.0.0.exe.tmp",
        "/releases/brock/latest/1.0.0/%00",
    ] {
        let got = feed(&r.t, Request::new("GET", bad)).await;
        assert!(
            [400, 404].contains(&got.status),
            "{bad} answered {} {}",
            got.status,
            got.text()
        );
        assert!(!got.text().contains("no\n") && got.text() != "no", "{bad}");
    }
    let post = r
        .t
        .send(Request::new("POST", "/releases/brock/latest/1.0.0/release.json").json(&json!({})))
        .await;
    assert_eq!(post.status, 405);
    assert_eq!(post.header("allow"), Some("GET"));
    r.t.stop(false).await;
    drop(r.dirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_feed_host_serves_the_store_without_the_prefix_and_boards_keep_their_hosts() {
    let r = rig(|home| release(home, "latest", "1.0.0", "done")).await;
    let port = door(&r.t).port();
    let host = format!("releases.localhost:{port}");
    let at = |path: &str| Request::new("GET", path).header("Host", &host);
    let got = request_at(door(&r.t), &at("/brock/latest/latest.yml")).await;
    assert_eq!(got.status, 200);
    assert!(got.text().contains("1.0.0/App-Setup-1.0.0.exe"));
    let ranged = at("/brock/latest/1.0.0/App-Setup-1.0.0.exe").header("Range", "bytes=0-3");
    assert_eq!(request_at(door(&r.t), &ranged).await.status, 206);
    assert_eq!(request_at(door(&r.t), &at("/")).await.status, 404);
    assert_eq!(request_at(door(&r.t), &at("/api/health")).await.status, 404);
    let board = Request::new("GET", "/").header("Host", &format!("alpha.localhost:{port}"));
    assert_eq!(request_at(door(&r.t), &board).await.status, 302);
    let evil = Request::new("GET", "/brock/latest/latest.yml").header("Host", "releases.evil.test");
    assert_eq!(request_at(door(&r.t), &evil).await.status, 421);
    r.t.stop(false).await;
    drop(r.dirs);
}

fn two_runs(home: &Path) {
    check_run(
        home,
        "brock",
        "run-b",
        "2026-10-29T10:05:00Z",
        "bbbbbbb1234567",
        "PR-2",
    );
    check_run(
        home,
        "brock",
        "run-a",
        "2026-10-29T10:00:00Z",
        "aaaaaaa1234567",
        "PR-1",
    );
    put(home, "ci/brock/run-t.json.tmp", "{}");
}

#[tokio::test(flavor = "multi_thread")]
async fn check_runs_are_listed_with_numbers_and_filtered() {
    let r = rig(two_runs).await;
    let all = r.t.get("/api/p/brock/checks").await.json();
    let ids: Vec<&str> = all["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter_map(|x| x["id"].as_str())
        .collect();
    assert_eq!(
        ids,
        ["C-2", "C-1"],
        "first started is C-1, newest listed first"
    );
    assert_eq!(all["runs"][1]["runId"], "run-a");
    assert_eq!(all["runs"][0]["steps"][0]["command"], "pnpm lint");
    let by_commit = r.t.get("/api/p/brock/checks?commit=aaaaaaa").await.json();
    assert_eq!(by_commit["runs"][0]["id"], "C-1");
    let by_pr = r.t.get("/api/p/brock/checks?pr=PR-2").await.json();
    assert_eq!(by_pr["runs"].as_array().map(Vec::len), Some(1));
    r.t.stop(false).await;
    drop(r.dirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn step_logs_are_served_and_a_run_is_indexed_when_its_file_appears() {
    let r = rig(two_runs).await;
    let log = r.t.get("/api/p/brock/checks/C-1/log?step=0").await;
    assert_eq!(
        (log.status, log.text().as_str()),
        (
            200, "lint ok
"
        )
    );
    assert_eq!(
        r.t.get("/api/p/brock/checks/C-1/log?step=3").await.status,
        404
    );
    assert_eq!(
        r.t.get("/api/p/brock/checks/C-9/log?step=0").await.status,
        404
    );
    assert_eq!(r.t.get("/api/p/brock/checks/C-1/log").await.status, 400);
    check_run(
        r.home(),
        "brock",
        "run-c",
        "2026-10-29T11:00:00Z",
        "ccccccc1234567",
        "PR-3",
    );
    let t = &r.t;
    let grown = until("the new run in the index", || async move {
        let body = t.get("/api/p/brock/checks").await.json();
        (body["runs"].as_array()?.len() == 3).then_some(body)
    })
    .await;
    assert_eq!(grown["runs"][0]["id"], "C-3");
    r.t.stop(false).await;
    drop(r.dirs);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_step_log_outside_the_store_is_refused() {
    let r = rig(|home| {
        check_run(
            home,
            "brock",
            "run-a",
            "2026-10-29T10:00:00Z",
            "aaaaaaa1",
            "",
        );
        put(home, "secret.log", "no");
        let outside = home.join("secret.log").display().to_string();
        let mut record: Value =
            serde_json::from_slice(&fs::read(home.join("ci/brock/run-a.json")).expect("record"))
                .expect("json");
        record["steps"]
            .as_array_mut()
            .expect("steps")
            .push(json!({ "name": "x", "log": outside }));
        record["steps"]
            .as_array_mut()
            .expect("steps")
            .push(json!({ "name": "y", "log": "../../secret.log" }));
        put(home, "ci/brock/run-a.json", record.to_string());
    })
    .await;
    assert_eq!(
        r.t.get("/api/p/brock/checks/C-1/log?step=0").await.status,
        200
    );
    assert_eq!(
        r.t.get("/api/p/brock/checks/C-1/log?step=1").await.status,
        404,
        "an absolute path"
    );
    assert_eq!(
        r.t.get("/api/p/brock/checks/C-1/log?step=2").await.status,
        404,
        "a path that climbs out"
    );
    r.t.stop(false).await;
    drop(r.dirs);
}

fn id_rows(body: &Value) -> Vec<(String, String)> {
    let text = |x: &Value, key: &str| x[key].as_str().unwrap_or_default().to_owned();
    let runs = body["runs"].as_array().expect("runs");
    let mut rows: Vec<(String, String)> = runs
        .iter()
        .map(|x| (text(x, "runId"), text(x, "id")))
        .collect();
    rows.sort();
    rows
}

#[tokio::test(flavor = "multi_thread")]
async fn check_numbers_stay_the_same_after_a_restart() {
    let home = tempfile::tempdir().expect("coord home");
    let dash = tempfile::tempdir().expect("dashboard home");
    check_run(
        home.path(),
        "brock",
        "run-a",
        "2026-10-29T10:00:00Z",
        "aaaaaaa1",
        "",
    );
    check_run(
        home.path(),
        "brock",
        "run-b",
        "2026-10-29T10:05:00Z",
        "bbbbbbb1",
        "",
    );
    let at = |h: &TempDir, d: &TempDir| {
        start_with(h.path().to_path_buf(), Some(d.path().to_path_buf()), |_| {})
    };
    let first = at(&home, &dash).await;
    let before = id_rows(&first.get("/api/p/brock/checks").await.json());
    assert_eq!(
        before,
        [
            ("run-a".into(), "C-1".into()),
            ("run-b".into(), "C-2".into())
        ]
    );
    first.stop(true).await;
    check_run(
        home.path(),
        "brock",
        "run-0",
        "2026-10-29T08:00:00Z",
        "0000000a",
        "",
    );
    let second = at(&home, &dash).await;
    let after = id_rows(&second.get("/api/p/brock/checks").await.json());
    let want = [("run-0", "C-3"), ("run-a", "C-1"), ("run-b", "C-2")];
    assert_eq!(after, want.map(|(a, b)| (a.to_owned(), b.to_owned())));
    second.stop(false).await;
}

struct Counted {
    calls: Arc<AtomicUsize>,
    answer: Result<String, String>,
}

#[async_trait]
impl ConsumerStatus for Counted {
    async fn status_json(&self) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.answer.clone()
    }
}

async fn registry_rig(source: ConsumerSource) -> (TestServer, TempDir, TempDir) {
    let home = tempfile::tempdir().expect("coord home");
    let pkg = tempfile::tempdir().expect("localpkg home");
    let index = json!({ "packages": {
        "@drizztdourden08/brock-core": { "version": "0.115.5", "tarball": "tgz/core.tgz", "project": "X:/brock", "head": "abc1234", "packedAt": "2026-10-29T09:00:00Z" },
    }});
    put(pkg.path(), "index.json", index.to_string());
    put(pkg.path(), "tgz/core.tgz", "12345678");
    let pkg_home = pkg.path().to_path_buf();
    let t = start_with(home.path().to_path_buf(), None, move |o| {
        o.records.localpkg_home = Some(pkg_home.clone());
        o.records.consumers = source.clone();
    })
    .await;
    (t, home, pkg)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_registry_lists_builds_and_consumers_and_asks_the_tool_once_in_30_seconds() {
    let calls = Arc::new(AtomicUsize::new(0));
    let status =
        r#"{ "builds": [], "consumers": [ { "root": "X:/rotp", "since": "t", "hook": true } ] }"#;
    let source = ConsumerSource::Custom(Arc::new(Counted {
        calls: Arc::clone(&calls),
        answer: Ok(status.into()),
    }));
    let (t, _home, _pkg) = registry_rig(source).await;
    let body = t.get("/api/registry").await.json();
    assert_eq!(body["builds"][0]["name"], "@drizztdourden08/brock-core");
    assert_eq!(body["builds"][0]["version"], "0.115.5");
    assert_eq!(body["builds"][0]["commit"], "abc1234");
    assert_eq!(body["builds"][0]["checkout"], "X:/brock");
    assert_eq!(body["builds"][0]["size"], 8);
    assert_eq!(body["consumers"][0]["path"], "X:/rotp");
    assert_eq!(body["consumers"][0]["on"], true);
    assert_eq!(body["scopeRedirected"], true);
    t.get("/api/registry").await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_tool_without_the_json_flag_leaves_the_consumers_empty_with_a_note() {
    let calls = Arc::new(AtomicUsize::new(0));
    let source = ConsumerSource::Custom(Arc::new(Counted {
        calls,
        answer: Err("localpkg status --json failed".into()),
    }));
    let (t, _home, _pkg) = registry_rig(source).await;
    let body = t.get("/api/registry").await.json();
    assert_eq!(body["consumers"], json!([]));
    assert_eq!(body["scopeRedirected"], false);
    assert_eq!(body["consumersNote"], "localpkg status --json failed");
    assert_eq!(body["builds"].as_array().map(Vec::len), Some(1));
    t.stop(false).await;
    let (off, _h, _p) = registry_rig(ConsumerSource::Off).await;
    let none = off.get("/api/registry").await.json();
    assert_eq!(none["consumers"], json!([]));
    off.stop(false).await;
}
