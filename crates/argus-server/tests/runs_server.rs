//! The run service inside the whole server, with real processes: the fixture launches of the
//! DESIGN 9 home, the run docs on the page stream, the Origin and content-type rules of the start
//! route, Stop ending the process tree, and a graceful restart. Ported from
//! `runs.server.test.mjs`.

use std::time::Duration;

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::{Value, json};

async fn start_server(home: Option<(std::path::PathBuf, std::path::PathBuf)>) -> TestServer {
    let fresh = home.is_none();
    let (coord_home, dashboard_home) = home.unzip();
    TestServer::start(TestOptions {
        seed: fresh,
        coord_home,
        dashboard_home,
        ..TestOptions::default()
    })
    .await
}

async fn page_start(t: &TestServer, name: &str) -> Value {
    let request = Request::new("POST", "/api/p/alpha/runs").json(&json!({ "name": name }));
    let reply = t.send(t.page(request)).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    reply.json()
}

async fn finished(t: &TestServer, rid: &str) -> Value {
    for _ in 0..200 {
        let run = t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone();
        if run["status"] != "running" && run["status"] != "queued" {
            return run;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("{rid} did not end");
}

#[cfg(windows)]
fn alive(pid: u32) -> bool {
    argus_native::process_alive(pid)
}

#[cfg(not(windows))]
fn alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

async fn gone(pid: u32) {
    for _ in 0..60 {
        if !alive(pid) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("process {pid} is still alive");
}

/// The run doc that says `exited`, as the page stream carries it.
async fn finished_doc(stream: &argus_testkit::SseReader) -> Value {
    let done = stream
        .next_where(
            |e| {
                e.event == "changes"
                    && e.data["changes"].as_array().is_some_and(|cs| {
                        cs.iter()
                            .any(|c| c["coll"] == "runs" && c["data"]["status"] == "exited")
                    })
            },
            15_000,
        )
        .await
        .expect("the finished run doc arrives over the stream");
    done.data["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["coll"] == "runs")
        .unwrap()["data"]
        .clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_fixture_launches_hello_runs_from_the_page_and_its_run_doc_reaches_the_stream() {
    let t = start_server(None).await;
    let launches = t.get("/api/p/alpha/launches").await.json();
    let names: Vec<&str> = launches["defs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["name"].as_str())
        .collect();
    assert_eq!(names, ["app", "fail", "hello", "slow"]);

    let stream = t.sse("/api/p/alpha/stream?c=runs,launches").await;
    assert!(stream.next(Some("ready"), 5000).await.is_some());
    let started = page_start(&t, "hello").await;
    assert_eq!(
        (&started["run"]["origin"], &started["run"]["by"]),
        (&json!("page"), &json!("human"))
    );
    let doc = finished_doc(&stream).await;
    for key in [
        "id",
        "name",
        "status",
        "agent",
        "startedAt",
        "lastLine",
        "origin",
        "exitCode",
    ] {
        assert!(doc.get(key).is_some(), "the run doc has {key}");
    }
    assert!(
        doc.get("cmd").is_none() && doc.get("cwd").is_none(),
        "the page never sees the command"
    );
    let rid = started["run"]["id"].as_str().unwrap();
    let logs = t.get(&format!("/api/p/alpha/runs/{rid}/logs")).await.json();
    assert!(
        logs["lines"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["stream"] == "out" && l["text"] == "hi")
    );
    let all = t.get("/api/runs?status=exited").await.json();
    assert!(
        all["runs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["scope"].is_string() && r["id"].is_string())
    );
    stream.close();
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cross_origin_start_is_refused_a_failing_launch_keeps_its_code_and_unknown_names_are_404()
{
    let t = start_server(None).await;
    let foreign = Request::new("POST", "/api/p/alpha/runs")
        .header("Origin", "http://evil.test")
        .header("Sec-Fetch-Site", "cross-site")
        .json(&json!({ "name": "hello" }));
    assert_eq!(t.send(foreign).await.status, 403);
    let form = Request::new("POST", "/api/p/alpha/runs")
        .header("Content-Type", "text/plain")
        .body(r#"{"name":"hello"}"#);
    assert_eq!(t.send(form).await.status, 415);
    let fail = t
        .write("POST", "/api/p/alpha/runs", &json!({ "name": "fail" }))
        .await
        .json();
    let run = finished(&t, fail["run"]["id"].as_str().unwrap()).await;
    assert_eq!(
        (&run["status"], &run["exitCode"]),
        (&json!("failed"), &json!(3))
    );
    let ghost = t
        .write("POST", "/api/p/alpha/runs", &json!({ "name": "ghost" }))
        .await;
    assert_eq!((ghost.status, ghost.code().as_str()), (404, "unknown_run"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn slow_play_then_already_then_stop_ends_the_process_tree() {
    let t = start_server(None).await;
    let first = page_start(&t, "slow").await;
    let second = page_start(&t, "slow").await;
    assert_eq!(
        (&second["already"], &second["run"]["id"]),
        (&json!(true), &first["run"]["id"])
    );
    let rid = first["run"]["id"].as_str().unwrap().to_owned();
    let pid = u32::try_from(first["run"]["pid"].as_u64().expect("a pid")).unwrap();
    let began = std::time::Instant::now();
    let request = Request::new("POST", &format!("/api/p/alpha/runs/{rid}/stop")).json(&json!({}));
    let stopped = t.send(t.page(request)).await.json();
    assert_eq!(stopped["run"]["status"], "stopped");
    assert!(began.elapsed() < Duration::from_secs(6));
    gone(pid).await;
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_graceful_restart_stops_hidden_runs_as_server_shutdown_and_the_counters_continue() {
    let t = start_server(None).await;
    let run = t
        .write("POST", "/api/p/alpha/runs", &json!({ "name": "slow" }))
        .await
        .json()["run"]
        .clone();
    let rid = run["id"].as_str().unwrap().to_owned();
    let pid = u32::try_from(run["pid"].as_u64().expect("a pid")).unwrap();
    let homes = t.stop(true).await;
    gone(pid).await;
    let t = start_server(Some(homes)).await;
    let after = t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone();
    assert_eq!(
        (&after["status"], &after["by"]),
        (&json!("stopped"), &json!("server shutdown"))
    );
    let fresh = t
        .write("POST", "/api/p/alpha/runs", &json!({ "name": "hello" }))
        .await
        .json()["run"]
        .clone();
    assert_ne!(fresh["id"], run["id"], "the counters continue");
    finished(&t, fresh["id"].as_str().unwrap()).await;
    t.stop(false).await;
}
