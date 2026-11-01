//! The agent reaper: a working or waiting agent whose transcript has not grown for
//! `agents.reapMs` is closed as done through the board's locked write, with a note; agents without
//! a transcript are never closed, a sleep gap between polls skips the poll. Ported from
//! `agents.reaper.test.mjs`.
//! K1: when a quiet agent is closed and when it is kept.
// test-tag: critical

use argus_agents::fakes::{
    JsonBoards, RecordingActivity, agent_record, board_json, thread_record, touch, user_line,
};
use argus_agents::{
    ActivitySink, AgentReaper, AgentsConfig, Board, Clock, FileSystem, ManualClock, Ports,
    ReaperOptions, StdFs,
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
    let clock = Arc::new(ManualClock::at(NOW));
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
        self.put_lead(&json!({}), agents);
    }

    fn put_lead(&self, lead_fields: &Value, agents: &[Value]) {
        let lead = thread_record("coordinator", "coordinator", lead_fields);
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

const SID: &str = "sess-0000-aaaa";

/// Claude's own data for the agent: its transcript in the thread session's folder, a task output, and a worktree.
struct Claude {
    home: std::path::PathBuf,
    temp: std::path::PathBuf,
}

impl Claude {
    fn new(w: &World) -> Self {
        let root = w.dir.path().join("claude");
        let claude = Self {
            home: root.join("home"),
            temp: root.join("tmp"),
        };
        fs::create_dir_all(claude.subagents()).unwrap();
        fs::create_dir_all(claude.tasks()).unwrap();
        claude
    }

    fn subagents(&self) -> std::path::PathBuf {
        self.home
            .join("projects")
            .join("X--proj")
            .join(SID)
            .join("subagents")
    }

    fn tasks(&self) -> std::path::PathBuf {
        self.temp.join("X--proj").join(SID).join("tasks")
    }

    fn config(&self, more: &Value) -> Value {
        let mut agents = json!({ "claudeHome": self.home, "claudeTemp": self.temp });
        for (key, value) in more.as_object().unwrap() {
            agents[key] = value.clone();
        }
        agents
    }

    /// A reaper that reads this Claude data (`more` are further `agents` keys).
    fn reaper(&self, w: &World, more: &Value) -> AgentReaper {
        w.reaper(&self.config(more), ReaperOptions::default())
    }

    /// A hook agent whose transcript sits in the session folder and is `quiet_min` quiet.
    fn agent(&self, id: &str, quiet_min: i64, at: i64, fields: &Value) -> Value {
        let file = self.subagents().join(format!("agent-{id}.jsonl"));
        fs::write(&file, user_line(at, &json!("go"))).unwrap();
        touch(&file, at - quiet_min * MIN);
        let mut record = json!({
            "source": "hook", "hookId": id, "claudeId": id,
            "transcriptFile": file.to_string_lossy()
        });
        for (key, value) in fields.as_object().unwrap() {
            record[key] = value.clone();
        }
        agent(&format!("A-{}", &id[id.len() - 2..]), &record)
    }

    /// A worktree folder with one file written `age_min` ago.
    fn worktree(w: &World, name: &str, age_min: i64) -> String {
        let dir = w.dir.path().join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        let file = dir.join("src").join("big.rs");
        fs::write(&file, "fn main() {}").unwrap();
        touch(&file, NOW - age_min * MIN);
        dir.to_string_lossy().into_owned()
    }
}

fn with_session(w: &World, agents: &[Value]) {
    w.put_lead(&json!({ "sessionId": SID }), agents);
}

fn none() -> Vec<String> {
    Vec::new()
}

#[test]
fn an_opus_agent_writing_files_in_its_worktree_is_not_closed_although_its_transcript_is_30_minutes_quiet()
 {
    let w = world();
    let c = Claude::new(&w);
    let tree = Claude::worktree(&w, "wt1", 2);
    let a = c.agent(
        "a0000000000000011",
        30,
        NOW,
        &json!({ "model": "opus", "worktree": tree }),
    );
    with_session(&w, &[a]);
    assert_eq!(c.reaper(&w, &json!({ "reapMs": 20 * MIN })).tick(), none());
    assert_eq!(w.agent_on_board("A-11")["status"], json!("working"));
}

#[test]
fn a_sonnet_agent_whose_worktree_shows_work_minutes_ago_is_kept_once_it_is_quiet_too_it_is_closed_with_the_newest_sign()
 {
    let w = world();
    let c = Claude::new(&w);
    let tree = Claude::worktree(&w, "wt2", 3);
    with_session(
        &w,
        &[c.agent("a0000000000000012", 30, NOW, &json!({ "worktree": tree }))],
    );
    assert_eq!(c.reaper(&w, &json!({})).tick(), none());
    touch(
        &w.dir.path().join("wt2").join("src").join("big.rs"),
        NOW - 26 * MIN,
    );
    assert_eq!(
        c.reaper(&w, &json!({})).tick(),
        vec!["alpha/A-12".to_owned()]
    );
    let closed = w.agent_on_board("A-12");
    let note = closed["note"].as_str().unwrap();
    assert!(
        note == "no activity for 26 min, closed automatically"
            || note == "no activity for 27 min, closed automatically",
        "{note}"
    );
    assert_eq!(closed["closedBy"], json!("reaper"));
}

#[test]
fn the_output_file_of_a_background_task_of_the_agent_counts_as_a_sign_of_life() {
    let w = world();
    let c = Claude::new(&w);
    with_session(&w, &[c.agent("a0000000000000013", 40, NOW, &json!({}))]);
    let out = c.tasks().join("a0000000000000013.output");
    fs::write(&out, "building...").unwrap();
    touch(&out, NOW - MIN);
    assert_eq!(c.reaper(&w, &json!({})).tick(), none());
    touch(&out, NOW - 41 * MIN);
    assert_eq!(
        c.reaper(&w, &json!({})).tick(),
        vec!["alpha/A-13".to_owned()],
        "a truly quiet one is still closed"
    );
}

#[test]
fn an_opus_level_agent_gets_at_least_60_minutes_haiku_and_sonnet_keep_the_20_minute_limit() {
    let w = world();
    let c = Claude::new(&w);
    let opus_45 = json!({ "model": "claude-opus-4-5", "modelUsed": "claude-opus-4-5" });
    with_session(
        &w,
        &[
            c.agent("a0000000000000014", 40, NOW, &json!({ "model": "opus" })),
            c.agent("a0000000000000015", 40, NOW, &json!({ "model": "sonnet" })),
            c.agent("a0000000000000016", 40, NOW, &opus_45),
        ],
    );
    assert_eq!(
        c.reaper(&w, &json!({})).tick(),
        vec!["alpha/A-15".to_owned()]
    );
    w.clock.advance(MIN);
    let later = NOW + MIN;
    touch(
        &c.subagents().join("agent-a0000000000000014.jsonl"),
        later - 62 * MIN,
    );
    assert_eq!(
        c.reaper(&w, &json!({})).tick(),
        vec!["alpha/A-14".to_owned()]
    );
    let note = w.agent_on_board("A-14")["note"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(note.starts_with("no activity for 62 min"), "{note}");
    assert_eq!(w.agent_on_board("A-16")["status"], json!("working"));
    let short_opus = c.reaper(&w, &json!({ "reapOpusMs": 30 * MIN }));
    assert_eq!(
        short_opus.tick(),
        vec!["alpha/A-16".to_owned()],
        "the opus limit is configurable"
    );
}

#[test]
fn a_record_without_a_worktree_name_or_with_one_that_is_not_a_folder_is_judged_by_the_transcript_and_the_task_alone()
 {
    let w = world();
    let c = Claude::new(&w);
    let fields = json!({ "worktree": "no-such-worktree" });
    with_session(&w, &[c.agent("a0000000000000017", 30, NOW, &fields)]);
    assert_eq!(
        c.reaper(&w, &json!({})).tick(),
        vec!["alpha/A-17".to_owned()]
    );
}
