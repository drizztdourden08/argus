//! The sub-agent transcript watcher on temp transcript files written line by line, with a manual
//! clock and a fake doc store: offset tailing, a partial last line, `tool_use` to `tool_result`
//! activity, the last message, the model, usage sums, the stall mark, the missing-file note, and
//! the end of tailing. Ported from `agents.watcher.test.mjs`.
//! K1: transcript tailing, activity, usage, stall and the doc update rate.
// test-tag: critical

use argus_agents::fakes::{
    JsonBoards, MemoryDocs, agent_record, assistant_line, text_block as text,
    tool_result as result, tool_use as tool, user_line,
};
use argus_agents::iso::format_iso;
use argus_agents::record::Record;
use argus_agents::tool_summary::tool_summary;
use argus_agents::watcher::{LastTool, Live, Tokens};
use argus_agents::{
    AgentWatcher, AgentsConfig, Board, Clock, FileSystem, LiveDocs, ManualClock, Ports, StdFs,
    WatcherOptions,
};
use serde_json::{Value, json};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

const T0: i64 = 1_792_756_800_000;

fn at(sec: i64) -> String {
    format_iso(T0 + sec * 1000)
}

fn assistant(sec: i64, content: &Value, extra: &Value) -> String {
    assistant_line(T0 + sec * 1000, content, extra)
}

fn user(sec: i64, content: &Value) -> String {
    user_line(T0 + sec * 1000, content)
}

fn write(file: &Path, content: &str) {
    fs::write(file, content).unwrap();
}

fn append(file: &Path, content: &str) {
    let mut handle = OpenOptions::new().append(true).open(file).unwrap();
    handle.write_all(content.as_bytes()).unwrap();
}

struct Harness {
    clock: Arc<ManualClock>,
    boards: Arc<JsonBoards>,
    docs: Arc<MemoryDocs>,
    watcher: AgentWatcher,
    rec: Record,
}

struct Setup<'a> {
    dir: &'a Path,
    status: &'a str,
    file: Option<&'a Path>,
    started_at: String,
    opts: WatcherOptions,
}

impl<'a> Setup<'a> {
    fn new(dir: &'a Path, file: &'a Path) -> Self {
        Self {
            dir,
            status: "working",
            file: Some(file),
            started_at: at(0),
            opts: WatcherOptions::default(),
        }
    }

    fn build(self) -> Harness {
        let clock = Arc::new(ManualClock::at(T0 + 1000));
        let boards = Arc::new(JsonBoards::new(self.dir, &["alpha"]));
        let docs = Arc::new(MemoryDocs::default());
        let file = self
            .file
            .map_or(Value::Null, |f| json!(f.to_string_lossy()));
        let fields = json!({
            "name": "builder", "status": self.status, "startedAt": self.started_at,
            "transcriptFile": file
        });
        let rec = agent_record("A-1", T0, &fields)
            .as_object()
            .cloned()
            .unwrap();
        docs.insert(
            "alpha",
            "A-1",
            json!({ "id": "A-1", "name": "builder", "status": self.status })
                .as_object()
                .cloned()
                .unwrap(),
        );
        let dyn_clock: Arc<dyn Clock> = clock.clone();
        let dyn_fs: Arc<dyn FileSystem> = Arc::new(StdFs);
        let dyn_docs: Arc<dyn LiveDocs> = docs.clone();
        let ports = Ports::new(boards.clone(), dyn_fs, dyn_clock);
        let watcher = AgentWatcher::new(ports, dyn_docs, self.opts);
        let harness = Harness {
            clock,
            boards,
            docs,
            watcher,
            rec,
        };
        harness.push();
        harness
    }
}

impl Harness {
    fn push(&self) {
        let board = Board {
            agents: vec![self.rec.clone()],
            threads: vec![],
        };
        self.boards.set_mirror("alpha", board);
    }

    fn edit(&mut self, change: impl FnOnce(&mut Record)) {
        change(&mut self.rec);
        self.push();
    }

    fn tick(&self) {
        self.watcher.tick();
    }

    fn live(&self) -> Option<Live> {
        self.watcher.live("alpha", "A-1")
    }

    fn assert_live(&self, turns: u64, last_message: &str) {
        let live = self.live().unwrap();
        assert_eq!(live.turns, turns);
        assert_eq!(live.last_message.as_deref(), Some(last_message));
    }

    fn applied(&self) -> Vec<Record> {
        self.docs.applied()
    }
}

fn start() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("agent-1.jsonl");
    (dir, file)
}

fn make(dir: &TempDir, file: &Path) -> Harness {
    Setup::new(dir.path(), file).build()
}

#[test]
fn reads_only_the_new_bytes_and_waits_for_the_rest_of_a_partial_last_line() {
    let (dir, f) = start();
    let whole = assistant(1, &json!([text("second message")]), &json!({}));
    write(
        &f,
        &(assistant(0, &json!([text("first message")]), &json!({})) + &whole[..40]),
    );
    let m = make(&dir, &f);
    m.tick();
    m.assert_live(1, "first message");
    append(&f, &whole[40..]);
    m.tick();
    m.assert_live(2, "second message");
    m.tick();
    assert_eq!(
        m.live().unwrap().turns,
        2,
        "nothing new: nothing is counted again"
    );
    append(&f, &assistant(2, &json!([text("third")]), &json!({})));
    m.tick();
    assert_eq!(m.live().unwrap().turns, 3);
}

#[test]
fn a_garbage_line_and_a_line_without_a_role_are_skipped() {
    let (dir, f) = start();
    let summary = json!({ "type": "summary", "summary": "x" });
    let lines = format!(
        "not json\n{summary}\n{}",
        assistant(1, &json!([text("real")]), &json!({}))
    );
    write(&f, &lines);
    let m = make(&dir, &f);
    m.tick();
    m.assert_live(1, "real");
}

#[test]
fn a_line_far_too_long_is_dropped_without_being_parsed_and_the_next_one_is_read() {
    let (dir, f) = start();
    let filler = "x".repeat(1000);
    let big = format!(r#"{{"type":"user","message":{{"role":"user","content":"{filler}"}}}}"#);
    write(
        &f,
        &(assistant(0, &json!([text("before")]), &json!({})) + &big[..700]),
    );
    let mut setup = Setup::new(dir.path(), &f);
    setup.opts.max_line_bytes = 600;
    setup.opts.chunk_bytes = 64;
    let m = setup.build();
    m.tick();
    append(
        &f,
        &format!(
            "{}\n{}",
            &big[700..],
            assistant(3, &json!([text("after")]), &json!({}))
        ),
    );
    m.tick();
    assert_eq!(m.live().unwrap().last_message.as_deref(), Some("after"));
    assert_eq!(m.live().unwrap().turns, 2);
}

#[test]
fn a_file_that_shrinks_is_read_again_from_its_start() {
    let (dir, f) = start();
    write(
        &f,
        &(assistant(0, &json!([text("one")]), &json!({}))
            + &assistant(1, &json!([text("two")]), &json!({}))),
    );
    let m = make(&dir, &f);
    m.tick();
    assert_eq!(m.live().unwrap().turns, 2);
    write(&f, &assistant(2, &json!([text("fresh")]), &json!({})));
    m.tick();
    assert_eq!(m.live().unwrap().turns, 1);
    assert_eq!(m.live().unwrap().last_message.as_deref(), Some("fresh"));
}

#[test]
fn activity_follows_tool_use_tool_result_and_the_next_assistant_line() {
    let (dir, f) = start();
    write(&f, &user(0, &json!("do the work")));
    let m = make(&dir, &f);
    m.tick();
    assert_eq!(m.live().unwrap().activity.as_deref(), Some("waiting"));
    let edit = tool(
        "t1",
        "Edit",
        &json!({ "file_path": "X:/repo/dashboard/pages/board.html" }),
    );
    append(
        &f,
        &assistant(2, &json!([text("Looking at the page"), edit]), &json!({})),
    );
    m.tick();
    let live = m.live().unwrap();
    assert_eq!(live.activity.as_deref(), Some("tool"));
    assert_eq!(live.activity_since, Some(at(2)));
    let expected = LastTool {
        name: "Edit".into(),
        summary: "X:/repo/dashboard/pages/board.html".into(),
    };
    assert_eq!(live.last_tool, Some(expected));
    assert_eq!(live.tool_calls, 1);
    activity_after_the_result(&m, &f);
}

fn activity_after_the_result(m: &Harness, f: &Path) {
    append(f, &user(9, &json!([result("t1")])));
    m.tick();
    assert_eq!(m.live().unwrap().activity.as_deref(), Some("waiting"));
    assert_eq!(m.live().unwrap().activity_since, Some(at(9)));
    append(
        f,
        &assistant(12, &json!([text("  Done with the edit.  ")]), &json!({})),
    );
    m.tick();
    let live = m.live().unwrap();
    assert_eq!(live.activity.as_deref(), Some("thinking"));
    assert_eq!(live.last_message.as_deref(), Some("Done with the edit."));
    assert_eq!(live.last_activity_at, Some(at(12)));
    assert_eq!(live.turns, 2);
}

#[test]
fn parallel_tool_calls_stay_tool_until_the_last_result_is_in() {
    let (dir, f) = start();
    let calls = json!([
        tool("a", "Read", &json!({ "file_path": "a.txt" })),
        tool("b", "Grep", &json!({ "pattern": "foo" }))
    ]);
    write(
        &f,
        &(assistant(0, &calls, &json!({})) + &user(1, &json!([result("a")]))),
    );
    let m = make(&dir, &f);
    m.tick();
    let live = m.live().unwrap();
    assert_eq!(live.activity.as_deref(), Some("tool"));
    assert_eq!(live.tool_calls, 2);
    assert_eq!(live.last_tool.unwrap().name, "Grep");
    append(&f, &user(2, &json!([result("b")])));
    m.tick();
    assert_eq!(m.live().unwrap().activity.as_deref(), Some("waiting"));
}

#[test]
fn the_last_message_is_the_last_assistant_text_block_cut_to_500_characters() {
    let (dir, f) = start();
    let bash = json!([tool("t", "Bash", &json!({ "command": "ls" }))]);
    write(
        &f,
        &(assistant(0, &json!([text(&"x".repeat(900))]), &json!({}))
            + &assistant(1, &bash, &json!({}))),
    );
    let m = make(&dir, &f);
    m.tick();
    assert_eq!(m.live().unwrap().last_message.unwrap().chars().count(), 500);
    assert_eq!(m.live().unwrap().turns, 2);
}

#[test]
fn tool_summaries_are_the_command_head_the_file_path_the_pattern_or_the_agent_description() {
    let long = json!({ "command": format!("node --test {}", "a".repeat(200)) });
    assert_eq!(tool_summary("Bash", &long).chars().count(), 80);
    assert_eq!(
        tool_summary("Bash", &json!({ "command": "git\nstatus" })),
        "git status"
    );
    assert_eq!(
        tool_summary("Write", &json!({ "file_path": "/a/b.txt" })),
        "/a/b.txt"
    );
    assert_eq!(
        tool_summary("Read", &json!({ "file_path": "/a/c.txt" })),
        "/a/c.txt"
    );
    assert_eq!(
        tool_summary("Grep", &json!({ "pattern": "TODO|FIXME" })),
        "TODO|FIXME"
    );
    let agent = json!({ "description": "Check the build", "prompt": "long" });
    assert_eq!(tool_summary("Agent", &agent), "Check the build");
    assert_eq!(tool_summary("Other", &json!({})), "");
}

#[test]
fn the_model_is_the_latest_assistant_line_usage_is_summed_and_a_streamed_message_counts_once() {
    let (dir, f) = start();
    let bash = json!([tool("t", "Bash", &json!({ "command": "ls" }))]);
    let second_usage =
        json!({ "input_tokens": 10, "output_tokens": 9, "cache_read_input_tokens": 100 });
    write(
        &f,
        &(assistant(
            0,
            &json!([text("a")]),
            &json!({ "model": "claude-haiku-4-5" }),
        ) + &assistant(
            1,
            &json!([text("b")]),
            &json!({ "id": "msg_1", "model": "claude-sonnet-5-5" }),
        ) + &assistant(
            2,
            &bash,
            &json!({ "id": "msg_1", "model": "claude-sonnet-5-5", "usage": second_usage }),
        )),
    );
    let m = make(&dir, &f);
    m.tick();
    let live = m.live().unwrap();
    assert_eq!(live.model_used.as_deref(), Some("claude-sonnet-5-5"));
    assert_eq!(
        live.tokens,
        Tokens {
            input: 20,
            output: 14,
            cache_read: 200
        }
    );
    assert_eq!(live.turns, 2);
}

#[test]
fn working_with_no_new_line_for_stall_ms_is_stalled_and_a_new_line_clears_it() {
    let (dir, f) = start();
    write(&f, &assistant(0, &json!([text("hi")]), &json!({})));
    let m = make(&dir, &f);
    m.tick();
    assert!(!m.live().unwrap().stalled);
    m.clock.set(T0 + 170_000);
    m.tick();
    assert!(!m.live().unwrap().stalled);
    m.clock.set(T0 + 181_000);
    m.tick();
    assert!(m.live().unwrap().stalled);
    assert_eq!(m.applied().last().unwrap()["live"]["stalled"], json!(true));
    append(&f, &assistant(182, &json!([text("back")]), &json!({})));
    m.clock.set(T0 + 183_000);
    m.tick();
    assert!(!m.live().unwrap().stalled);
}

#[test]
fn the_threshold_comes_from_config_agents_stall_ms() {
    let (dir, f) = start();
    write(&f, &assistant(0, &json!([text("hi")]), &json!({})));
    let mut setup = Setup::new(dir.path(), &f);
    setup.opts = WatcherOptions::default()
        .configured(&AgentsConfig::from_json(&json!({ "stallMs": 30_000 })));
    let m = setup.build();
    m.clock.set(T0 + 31_000);
    m.tick();
    assert!(m.live().unwrap().stalled);
}

#[test]
fn only_a_working_agent_can_stall() {
    let (dir, f) = start();
    write(&f, &assistant(0, &json!([text("hi")]), &json!({})));
    let mut setup = Setup::new(dir.path(), &f);
    setup.status = "waiting";
    let m = setup.build();
    m.clock.set(T0 + 600_000);
    m.tick();
    assert!(!m.live().unwrap().stalled);
}

#[test]
fn a_transcript_still_missing_20_s_after_the_start_sets_the_note_once() {
    let (dir, f) = start();
    let m = make(&dir, &f);
    m.clock.set(T0 + 10_000);
    m.tick();
    assert!(m.live().is_none());
    m.clock.set(T0 + 21_000);
    m.tick();
    assert_eq!(
        m.live().unwrap().live_note.as_deref(),
        Some("no transcript found")
    );
    let count = m.applied().len();
    m.clock.set(T0 + 30_000);
    m.tick();
    m.clock.set(T0 + 40_000);
    m.tick();
    assert_eq!(
        m.applied().len(),
        count,
        "no second update for the same note"
    );
    write(&f, &assistant(41, &json!([text("started")]), &json!({})));
    m.clock.set(T0 + 42_000);
    m.tick();
    assert_eq!(m.live().unwrap().live_note, None);
    assert_eq!(m.live().unwrap().last_message.as_deref(), Some("started"));
}

#[test]
fn a_changed_doc_is_set_at_most_once_a_second_with_the_live_fields_under_live() {
    let (dir, f) = start();
    write(&f, &assistant(0, &json!([text("one")]), &json!({})));
    let m = make(&dir, &f);
    m.tick();
    assert_eq!(m.applied().len(), 1);
    assert_eq!(m.applied()[0]["live"]["lastMessage"], json!("one"));
    assert_eq!(
        m.applied()[0]["name"],
        json!("builder"),
        "the rest of the doc is kept"
    );
    append(&f, &assistant(1, &json!([text("two")]), &json!({})));
    m.tick();
    assert_eq!(m.applied().len(), 1, "same second: held back");
    m.clock.advance(1000);
    m.tick();
    assert_eq!(m.applied().len(), 2);
    assert_eq!(m.applied()[1]["live"]["lastMessage"], json!("two"));
    m.clock.advance(1000);
    m.tick();
    assert_eq!(m.applied().len(), 2, "nothing changed, nothing set");
}

#[test]
fn when_the_record_is_done_the_last_lines_are_read_once_and_the_file_is_left_alone() {
    let (dir, f) = start();
    write(
        &f,
        &assistant(0, &json!([text("working on it")]), &json!({})),
    );
    let mut m = make(&dir, &f);
    m.tick();
    append(
        &f,
        &assistant(5, &json!([text("All finished.")]), &json!({})),
    );
    let finished = at(6);
    m.edit(|rec| {
        rec.insert("status".into(), json!("done"));
        rec.insert("finishedAt".into(), json!(finished));
    });
    m.clock.advance(1000);
    m.tick();
    assert_eq!(
        m.live().unwrap().last_message.as_deref(),
        Some("All finished.")
    );
    assert_eq!(
        m.applied().last().unwrap()["live"]["lastMessage"],
        json!("All finished.")
    );
    let count = m.applied().len();
    append(&f, &assistant(7, &json!([text("ignored")]), &json!({})));
    m.clock.advance(5000);
    m.tick();
    assert_eq!(
        m.live().unwrap().last_message.as_deref(),
        Some("All finished.")
    );
    assert_eq!(m.applied().len(), count);
}

#[test]
fn an_agent_finished_within_ten_minutes_is_read_once_at_start_and_an_older_one_is_not() {
    let (dir, f) = start();
    write(&f, &assistant(0, &json!([text("old news")]), &json!({})));
    let mut setup = Setup::new(dir.path(), &f);
    setup.status = "done";
    let mut recent = setup.build();
    recent.edit(|rec| {
        rec.insert("finishedAt".into(), json!(at(1)));
    });
    recent.tick();
    assert_eq!(
        recent.live().unwrap().last_message.as_deref(),
        Some("old news")
    );
    let mut setup = Setup::new(dir.path(), &f);
    setup.status = "done";
    let mut old = setup.build();
    old.edit(|rec| {
        rec.insert("finishedAt".into(), json!(at(1)));
    });
    old.clock.set(T0 + 11 * 60_000);
    old.tick();
    assert!(old.live().is_none());
}

#[test]
fn an_agent_with_no_transcript_file_is_not_tracked_and_a_removed_record_drops_its_state() {
    let (dir, f) = start();
    write(&f, &assistant(0, &json!([text("hi")]), &json!({})));
    let mut setup = Setup::new(dir.path(), &f);
    setup.file = None;
    let none = setup.build();
    none.tick();
    assert_eq!(none.watcher.size(), 0);
    let mut m = make(&dir, &f);
    m.tick();
    assert_eq!(m.watcher.size(), 1);
    m.edit(|rec| {
        rec.shift_remove("transcriptFile");
    });
    m.tick();
    assert_eq!(m.watcher.size(), 0);
}

#[test]
fn overlay_puts_the_live_state_on_the_exported_agents_doc_and_leaves_other_docs_alone() {
    let (dir, f) = start();
    write(&f, &assistant(0, &json!([text("overlaid")]), &json!({})));
    let m = make(&dir, &f);
    m.tick();
    let mut tracked = json!({ "id": "A-1", "status": "working" })
        .as_object()
        .cloned()
        .unwrap();
    let mut unknown = json!({ "id": "A-9" }).as_object().cloned().unwrap();
    assert!(m.watcher.overlay_doc("alpha", "A-1", &mut tracked));
    assert!(!m.watcher.overlay_doc("alpha", "A-9", &mut unknown));
    assert_eq!(tracked["live"]["lastMessage"], json!("overlaid"));
    assert!(!unknown.contains_key("live"));
}
