//! The live fields of a sub-agent reach the agents doc through the exporter's overlay (and the
//! watcher's own updates), survive a resync, and never get into the board file. The exporter is
//! played by `overlay_doc` over a record read from the board. Ported from `agents.export.test.mjs`.

use argus_agents::fakes::{
    JsonBoards, ManualClock, MemoryDocs, agent_record, assistant_line, board_json, text_block,
    thread_record, tool_result, tool_use, user_line,
};
use argus_agents::record::Record;
use argus_agents::{
    AgentWatcher, BoardStore, Clock, FileSystem, LiveDocs, Ports, StdFs, WatcherOptions,
};
use serde_json::{Value, json};
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

const T0: i64 = 1_792_756_800_000;

struct World {
    _dir: TempDir,
    transcript: PathBuf,
    boards: Arc<JsonBoards>,
    docs: Arc<MemoryDocs>,
    clock: Arc<ManualClock>,
    watcher: AgentWatcher,
}

fn world() -> World {
    let dir = tempfile::tempdir().unwrap();
    let transcript = dir.path().join("agent-A-1.jsonl");
    let reading = json!([
        text_block("Reading the form"),
        tool_use("t1", "Read", &json!({ "file_path": "form.js" }))
    ]);
    let usage = json!({ "usage": { "input_tokens": 7, "output_tokens": 3 } });
    fs::write(&transcript, assistant_line(T0, &reading, &usage)).unwrap();
    let builder = json!({ "name": "builder", "transcriptFile": transcript.to_string_lossy() });
    let checker = json!({ "name": "checker", "status": "done", "finishedAt": null });
    let board = board_json(
        &[thread_record("coordinator", "coordinator", &json!({}))],
        &[
            agent_record("A-1", T0, &builder),
            agent_record("A-2", T0, &checker),
        ],
    );
    let boards = Arc::new(JsonBoards::new(dir.path(), &["alpha"]));
    boards.save("alpha", &board);
    let docs = Arc::new(MemoryDocs::default());
    for id in ["A-1", "A-2"] {
        docs.insert(
            "alpha",
            id,
            board["agents"][id].as_object().cloned().unwrap(),
        );
    }
    let clock = Arc::new(ManualClock::new(T0 + 1000));
    let (dyn_clock, dyn_docs): (Arc<dyn Clock>, Arc<dyn LiveDocs>) = (clock.clone(), docs.clone());
    let dyn_fs: Arc<dyn FileSystem> = Arc::new(StdFs);
    let ports = Ports::new(boards.clone(), dyn_fs, dyn_clock);
    let watcher = AgentWatcher::new(ports, dyn_docs, WatcherOptions::default());
    World {
        _dir: dir,
        transcript,
        boards,
        docs,
        clock,
        watcher,
    }
}

impl World {
    fn board_record(&self, id: &str) -> Record {
        let board = self.boards.snapshot("alpha").unwrap();
        board
            .agents
            .into_iter()
            .find(|a| a["id"] == json!(id))
            .unwrap()
    }

    fn exported(&self, id: &str) -> Record {
        let mut data = self.board_record(id);
        self.watcher.overlay_doc("alpha", id, &mut data);
        data
    }
}

#[test]
fn the_first_export_carries_nothing_live_until_the_watcher_has_read_the_transcript() {
    let w = world();
    assert!(!w.exported("A-1").contains_key("live"));
}

#[test]
fn a_tick_puts_the_live_fields_on_the_agents_doc_and_sets_it_once() {
    let w = world();
    w.watcher.tick();
    let sets = w.docs.applied();
    assert_eq!(sets.len(), 1);
    let doc = w.docs.doc("alpha", "A-1").unwrap();
    assert_eq!(doc["live"]["lastMessage"], json!("Reading the form"));
    assert_eq!(doc["live"]["activity"], json!("tool"));
    assert_eq!(
        doc["live"]["lastTool"],
        json!({ "name": "Read", "summary": "form.js" })
    );
    assert_eq!(doc["live"]["modelUsed"], json!("claude-sonnet-5-5"));
    assert_eq!(doc["live"]["tokens"]["input"], json!(7));
    assert_eq!(
        doc["name"],
        json!("builder"),
        "the record fields are still there"
    );
    assert!(
        !w.docs.doc("alpha", "A-2").unwrap().contains_key("live"),
        "the finished agent has no transcript"
    );
}

#[test]
fn the_board_file_keeps_only_the_path() {
    let w = world();
    w.watcher.tick();
    let raw = fs::read_to_string(w.boards.path("alpha")).unwrap();
    assert!(raw.contains("transcriptFile"));
    assert!(!raw.contains("lastMessage"));
    assert!(!raw.contains("\"live\""));
}

#[test]
fn a_resync_of_the_board_keeps_the_live_fields_and_writes_nothing() {
    let w = world();
    w.watcher.tick();
    let writes = w.docs.applied().len();
    let doc = w.exported("A-1");
    assert_eq!(doc["live"]["lastMessage"], json!("Reading the form"));
    w.clock.advance(5000);
    w.watcher.tick();
    assert_eq!(w.docs.applied().len(), writes);
}

#[test]
fn a_changed_board_record_is_exported_with_the_live_fields_on_top() {
    let w = world();
    w.watcher.tick();
    let mut board: Value = w.boards.load("alpha");
    board["agents"]["A-1"]["note"] = json!("halfway");
    w.boards.save("alpha", &board);
    let doc = w.exported("A-1");
    assert_eq!(doc["note"], json!("halfway"));
    assert_eq!(doc["live"]["lastMessage"], json!("Reading the form"));
}

#[test]
fn new_transcript_lines_show_up_in_the_doc_on_the_next_tick() {
    let w = world();
    w.watcher.tick();
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&w.transcript)
        .unwrap();
    let editing = json!([text_block("Form read, now editing")]);
    let lines = user_line(T0 + 1, &json!([tool_result("t1")]))
        + &assistant_line(T0 + 2, &editing, &json!({}));
    file.write_all(lines.as_bytes()).unwrap();
    w.clock.advance(1100);
    w.watcher.tick();
    let doc = w.docs.doc("alpha", "A-1").unwrap();
    assert_eq!(doc["live"]["lastMessage"], json!("Form read, now editing"));
    assert_eq!(doc["live"]["activity"], json!("thinking"));
    assert_eq!(doc["live"]["turns"], json!(2));
}
