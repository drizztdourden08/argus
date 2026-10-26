//! The built-in terminal of a launcher's Play (DESIGN.md sections 5 and 13.7): the run is
//! started hidden, colour hints in its environment, a stdin pipe for the terminal's input
//! row, a run doc whose lastLine moves at most four times a second and never shows the
//! dashboard's own notes, and the note when a command returns right away (runs.terminal).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use argus_launchers::{RunDef, RunEvents, RunKind, RunRow, RunStatus, SpawnRequest, spawn_options};
use tokio::time::Instant;

use crate::common::{Env, EnvOptions, cli, page};

fn play(env: &Env, name: &str, cmd: &str) -> RunDef {
    RunDef {
        kind: RunKind::Launch,
        hand: true,
        bundle: Some("term".to_owned()),
        label: Some("Play term".to_owned()),
        ..env.def(name, cmd)
    }
}

fn request(visible: bool, stdin: bool) -> SpawnRequest {
    SpawnRequest {
        cmd: "x".to_owned(),
        cwd: ".".into(),
        env: std::collections::BTreeMap::new(),
        visible,
        stdin,
    }
}

#[test]
fn spawn_options_a_hidden_run_has_no_console_window_a_window_run_does() {
    let hidden = spawn_options(&request(false, true));
    assert!(hidden.hide_window && hidden.stdin_pipe);
    assert_eq!(hidden.shell, if cfg!(windows) { "cmd.exe" } else { "sh" });
    let window = spawn_options(&request(true, false));
    assert!(!window.hide_window);
    assert!(
        window.detached,
        "a console window survives a dashboard restart"
    );
    assert!(!window.stdin_pipe);
    if cfg!(windows) {
        assert_eq!(window.shell_args, ["/d", "/s", "/c"]);
    }
}

#[tokio::test(start_paused = true)]
async fn a_bundle_run_is_hidden_colour_is_asked_for_and_its_output_is_streamed() {
    let env = Env::new();
    env.define(
        "alpha",
        vec![
            play(&env, "bundle-term", "colour"),
            env.def("plain", "plain"),
        ],
    );
    let run = env.svc.start("alpha", "bundle-term", &page()).unwrap().run;
    assert!(!run.visible && run.stdin);
    let first = &env.host.requests()[0];
    assert_eq!(first.env.get("FORCE_COLOR").map(String::as_str), Some("1"));
    assert_eq!(
        first.env.get("TERM").map(String::as_str),
        Some("xterm-256color")
    );
    assert!(first.stdin && !first.visible);
    let pid = env.pid(0);
    env.host.say(pid, "\u{1b}[32mgreen ok\u{1b}[0m\n").await;
    env.host.complain(pid, "warn\n").await;
    env.host.exit(pid, Some(0)).await;
    env.finished("alpha", &run.id).await;
    let stored = env.text("alpha", &run.id, Some("out"));
    assert!(
        stored.contains("\u{1b}[32mgreen ok\u{1b}[0m"),
        "the stored line keeps its colour codes"
    );
    let last = env.events.doc("alpha", &run.id).unwrap()["lastLine"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(last == "green ok" || last == "warn");
    let other = env.svc.start("alpha", "plain", &cli()).unwrap().run;
    assert!(!other.stdin);
    assert!(!env.host.requests()[1].env.contains_key("FORCE_COLOR"));
}

#[tokio::test(start_paused = true)]
async fn stdin_a_line_goes_to_the_process_and_is_noted_in_the_log() {
    let env = Env::new();
    env.define("alpha", vec![play(&env, "bundle-term", "echo")]);
    let run = env.svc.start("alpha", "bundle-term", &page()).unwrap().run;
    let pid = env.pid(0);
    env.host.say(pid, "ready\n").await;
    assert_eq!(env.events.doc("alpha", &run.id).unwrap()["stdin"], true);
    assert_eq!(
        env.svc
            .send_input("alpha", &run.id, "hello world")
            .await
            .unwrap(),
        12
    );
    assert_eq!(env.host.stdin_text(pid).await, "hello world\n");
    env.svc
        .send_input("alpha", &run.id, "second\n")
        .await
        .unwrap();
    assert_eq!(
        env.host.stdin_text(pid).await,
        "second\n",
        "a trailing newline is not doubled"
    );
    env.svc.stop("alpha", &run.id, "human").await.unwrap();
    let sys = env.text("alpha", &run.id, Some("sys"));
    assert!(sys.contains("> hello world") && sys.contains("> second"));
    assert_eq!(env.events.doc("alpha", &run.id).unwrap()["stdin"], false);
    let late = env
        .svc
        .send_input("alpha", &run.id, "late")
        .await
        .unwrap_err();
    assert_eq!((late.status, late.code), (409, "conflict"));
    assert!(late.message.contains("not running"));
}

#[tokio::test(start_paused = true)]
async fn stdin_refusals_over_4k_no_pipe_unknown_run_and_a_bad_id() {
    let env = Env::new();
    let compile = RunDef {
        kind: RunKind::Build,
        ..env.def("build", "build")
    };
    env.define("alpha", vec![play(&env, "bundle-term", "echo"), compile]);
    let term = env.svc.start("alpha", "bundle-term", &page()).unwrap().run;
    assert_eq!(
        env.svc
            .send_input("alpha", &term.id, &"x".repeat(4096))
            .await
            .unwrap(),
        4097
    );
    let big = env
        .svc
        .send_input("alpha", &term.id, &"x".repeat(4097))
        .await
        .unwrap_err();
    assert_eq!((big.status, big.code), (413, "too_large"));
    assert_eq!(
        env.svc.send_input("alpha", &term.id, "").await.unwrap(),
        1,
        "an empty line is a valid Enter"
    );
    let no_pipe = env.svc.start("alpha", "build", &cli()).unwrap().run;
    assert!(!no_pipe.stdin);
    let none = env
        .svc
        .send_input("alpha", &no_pipe.id, "a")
        .await
        .unwrap_err();
    assert_eq!(none.status, 409);
    assert!(none.message.contains("no stdin"));
    assert_eq!(
        env.svc
            .send_input("alpha", "R-99", "a")
            .await
            .unwrap_err()
            .status,
        404
    );
    assert_eq!(
        env.svc
            .send_input("alpha", "nope", "a")
            .await
            .unwrap_err()
            .status,
        400
    );
    env.svc.stop("alpha", &term.id, "human").await.unwrap();
    env.svc.stop("alpha", &no_pipe.id, "human").await.unwrap();
}

/// Remembers when each distinct `lastLine` reached a run doc.
#[derive(Default)]
struct Stamps(Mutex<Vec<(Instant, Option<String>)>>);

impl RunEvents for Stamps {
    fn project(&self, row: &RunRow) {
        let mut stamps = self.0.lock().unwrap();
        if stamps.last().is_none_or(|(_, line)| *line != row.last_line) {
            stamps.push((Instant::now(), row.last_line.clone()));
        }
    }
    fn unproject(&self, _scope: &str, _rid: &str) {}
    fn reproject(&self, _scope: &str, _rows: &[RunRow]) {}
    fn log_line(&self, _event: &argus_launchers::LogEvent) {}
    fn warn(&self, _message: &str, _detail: &str) {}
}

#[tokio::test(start_paused = true)]
async fn last_line_reaches_the_run_doc_at_most_every_250_ms_as_plain_text() {
    let stamps = Arc::new(Stamps::default());
    let env = Env::build(EnvOptions {
        events: Some(Arc::clone(&stamps) as Arc<dyn RunEvents>),
        ..EnvOptions::default()
    });
    env.define(
        "alpha",
        vec![
            play(&env, "bundle-term", "chatty"),
            env.def("quiet", "quiet"),
        ],
    );
    let run = env.svc.start("alpha", "bundle-term", &page()).unwrap().run;
    for n in 1..=160 {
        env.host
            .say(env.pid(0), &format!("\u{1b}[36mline {n}\u{1b}[0m\n"))
            .await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    env.host.exit(env.pid(0), Some(0)).await;
    env.finished("alpha", &run.id).await;
    let seen = stamps.0.lock().unwrap().clone();
    let lines: Vec<&String> = seen.iter().filter_map(|(_, l)| l.as_ref()).collect();
    assert!(
        lines.len() >= 3,
        "the doc follows the output (saw {})",
        lines.len()
    );
    assert!(
        lines
            .iter()
            .all(|l| l.starts_with("line ") && !l.contains('\u{1b}'))
    );
    let progress: Vec<Instant> = seen
        .iter()
        .filter(|(_, l)| l.is_some())
        .map(|(at, _)| *at)
        .collect();
    for pair in progress[..progress.len() - 1].windows(2) {
        assert!(
            pair[1] - pair[0] >= Duration::from_millis(200),
            "{:?} apart",
            pair[1] - pair[0]
        );
    }
    let quiet = env.svc.start("alpha", "quiet", &cli()).unwrap().run;
    env.host.exit(env.pid(1), Some(0)).await;
    assert_eq!(env.finished("alpha", &quiet.id).await.last_line, None);
}

#[tokio::test(start_paused = true)]
async fn a_start_that_cannot_happen_is_final_at_once_and_says_why() {
    let env = Env::new();
    let gone = RunDef {
        cwd: env
            .dir
            .path()
            .join("no-such-folder")
            .to_string_lossy()
            .into_owned(),
        ..play(&env, "bundle-term", "never")
    };
    env.define("alpha", vec![gone]);
    let run = env.svc.start("alpha", "bundle-term", &page()).unwrap().run;
    assert_eq!(
        run.status,
        RunStatus::Failed,
        "the start itself answers with the failure"
    );
    assert!(
        run.error
            .unwrap()
            .starts_with("failed to start: working directory is not usable")
    );
    assert!(
        env.text("alpha", &run.id, Some("sys"))
            .contains("failed to start: working directory is not usable")
    );
    assert!(run.started_at.is_none());
}

#[tokio::test(start_paused = true)]
async fn a_command_that_returns_at_once_with_code_0_gets_one_note_failures_and_tasks_do_not() {
    let env = Env::new();
    let quiet = RunDef {
        bundle: Some("quiet".to_owned()),
        ..play(&env, "bundle-quiet", "q")
    };
    let fail = RunDef {
        bundle: Some("fail".to_owned()),
        ..play(&env, "bundle-fail", "f")
    };
    env.define(
        "alpha",
        vec![
            play(&env, "bundle-term", "t"),
            quiet,
            fail,
            env.def("task", "x"),
        ],
    );
    let hint = "the command returned; a program it started may still be running";
    let mut sys_of = Vec::new();
    for (index, (name, code)) in [
        ("bundle-term", 0),
        ("bundle-quiet", 0),
        ("bundle-fail", 2),
        ("task", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let run = env.svc.start("alpha", name, &page()).unwrap().run;
        if index == 0 {
            env.host.say(env.pid(0), "starting the app\n").await;
        }
        env.host.exit(env.pid(index), Some(code)).await;
        env.finished("alpha", &run.id).await;
        sys_of.push(env.texts("alpha", &run.id, Some("sys")));
    }
    let noted: Vec<bool> = sys_of
        .iter()
        .map(|sys| sys.iter().any(|l| l == hint))
        .collect();
    assert_eq!(noted, [true, true, false, false]);
    let first = &sys_of[0];
    assert!(
        first[first.len() - 2].contains("exited with code 0"),
        "the note comes after the exit line"
    );
    assert_eq!(first.last().map(String::as_str), Some(hint));
    assert_eq!(
        env.events.doc("alpha", "R-1").unwrap()["lastLine"],
        "starting the app"
    );
}
