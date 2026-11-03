//! The run routes over real HTTP, with pipes the test drives by hand: start, already, stop,
//! list, logs, the log stream, stdin, attachments, launches and the error shape. Ported from
//! `runs.http.test.mjs` and the HTTP side of `runs.cli.test.mjs`; the real processes of
//! `runs.server.test.mjs` are in `runs_server.rs`, the stored rows in `runs_store.rs`.
//! K2, K3, K4: the run routes, logs, streams, attachments and launches over a fake process host.
// test-tag: standard

use crate::runs_rig::{Rig, until};
use argus_testkit::{Event, Request, SseReader, TestOptions, png};
use serde_json::{Value, json};

fn pid_of(run: &Value) -> u32 {
    u32::try_from(run["pid"].as_u64().expect("a pid")).expect("a pid fits")
}

fn ids(list: &Value, field: &str) -> Vec<String> {
    let rows = list[field].as_array().cloned().unwrap_or_default();
    rows.iter()
        .map(|r| {
            format!(
                "{}{}",
                r["scope"]
                    .as_str()
                    .map_or(String::new(), |s| format!("{s}/")),
                r["id"].as_str().unwrap_or_default()
            )
        })
        .collect()
}

/// Every event until the server ends the stream.
async fn drain(sse: &SseReader) -> Vec<Event> {
    let mut out = Vec::new();
    while let Some(event) = sse.next(None, 5000).await {
        out.push(event);
    }
    out
}

async fn start(r: &Rig, body: Value) -> argus_testkit::Reply {
    r.cli("POST", "/api/p/alpha/runs", &body).await
}

/// The owner's page presses Play on a launch.
async fn play(r: &Rig, name: &str, extra: Value) -> argus_testkit::Reply {
    let mut body = json!({ "name": name });
    body.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    r.page("POST", "/api/p/alpha/runs", &body).await
}

fn lines(events: &[Event]) -> Vec<&Event> {
    events.iter().filter(|e| e.event == "log").collect()
}

/// The run reads: the command is shown, and the three listings filter by status, name and scope.
async fn listings(r: &Rig) {
    let one = r.run("R-1").await;
    assert!(
        one["cmd"].is_string() && one["cwd"].is_string(),
        "the API shows the command"
    );
    let running = r.t.get("/api/p/alpha/runs?status=running").await.json();
    assert_eq!(ids(&running, "runs"), ["alpha/R-1"]);
    let named = r.t.get("/api/p/alpha/runs?name=hello&limit=5").await.json();
    assert_eq!(ids(&named, "runs"), ["alpha/R-2"]);
    let everywhere = r.t.get("/api/runs?status=running").await.json();
    assert_eq!(ids(&everywhere, "runs"), ["alpha/R-1"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn start_already_get_list_and_stop_the_page_is_human_the_cli_is_not() {
    let r = Rig::start().await;
    r.define(&[("slow", json!({})), ("hello", json!({}))]);
    let page = r
        .page(
            "POST",
            "/api/p/alpha/runs",
            &json!({ "name": "slow", "reason": "try it" }),
        )
        .await
        .json();
    let run = &page["run"];
    assert_eq!(page["already"], false);
    assert_eq!(
        [
            &run["origin"],
            &run["by"],
            &run["reason"],
            &run["id"],
            &run["status"]
        ],
        [
            &json!("page"),
            &json!("human"),
            &json!("try it"),
            &json!("R-1"),
            &json!("running")
        ]
    );
    let again = r
        .cli("POST", "/api/p/alpha/runs", &json!({ "name": "slow" }))
        .await
        .json();
    assert_eq!(
        (&again["already"], &again["run"]["id"]),
        (&json!(true), &json!("R-1"))
    );

    let body = json!({ "name": "hello", "by": "worker-1", "agent": "A-3" });
    let cli = r.cli("POST", "/api/p/alpha/runs", &body).await.json()["run"].clone();
    assert_eq!(
        [&cli["origin"], &cli["by"], &cli["agent"]],
        [&json!("cli"), &json!("worker-1"), &json!("A-3")]
    );
    r.host.exit(pid_of(&cli), Some(0)).await;
    r.until_status("R-2", "exited").await;

    listings(&r).await;

    let stopped = r
        .page("POST", "/api/p/alpha/runs/R-1/stop", &json!({}))
        .await
        .json();
    assert_eq!(stopped["run"]["status"], "stopped");
    assert_eq!(stopped["run"]["by"], "human");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn launches_hide_env_values_and_errors_use_the_one_shape() {
    let r = Rig::start().await;
    r.define(&[
        (
            "build",
            json!({ "kind": "build", "env": { "TOKEN": "abc" } }),
        ),
        ("app", json!({ "hand": true, "visible": true })),
    ]);
    let launches = r.t.get("/api/p/alpha/launches").await;
    let defs = launches.json()["defs"].clone();
    let names: Vec<&str> = defs
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["name"].as_str())
        .collect();
    assert_eq!(names, ["app", "build"]);
    assert_eq!(defs[1]["env"], json!({}));
    assert_eq!(defs[1]["timeoutSec"], 0, "the board said 0");
    assert_eq!(defs[0]["hand"], true);
    assert!(!launches.text().contains("abc"));

    let unknown = start(&r, json!({ "name": "ghost" })).await;
    assert_eq!(
        (unknown.status, unknown.code().as_str()),
        (404, "unknown_run")
    );
    assert!(
        unknown.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("ghost")
    );
    assert_eq!(start(&r, json!({})).await.status, 400);
    assert_eq!(
        r.t.get("/api/p/alpha/runs/R-99").await.code(),
        "unknown_run"
    );
    assert_eq!(r.t.get("/api/p/alpha/runs/xx").await.status, 400);
    assert_eq!(r.t.get("/api/p/Bad_Key/launches").await.status, 400);
    assert_eq!(
        r.cli("POST", "/api/p/alpha/runs/R-99/stop", &json!({}))
            .await
            .status,
        404
    );
    let sneaky = start(&r, json!({ "name": "ghost", "cmd": "calc.exe" })).await;
    assert_eq!(sneaky.status, 404, "only the name counts");
    let array = r.cli("POST", "/api/p/alpha/runs", &json!([1])).await;
    assert_eq!((array.status, array.code().as_str()), (400, "bad_request"));
    let visible = start(&r, json!({ "name": "build", "visible": true })).await;
    assert_eq!(visible.status, 400, "only a handed build has a window");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn logs_page_by_cursor_and_by_tail() {
    let r = Rig::start().await;
    r.define(&[("job", json!({}))]);
    let pid = pid_of(&r.begin("job").await);
    let text = "n1
n2
n3
n4
n5
n6
";
    r.host.say(pid, text).await;
    r.host.exit(pid, Some(0)).await;
    r.until_status("R-1", "exited").await;
    let page1 =
        r.t.get("/api/p/alpha/runs/R-1/logs?after=0&limit=3")
            .await
            .json();
    assert_eq!(
        (page1["lines"].as_array().unwrap().len(), &page1["next"]),
        (3, &json!(3))
    );
    let page2 = r.t.get("/api/p/alpha/runs/R-1/logs?after=3").await.json();
    assert_eq!(page2["lines"][0]["seq"], 4);
    let tail =
        r.t.get("/api/p/alpha/runs/R-1/logs?tail=2&stream=out")
            .await
            .json();
    let texts: Vec<&str> = tail["lines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l["text"].as_str())
        .collect();
    assert_eq!(texts, ["n5", "n6"]);
    assert_eq!(tail["dropped"], 0);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_log_stream_replays_goes_live_sends_status_and_ends_after_the_final_one() {
    let r = Rig::start().await;
    r.define(&[("job", json!({}))]);
    let pid = pid_of(&r.begin("job").await);
    r.host.say(pid, "first\n").await;
    r.until_line("R-1", "first").await;
    let sse = r.tail("R-1", "?after=0").await;
    assert_eq!(sse.status, 200);
    assert_eq!(
        sse.headers
            .iter()
            .find(|(k, _)| k == "content-type")
            .map(|(_, v)| v.as_str()),
        Some("text/event-stream; charset=utf-8")
    );
    r.host.say(pid, "second\n").await;
    r.host.say(pid, "third\n").await;
    r.until_line("R-1", "third").await;
    r.host.exit(pid, Some(0)).await;
    let events = drain(&sse).await;
    let logs = lines(&events);
    let out: Vec<&str> = logs
        .iter()
        .filter(|e| e.data["stream"] == "out")
        .filter_map(|e| e.data["text"].as_str())
        .collect();
    assert_eq!(out, ["first", "second", "third"]);
    let seqs: Vec<u64> = logs.iter().filter_map(|e| e.data["seq"].as_u64()).collect();
    assert!(
        seqs.windows(2).all(|w| w[0] < w[1]),
        "every line once, in order"
    );
    let ids: Vec<u64> = logs
        .iter()
        .filter_map(|e| e.id.as_deref()?.parse().ok())
        .collect();
    assert_eq!(ids, seqs, "the SSE id is the log seq");
    let last = events.last().expect("events");
    assert_eq!(
        (last.event.as_str(), &last.data["run"]["status"]),
        ("status", &json!("exited"))
    );
    assert!(
        logs.last().unwrap().data["text"]
            .as_str()
            .unwrap()
            .contains("exited with code 0")
    );
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_finished_run_replays_and_closes_at_once_and_resumes_after_the_last_event_id() {
    let r = Rig::start().await;
    r.define(&[("job", json!({}))]);
    let pid = pid_of(&r.begin("job").await);
    r.host.say(pid, "a\nb\n").await;
    r.host.exit(pid, Some(0)).await;
    r.until_status("R-1", "exited").await;
    let all = drain(&r.tail("R-1", "").await).await;
    let total = lines(&all).len();
    assert!(total >= 4, "sys start, a, b, sys exit");
    let resumed =
        r.t.sse_with(
            Request::new("GET", "/api/p/alpha/runs/R-1/logs/stream").header("Last-Event-ID", "2"),
        )
        .await;
    assert_eq!(lines(&drain(&resumed).await).len(), total - 2);
    let missing = r.tail("R-9", "").await;
    assert_eq!(missing.status, 404);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stopping_a_run_ends_its_stream_and_a_closed_client_leaves_no_listener() {
    let r = Rig::start().await;
    r.define(&[("slow", json!({}))]);
    let (logs, bus) = (
        r.t.svc.run_logs.listener_count(),
        r.t.svc.bus.listener_count(),
    );
    let pid = pid_of(&r.begin("slow").await);
    r.host.say(pid, "up\n").await;
    r.until_line("R-1", "up").await;
    let watcher = r.tail("R-1", "").await;
    until("the stream to subscribe", || async {
        (r.t.svc.run_logs.listener_count() > logs).then_some(())
    })
    .await;
    r.cli("POST", "/api/p/alpha/runs/R-1/stop", &json!({}))
        .await;
    let events = drain(&watcher).await;
    let last = events.last().expect("events");
    assert_eq!(
        (last.event.as_str(), &last.data["run"]["status"]),
        ("status", &json!("stopped"))
    );
    until("the stream to unsubscribe", || async {
        (r.t.svc.run_logs.listener_count() == logs && r.t.svc.bus.listener_count() == bus)
            .then_some(())
    })
    .await;

    r.begin("slow").await;
    let walker = r.tail("R-2", "").await;
    assert!(walker.next(Some("log"), 3000).await.is_some());
    walker.close();
    until("the listeners to go after the client left", || async {
        (r.t.svc.run_logs.listener_count() == logs && r.t.svc.bus.listener_count() == bus)
            .then_some(())
    })
    .await;
    r.t.stop(false).await;
}

/// A rig with attachments capped at 100 bytes and the run `R-1` of the job already exited.
async fn rig_with_an_exited_run() -> Rig {
    let config = json!({ "attach": { "maxBytes": 100 } });
    let r = Rig::with(TestOptions {
        config,
        ..TestOptions::default()
    })
    .await;
    r.define(&[("job", json!({}))]);
    let pid = pid_of(&r.begin("job").await);
    r.host.exit(pid, Some(0)).await;
    r.until_status("R-1", "exited").await;
    r
}

#[tokio::test(flavor = "multi_thread")]
async fn attachments_a_raw_image_in_the_listing_out_wrong_bytes_and_a_big_body_refused() {
    let r = rig_with_an_exited_run().await;
    let send = |headers: Vec<(&str, &str)>, body: Vec<u8>, page: bool| {
        let mut request = Request::new(
            "POST",
            "/api/p/alpha/runs/R-1/attachments?caption=the%20screen",
        )
        .body(body);
        for (k, v) in headers {
            request = request.header(k, v);
        }
        let request = if page { r.t.page(request) } else { request };
        async { r.t.send(request).await }
    };
    let ok = send(vec![("Content-Type", "image/png")], png(), true).await;
    assert_eq!(ok.status, 200, "{}", ok.text());
    let attachment = ok.json()["attachment"].clone();
    let url = attachment["url"].as_str().unwrap().to_owned();
    assert!(url.starts_with("/files/alpha/a/ra-"), "{url}");
    assert_eq!(attachment["caption"], "the screen");
    let list = r.t.get("/api/p/alpha/runs/R-1/attachments").await.json();
    assert_eq!(
        (
            list["attachments"].as_array().unwrap().len(),
            &list["attachments"][0]["by"]
        ),
        (1, &json!("human"))
    );
    let served = r.t.get(&url).await;
    assert_eq!((served.status, served.body), (200, png()));
    let bad = send(
        vec![("Content-Type", "image/png")],
        b"not an image".to_vec(),
        false,
    )
    .await;
    assert_eq!(
        (bad.status, bad.code().as_str()),
        (415, "unsupported_media")
    );
    let big = send(vec![("Content-Type", "image/png")], vec![0; 300], false).await;
    assert_eq!(big.status, 413);
    let gone =
        r.t.send(
            Request::new("POST", "/api/p/alpha/runs/R-7/attachments")
                .header("Content-Type", "image/png")
                .body(png()),
        )
        .await;
    assert_eq!(gone.status, 404);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_launcher_or_bundle_runs_hidden_with_stdin_and_colour_a_console_only_with_window() {
    let r = Rig::start().await;
    let bundle = json!({ "kind": "launch", "hand": true, "visible": true, "bundle": "app-1" });
    let launcher = json!({ "kind": "launch", "hand": true, "visible": true, "window": true, "launcher": "win-2" });
    r.define(&[("bundle-app-1", bundle), ("launcher-win-2", launcher)]);
    let hidden = play(&r, "bundle-app-1", json!({})).await.json()["run"].clone();
    assert_eq!(
        (&hidden["visible"], &hidden["stdin"], &hidden["origin"]),
        (&json!(false), &json!(true), &json!("page"))
    );
    let asked = play(&r, "bundle-app-1", json!({ "visible": true }))
        .await
        .json()["run"]
        .clone();
    assert_eq!(asked["id"], hidden["id"], "still running: already");
    let cli = r
        .cli(
            "POST",
            "/api/p/alpha/runs",
            &json!({ "name": "bundle-app-1", "visible": true }),
        )
        .await;
    assert_eq!(
        cli.status, 400,
        "a bundle without window has no console to show"
    );
    let win = play(&r, "launcher-win-2", json!({})).await.json()["run"].clone();
    assert_eq!(win["visible"], true);
    let requests = r.host.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!((requests[0].visible, requests[0].stdin), (false, true));
    assert_eq!(
        requests[0].env.get("FORCE_COLOR").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        requests[0].env.get("TERM").map(String::as_str),
        Some("xterm-256color")
    );
    assert_eq!(
        requests[0].env.get("COORD_PROJECT").map(String::as_str),
        Some("alpha")
    );
    assert_eq!(
        (requests[1].visible, requests[1].env.get("FORCE_COLOR")),
        (true, None)
    );
    let colon = r
        .page(
            "POST",
            "/api/p/alpha/runs",
            &json!({ "name": "bundle:app-1" }),
        )
        .await;
    assert_eq!((colon.status, colon.code().as_str()), (400, "bad_request"));
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn shadow_mode_reads_and_refuses_every_write() {
    let r = Rig::with(TestOptions {
        shadow: true,
        ..TestOptions::default()
    })
    .await;
    r.define(&[("job", json!({}))]);
    assert_eq!(r.t.get("/api/p/alpha/launches").await.status, 200);
    assert_eq!(r.t.get("/api/p/alpha/runs").await.json()["runs"], json!([]));
    r.refuses_every_run(false).await;
    r.t.stop(false).await;
}
