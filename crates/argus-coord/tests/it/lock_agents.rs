//! I-144: `coord lock --agent A-n` labels a claim so two agents of one thread queue and release
//! apart. The holder stays the thread; the record carries `holderAgent`.

// test-tag: critical

use crate::work_board::Work;
use serde_json::json;

/// A board where agent A-1 of the worker holds `build` and agent A-2 waits.
fn two_agents() -> Work {
    let work = Work::new();
    work.worker_ok(&["lock", "claim", "build", "--agent", "A-1"]);
    work.worker_ok(&["lock", "claim", "build", "--agent", "A-2"]);
    work
}

#[test]
fn holder_agent_is_recorded_and_the_holder_stays_the_thread() {
    let work = Work::new();
    let said = work.worker_ok(&["lock", "claim", "build", "--agent", "A-1"]);
    assert!(
        said.contains("coord lock release build --agent A-1"),
        "{said}"
    );
    let lock = work.board()["locks"]["build"].clone();
    assert_eq!(lock["holder"], "worker");
    assert_eq!(lock["holderAgent"], "A-1");
    assert!(work.worker_ok(&["lock", "list"]).contains("worker#A-1"));
}

#[test]
fn two_agents_of_one_thread_queue_apart() {
    let work = two_agents();
    let lock = work.board()["locks"]["build"].clone();
    assert_eq!(lock["queue"], json!(["worker#A-2"]));
    let again = work.worker_ok(&["lock", "claim", "build", "--agent", "A-1"]);
    assert!(again.contains("you already hold it"), "{again}");
    let list = work.worker_ok(&["lock", "list"]);
    assert!(list.contains("worker#A-1"), "{list}");
    assert!(list.contains("worker#A-2"), "{list}");
}

#[test]
fn a_release_hands_the_lock_to_the_next_agent() {
    let work = two_agents();
    let said = work.worker_ok(&["lock", "release", "build", "--agent", "A-1"]);
    assert!(said.contains("passed to worker#A-2."), "{said}");
    let lock = work.board()["locks"]["build"].clone();
    assert_eq!(lock["holder"], "worker");
    assert_eq!(lock["holderAgent"], "A-2");
    assert_eq!(lock["queue"], json!([]));
    work.worker_ok(&["lock", "release", "build", "--agent", "A-2"]);
    assert_eq!(work.board()["locks"]["build"]["holderAgent"], json!(null));
}

#[test]
fn a_release_by_the_wrong_agent_is_refused() {
    let work = two_agents();
    let ran = work.worker(&["lock", "release", "build", "--agent", "A-2"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.contains("build: held by agent A-1, not A-2."),
        "{}",
        ran.err
    );
    assert_eq!(work.board()["locks"]["build"]["holderAgent"], "A-1");
}

#[test]
fn drop_leaves_the_queue_as_that_agent_only() {
    let work = two_agents();
    work.worker_ok(&["lock", "drop", "build"]);
    assert_eq!(
        work.board()["locks"]["build"]["queue"],
        json!(["worker#A-2"])
    );
    work.worker_ok(&["lock", "drop", "build", "--agent", "A-2"]);
    assert_eq!(work.board()["locks"]["build"]["queue"], json!([]));
}
