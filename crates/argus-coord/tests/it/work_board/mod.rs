//! A seeded `alpha` board for the work verb tests: a lead, a worker and one item (I-1), in
//! a temp home, and calls of `coord` as the worker or the lead. Each test binary uses only
//! part of it.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::common::{Home, Ran, coord};
use crate::support::differ::FixtureBoard;

/// A temp home with the seeded board and the folder the calls run in.
pub struct Work {
    pub home: Home,
    pub cwd: PathBuf,
}

impl Work {
    pub fn new() -> Self {
        let home = Home::new();
        let cwd = home.folder("work");
        let board = FixtureBoard::new("alpha")
            .thread("lead", "lead", None)
            .thread("worker", "thread", None)
            .item("Login", "worker", "todo", &[]);
        let dir = home.coord_home().join("alpha");
        fs::create_dir_all(&dir).expect("board folder");
        let text = board.to_json(&cwd.to_string_lossy());
        fs::write(dir.join("board.json"), text).expect("board file");
        Work { home, cwd }
    }

    /// `coord <words> --project alpha --as <who>`.
    pub fn as_thread(&self, who: &str, words: &[&str]) -> Ran {
        let mut argv: Vec<&str> = words.to_vec();
        argv.extend(["--project", "alpha", "--as", who]);
        coord(&argv, self.home.env(), &self.cwd)
    }

    pub fn worker(&self, words: &[&str]) -> Ran {
        self.as_thread("worker", words)
    }

    /// Runs `words` as the worker, asserts it passed and gives its output.
    pub fn worker_ok(&self, words: &[&str]) -> String {
        let ran = self.worker(words);
        assert_eq!(ran.code, 0, "{words:?}\n{}\n{}", ran.out, ran.err);
        ran.out
    }

    pub fn lead(&self, words: &[&str]) -> Ran {
        self.as_thread("lead", words)
    }

    /// The board file as JSON.
    pub fn board(&self) -> Value {
        let text = fs::read_to_string(self.home.coord_home().join("alpha").join("board.json"))
            .expect("board file");
        serde_json::from_str(&text).expect("board json")
    }

    /// Adds a plain item the way an older board holds one (a bare `item add` is refused now:
    /// a task needs a parent). It is a copy of I-1 with the next id and `title`.
    pub fn seed_item(&self, title: &str) -> String {
        let mut board = self.board();
        let n = board["counters"]["I"].as_u64().unwrap_or(1) + 1;
        let id = format!("I-{n}");
        let mut item = board["items"]["I-1"].clone();
        item["id"] = Value::from(id.clone());
        item["title"] = Value::from(title);
        board["items"][id.as_str()] = item;
        board["counters"]["I"] = Value::from(n);
        self.save(&board);
        id
    }

    /// An open epic with a fixed id (`I-90`) that leaves the counter alone, for a story or a task
    /// to sit under without moving the ids a test counts on.
    pub fn seed_epic(&self) -> &'static str {
        let mut board = self.board();
        let mut item = board["items"]["I-1"].clone();
        item["id"] = Value::from("I-90");
        item["title"] = Value::from("Epic");
        item["type"] = Value::from("epic");
        board["items"]["I-90"] = item;
        self.save(&board);
        "I-90"
    }

    /// Writes a changed board back, as `store.mjs` does.
    pub fn save(&self, board: &Value) {
        let text = serde_json::to_string_pretty(board).expect("board json");
        let file = self.home.coord_home().join("alpha").join("board.json");
        fs::write(
            file,
            format!(
                "{text}
"
            ),
        )
        .expect("board file");
    }
}
