//! Run lifecycle: exited, failed, a refused start, the run docs the pages receive, who may
//! start what, and the end listeners (runs.lifecycle).

use std::sync::{Arc, Mutex};

use argus_launchers::{Origin, RunKind, RunStatus, StartOptions};

use crate::common::{Env, cli, page};

#[tokio::test(start_paused = true)]
async fn a_run_exits_0_logs_its_output_and_shows_up_as_a_run_doc() {
    let env = Env::new();
    env.define("alpha", vec![env.def("hello", "echo hi")]);
    let started = env.svc.start("alpha", "hello", &cli()).unwrap();
    assert!(!started.already);
    assert_eq!(started.run.id, "R-1");
    assert!(!started.run.visible);
    assert_eq!(started.run.origin, Origin::Cli);
    assert_eq!(started.run.by, "tester");
    let pid = env.pid(0);
    env.host.say(pid, "hi\n").await;
    env.host.complain(pid, "oops\n").await;
    env.host.exit(pid, Some(0)).await;
    let done = env.finished("alpha", "R-1").await;
    assert_eq!(done.status, RunStatus::Exited);
    assert_eq!(done.exit_code, Some(0));
    assert!(done.started_at.is_some() && done.ended_at.is_some());
    assert_eq!(env.text("alpha", "R-1", Some("out")), "hi");
    assert_eq!(env.text("alpha", "R-1", Some("err")), "oops");
    let all = env.text("alpha", "R-1", None);
    assert!(all.contains("started by tester") && all.contains("exited with code 0"));
    assert!(done.log_bytes > 0);
}

#[tokio::test(start_paused = true)]
async fn the_page_doc_has_no_command_and_ends_in_the_final_status() {
    let env = Env::new();
    env.define("alpha", vec![env.def("hello", "echo hi")]);
    env.start("alpha", "hello");
    let pid = env.pid(0);
    env.host.say(pid, "hi\n").await;
    env.host.exit(pid, Some(0)).await;
    env.finished("alpha", "R-1").await;
    let doc = env.events.doc("alpha", "R-1").unwrap();
    assert_eq!(doc["status"], "exited");
    assert_eq!(
        doc["lastLine"], "hi",
        "the program's line, not the exit note"
    );
    assert!(doc.get("cmd").is_none() && doc.get("cwd").is_none());
    let statuses = env.events.statuses("alpha", "R-1");
    assert_eq!(statuses[..2], ["queued", "running"]);
    assert_eq!(statuses.last().map(String::as_str), Some("exited"));
}

#[tokio::test(start_paused = true)]
async fn a_non_zero_exit_is_failed_and_keeps_the_code() {
    let env = Env::new();
    env.define("alpha", vec![env.def("fail", "exit 3")]);
    env.start("alpha", "fail");
    env.host.exit(env.pid(0), Some(3)).await;
    let done = env.finished("alpha", "R-1").await;
    assert_eq!(done.status, RunStatus::Failed);
    assert_eq!(done.exit_code, Some(3));
    assert!(
        env.text("alpha", "R-1", None)
            .contains("exited with code 3")
    );
}

#[tokio::test(start_paused = true)]
async fn an_ended_run_can_be_failed_after_the_fact_and_a_live_one_cannot() {
    let env = Env::new();
    env.define("alpha", vec![env.def("hello", "echo hi")]);
    env.start("alpha", "hello");
    let pid = env.pid(0);
    assert_eq!(
        env.svc.fail_ended("alpha", "R-1", "harm"),
        None,
        "still live"
    );
    env.host.exit(pid, Some(0)).await;
    env.finished("alpha", "R-1").await;
    let failed = env.svc.fail_ended("alpha", "R-1", "harm").unwrap();
    assert_eq!(
        (failed.status, failed.error.as_deref(), failed.exit_code),
        (RunStatus::Failed, Some("harm"), Some(0))
    );
    assert_eq!(
        env.svc.get("alpha", "R-1").unwrap().status,
        RunStatus::Failed
    );
    assert_eq!(env.svc.fail_ended("alpha", "R-9", "harm"), None, "unknown");
}

#[tokio::test(start_paused = true)]
async fn a_refused_spawn_fails_the_run_at_once_with_the_reason() {
    let env = Env::new();
    env.define("alpha", vec![env.def("app", "anything")]);
    env.host.fail_spawns(Some("spawn EACCES"));
    let run = env.start("alpha", "app");
    assert_eq!(run.status, RunStatus::Failed);
    assert_eq!(run.error.as_deref(), Some("failed to start: spawn EACCES"));
    assert!(run.started_at.is_none() && run.pid.is_none());
    let sys = env.text("alpha", "R-1", Some("sys"));
    assert!(sys.contains("failed to start: spawn EACCES"));
}

#[tokio::test(start_paused = true)]
async fn an_unknown_name_is_unknown_run_and_nothing_is_stored() {
    let env = Env::new();
    env.define("alpha", vec![env.def("hello", "echo hi")]);
    let ghost = env.svc.start("alpha", "ghost", &cli()).unwrap_err();
    assert_eq!((ghost.status, ghost.code), (404, "unknown_run"));
    let nobody = env.svc.start("nobody", "hello", &cli()).unwrap_err();
    assert_eq!(nobody.code, "unknown_run");
    let bad = env.svc.start("alpha", "Bad Name", &cli()).unwrap_err();
    assert_eq!(bad.code, "bad_request");
    let agent = StartOptions {
        agent: Some("x".to_owned()),
        ..cli()
    };
    assert_eq!(
        env.svc.start("alpha", "hello", &agent).unwrap_err().code,
        "bad_request"
    );
    assert!(env.list("alpha").is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_def_at_its_parallel_limit_returns_the_live_run_with_already() {
    let env = Env::new();
    env.define("alpha", vec![env.def("slow", "sleep")]);
    let first = env.svc.start("alpha", "slow", &cli()).unwrap();
    let a = env.svc.start("alpha", "slow", &cli()).unwrap();
    let b = env.svc.start("alpha", "slow", &cli()).unwrap();
    assert!(a.already && b.already);
    assert_eq!(a.run.id, first.run.id);
    assert_eq!(env.list("alpha").len(), 1);
    env.svc
        .stop("alpha", &first.run.id, "tester")
        .await
        .unwrap();
    let again = env.svc.start("alpha", "slow", &cli()).unwrap();
    assert!(!again.already);
    assert_eq!(again.run.id, "R-2");
}

#[tokio::test(start_paused = true)]
async fn parallel_2_lets_two_run_and_the_third_gets_already() {
    let env = Env::new();
    let def = argus_launchers::RunDef {
        parallel: 2,
        ..env.def("slow", "sleep")
    };
    env.define("alpha", vec![def]);
    let outcomes: Vec<_> = (0..3)
        .map(|_| env.svc.start("alpha", "slow", &cli()).unwrap())
        .collect();
    let already: Vec<bool> = outcomes.iter().map(|o| o.already).collect();
    assert_eq!(already, [false, false, true]);
    assert_ne!(outcomes[0].run.id, outcomes[1].run.id);
}

#[tokio::test(start_paused = true)]
async fn the_visible_rule_page_and_cli_for_a_handed_build_never_mcp() {
    let env = Env::with_limits(argus_launchers::RunLimits {
        max_concurrent: 0,
        ..argus_launchers::RunLimits::default()
    });
    let eight = |hand: bool, visible: bool, name: &str| argus_launchers::RunDef {
        hand,
        visible,
        parallel: 8,
        ..env.def(name, "x")
    };
    let defs = vec![
        eight(true, true, "app"),
        eight(true, false, "handonly"),
        eight(false, false, "plain"),
    ];
    env.define("alpha", defs);
    let visible = |name: &str, options: StartOptions| {
        env.svc.start("alpha", name, &options).unwrap().run.visible
    };
    let wants = |wanted: bool, base: StartOptions| StartOptions {
        visible: Some(wanted),
        ..base
    };
    assert!(visible("app", page()));
    assert!(!visible("app", wants(false, page())));
    assert!(!visible("app", cli()));
    assert!(visible("app", wants(true, cli())));
    let mcp = StartOptions {
        origin: Origin::Mcp,
        ..wants(true, StartOptions::default())
    };
    assert!(!visible("app", mcp));
    assert!(!visible("plain", wants(true, page())));
    assert!(!visible("handonly", page()));
    let refused = env.svc.start("alpha", "plain", &wants(true, cli()));
    assert_eq!(refused.unwrap_err().code, "bad_request");
}

#[tokio::test(start_paused = true)]
async fn the_run_sees_its_names_and_env_and_the_stored_cmd_is_redacted() {
    let env = Env::new();
    let mut def = env.def("envy", "node env.js --token=supersecretvalue");
    def.env.insert("GREETING".to_owned(), "hello".to_owned());
    def.env
        .insert("API_TOKEN".to_owned(), "tok-12345678".to_owned());
    env.define("alpha", vec![def]);
    let run = env.start("alpha", "envy");
    let request = &env.host.requests()[0];
    let get = |key: &str| request.env.get(key).map(String::as_str);
    assert_eq!(get("COORD_PROJECT"), Some("alpha"));
    assert_eq!(get("COORD_RUN_ID"), Some("R-1"));
    assert_eq!(get("COORD_DASHBOARD"), Some("1"));
    assert_eq!((get("GREETING"), get("FORCE_COLOR")), (Some("hello"), None));
    assert_eq!(request.cmd, "node env.js --token=supersecretvalue");
    assert!(!run.cmd.contains("supersecretvalue") && run.cmd.contains("[redacted]"));
    assert!(
        env.svc.defs("alpha").unwrap()[0].env.is_empty(),
        "env values never leave the board"
    );
}

#[tokio::test(start_paused = true)]
async fn a_missing_cwd_fails_the_run_at_once_with_the_reason() {
    let env = Env::new();
    let def = argus_launchers::RunDef {
        cwd: "Z:/definitely/not/here".to_owned(),
        ..env.def("gone", "echo hi")
    };
    env.define("alpha", vec![def]);
    let run = env.start("alpha", "gone");
    assert_eq!(run.status, RunStatus::Failed);
    assert!(run.error.unwrap().contains("working directory"));
    assert!(run.pid.is_none());
    assert_eq!(run.kind, RunKind::Task);
}

#[tokio::test(start_paused = true)]
async fn end_listeners_hear_failed_and_exited_runs_and_a_panic_stops_nobody() {
    let env = Env::new();
    env.define(
        "alpha",
        vec![env.def("bad", "bad"), env.def("good", "good")],
    );
    let heard = Arc::new(Mutex::new(Vec::new()));
    env.svc.on_end(|_| panic!("a broken listener"));
    let sink = Arc::clone(&heard);
    env.svc.on_end(move |row| {
        sink.lock()
            .unwrap()
            .push((row.name.clone(), row.status, row.exit_code));
    });
    env.start("alpha", "bad");
    env.host.exit(env.pid(0), Some(1)).await;
    env.finished("alpha", "R-1").await;
    env.start("alpha", "good");
    env.host.exit(env.pid(1), Some(0)).await;
    env.finished("alpha", "R-2").await;
    let heard = heard.lock().unwrap().clone();
    assert_eq!(
        heard,
        [
            ("bad".to_owned(), RunStatus::Failed, Some(1)),
            ("good".to_owned(), RunStatus::Exited, Some(0)),
        ]
    );
    assert_eq!(env.events.warnings().len(), 2, "each panic is reported");
}

#[tokio::test(start_paused = true)]
async fn finish_listeners_see_the_whole_log_of_a_run_that_is_not_yet_reported_ended() {
    let env = Env::new();
    env.define("alpha", vec![env.def("quick", "quick")]);
    env.svc.on_finish(|_, _| panic!("a broken listener"));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    env.svc.on_finish(move |svc, row| {
        let options = argus_launchers::LogOptions::default();
        let page = svc.logs(&row.scope, &row.id, &options).unwrap();
        let out: Vec<String> = page.lines.iter().map(|l| l.text.clone()).collect();
        let live = svc.get(&row.scope, &row.id).unwrap().status.is_live();
        sink.lock().unwrap().push((out, live));
    });
    env.start("alpha", "quick");
    let pid = env.pid(0);
    env.host.say(pid, "first\nlast\n").await;
    env.host.exit(pid, Some(0)).await;
    env.finished("alpha", "R-1").await;
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    let (lines, live) = &seen[0];
    assert!(
        lines.iter().any(|l| l == "last"),
        "the log is whole: {lines:?}"
    );
    assert!(live, "the run is not reported ended yet");
    assert_eq!(env.events.warnings().len(), 1, "the panic is reported");
}
