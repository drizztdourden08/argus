//! The HTTP side the Node `coord run` client relies on, driven by the real client: the Node coord
//! CLI in a child process against an Argus test server with real processes. Ported from
//! `runs.cli.test.mjs`; every case needs `ARGUS_COORD_CLI` and prints why it returns without it.
//!
//! Not ported here, with the reason: the refusals of `run define` and `run undefine` (the
//! sentences are the CLI's own and need no server), the exit-3 message of a stopped dashboard and
//! the client that starts a stopped dashboard from an install dir (both are client behaviour
//! with a fake install; `argus start` takes the second over at the switch-over). They run with
//! `argus coord` in slice 8.
//! E2: the real Node coord run client against a server with real processes.
// test-tag: e2e

use std::path::Path;

use argus_testkit::{CliOut, CoordCli, TestOptions, TestServer};
use serde_json::{Value, json};
use tempfile::TempDir;

struct Env {
    t: TestServer,
    cli: CoordCli,
    dir: TempDir,
}

impl Env {
    /// A server with real processes, a board `alpha` and its two threads made by the real verbs.
    async fn start() -> Option<Env> {
        let cli = CoordCli::locate()?;
        let t = TestServer::start(TestOptions::default()).await;
        let env = Env {
            t,
            cli,
            dir: tempfile::tempdir().expect("a temp folder"),
        };
        let root = env.dir.path().display().to_string();
        for args in [
            vec!["board", "init", "--project", "alpha", "--name", "Alpha"],
            vec![
                "thread",
                "register",
                "lead",
                "--role",
                "lead",
                "--project",
                "alpha",
            ],
            vec![
                "thread",
                "register",
                "worker",
                "--subject",
                "work",
                "--project",
                "alpha",
                "--root",
                &root,
            ],
        ] {
            let out = env.coord(&args).await;
            assert_eq!(out.status, 0, "{args:?}: {}", out.stderr);
        }
        Some(env)
    }

    async fn coord(&self, args: &[&str]) -> CliOut {
        self.cli.run(&self.t, args, self.dir.path(), &[]).await
    }

    /// A verb of a project, as the lead unless `rest` says otherwise.
    async fn run(&self, verb: &[&str], rest: &[&str]) -> CliOut {
        let mut args = vec!["run"];
        args.extend(verb);
        args.extend(["--project", "alpha"]);
        args.extend(rest);
        self.coord(&args).await
    }

    /// A launch whose command is a Node script written to the temp folder.
    async fn define(&self, name: &str, source: &str) {
        let file = self.dir.path().join(format!("{name}.cjs"));
        std::fs::write(&file, source).expect("the script");
        let cmd = format!("node \"{}\"", file.display());
        let out = self
            .run(&["define", name, "--cmd", &cmd], &["--as", "lead"])
            .await;
        assert_eq!(out.status, 0, "{}", out.stderr);
    }

    async fn run_doc(&self, rid: &str) -> Value {
        self.t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone()
    }

    fn board(&self) -> Value {
        self.t.board("alpha")
    }
}

fn lines(text: &str) -> Vec<&str> {
    text.lines().map(str::trim_end).collect()
}

fn has_line(text: &str, wanted: &str) -> bool {
    lines(text).contains(&wanted)
}

fn cwd_of(dir: &Path) -> String {
    dir.display().to_string()
}

/// The first `run define`: the board file holds the launch and the server lists it without env.
async fn first_define(env: &Env) {
    let build = [
        "define",
        "build",
        "--cmd",
        "npm run build",
        "--kind",
        "build",
    ];
    let first = env
        .run(
            &build,
            &[
                "--env",
                "API_TOKEN=s3cret",
                "--env",
                "MODE=prod",
                "--as",
                "lead",
            ],
        )
        .await;
    assert_eq!(first.status, 0, "{}", first.stderr);
    assert!(
        first.stdout.contains("defined build: build"),
        "{}",
        first.stdout
    );
    assert!(first.stdout.contains("timeout 1800s"), "{}", first.stdout);
    let def = env.board()["runDefs"]["build"].clone();
    assert_eq!(
        json!({ "cmd": def["cmd"], "kind": def["kind"], "hand": def["hand"], "visible": def["visible"],
                "timeoutSec": def["timeoutSec"], "parallel": def["parallel"], "rev": def["rev"],
                "definedBy": def["definedBy"] }),
        json!({ "cmd": "npm run build", "kind": "build", "hand": false, "visible": false,
                "timeoutSec": 1800, "parallel": 1, "rev": 1, "definedBy": "lead" })
    );
    assert_eq!(def["env"], json!({ "API_TOKEN": "s3cret", "MODE": "prod" }));
    assert_eq!(
        def["cwd"].as_str().map(|c| c.replace('\\', "/")),
        Some(cwd_of(env.dir.path()).replace('\\', "/")),
        "cwd defaults to the caller cwd"
    );
    let served = env.t.get("/api/p/alpha/launches").await.text();
    assert!(
        served.contains("npm run build") && !served.contains("s3cret"),
        "the launch is listed without its env: {served}"
    );
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn define_defs_and_undefine_work_on_the_board_file_and_the_server_shows_the_launch() {
    let Some(env) = Env::start().await else {
        return;
    };
    first_define(&env).await;
    let again = env
        .run(
            &[
                "define",
                "build",
                "--cmd",
                "npm run build:all",
                "--kind",
                "build",
                "--timeout",
                "0",
            ],
            &["--as", "lead"],
        )
        .await;
    assert!(again.stdout.contains("updated build"), "{}", again.stdout);
    assert_eq!(env.board()["runDefs"]["build"]["rev"], 2);
    let hand = env
        .run(
            &[
                "define",
                "app",
                "--cmd",
                "start /wait app.exe",
                "--hand",
                "--visible",
                "--parallel",
                "1",
            ],
            &["--as", "lead"],
        )
        .await;
    assert_eq!(hand.status, 0, "{}", hand.stderr);
    assert!(hand.stdout.contains("hand (visible)"), "{}", hand.stdout);
    let listed = env.run(&["defs", "--json"], &[]).await;
    let parsed: Value = serde_json::from_str(&listed.stdout).expect("defs --json prints JSON");
    let names: Vec<&str> = parsed["defs"]
        .as_array()
        .expect("defs")
        .iter()
        .filter_map(|d| d["name"].as_str())
        .collect();
    assert_eq!(names, ["app", "build"]);
    assert!(
        !listed.stdout.contains("s3cret"),
        "defs never prints env values"
    );
    undefine_build(&env).await;
}

async fn undefine_build(env: &Env) {
    let gone = env.run(&["undefine", "build"], &["--as", "lead"]).await;
    assert_eq!(gone.status, 0, "{}", gone.stderr);
    let keys: Vec<String> = env.board()["runDefs"]
        .as_object()
        .expect("runDefs")
        .keys()
        .cloned()
        .collect();
    assert_eq!(keys, ["app"]);
}

/// `start --wait` streams the output and exits with the run's exit code.
async fn waited_runs(env: &Env) {
    let hello = env
        .run(&["start", "hello", "--wait", "--reason", "smoke"], &[])
        .await;
    assert_eq!(hello.status, 0, "{}", hello.stderr);
    assert!(
        hello.stdout.contains("R-1 hello queued") || hello.stdout.contains("R-1 hello running"),
        "{}",
        hello.stdout
    );
    assert!(has_line(&hello.stdout, "hi"), "{}", hello.stdout);
    assert!(hello.stderr.contains("careful"), "{}", hello.stderr);
    assert!(
        hello.stderr.contains("R-1 hello exited (code 0)"),
        "{}",
        hello.stderr
    );
    assert_eq!(env.run_doc("R-1").await["reason"], "smoke");

    let fail = env.run(&["start", "fail", "--wait"], &[]).await;
    assert_eq!(fail.status, 3, "the exit code is the run exit code");
    assert!(has_line(&fail.stdout, "before"), "{}", fail.stdout);
    assert!(
        fail.stderr.contains("R-2 fail failed (code 3)"),
        "{}",
        fail.stderr
    );
}

/// A run left running: a second start is already, status and list read it.
async fn live_run(env: &Env) {
    let first = env.run(&["start", "slow", "--agent", "A-7"], &[]).await;
    assert_eq!(first.status, 0, "{}", first.stderr);
    assert!(first.stdout.contains("R-3 slow"), "{}", first.stdout);
    let second = env.run(&["start", "slow"], &[]).await;
    assert!(
        second
            .stdout
            .contains("R-3 slow running (already live, nothing new started)"),
        "{}",
        second.stdout
    );
    let live = env.run_doc("R-3").await;
    assert_eq!(live["agent"], "A-7");
    assert_eq!(live["by"], "worker", "the cwd is the worker's checkout");

    let status = env.run(&["status", "R-3"], &[]).await;
    assert!(
        status.stdout.contains("R-3") && status.stdout.contains("running"),
        "{}",
        status.stdout
    );
    assert!(status.stdout.contains("agent: A-7"), "{}", status.stdout);
    let status_json = env.run(&["status", "R-3", "--json"], &[]).await;
    let parsed: Value = serde_json::from_str(&status_json.stdout).expect("JSON");
    assert_eq!(parsed["run"]["status"], "running");
    let running = env.run(&["list", "--status", "running"], &[]).await;
    assert!(
        running.stdout.contains("R-3") && !running.stdout.contains("R-1"),
        "{}",
        running.stdout
    );
    let named = env.run(&["list", "--name", "hello", "--json"], &[]).await;
    let named: Value = serde_json::from_str(&named.stdout).expect("JSON");
    let ids: Vec<&str> = named["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter_map(|r| r["id"].as_str())
        .collect();
    assert_eq!(ids, ["R-1"]);
}

/// Stop, the refusals of the verbs and the logs of finished runs.
async fn stops_refusals_and_logs(env: &Env) {
    let stopped = env.run(&["stop", "R-3"], &[]).await;
    assert!(
        stopped.stdout.contains("R-3 slow stopped"),
        "{}",
        stopped.stdout
    );
    assert!(
        env.run(&["stop", "R-99"], &[])
            .await
            .stderr
            .contains("no run R-99")
    );
    assert!(
        env.run(&["stop", "nope"], &[])
            .await
            .stderr
            .contains("run id")
    );
    assert!(
        env.run(&["start", "ghost"], &[])
            .await
            .stderr
            .contains("no launch \"ghost\"")
    );
    let visible = env.run(&["start", "hello", "--visible"], &[]).await;
    assert!(
        visible.stderr.contains("--visible is only for"),
        "{}",
        visible.stderr
    );
    let agent = env.run(&["start", "hello", "--agent", "bob"], &[]).await;
    assert!(agent.stderr.contains("--agent"), "{}", agent.stderr);

    let logs = env.run(&["logs", "R-1"], &[]).await;
    assert!(has_line(&logs.stdout, "hi"), "{}", logs.stdout);
    assert!(
        logs.stderr.contains("# started by worker"),
        "{}",
        logs.stderr
    );
    let tail = env.run(&["logs", "R-2", "--tail", "1"], &[]).await;
    assert!(
        tail.stderr.contains("exited with code 3"),
        "{}",
        tail.stderr
    );
    assert!(!tail.stdout.contains("before"), "{}", tail.stdout);
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn start_wait_streams_the_output_and_exits_with_the_run_exit_code() {
    let Some(env) = Env::start().await else {
        return;
    };
    env.define("hello", r#"console.log("hi"); console.error("careful")"#)
        .await;
    env.define("fail", r#"console.log("before"); process.exit(3)"#)
        .await;
    env.define("slow", "setInterval(() => {}, 1000)").await;
    waited_runs(&env).await;
    live_run(&env).await;
    stops_refusals_and_logs(&env).await;
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_thread_caller_is_recorded_as_the_starter() {
    let Some(env) = Env::start().await else {
        return;
    };
    env.define("quick", r#"console.log("q")"#).await;
    let out = env
        .run(&["start", "quick", "--wait"], &["--as", "worker"])
        .await;
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert_eq!(env.run_doc("R-1").await["by"], "worker");
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn logs_follow_tails_a_live_run_until_it_ends() {
    let Some(env) = Env::start().await else {
        return;
    };
    let source = "let n = 0; const t = setInterval(() => { console.log('tick ' + ++n); if (n === 4) clearInterval(t); }, 200)";
    env.define("ticker", source).await;
    let started = env
        .t
        .write(
            "POST",
            "/api/p/alpha/runs",
            &json!({ "name": "ticker", "by": "test" }),
        )
        .await;
    assert_eq!(started.status, 200, "{}", started.text());
    let follow = env.run(&["logs", "R-1", "--follow"], &[]).await;
    assert_eq!(follow.status, 0, "{}", follow.stderr);
    assert_eq!(
        lines(&follow.stdout),
        ["tick 1", "tick 2", "tick 3", "tick 4"]
    );
    assert!(
        follow.stderr.contains("R-1 ticker exited"),
        "{}",
        follow.stderr
    );
}
