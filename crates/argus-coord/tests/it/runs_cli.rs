//! Ports `runs.cli.test.mjs`: `coord run ...` end to end against a temp `COORD_HOME`, talking to
//! the real run routes of an Argus test server over HTTP: the board-only verbs, `start --wait`
//! streaming the output and ending with the exit code of the run, the caller recorded as the
//! starter, `logs --follow`, the exit 3 of a dashboard that cannot be reached, and the client
//! that starts a stopped dashboard. Runs are real `node` processes.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::common::{Home, Ran, coord};
use crate::remote::{Rig, alpha_with_worker, env_plus, ok, read_board_at, refused};
use serde_json::{Value, json};

const P: [&str; 2] = ["--project", "alpha"];
const AS: [&str; 2] = ["--as", "coordinator"];

fn seed(coord_home: &Path, folder: &Path) {
    argus_testkit::write_board(coord_home, "alpha", &alpha_with_worker(folder));
}

/// `coord run <words> --project alpha <extra>`.
fn run_argv<'a>(words: &[&'a str], extra: &[&'a str]) -> Vec<&'a str> {
    ["run"]
        .iter()
        .chain(words)
        .chain(&P)
        .chain(extra)
        .copied()
        .collect()
}

fn run(rig: &Rig, words: &[&str], extra: &[&str]) -> Ran {
    rig.run(&run_argv(words, extra))
}

/// A launch whose command is a Node script written to the rig folder.
fn define(rig: &Rig, name: &str, source: &str) {
    let file = rig.dir.path().join(format!("{name}.cjs"));
    fs::write(&file, source).expect("script");
    let cmd = format!("node \"{}\"", file.display());
    ok(&run(rig, &["define", name, "--cmd", &cmd], &AS));
}

async fn run_doc(rig: &Rig, rid: &str) -> Value {
    rig.t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone()
}

fn same_folder(a: &Value, b: &Path) -> bool {
    let flat = |s: &str| s.replace('\\', "/").to_lowercase();
    a.as_str()
        .is_some_and(|a| flat(a) == flat(&b.to_string_lossy()))
}

/// A home with the board `alpha`, and `coord run <words> --project alpha <extra>` in it.
struct Local {
    home: Home,
}

impl Local {
    fn new() -> Local {
        let home = Home::new();
        seed(&home.coord_home(), home.path());
        Local { home }
    }

    fn go(&self, words: &[&str], extra: &[&str]) -> Ran {
        coord(&run_argv(words, extra), self.home.env(), self.home.path())
    }

    fn defs(&self) -> Value {
        read_board_at(&self.home.coord_home(), "alpha")["runDefs"].clone()
    }
}

#[test]
fn define_records_the_launch_and_a_second_define_updates_it() {
    let local = Local::new();
    let build = [
        "define",
        "build",
        "--cmd",
        "npm run build",
        "--kind",
        "build",
    ];
    let first = local.go(
        &build,
        &[
            "--env",
            "API_TOKEN=s3cret",
            "--env",
            "MODE=prod",
            "--as",
            "coordinator",
        ],
    );
    let out = ok(&first);
    assert!(
        out.contains("defined build: build") && out.contains("timeout 1800s"),
        "{out}"
    );
    let def = local.defs()["build"].clone();
    let seen = json!([
        def["cmd"],
        def["kind"],
        def["hand"],
        def["visible"],
        def["timeoutSec"],
        def["parallel"],
        def["rev"],
        def["definedBy"]
    ]);
    assert_eq!(
        seen,
        json!([
            "npm run build",
            "build",
            false,
            false,
            1800,
            1,
            1,
            "coordinator"
        ])
    );
    assert_eq!(def["env"], json!({ "API_TOKEN": "s3cret", "MODE": "prod" }));
    assert!(
        same_folder(&def["cwd"], local.home.path()),
        "cwd defaults to the caller cwd: {}",
        def["cwd"]
    );
}

#[test]
fn a_second_define_updates_the_launch_and_a_hand_launch_may_be_visible() {
    let local = Local::new();
    local.go(
        &[
            "define",
            "build",
            "--cmd",
            "npm run build",
            "--kind",
            "build",
        ],
        &AS,
    );
    let again = local.go(
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
        &AS,
    );
    assert!(ok(&again).contains("updated build"));
    assert_eq!(local.defs()["build"]["rev"], 2);
    let hand = local.go(
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
        &AS,
    );
    assert!(ok(&hand).contains("hand (visible)"));
}

#[test]
fn defs_lists_the_launches_without_their_env_values_and_undefine_removes_one() {
    let local = Local::new();
    local.go(
        &[
            "define",
            "build",
            "--cmd",
            "npm run build",
            "--env",
            "API_TOKEN=s3cret",
        ],
        &AS,
    );
    local.go(
        &[
            "define",
            "app",
            "--cmd",
            "start /wait app.exe",
            "--hand",
            "--visible",
        ],
        &AS,
    );
    let parsed: Value =
        serde_json::from_str(ok(&local.go(&["defs", "--json"], &[]))).expect("json");
    let names: Vec<&Value> = parsed["defs"]
        .as_array()
        .expect("defs")
        .iter()
        .map(|d| &d["name"])
        .collect();
    assert_eq!(names, [&json!("app"), &json!("build")]);
    assert!(
        !parsed.to_string().contains("s3cret"),
        "defs never prints env values"
    );
    let text = ok(&local.go(&["defs"], &[]))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("app task hand,visible"), "{text}");
    ok(&local.go(&["undefine", "build"], &AS));
    refused(
        &local.go(&["undefine", "build"], &AS),
        1,
        "no launch \"build\"",
    );
    assert_eq!(
        local
            .defs()
            .as_object()
            .map(|m| m.keys().cloned().collect::<Vec<_>>()),
        Some(vec!["app".to_owned()])
    );
}

#[test]
fn define_and_undefine_refuse_what_is_not_the_coordinators_or_not_valid() {
    let local = Local::new();
    let worker = ["--as", "worker"];
    let refusals: [(&[&str], &[&str], &str); 9] = [
        (
            &["define", "evil", "--cmd", "calc"],
            &worker,
            "only the coordinator",
        ),
        (
            &["define", "x", "--cmd", "echo", "--visible"],
            &AS,
            "--visible only goes with --hand",
        ),
        (&["define", "Bad Name", "--cmd", "echo"], &AS, "lowercase"),
        (&["define", "x"], &AS, "--cmd"),
        (
            &["define", "x", "--cmd", "echo", "--cwd", "nope"],
            &AS,
            "not a folder",
        ),
        (
            &["define", "x", "--cmd", "echo", "--kind", "weird"],
            &AS,
            "--kind",
        ),
        (
            &["define", "x", "--cmd", "echo", "--env", "NOEQUALS"],
            &AS,
            "KEY=VALUE",
        ),
        (
            &["define", "x", "--cmd", "echo", "--timeout", "soon"],
            &AS,
            "--timeout",
        ),
        (&["undefine", "build"], &worker, "only the coordinator"),
    ];
    for (words, extra, text) in refusals {
        refused(&local.go(words, extra), 1, text);
    }
    refused(&local.go(&["nonsense"], &[]), 1, "run verb must be one of");
}

/// A rig with three launches: `hello`, `fail` and a `slow` one that never ends by itself.
async fn launches() -> Rig {
    let rig = Rig::start(json!({}), seed).await;
    define(
        &rig,
        "hello",
        "console.log(\"hi\"); console.error(\"careful\")",
    );
    define(&rig, "fail", "console.log(\"before\"); process.exit(3)");
    define(&rig, "slow", "setInterval(() => {}, 1000)");
    rig
}

#[tokio::test(flavor = "multi_thread")]
async fn start_wait_streams_the_output_and_exits_with_the_run_exit_code() {
    let rig = launches().await;
    let hello = run(
        &rig,
        &["start", "hello", "--wait", "--reason", "smoke"],
        &[],
    );
    let out = ok(&hello);
    assert!(out.starts_with("R-1 hello "), "{out}");
    assert!(out.lines().any(|l| l == "hi"), "{out}");
    assert!(hello.err.contains("careful"), "{}", hello.err);
    assert!(
        hello.err.contains("R-1 hello exited (code 0) in "),
        "{}",
        hello.err
    );
    assert_eq!(run_doc(&rig, "R-1").await["reason"], "smoke");
    let fail = run(&rig, &["start", "fail", "--wait"], &[]);
    assert_eq!(fail.code, 3, "the exit code is the run exit code");
    assert!(fail.out.lines().any(|l| l == "before"), "{}", fail.out);
    assert!(
        fail.err.contains("R-2 fail failed (code 3)"),
        "{}",
        fail.err
    );
    let logs = run(&rig, &["logs", "R-1"], &[]);
    assert!(ok(&logs).lines().any(|l| l == "hi"), "{}", logs.out);
    assert!(logs.err.contains("# started by worker"), "{}", logs.err);
    let tail = run(&rig, &["logs", "R-2", "--tail", "1"], &[]);
    assert!(
        tail.err.contains("exited with code 3") && !tail.out.contains("before"),
        "{} {}",
        tail.out,
        tail.err
    );
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_live_run_is_started_once_listed_shown_and_stopped() {
    let rig = launches().await;
    ok(&run(&rig, &["start", "hello", "--wait"], &[]));
    let first = run(&rig, &["start", "slow", "--agent", "A-7"], &[]);
    assert!(ok(&first).starts_with("R-2 slow "), "{}", first.out);
    let second = run(&rig, &["start", "slow"], &[]);
    assert!(
        ok(&second).contains("R-2 slow running (already live, nothing new started)"),
        "{}",
        second.out
    );
    let slow = run_doc(&rig, "R-2").await;
    assert_eq!(
        (slow["agent"].clone(), slow["by"].clone()),
        (json!("A-7"), json!("worker")),
        "the cwd is the worker's checkout"
    );
    let status = ok(&run(&rig, &["status", "R-2"], &[]))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        status.contains("R-2 slow running") && status.contains("agent: A-7"),
        "{status}"
    );
    let json: Value =
        serde_json::from_str(ok(&run(&rig, &["status", "R-2", "--json"], &[]))).expect("json");
    assert_eq!(json["run"]["status"], "running");
    let list = run(&rig, &["list", "--status", "running"], &[]);
    assert!(
        ok(&list).contains("R-2") && !list.out.contains("R-1"),
        "{}",
        list.out
    );
    let named: Value =
        serde_json::from_str(ok(&run(&rig, &["list", "--name", "hello", "--json"], &[])))
            .expect("json");
    assert_eq!(
        named["runs"]
            .as_array()
            .map(|r| r.iter().map(|x| x["id"].clone()).collect::<Vec<_>>()),
        Some(vec![json!("R-1")])
    );
    let stopped = run(&rig, &["stop", "R-2"], &[]);
    assert!(ok(&stopped).contains("R-2 slow stopped"), "{}", stopped.out);
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn start_stop_and_status_refuse_a_bad_name_id_or_agent() {
    let rig = launches().await;
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

#[tokio::test(flavor = "multi_thread")]
async fn a_thread_caller_is_recorded_as_the_starter() {
    let rig = Rig::start(json!({}), seed).await;
    define(&rig, "quick", "console.log(\"q\")");
    ok(&run(
        &rig,
        &["start", "quick", "--wait"],
        &["--as", "worker"],
    ));
    assert_eq!(run_doc(&rig, "R-1").await["by"], "worker");
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn logs_follow_tails_a_live_run_until_it_ends() {
    let rig = Rig::start(json!({}), seed).await;
    let ticker = "let n = 0; const t = setInterval(() => { console.log(\"tick \" + ++n); if (n === 4) clearInterval(t); }, 200)";
    define(&rig, "ticker", ticker);
    let started = rig
        .t
        .write(
            "POST",
            "/api/p/alpha/runs",
            &json!({ "name": "ticker", "by": "test" }),
        )
        .await;
    assert_eq!(started.status, 200);
    let follow = run(&rig, &["logs", "R-1", "--follow"], &[]);
    let out = ok(&follow);
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["tick 1", "tick 2", "tick 3", "tick 4"]
    );
    assert!(follow.err.contains("R-1 ticker exited"), "{}", follow.err);
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
        "define",
        "quick",
        "--cmd",
        "echo hi",
        "--as",
        "coordinator",
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

const FAKE_DASHBOARD: &str = r#"import http from 'node:http';
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
const home = process.env.DASHBOARD_HOME || join(process.env.COORD_HOME, 'dashboard');
if (process.argv[2] === 'start') {
  spawn(process.execPath, [fileURLToPath(import.meta.url), 'serve'], { detached: true, stdio: 'ignore', windowsHide: true, env: process.env }).unref();
} else {
  const server = http.createServer((req, res) => {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(req.url.startsWith('/api/health') ? '{"ok":true}' : '{"runs":[]}');
  });
  server.listen(0, '127.0.0.1', () => {
    writeFileSync(join(home, 'server.pid'), JSON.stringify({ pid: process.pid, port: server.address().port }));
    writeFileSync(join(home, 'fake.pid'), String(process.pid));
  });
}
"#;

fn kill(pid: &str) {
    let mut command = if cfg!(windows) {
        Command::new("taskkill")
    } else {
        Command::new("kill")
    };
    let args: Vec<&str> = if cfg!(windows) {
        vec!["/PID", pid, "/F", "/T"]
    } else {
        vec![pid]
    };
    command.args(args).output().ok();
}

/// A temp home with the board `alpha`, a fake install whose `dashboard.mjs` starts a fake
/// server, and a `config.json` naming that install. Returns the home and the install folder.
fn home_with_fake_install() -> (Home, std::path::PathBuf) {
    let home = Home::new();
    seed(&home.coord_home(), home.path());
    let install = home.path().join("fake-install");
    fs::create_dir_all(install.join("bin")).expect("bin");
    fs::write(install.join("bin").join("dashboard.mjs"), FAKE_DASHBOARD).expect("fake dashboard");
    let config = json!({ "port": 1, "token": "b".repeat(64), "installDir": install });
    fs::write(
        home.dashboard_home().join("config.json"),
        config.to_string(),
    )
    .expect("config");
    (home, install)
}

fn node_missing() -> bool {
    let missing = Command::new("node").arg("--version").output().is_err();
    if missing {
        eprintln!("skipped: node is not installed");
    }
    missing
}

fn fake_pid(home: &Home) -> String {
    let pid = fs::read_to_string(home.dashboard_home().join("fake.pid")).unwrap_or_default();
    pid.trim().to_owned()
}

#[test]
fn the_client_starts_a_stopped_dashboard_hidden_and_detached_and_then_calls_it() {
    if node_missing() {
        return;
    }
    let (home, install) = home_with_fake_install();
    let wait = [
        ("COORD_DASHBOARD_WAIT_MS", "10000"),
        ("COORD_DASHBOARD_INSTALL", &install.to_string_lossy()),
    ];
    let ran = coord(
        &["run", "list", "--project", "alpha"],
        env_plus(&home.env(), &wait),
        home.path(),
    );
    let pid = fake_pid(&home);
    kill(&pid);
    assert!(
        ok(&ran).contains("no runs match"),
        "{} {}",
        ran.out,
        ran.err
    );
    assert!(
        pid.parse::<u32>().is_ok_and(|p| p > 0),
        "the fake dashboard started: {pid:?}"
    );
}

/// The autostart as a real child with piped output, as the Node suite and the hooks run it: the
/// server it starts must not hold those pipes, or the caller waits for it to exit.
#[test]
fn the_server_the_client_starts_does_not_hold_the_pipes_of_its_caller() {
    if node_missing() {
        return;
    }
    let Some(exe) = argus_testkit::argus_exe() else {
        eprintln!("skipped: no argus binary (set ARGUS_EXE or run `cargo build -p argus`)");
        return;
    };
    let (home, install) = home_with_fake_install();
    let mut command = Command::new(exe);
    command
        .args(["coord", "run", "list", "--project", "alpha"])
        .current_dir(home.path())
        .env("COORD_HOME", home.coord_home())
        .env("DASHBOARD_HOME", home.dashboard_home())
        .env("COORD_DASHBOARD_WAIT_MS", "10000")
        .env("COORD_DASHBOARD_INSTALL", &install)
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("COORD_PROJECT")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0800_0000);
    let child = command.spawn().expect("argus coord");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || tx.send(child.wait_with_output()).ok());
    let finished = rx.recv_timeout(std::time::Duration::from_secs(25));
    let pid = fake_pid(&home);
    kill(&pid);
    let out = finished
        .expect("the call returned while the server it started was still running")
        .expect("output");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("no runs match"),
        "{out:?}"
    );
}
