//! The agent reconciler: the board's agent records against the sub-agent transcripts and task
//! outputs Claude keeps for the owning session. Temp folders stand in for `~/.claude/projects` and
//! the temp `claude` folder (config `agents.claudeHome` and `agents.claudeTemp`), a manual clock
//! drives the timeouts, and the wake engine records what it is given. The shapes are the real
//! ones found on disk: `agent-<id>.jsonl`, `agent-<id>.meta.json` with `finished: true` once the
//! turn ended, `<sessionId>/tasks/<id>.output`. Ported from `agents.reconciler.test.mjs`.
//! K1: how board records are linked, woken, closed and reopened from Claude data.
// test-tag: critical

use argus_agents::fakes::{
    JsonBoards, RecordingWakes, agent_record, board_json, thread_record, touch,
};
use argus_agents::iso::format_iso;
use argus_agents::reconciler::{Class, Counts, Finding};
use argus_agents::{
    AgentReconciler, AgentsConfig, Board, Clock, FileSystem, ManualClock, Ports, ReconcilerOptions,
    RunReport, StdFs, SystemClock, WakeEvent, WakeSink,
};
use serde_json::{Value, json};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

const MIN: i64 = 60_000;
const SID: &str = "sess-0000-aaaa";
const SID_UI: &str = "sess-0000-bbbb";
const PROJ: &str = "X--proj";
const FINAL: &str = "All done: the final answer.";

struct World {
    _home: TempDir,
    claude: TempDir,
    boards: Arc<JsonBoards>,
    clock: Arc<ManualClock>,
    wakes: Arc<RecordingWakes>,
    now: i64,
}

struct Sub {
    description: &'static str,
    model: Option<&'static str>,
    quiet: i64,
    ended: bool,
    end_turn: bool,
    flag: bool,
    dir: Option<PathBuf>,
}

impl Default for Sub {
    fn default() -> Self {
        Self {
            description: "work",
            model: None,
            quiet: 0,
            ended: false,
            end_turn: false,
            flag: true,
            dir: None,
        }
    }
}

fn line(role: &str, text: &str, stop: Option<&str>, at: i64) -> String {
    let mut message = json!({ "role": role, "content": [{ "type": "text", "text": text }] });
    if let Some(stop) = stop {
        message["stop_reason"] = json!(stop);
    }
    format!(
        "{}\n",
        json!({ "type": role, "message": message, "timestamp": format_iso(at) })
    )
}

fn tool_line() -> String {
    let call =
        json!({ "type": "tool_use", "id": "t1", "name": "Bash", "input": { "command": "ls" } });
    let message = json!({ "role": "assistant", "content": [call], "stop_reason": "tool_use" });
    format!("{}\n", json!({ "type": "assistant", "message": message }))
}

fn world() -> World {
    let home = tempfile::tempdir().unwrap();
    let claude = tempfile::tempdir().unwrap();
    for sid in [SID, SID_UI] {
        fs::create_dir_all(
            claude
                .path()
                .join("home")
                .join("projects")
                .join(PROJ)
                .join(sid),
        )
        .unwrap();
    }
    let now = SystemClock.now_ms();
    let boards = Arc::new(JsonBoards::new(home.path(), &["alpha"]));
    let main = thread_record("main", "lead", &json!({}))
        .as_object()
        .cloned()
        .unwrap();
    boards.set_mirror(
        "portfolio",
        Board {
            agents: vec![],
            threads: vec![main],
            ..Board::default()
        },
    );
    World {
        _home: home,
        claude,
        boards,
        clock: Arc::new(ManualClock::at(now)),
        wakes: Arc::new(RecordingWakes::default()),
        now,
    }
}

impl World {
    fn sub_dir(&self, sid: &str) -> PathBuf {
        self.claude
            .path()
            .join("home")
            .join("projects")
            .join(PROJ)
            .join(sid)
            .join("subagents")
    }

    fn task_dir(&self) -> PathBuf {
        self.claude
            .path()
            .join("tmp")
            .join(PROJ)
            .join(SID)
            .join("tasks")
    }

    fn at(&self, ms: i64) -> String {
        format_iso(self.now + ms)
    }

    fn agent(&self, id: &str, fields: &Value) -> Value {
        agent_record(id, self.now, fields)
    }

    fn touch(&self, file: &std::path::Path, quiet_min: i64) {
        touch(file, self.clock.now_ms() - quiet_min * MIN);
    }

    fn subagent(&self, id: &str, sub: Sub) -> String {
        let dir = sub.dir.unwrap_or_else(|| self.sub_dir(SID));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join(format!("agent-{id}.jsonl"));
        let last = if sub.end_turn {
            line("assistant", FINAL, Some("end_turn"), self.now)
        } else {
            tool_line()
        };
        fs::write(&file, line("user", "go", None, self.now) + &last).unwrap();
        let meta = dir.join(format!("agent-{id}.meta.json"));
        let mut body = json!({ "agentType": "general-purpose", "description": sub.description, "toolUseId": format!("toolu_{id}"), "spawnDepth": 1 });
        if let Some(model) = sub.model {
            body["model"] = json!(model);
        }
        if sub.ended && sub.flag {
            body["finished"] = json!(true);
        }
        fs::write(&meta, body.to_string()).unwrap();
        self.touch(&file, sub.quiet);
        self.touch(&meta, sub.quiet);
        file.to_string_lossy().into_owned()
    }

    fn resume(&self, id: &str, quiet: i64) {
        let dir = self.sub_dir(SID);
        let file = dir.join(format!("agent-{id}.jsonl"));
        let mut handle = OpenOptions::new().append(true).open(&file).unwrap();
        handle.write_all(tool_line().as_bytes()).unwrap();
        let body =
            json!({ "agentType": "general-purpose", "description": "work", "spawnDepth": 1 });
        fs::write(dir.join(format!("agent-{id}.meta.json")), body.to_string()).unwrap();
        self.touch(&file, quiet);
    }

    fn put(&self, agents: &[Value]) {
        let sessions = json!({ "root": "X:\\proj" });
        let mut lead = thread_record("lead", "lead", &sessions);
        lead["sessionId"] = json!(SID);
        let mut ui = thread_record("ui", "thread", &sessions);
        ui["sessionId"] = json!(SID_UI);
        self.boards.save("alpha", &board_json(&[lead, ui], agents));
    }

    fn edit_board(&self, change: impl FnOnce(&mut Value)) {
        let mut board = self.boards.load("alpha");
        change(&mut board);
        self.boards.save("alpha", &board);
    }

    fn a(&self, id: &str) -> Value {
        self.boards.load("alpha")["agents"][id].clone()
    }

    fn reconciler(&self, wake: bool) -> AgentReconciler {
        let cfg = json!({
            "claudeHome": self.claude.path().join("home").to_string_lossy(),
            "claudeTemp": self.claude.path().join("tmp").to_string_lossy(),
            "reconcileMs": 60 * MIN
        });
        let clock: Arc<dyn Clock> = self.clock.clone();
        let fs: Arc<dyn FileSystem> = Arc::new(StdFs);
        let mut ports = Ports::new(self.boards.clone(), fs, clock);
        if wake {
            let sink: Arc<dyn WakeSink> = self.wakes.clone();
            ports = ports.with_wakes(sink);
        }
        let opts = ReconcilerOptions::default().configured(&AgentsConfig::from_json(&cfg));
        AgentReconciler::new(ports, opts)
    }

    fn events(&self) -> Vec<WakeEvent> {
        self.wakes.events()
    }

    fn advance(&self, min: i64) {
        self.clock.advance(min * MIN);
    }
}

fn counts(run: &RunReport) -> Counts {
    run.projects["alpha"]
}

fn found<'a>(run: &'a RunReport, id: &str) -> Option<&'a Finding> {
    run.findings.iter().find(|f| f.id == id)
}

fn ids(wakes: &[WakeEvent]) -> Vec<String> {
    wakes
        .iter()
        .map(|e| format!("{}:{}", e.subject, e.reason))
        .collect()
}

#[test]
fn a_manual_record_gets_the_transcript_id_and_hook_id_of_the_sub_agent_whose_description_reads_like_it()
 {
    let w = world();
    let started = w.at(-2 * MIN);
    let ids_by = [
        ("A-44", "a8e4bd817ac9c4128"),
        ("A-48", "a75bc95bcaf0a7a96"),
        ("A-53", "adeccacd42e92e27b"),
    ];
    w.subagent(
        ids_by[0].1,
        Sub {
            description: "Rebuild Controllers screen to approved design",
            model: Some("opus"),
            ..Sub::default()
        },
    );
    w.subagent(
        ids_by[1].1,
        Sub {
            description: "R1: restore the apworld on master",
            model: Some("sonnet"),
            ..Sub::default()
        },
    );
    w.subagent(
        ids_by[2].1,
        Sub {
            description: "R3: build the Archipelia lobby client",
            model: Some("sonnet"),
            ..Sub::default()
        },
    );
    w.put(&[
        w.agent("A-44", &json!({ "name": "controllers-opus", "task": "Rebuild Controllers to approved wireframe: live nav, no nested boxes", "model": "opus", "branch": "agent/lead-controllers-live", "startedAt": started })),
        w.agent("A-48", &json!({ "name": "apworld-r1", "task": "R1: restore apworld package, reproducible build", "startedAt": started })),
        w.agent("A-53", &json!({ "name": "lobby-r3", "task": "R3: lobby client vs Archipelia fake server", "startedAt": started })),
    ]);
    let run = w.reconciler(true).tick();
    for (agent, id) in ids_by {
        assert_eq!(w.a(agent)["claudeId"], json!(id));
        assert_eq!(w.a(agent)["hookId"], json!(id));
        assert_eq!(w.a(agent)["updatedBy"], json!("dashboard"));
    }
    let file = w.sub_dir(SID).join("agent-a8e4bd817ac9c4128.jsonl");
    assert_eq!(w.a("A-44")["transcriptFile"], json!(file.to_string_lossy()));
    assert_eq!(counts(&run).linked, 3);
    assert_eq!(
        counts(&run).alive,
        3,
        "they are growing: nothing else to do"
    );
    assert_eq!(w.events(), vec![]);
}

#[test]
fn no_confident_match_no_link() {
    let w = world();
    let started = w.at(-2 * MIN);
    let sonnet = |description| Sub {
        description,
        model: Some("sonnet"),
        ..Sub::default()
    };
    w.subagent("a0000000000000001", sonnet("Rebuild settings"));
    w.subagent("a0000000000000002", sonnet("Rebuild Controllers screen"));
    w.subagent("a0000000000000003", sonnet("Restore apworld package"));
    w.put(&[
        w.agent("A-1", &json!({ "name": "rebuild-all", "task": "rebuild everything", "startedAt": started })),
        w.agent("A-2", &json!({ "name": "controllers", "task": "Rebuild Controllers screen", "model": "opus", "startedAt": started })),
        w.agent("A-3", &json!({ "name": "apworld", "task": "Restore apworld package", "startedAt": w.at(-5 * 60 * MIN) })),
        w.agent("A-4", &json!({ "name": "apworld-again", "task": "Restore apworld package", "hookId": "a0000000000000003", "claudeId": "a0000000000000003", "startedAt": started })),
    ]);
    let run = w.reconciler(true).tick();
    assert!(w.a("A-1").get("claudeId").is_none(), "one shared word");
    assert!(
        w.a("A-2").get("claudeId").is_none(),
        "opus asked, the transcript ran sonnet"
    );
    assert!(
        w.a("A-3").get("claudeId").is_none(),
        "started five hours before the transcript"
    );
    assert_eq!(counts(&run).linked, 0);
}

#[test]
fn two_records_that_read_alike_and_two_transcripts_that_read_alike_are_not_guessed_at() {
    let w = world();
    for id in ["a0000000000000011", "a0000000000000012"] {
        w.subagent(
            id,
            Sub {
                description: "Fix lint errors in the desktop app",
                ..Sub::default()
            },
        );
    }
    let task = json!({ "name": "lint-fix", "task": "Fix lint errors desktop app", "startedAt": w.at(-2 * MIN) });
    w.put(&[w.agent("A-1", &task)]);
    w.reconciler(true).tick();
    assert!(
        w.a("A-1").get("claudeId").is_none(),
        "two equally good transcripts for one record"
    );
}

#[test]
fn a_transcript_of_an_earlier_session_of_the_same_project_folder_can_be_the_one() {
    let w = world();
    let old = Sub {
        description: "Check the Google Cloud servers",
        dir: Some(w.sub_dir("sess-0000-cccc")),
        ..Sub::default()
    };
    w.subagent("a0000000000000021", old);
    let task = json!({ "name": "gcp-restore", "task": "Check and restore the Google Cloud servers", "startedAt": w.at(-2 * MIN) });
    w.put(&[w.agent("A-1", &task)]);
    w.reconciler(true).tick();
    assert_eq!(w.a("A-1")["claudeId"], json!("a0000000000000021"));
}

#[test]
fn a_session_that_belongs_to_another_thread_is_never_offered_to_this_one() {
    let w = world();
    let other = Sub {
        description: "Check the Google Cloud servers",
        dir: Some(w.sub_dir(SID_UI)),
        ..Sub::default()
    };
    w.subagent("a0000000000000031", other);
    let task = json!({ "name": "gcp-restore", "task": "Check and restore the Google Cloud servers", "startedAt": w.at(-2 * MIN) });
    w.put(&[w.agent("A-1", &task)]);
    w.reconciler(true).tick();
    assert!(w.a("A-1").get("claudeId").is_none());
}

#[test]
fn claude_id_is_looked_up_first_a_record_with_an_id_and_no_transcript_gets_the_path() {
    let w = world();
    w.subagent(
        "a0000000000000041",
        Sub {
            description: "something else entirely",
            ..Sub::default()
        },
    );
    w.put(&[w.agent(
        "A-1",
        &json!({ "name": "zzz", "task": "qqq", "claudeId": "a0000000000000041" }),
    )]);
    w.reconciler(true).tick();
    let file = w.sub_dir(SID).join("agent-a0000000000000041.jsonl");
    assert_eq!(w.a("A-1")["transcriptFile"], json!(file.to_string_lossy()));
}

#[test]
fn a_record_that_only_has_hook_id_gets_claude_id() {
    let w = world();
    let file = w.subagent("a0000000000000051", Sub::default());
    w.put(&[w.agent(
        "A-1",
        &json!({ "hookId": "a0000000000000051", "transcriptFile": file, "source": "hook" }),
    )]);
    w.reconciler(true).tick();
    assert_eq!(w.a("A-1")["claudeId"], json!("a0000000000000051"));
    assert!(w.a("A-1").get("idMismatch").is_none());
}

#[test]
fn ids_that_disagree_are_flagged_and_repaired_from_the_file_when_exactly_one_has_a_transcript() {
    let w = world();
    let file = w.subagent("a0000000000000061", Sub::default());
    w.put(&[w.agent(
        "A-1",
        &json!({ "claudeId": "a0000000000000061", "hookId": "a-wrong-id", "transcriptFile": file }),
    )]);
    w.reconciler(true).tick();
    let a = w.a("A-1");
    assert_eq!(a["idMismatch"], json!(true));
    assert_eq!(a["claudeId"], json!("a0000000000000061"));
    assert_eq!(
        a["hookId"],
        json!("a0000000000000061"),
        "the older name follows"
    );
    assert_eq!(a["transcriptFile"], json!(file));
}

#[test]
fn a_wrong_transcript_file_is_repaired_from_the_claude_id_and_when_both_ids_have_one_it_is_only_flagged()
 {
    let w = world();
    w.subagent("a0000000000000071", Sub::default());
    let other = w.subagent("a0000000000000072", Sub::default());
    let nowhere = w.sub_dir(SID).join("agent-nowhere.jsonl");
    w.put(&[
        w.agent("A-1", &json!({ "claudeId": "a0000000000000071", "transcriptFile": nowhere.to_string_lossy() })),
        w.agent("A-2", &json!({ "claudeId": "a0000000000000071b", "transcriptFile": other })),
    ]);
    w.subagent("a0000000000000071b", Sub::default());
    w.reconciler(true).tick();
    let repaired = w.sub_dir(SID).join("agent-a0000000000000071.jsonl");
    assert_eq!(
        w.a("A-1")["transcriptFile"],
        json!(repaired.to_string_lossy())
    );
    assert_eq!(w.a("A-1")["idMismatch"], json!(true));
    assert_eq!(
        w.a("A-2")["idMismatch"],
        json!(true),
        "both ids have a transcript: which one is right is unknown"
    );
    assert_eq!(w.a("A-2")["claudeId"], json!("a0000000000000071b"));
    assert_eq!(w.a("A-2")["transcriptFile"], json!(other));
}

#[test]
fn two_records_with_one_claude_id_are_one_agent_the_hook_bound_record_stays_and_the_other_is_closed()
 {
    let w = world();
    let id = "a8e4bd817ac9c4128";
    let file = w.subagent(id, Sub::default());
    let hooked = json!({ "name": "general-purpose-4128", "task": "general-purpose", "model": "opus", "status": "done", "finishedAt": w.at(-5 * MIN), "source": "hook", "hookId": id, "claudeId": id, "transcriptFile": file });
    let lead = json!({ "name": "controllers-opus", "task": "Rebuild Controllers", "model": "opus", "modelWhy": "owner asked: Opus rebuild", "item": "I-79", "checkinMin": 20, "claudeId": id });
    w.put(&[w.agent("A-54", &hooked), w.agent("A-44", &lead)]);
    let run = w.reconciler(true).tick();
    let dup = w.a("A-44");
    assert_eq!(dup["status"], json!("stopped"));
    assert_eq!(dup["duplicateOf"], json!("A-54"));
    assert_eq!(
        dup["note"],
        json!("duplicate of A-54 (same Claude agent id), closed automatically")
    );
    assert_eq!(counts(&run).duplicates, 1);
    assert_keeper_took_the_leads_fields(&w.a("A-54"));
}

fn assert_keeper_took_the_leads_fields(keeper: &Value) {
    assert_eq!(keeper["name"], json!("controllers-opus"));
    assert_eq!(keeper["item"], json!("I-79"));
    assert_eq!(keeper["checkinMin"], json!(20));
    assert_eq!(keeper["modelWhy"], json!("owner asked: Opus rebuild"));
    assert_eq!(keeper["source"], json!("hook"));
    assert_eq!(
        keeper["status"],
        json!("working"),
        "its transcript grew after the hook closed it: it was resumed"
    );
}

#[test]
fn alive_a_transcript_that_grew_within_the_stall_window_changes_nothing_and_sends_nothing() {
    let w = world();
    let file = w.subagent(
        "a0000000000000081",
        Sub {
            quiet: 1,
            ..Sub::default()
        },
    );
    w.put(&[w.agent(
        "A-1",
        &json!({ "claudeId": "a0000000000000081", "transcriptFile": file }),
    )]);
    let before = fs::read_to_string(w.boards.path("alpha")).unwrap();
    let run = w.reconciler(true).tick();
    assert_eq!(counts(&run).alive, 1);
    assert_eq!(
        fs::read_to_string(w.boards.path("alpha")).unwrap(),
        before,
        "the board file was not even rewritten"
    );
    assert_eq!(w.events(), vec![]);
}

#[test]
fn quiet_a_transcript_that_stopped_growing_with_its_turn_not_ended_is_left_to_the_reaper() {
    let w = world();
    let file = w.subagent(
        "a0000000000000091",
        Sub {
            quiet: 12,
            ..Sub::default()
        },
    );
    w.put(&[w.agent(
        "A-1",
        &json!({ "claudeId": "a0000000000000091", "transcriptFile": file }),
    )]);
    w.advance(40);
    let run = w.reconciler(true).tick();
    assert_eq!(counts(&run).quiet, 1);
    assert_eq!(w.a("A-1")["status"], json!("working"));
    assert_eq!(w.events(), vec![]);
    assert_eq!(found(&run, "A-1").unwrap().class, Class::Quiet);
}

#[test]
fn a_task_output_counts_too_alive_while_it_grows_and_a_shell_exit_marker_means_it_ended() {
    let w = world();
    fs::create_dir_all(w.task_dir()).unwrap();
    let live = w.task_dir().join("b1live.output");
    fs::write(&live, "building...\n").unwrap();
    w.touch(&live, 1);
    let done = w.task_dir().join("b2done.output");
    fs::write(&done, "built\n\n[exited with code 0]\n").unwrap();
    w.touch(&done, 10);
    w.put(&[
        w.agent("A-1", &json!({ "claudeId": "b1live" })),
        w.agent("A-2", &json!({ "claudeId": "b2done" })),
    ]);
    let run = w.reconciler(true).tick();
    assert!(
        found(&run, "A-1").is_none(),
        "alive agents are only counted"
    );
    assert_eq!(counts(&run).alive, 1);
    assert_eq!(found(&run, "A-2").unwrap().class, Class::Idle);
}

fn ended(w: &World, sub: Sub) -> String {
    let file = w.subagent(
        "a8e4bd817ac9c4128",
        Sub {
            ended: true,
            end_turn: true,
            ..sub
        },
    );
    let fields = json!({ "name": "controllers-opus", "model": "opus", "claudeId": "a8e4bd817ac9c4128", "transcriptFile": file });
    w.put(&[w.agent("A-44", &fields)]);
    file
}

#[test]
fn an_ended_turn_is_not_closed_and_not_woken_before_the_idle_time() {
    let w = world();
    ended(
        &w,
        Sub {
            quiet: 3,
            ..Sub::default()
        },
    );
    let r = w.reconciler(true);
    let run = r.tick();
    assert_eq!(counts(&run).idle, 1);
    assert_eq!(found(&run, "A-44").unwrap().class, Class::Idle);
    assert_eq!(w.events(), vec![]);
    assert!(w.a("A-44")["idleSince"].is_string());
    w.advance(3);
    r.tick();
    assert_eq!(ids(&w.events()), vec!["A-44:agent-idle".to_owned()]);
}

#[test]
fn an_idle_agent_gets_one_wake_with_the_id_then_is_closed_as_finished_after_the_clear_period() {
    let w = world();
    ended(
        &w,
        Sub {
            quiet: 6,
            ..Sub::default()
        },
    );
    let r = w.reconciler(true);
    r.tick();
    let sent = w.events();
    assert_eq!(sent.len(), 1);
    assert_eq!(
        (sent[0].scope.as_str(), sent[0].thread.as_str()),
        ("alpha", "lead")
    );
    assert_eq!(
        sent[0].text,
        "A-44 controllers-opus (opus, id a8e4bd817ac9c4128) ended its turn: resume it by id (SendMessage to the agent id) or close the record (coord agent done A-44)."
    );
    assert!(w.a("A-44")["idleSince"].is_string() && w.a("A-44")["idleWokeAt"].is_string());
    assert_eq!(w.a("A-44")["status"], json!("working"));
    w.advance(10);
    r.tick();
    assert_eq!(w.events().len(), 1, "not again");
    w.advance(10);
    r.tick();
    assert_eq!(
        w.a("A-44")["status"],
        json!("working"),
        "only 20 of the 30 minutes"
    );
    w.advance(11);
    let run = r.tick();
    assert_closed_as_finished(&w, &run);
}

#[test]
fn a_record_closed_as_finished_hands_its_items_back_to_its_thread() {
    let w = world();
    ended(
        &w,
        Sub {
            quiet: 6,
            ..Sub::default()
        },
    );
    w.edit_board(|board| {
        board["agents"]["A-44"]["items"] = json!(["I-1", "I-2"]);
        board["items"] = json!({
            "I-1": { "id": "I-1", "assignee": "agent:A-44", "rev": 1 },
            "I-2": { "id": "I-2", "assignee": "ui", "rev": 1 },
        });
    });
    let r = w.reconciler(true);
    r.tick();
    w.advance(31);
    r.tick();
    assert_eq!(w.a("A-44")["status"], json!("done"));
    let items = w.boards.load("alpha")["items"].clone();
    assert_eq!(items["I-1"]["assignee"], json!("lead"));
    assert_eq!(items["I-1"]["rev"], json!(2));
    assert_eq!(items["I-2"]["assignee"], json!("ui"));
}

fn assert_closed_as_finished(w: &World, run: &RunReport) {
    let closed = w.a("A-44");
    assert_eq!(closed["status"], json!("done"));
    assert_eq!(closed["note"], json!("finished, closed automatically"));
    assert_eq!(closed["result"], json!(FINAL));
    assert_eq!(closed["closedBy"], json!("reconciler"));
    assert!(closed.get("idleSince").is_none());
    assert_eq!(counts(run).closed, 1);
    assert_eq!(
        w.events().len(),
        1,
        "a finished close is not a fault: the main lead hears nothing"
    );
}

#[test]
fn growth_after_the_resume_revives_it_to_alive_and_forgets_the_idle_clock() {
    let w = world();
    ended(
        &w,
        Sub {
            quiet: 6,
            ..Sub::default()
        },
    );
    let r = w.reconciler(true);
    r.tick();
    assert_eq!(w.events().len(), 1);
    w.advance(10);
    w.resume("a8e4bd817ac9c4128", 0);
    let run = r.tick();
    assert_eq!(counts(&run).alive, 1);
    let a = w.a("A-44");
    assert_eq!(a["status"], json!("working"));
    assert!(a.get("idleSince").is_none() && a.get("idleWokeAt").is_none());
    w.advance(60);
    r.tick();
    assert_eq!(
        w.a("A-44")["status"],
        json!("working"),
        "quiet now, but not ended: nothing here closes it"
    );
}

#[test]
fn an_older_claude_code_writes_no_flag_a_last_end_turn_on_a_quiet_transcript_means_the_same() {
    let w = world();
    let file = w.subagent(
        "a0000000000000101",
        Sub {
            quiet: 10,
            ended: true,
            flag: false,
            end_turn: true,
            ..Sub::default()
        },
    );
    w.put(&[w.agent(
        "A-1",
        &json!({ "claudeId": "a0000000000000101", "transcriptFile": file }),
    )]);
    let run = w.reconciler(true).tick();
    assert_eq!(counts(&run).idle, 1);
    assert_eq!(w.events()[0].reason, "agent-idle");
}

#[test]
fn without_the_wake_service_the_clock_runs_from_the_end_of_the_turn() {
    let w = world();
    ended(
        &w,
        Sub {
            quiet: 40,
            ..Sub::default()
        },
    );
    let r = w.reconciler(false);
    let run = r.tick();
    assert_eq!(
        counts(&run).closed,
        0,
        "the dashboard was just started: a full period first"
    );
    w.advance(31);
    r.tick();
    assert_eq!(w.a("A-44")["status"], json!("done"));
}

fn old(w: &World, min: i64, fields: &Value) -> Value {
    let mut base = json!({ "name": "controllers-opus", "model": "opus", "startedAt": w.at(-min * MIN), "updatedAt": w.at(-min * MIN) });
    for (key, value) in fields.as_object().unwrap() {
        base[key] = value.clone();
    }
    w.agent(base["id"].as_str().unwrap_or("A-44"), &base)
}

#[test]
fn a_recent_record_or_one_the_lead_touched_lately_is_only_pending() {
    let w = world();
    w.put(&[
        old(&w, 10, &json!({})),
        old(
            &w,
            60,
            &json!({ "id": "A-45", "name": "other", "updatedAt": w.at(-3 * MIN) }),
        ),
    ]);
    let run = w.reconciler(true).tick();
    assert_eq!(counts(&run).pending, 2);
    assert_eq!(counts(&run).orphan, 0);
    assert_eq!(w.events(), vec![]);
}

#[test]
fn an_orphan_wakes_its_thread_once_is_closed_as_lost_after_the_clear_period_and_the_main_lead_is_told()
 {
    let w = world();
    w.put(&[old(&w, 20, &json!({}))]);
    let r = w.reconciler(true);
    let run = r.tick();
    assert_eq!(counts(&run).orphan, 1);
    assert_orphan_told_once(&w);
    w.advance(20);
    r.tick();
    w.advance(5);
    r.tick();
    assert_eq!(w.events().len(), 1, "once");
    assert_eq!(
        w.a("A-44")["status"],
        json!("working"),
        "only 25 of the 30 minutes"
    );
    w.advance(6);
    let run = r.tick();
    assert_eq!(counts(&run).closed, 1);
    assert_closed_as_lost(&w);
}

fn assert_orphan_told_once(w: &World) {
    let sent = w.events();
    assert_eq!(sent.len(), 1);
    assert_eq!(ids(&sent), vec!["A-44:agent-orphan".to_owned()]);
    assert_eq!(
        (sent[0].scope.as_str(), sent[0].thread.as_str()),
        ("alpha", "lead")
    );
    assert_eq!(
        sent[0].text,
        "A-44 controllers-opus (opus) is on the board but Claude has no sub-agent or background task for it: finished or dead? close it (coord agent done A-44) or tell me it is running."
    );
    let flagged = w.a("A-44");
    assert!(flagged["orphanSince"].is_string() && flagged["orphanWokeAt"].is_string());
    assert_eq!(flagged["status"], json!("working"));
}

fn assert_closed_as_lost(w: &World) {
    let lost = w.a("A-44");
    assert_eq!(lost["status"], json!("stopped"));
    assert_eq!(lost["lost"], json!(true));
    assert_eq!(
        lost["note"],
        json!("no Claude task found, closed automatically")
    );
    assert_eq!(lost["reaped"], json!(true));
    assert!(lost.get("orphanSince").is_none());
    let sent = w.events();
    assert_eq!(sent.len(), 2);
    assert_eq!(
        (
            sent[1].scope.as_str(),
            sent[1].thread.as_str(),
            sent[1].reason.as_str()
        ),
        ("portfolio", "main", "wake-fault")
    );
    assert!(sent[1].text.starts_with("alpha: the agent reconciler closed 1 record(s) as lost, no Claude task found (alpha/A-44 controllers-opus)"));
    assert!(!sent[1].text.contains('\n'));
}

#[test]
fn any_update_of_the_record_starts_the_clock_over() {
    let w = world();
    w.put(&[old(&w, 20, &json!({}))]);
    let r = w.reconciler(true);
    r.tick();
    let touched = w.at(MIN);
    w.edit_board(|b| {
        let a = &mut b["agents"]["A-44"];
        let rev = a["rev"].as_i64().unwrap() + 1;
        a["note"] = json!("alive per harness");
        a["rev"] = json!(rev);
        a["updatedAt"] = json!(touched);
    });
    w.advance(2);
    let run = r.tick();
    assert_eq!(counts(&run).pending, 1);
    assert!(w.a("A-44").get("orphanSince").is_none());
    w.advance(60);
    r.tick();
    let told = w
        .events()
        .iter()
        .filter(|e| e.reason == "agent-orphan")
        .count();
    assert_eq!(told, 2, "it is an orphan again an hour on: told again");
    assert_eq!(w.a("A-44")["status"], json!("working"));
}

#[test]
fn the_owning_thread_that_ended_falls_back_to_the_lead() {
    let w = world();
    w.put(&[old(&w, 20, &json!({ "thread": "ui" }))]);
    w.edit_board(|b| b["threads"]["ui"]["status"] = json!("done"));
    w.reconciler(true).tick();
    assert_eq!(w.events()[0].thread, "lead");
}

#[test]
fn a_thread_with_no_session_id_or_one_claude_has_no_folder_for_is_unknown_never_an_orphan() {
    let w = world();
    w.put(&[
        old(&w, 500, &json!({ "thread": "ui" })),
        old(&w, 500, &json!({ "id": "A-45", "thread": "lead" })),
    ]);
    w.edit_board(|b| {
        b["threads"]["ui"]
            .as_object_mut()
            .unwrap()
            .remove("sessionId");
        b["threads"]["lead"]["sessionId"] = json!("sess-9999-zzzz");
    });
    let r = w.reconciler(true);
    let run = r.tick();
    assert_eq!(counts(&run).unknown, 2);
    assert_eq!(w.events(), vec![]);
    w.advance(600);
    r.tick();
    assert_eq!(w.a("A-44")["status"], json!("working"));
    assert_eq!(w.a("A-45")["status"], json!("working"));
}

#[test]
fn a_restart_does_not_send_the_wake_again_and_waits_a_full_period_before_closing() {
    let w = world();
    w.put(&[old(&w, 20, &json!({}))]);
    w.reconciler(true).tick();
    assert_eq!(w.events().len(), 1);
    w.advance(40);
    let again = w.reconciler(true);
    again.tick();
    assert_eq!(
        w.events().len(),
        1,
        "the flags are on the record: nothing is sent again"
    );
    assert_eq!(
        w.a("A-44")["status"],
        json!("working"),
        "the dashboard just started"
    );
    w.advance(31);
    again.tick();
    assert_eq!(w.a("A-44")["status"], json!("stopped"));
}

#[test]
fn a_poll_long_after_the_previous_one_the_machine_slept_only_reports() {
    let w = world();
    w.put(&[old(&w, 20, &json!({}))]);
    let r = w.reconciler(true);
    r.tick();
    w.advance(4 * 60);
    let run = r.tick();
    assert!(run.skipped);
    assert_eq!(w.a("A-44")["status"], json!("working"));
    w.advance(2);
    r.tick();
    assert_eq!(w.a("A-44")["status"], json!("stopped"));
}

#[test]
fn without_the_wake_service_an_orphan_is_flagged_and_closed_on_its_own_clock() {
    let w = world();
    w.put(&[old(&w, 20, &json!({}))]);
    let r = w.reconciler(false);
    r.tick();
    assert!(w.a("A-44")["orphanSince"].is_string());
    assert!(w.a("A-44").get("orphanWokeAt").is_none());
    w.advance(31);
    r.tick();
    assert_eq!(w.a("A-44")["status"], json!("stopped"));
    assert_eq!(w.events(), vec![]);
}

#[test]
fn records_of_other_threads_are_judged_by_their_own_session_never_by_the_leads_transcripts() {
    let w = world();
    w.subagent(
        "a0000000000000111",
        Sub {
            description: "R3: build the lobby client",
            ..Sub::default()
        },
    );
    let task = json!({ "name": "lobby-r3", "task": "R3: build the lobby client", "startedAt": w.at(-2 * MIN) });
    let ui = json!({ "name": "lobby-r3", "task": "R3: build the lobby client", "thread": "ui", "startedAt": w.at(-2 * MIN) });
    w.put(&[w.agent("A-1", &ui), w.agent("A-2", &task)]);
    w.reconciler(true).tick();
    assert_eq!(w.a("A-2")["claudeId"], json!("a0000000000000111"));
    assert!(
        w.a("A-1").get("claudeId").is_none(),
        "ui has its own session, which holds nothing"
    );
}

#[test]
fn finished_archived_and_deleted_records_are_not_looked_at_and_nothing_is_written() {
    let w = world();
    w.put(&[
        w.agent("A-1", &json!({ "status": "done", "finishedAt": w.at(-3000 * MIN), "startedAt": w.at(-4000 * MIN) })),
        w.agent("A-2", &json!({ "archivedAt": w.at(0) })),
        w.agent("A-3", &json!({ "deletedAt": w.at(0) })),
    ]);
    let before = fs::read_to_string(w.boards.path("alpha")).unwrap();
    let run = w.reconciler(true).tick();
    assert_eq!(counts(&run), Counts::default());
    assert_eq!(w.events(), vec![]);
    assert_eq!(fs::read_to_string(w.boards.path("alpha")).unwrap(), before);
}

#[test]
fn a_record_the_lead_stopped_stays_stopped_while_its_transcript_is_quiet_or_stopped_growing_with_it()
 {
    let w = world();
    let file = w.subagent(
        "a0000000000000121",
        Sub {
            quiet: 20,
            ..Sub::default()
        },
    );
    let stopped = json!({ "status": "stopped", "finishedAt": w.at(-10 * MIN), "claudeId": "a0000000000000121", "transcriptFile": file });
    w.put(&[w.agent("A-1", &stopped)]);
    let r = w.reconciler(true);
    r.tick();
    assert_eq!(w.a("A-1")["status"], json!("stopped"));
    w.resume("a0000000000000121", 1);
    let finished = w.at(-MIN);
    w.edit_board(|b| b["agents"]["A-1"]["finishedAt"] = json!(finished));
    r.tick();
    assert_eq!(
        w.a("A-1")["status"],
        json!("stopped"),
        "it grew only seconds after the stop: the stop is still taking effect"
    );
}

#[test]
fn a_record_the_hook_closed_at_the_end_of_a_turn_comes_back_to_working_when_the_agent_grows_again()
{
    let w = world();
    let file = w.subagent("a0000000000000131", Sub::default());
    let closed = json!({ "source": "hook", "hookId": "a0000000000000131", "claudeId": "a0000000000000131", "transcriptFile": file, "status": "done", "finishedAt": w.at(-5 * MIN), "result": "first answer" });
    w.put(&[w.agent("A-1", &closed)]);
    let run = w.reconciler(true).tick();
    let a = w.a("A-1");
    assert_eq!(a["status"], json!("working"));
    assert_eq!(a["finishedAt"], json!(null));
    assert_eq!(a["note"], json!("resumed, reopened automatically"));
    assert_eq!(counts(&run).reopened, 1);
}

#[test]
fn a_stopped_record_whose_agent_keeps_growing_minutes_after_is_reopened_a_lost_or_duplicate_one_never()
 {
    let w = world();
    let file = w.subagent("a0000000000000151", Sub::default());
    let gone = w.subagent("a0000000000000152", Sub::default());
    let stopped = |id: &str, file: &str| json!({ "status": "stopped", "finishedAt": w.at(-10 * MIN), "claudeId": id, "transcriptFile": file });
    let mut second = stopped("a0000000000000152", &gone);
    second["lost"] = json!(true);
    w.put(&[
        w.agent("A-1", &stopped("a0000000000000151", &file)),
        w.agent("A-2", &second),
    ]);
    w.reconciler(true).tick();
    assert_eq!(w.a("A-1")["status"], json!("working"));
    assert_eq!(w.a("A-2")["status"], json!("stopped"));
}

#[test]
fn a_record_closed_a_moment_ago_whose_transcript_did_not_grow_stays_closed() {
    let w = world();
    let file = w.subagent(
        "a0000000000000141",
        Sub {
            quiet: 5,
            ended: true,
            end_turn: true,
            ..Sub::default()
        },
    );
    let closed = json!({ "source": "hook", "hookId": "a0000000000000141", "claudeId": "a0000000000000141", "transcriptFile": file, "status": "done", "finishedAt": w.at(-5 * MIN + 1000) });
    w.put(&[w.agent("A-1", &closed)]);
    w.reconciler(true).tick();
    assert_eq!(w.a("A-1")["status"], json!("done"));
}

// The idle reaper closed it from a quiet transcript while the agent was in a long tool call (a big write, a build).
fn reaped(w: &World, id: &str, file: &str, extra: &Value) -> Value {
    let mut fields = json!({
        "source": "hook", "hookId": id, "claudeId": id, "transcriptFile": file, "model": "opus",
        "status": "done", "finishedAt": w.at(-10 * MIN), "reaped": true, "closedBy": "reaper",
        "note": "no activity for 25 min, closed automatically"
    });
    for (key, value) in extra.as_object().unwrap() {
        fields[key] = value.clone();
    }
    w.agent("A-1", &fields)
}

fn worktree_with(w: &World, name: &str, age_min: i64) -> String {
    let dir = w.claude.path().join(name);
    fs::create_dir_all(dir.join("src")).unwrap();
    let file = dir.join("src").join("big.rs");
    fs::write(
        &file,
        "fn main() {}
",
    )
    .unwrap();
    w.touch(&file, age_min);
    dir.to_string_lossy().into_owned()
}

#[test]
fn a_record_the_reaper_closed_while_the_agent_was_still_writing_files_in_its_worktree_is_reopened_and_the_note_says_why()
 {
    let w = world();
    let file = w.subagent(
        "a0000000000000161",
        Sub {
            quiet: 30,
            ..Sub::default()
        },
    );
    let tree = worktree_with(&w, "wt-live", 1);
    w.put(&[reaped(
        &w,
        "a0000000000000161",
        &file,
        &json!({ "worktree": tree }),
    )]);
    let run = w.reconciler(true).tick();
    let a = w.a("A-1");
    assert_eq!(a["status"], json!("working"));
    assert_eq!(a["finishedAt"], json!(null));
    assert_eq!(
        a["note"],
        json!(
            "reopened automatically: its Claude task is still running (worktree active 60s ago) although the record was closed as idle"
        )
    );
    assert_eq!(a["reaped"], json!(null));
    assert_eq!(a["closedBy"], json!(null));
    assert_eq!(counts(&run).reopened, 1);
    assert_eq!(
        found(&run, "A-1").unwrap().detail,
        "worktree active 60s ago, after the record was closed by the reaper"
    );
}

#[test]
fn the_output_file_of_a_running_background_task_of_the_agent_brings_the_closed_record_back_too() {
    let w = world();
    let file = w.subagent(
        "a0000000000000162",
        Sub {
            quiet: 30,
            ..Sub::default()
        },
    );
    fs::create_dir_all(w.task_dir()).unwrap();
    let out = w.task_dir().join("a0000000000000162.output");
    fs::write(
        &out,
        "compiling...
",
    )
    .unwrap();
    touch(&out, w.clock.now_ms() - 30_000);
    w.put(&[reaped(&w, "a0000000000000162", &file, &json!({}))]);
    w.reconciler(true).tick();
    assert_eq!(w.a("A-1")["status"], json!("working"));
    assert_eq!(
        w.a("A-1")["note"],
        json!(
            "reopened automatically: its Claude task is still running (task active 30s ago) although the record was closed as idle"
        )
    );
}

#[test]
fn a_record_the_reaper_closed_whose_worktree_is_quiet_too_or_whose_turn_really_ended_stays_closed()
{
    let w = world();
    let quiet = w.subagent(
        "a0000000000000163",
        Sub {
            quiet: 30,
            ..Sub::default()
        },
    );
    let ended = w.subagent(
        "a0000000000000164",
        Sub {
            quiet: 30,
            ended: true,
            end_turn: true,
            ..Sub::default()
        },
    );
    let old_tree = worktree_with(&w, "wt-old", 30);
    let fresh_tree = worktree_with(&w, "wt-fresh", 1);
    let mut second = reaped(
        &w,
        "a0000000000000164",
        &ended,
        &json!({ "worktree": fresh_tree }),
    );
    second["id"] = json!("A-2");
    w.put(&[
        reaped(
            &w,
            "a0000000000000163",
            &quiet,
            &json!({ "worktree": old_tree }),
        ),
        second,
    ]);
    w.reconciler(true).tick();
    assert_eq!(w.a("A-1")["status"], json!("done"));
    assert_eq!(w.a("A-2")["status"], json!("done"));
}
