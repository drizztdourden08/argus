//! Specs of the supervisor against a scripted child: K1 a message during a turn is held and goes
//! as one after the result, S2 a dead child refuses messages and its restart gets the turn that
//! was lost, K4 the billing guard ends a child that is not on the API key.

// test-tag: critical

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::fake::FakeHost;
use super::{Config, Handle, Link, Observer, start};
use crate::ports::{ChildSpec, HostFault};
use crate::session::env::build_env;
use crate::session::guard::GuardFault;
use crate::session::state::Status;

const INIT: &str = r#"{"type":"system","subtype":"init","session_id":"s1","apiKeySource":"ANTHROPIC_API_KEY"}"#;
const INIT_OAUTH: &str =
    r#"{"type":"system","subtype":"init","session_id":"s1","apiKeySource":"none"}"#;
const RESULT: &str =
    r#"{"type":"result","subtype":"success","session_id":"s1","total_cost_usd":0.25}"#;

#[derive(Default)]
struct Recorder {
    started: AtomicUsize,
    stopped: AtomicUsize,
    faults: Mutex<Vec<String>>,
}

impl Observer for Recorder {
    fn started(&self, _link: &Link) {
        self.started.fetch_add(1, Ordering::SeqCst);
    }

    fn stopped(&self) {
        self.stopped.fetch_add(1, Ordering::SeqCst);
    }

    fn guard_failed(&self, fault: &GuardFault) {
        self.faults.lock().unwrap().push(fault.to_string());
    }
}

struct Rig {
    host: FakeHost,
    observer: Arc<Recorder>,
    handle: Handle,
}

fn spec() -> Result<ChildSpec, HostFault> {
    let key = crate::ports::ApiKey::new("sk-test");
    Ok(ChildSpec {
        program: "claude".into(),
        args: Vec::new(),
        env: build_env(Vec::new(), &key, 30),
        cwd: ".".into(),
        stderr_log: "stderr.log".into(),
    })
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(10)).await;
}

async fn rig(restart_max: u32) -> Rig {
    let host = FakeHost::default();
    let observer = Arc::new(Recorder::default());
    let config = Config {
        spec: Box::new(|_| spec()),
        restart_max,
        resume_from: None,
    };
    let handle = start(Arc::new(host.clone()), config, observer.clone());
    settle().await;
    Rig {
        host,
        observer,
        handle,
    }
}

#[tokio::test(start_paused = true)]
async fn the_first_message_is_written_at_once_and_a_later_one_waits_for_the_result() {
    let rig = rig(3).await;
    let pid = rig.host.pids()[0];
    assert!(rig.handle.link.send(String::from("one")));
    settle().await;
    assert_eq!(rig.host.texts(pid), ["one"]);
    rig.host.say(pid, INIT);
    assert!(rig.handle.link.send(String::from("two")));
    assert!(rig.handle.link.send(String::from("three")));
    settle().await;
    assert_eq!(rig.host.texts(pid), ["one"], "held while the turn runs");
    rig.host.say(pid, RESULT);
    settle().await;
    assert_eq!(rig.host.texts(pid), ["one", "two\n\nthree"]);
}

#[tokio::test(start_paused = true)]
async fn an_idle_child_takes_each_message_as_it_comes() {
    let rig = rig(3).await;
    let pid = rig.host.pids()[0];
    assert!(rig.handle.link.send(String::from("one")));
    settle().await;
    rig.host.say(pid, INIT);
    rig.host.say(pid, RESULT);
    settle().await;
    assert_eq!(rig.handle.state.borrow().status, Status::Idle);
    assert!(rig.handle.link.send(String::from("two")));
    settle().await;
    assert_eq!(rig.host.texts(pid), ["one", "two"]);
    assert_eq!(rig.handle.state.borrow().status, Status::Turn);
}

#[tokio::test(start_paused = true)]
async fn a_dead_child_refuses_messages_and_its_restart_gets_the_lost_turn() {
    let rig = rig(3).await;
    let first = rig.host.pids()[0];
    assert!(rig.handle.link.send(String::from("lost")));
    settle().await;
    rig.host.die(first);
    settle().await;
    assert!(!rig.handle.link.send(String::from("refused")), "no child is up");
    assert_eq!(rig.observer.stopped.load(Ordering::SeqCst), 1);
    tokio::time::sleep(Duration::from_secs(2)).await;
    let pids = rig.host.pids();
    assert_eq!(pids.len(), 2);
    assert_eq!(rig.host.texts(pids[1]), ["lost"]);
    assert_eq!(rig.observer.started.load(Ordering::SeqCst), 2);
}

#[tokio::test(start_paused = true)]
async fn a_message_given_during_the_backoff_goes_to_the_next_child() {
    let rig = rig(3).await;
    let first = rig.host.pids()[0];
    rig.host.die(first);
    settle().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let second = rig.host.pids()[1];
    assert!(rig.handle.link.send(String::from("after")));
    settle().await;
    assert_eq!(rig.host.texts(second), ["after"]);
}

#[tokio::test(start_paused = true)]
async fn restarts_stop_past_the_maximum_and_the_session_stays_crashed() {
    let rig = rig(1).await;
    rig.host.die(rig.host.pids()[0]);
    tokio::time::sleep(Duration::from_secs(2)).await;
    rig.host.die(rig.host.pids()[1]);
    tokio::time::sleep(Duration::from_secs(60)).await;
    assert_eq!(rig.host.pids().len(), 2);
    assert_eq!(rig.handle.state.borrow().status, Status::Crashed);
    assert!(rig.handle.task.is_finished());
}

#[tokio::test(start_paused = true)]
async fn a_child_that_is_not_on_the_api_key_is_ended_and_not_restarted() {
    let rig = rig(3).await;
    let pid = rig.host.pids()[0];
    assert!(rig.handle.link.send(String::from("one")));
    rig.host.say(pid, INIT_OAUTH);
    tokio::time::sleep(Duration::from_secs(60)).await;
    assert_eq!(rig.host.killed(), [pid]);
    assert_eq!(rig.host.pids().len(), 1);
    assert_eq!(rig.observer.faults.lock().unwrap().len(), 1);
    assert_eq!(rig.handle.state.borrow().status, Status::Stopped);
    assert!(!rig.handle.link.send(String::from("two")));
}

#[tokio::test(start_paused = true)]
async fn stop_ends_the_child_by_pid_and_the_supervisor() {
    let rig = rig(3).await;
    let pid = rig.host.pids()[0];
    rig.handle.link.stop();
    settle().await;
    assert_eq!(rig.host.stopped(), [pid]);
    assert_eq!(rig.handle.state.borrow().status, Status::Stopped);
    assert!(rig.handle.task.is_finished());
}
