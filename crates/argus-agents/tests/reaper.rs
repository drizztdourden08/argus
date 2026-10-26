//! The agent reaper: a working or waiting agent whose transcript has not grown for
//! `agents.reapMs` is closed as done through the board's locked write, with a note; agents without
//! a transcript are never closed, a sleep gap between polls skips the poll. Ported from
//! `agents.reaper.test.mjs`.

use argus_agents::fakes::{
    JsonBoards, ManualClock, RecordingActivity, agent_record, board_json, thread_record, touch,
    user_line,
};
use argus_agents::{
    ActivitySink, AgentReaper, AgentsConfig, Board, Clock, FileSystem, Ports, ReaperOptions, StdFs,
};
use serde_json::{Value, json};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

const MIN: i64 = 60_000;
const NOW: i64 = 1_792_756_800_000;

struct World {
    dir: TempDir,
    boards: Arc<JsonBoards>,
    clock: Arc<ManualClock>,
    activity: Arc<RecordingActivity>,
}

fn world() -> World {
    let dir = tempfile::tempdir().unwrap();
    let boards = Arc::new(JsonBoards::new(dir.path(), &["alpha"]));
    let clock = Arc::new(ManualClock::new(NOW));
    let activity = Arc::new(RecordingActivity::default());
    World {
        dir,
        boards,
        clock,
        activity,
    }
}

fn agent(id: &str, fields: &Value) -> Value {
    agent_record(id, NOW, fields)
}

impl World {
    fn transcript(&self, name: &str, quiet_min: i64) -> String {
        let file = self.dir.path().join(format!("{name}.jsonl"));
        fs::write(&file, user_line(NOW, &json!("go"))).unwrap();
        touch(&file, NOW - quiet_min * MIN);
        file.to_string_lossy().into_owned()
    }

    fn put(&self, agents: &[Value]) {
        let lead = thread_record("coordinator", "coordinator", &json!({}));
        self.boards.save("alpha", &board_json(&[lead], agents));
    }

    fn agent_on_board(&self, id: &str) -> Value {
        self.boards.load("alpha")["agents"][id].clone()
    }

    fn reaper(&self, config: &Value, opts: ReaperOptions) -> AgentReaper {
        let dyn_clock: Arc<dyn Clock> = self.clock.clone();
        let dyn_fs: Arc<dyn FileSystem> = Arc::new(StdFs);
        let sink: Arc<dyn ActivitySink> = self.activity.clone();
        let ports = Ports::new(self.boards.clone(), dyn_fs, dyn_clock).with_activity(sink);
        AgentReaper::new(ports, opts.configured(&AgentsConfig::from_json(config)))
    }

    fn plain(&self) -> AgentReaper {
        self.reaper(&json!({}), ReaperOptions::default())
    }
}

#[test]
fn a_working_hook_agent_with_a_transcript_quiet_for_more_than_20_minutes_is_closed_as_done_with_a_note()
 {
    let w = world();
    let file = w.transcript("a11", 25);
    w.put(&[agent(
        "A-11",
        &json!({ "source": "hook", "hookId": "h-11", "transcriptFile": file }),
    )]);
    assert_eq!(w.plain().tick(), vec!["alpha/A-11".to_owned()]);
    let a = w.agent_on_board("A-11");
    assert_eq!(a["status"], json!("done"));
    assert_eq!(
        a["note"],
        json!("no activity for 25 min, closed automatically")
    );
    assert!(
        a["finishedAt"]
            .as_str()
            .is_some_and(|t| t.starts_with("2026-"))
    );
    assert_eq!(a["reaped"], json!(true));
    assert_eq!(a["updatedBy"], json!("dashboard"));
    assert_eq!(a["rev"], json!(2));
}

#[test]
fn every_close_is_one_line_in_the_activity_log_after_the_board_was_written() {
    let w = world();
    let file = w.transcript("a12", 30);
    w.put(&[agent(
        "A-12",
        &json!({ "source": "hook", "hookId": "h-12", "transcriptFile": file }),
    )]);
    w.plain().tick();
    assert_eq!(w.agent_on_board("A-12")["status"], json!("done"));
    let lines = w.activity.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0].text,
        "reaper closed alpha/A-12 (agent-A-12): no activity 30 min"
    );
    assert_eq!(lines[0].kind, "agent");
    assert_eq!(lines[0].project, "alpha");
    assert_eq!(lines[0].thread.as_deref(), Some("coordinator"));
}

#[test]
fn a_waiting_agent_is_closed_too_and_stop_requested_finished_or_hidden_ones_are_left_alone() {
    let w = world();
    let file = |n: &str| w.transcript(n, 40);
    let finished = json!(argus_agents::iso::format_iso(NOW));
    w.put(&[
        agent(
            "A-21",
            &json!({ "status": "waiting", "transcriptFile": file("b21") }),
        ),
        agent(
            "A-22",
            &json!({ "status": "stop-requested", "transcriptFile": file("b22") }),
        ),
        agent(
            "A-23",
            &json!({ "status": "done", "finishedAt": finished, "transcriptFile": file("b23") }),
        ),
        agent(
            "A-24",
            &json!({ "archivedAt": finished, "transcriptFile": file("b24") }),
        ),
        agent(
            "A-25",
            &json!({ "deletedAt": finished, "transcriptFile": file("b25") }),
        ),
    ]);
    assert_eq!(w.plain().tick(), vec!["alpha/A-21".to_owned()]);
    assert_eq!(w.agent_on_board("A-22")["status"], json!("stop-requested"));
    assert_eq!(w.agent_on_board("A-23")["note"], json!(null));
    assert_eq!(w.agent_on_board("A-24")["status"], json!("working"));
    assert_eq!(w.agent_on_board("A-25")["status"], json!("working"));
}

#[test]
fn an_agent_whose_transcript_grew_within_the_quiet_time_stays() {
    let w = world();
    let file = w.transcript("c31", 19);
    w.put(&[agent("A-31", &json!({ "transcriptFile": file }))]);
    assert_eq!(w.plain().tick(), Vec::<String>::new());
    assert_eq!(w.agent_on_board("A-31")["status"], json!("working"));
}

#[test]
fn an_agent_with_no_transcript_a_manual_record_is_never_closed_however_old() {
    let w = world();
    let old = argus_agents::iso::format_iso(NOW - 600 * MIN);
    w.put(&[
        agent("A-41", &json!({ "startedAt": old })),
        agent("A-42", &json!({ "transcriptFile": null, "startedAt": old })),
    ]);
    assert_eq!(w.plain().tick(), Vec::<String>::new());
    assert_eq!(w.agent_on_board("A-41")["status"], json!("working"));
    assert_eq!(w.agent_on_board("A-42")["status"], json!("working"));
}

#[test]
fn a_transcript_file_that_is_not_there_counts_from_the_agent_start() {
    let w = world();
    let nowhere = w
        .dir
        .path()
        .join("nowhere.jsonl")
        .to_string_lossy()
        .into_owned();
    let started = |min: i64| argus_agents::iso::format_iso(NOW - min * MIN);
    w.put(&[
        agent(
            "A-51",
            &json!({ "transcriptFile": nowhere, "startedAt": started(30) }),
        ),
        agent(
            "A-52",
            &json!({ "transcriptFile": nowhere, "startedAt": started(5) }),
        ),
    ]);
    assert_eq!(w.plain().tick(), vec!["alpha/A-51".to_owned()]);
}

#[test]
fn the_quiet_time_is_config_agents_reap_ms() {
    let w = world();
    let file = w.transcript("d61", 8);
    w.put(&[agent("A-61", &json!({ "transcriptFile": file }))]);
    assert_eq!(w.plain().tick(), Vec::<String>::new());
    let short_reaper = w.reaper(&json!({ "reapMs": 5 * MIN }), ReaperOptions::default());
    assert_eq!(short_reaper.tick(), vec!["alpha/A-61".to_owned()]);
    assert!(
        w.agent_on_board("A-61")["note"]
            .as_str()
            .unwrap()
            .starts_with("no activity for 8 min")
    );
}

#[test]
fn what_the_board_file_says_wins_over_the_mirror() {
    let w = world();
    let file = w.transcript("e71", 30);
    let stale = agent("A-71", &json!({ "transcriptFile": file }));
    let finished = json!({ "status": "done", "finishedAt": argus_agents::iso::format_iso(NOW), "result": "finished a moment ago" });
    let mut settled = stale.clone();
    for (key, value) in finished.as_object().unwrap() {
        settled[key] = value.clone();
    }
    w.put(&[settled]);
    let mirror = Board {
        agents: vec![stale.as_object().cloned().unwrap()],
        threads: vec![],
    };
    w.boards.set_mirror("alpha", mirror);
    assert_eq!(w.plain().tick(), Vec::<String>::new());
    assert_eq!(
        w.agent_on_board("A-71")["result"],
        json!("finished a moment ago")
    );
    assert_eq!(w.agent_on_board("A-71")["note"], json!(null));
}

#[test]
fn a_poll_long_after_the_previous_one_the_machine_slept_closes_nothing() {
    let w = world();
    let file = w.transcript("f81", 15);
    w.put(&[agent("A-81", &json!({ "transcriptFile": file }))]);
    let reaper = w.reaper(
        &json!({}),
        ReaperOptions {
            poll_ms: MIN,
            ..ReaperOptions::default()
        },
    );
    let none = Vec::<String>::new();
    assert_eq!(reaper.tick(), none, "quiet for 15 min: not yet");
    w.clock.advance(10 * MIN);
    assert_eq!(
        reaper.tick(),
        none,
        "the last poll was 10 min ago: the agents may have been frozen too"
    );
    assert_eq!(w.agent_on_board("A-81")["status"], json!("working"));
    w.clock.advance(MIN);
    assert_eq!(
        reaper.tick(),
        vec!["alpha/A-81".to_owned()],
        "the next regular poll closes it"
    );
}
