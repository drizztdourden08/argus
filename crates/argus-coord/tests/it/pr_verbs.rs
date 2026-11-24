//! K2: the lines and records of coord pr open, list and close.

// test-tag: critical

use crate::common::{Home, Ran, coord};
use crate::remote::{alpha_with_worker, lines, ok, read_board_at, refused};
use argus_coord::Env;
use serde_json::{Value, json};

const AS: [&str; 2] = ["--as", "lead"];
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

/// A home with the board `alpha`, and `coord <words> --project alpha --as lead` in it.
pub struct Local {
    pub home: Home,
}

impl Local {
    pub fn new() -> Local {
        let home = Home::new();
        let mut board = alpha_with_worker(home.path());
        board["items"] = json!({ "I-1": { "id": "I-1", "title": "An item", "thread": "worker", "status": "todo" } });
        home.write_board("alpha", &board);
        Local { home }
    }

    pub fn go(&self, words: &[&str]) -> Ran {
        self.go_in(words, self.home.env())
    }

    pub fn go_in(&self, words: &[&str], env: Env) -> Ran {
        let argv: Vec<&str> = words
            .iter()
            .chain(&["--project", "alpha"])
            .chain(&AS)
            .copied()
            .collect();
        coord(&argv, env, self.home.path())
    }

    pub fn board(&self) -> Value {
        read_board_at(&self.home.coord_home(), "alpha")
    }

    pub fn open(&self, title: &str, head: &str) -> Ran {
        self.go(&["pr", "open", title, "--head", head, "--commit", SHA])
    }
}

#[test]
fn open_records_the_pr_and_list_shows_it_as_a_table_and_as_json() {
    let local = Local::new();
    let said = local.go(&[
        "pr", "open", "Add x", "--head", "feat/x", "--commit", SHA, "--item", "I-1", "--body",
        "why",
    ]);
    assert_eq!(ok(&said).trim(), "PR-1 opened: feat/x into main at 0123456");
    let pr = &local.board()["prs"]["PR-1"];
    assert_eq!(
        (pr["status"].as_str(), pr["base"].as_str()),
        (Some("open"), Some("main"))
    );
    assert_eq!(
        (pr["headCommit"].as_str(), pr["item"].as_str()),
        (Some(SHA), Some("I-1"))
    );
    assert_eq!(pr["replay"]["state"], "pending");
    let table = local.go(&["pr", "list"]);
    let table = ok(&table);
    let shown = lines(table);
    assert!(shown[0].starts_with("pr"), "{table}");
    assert!(
        shown[1].contains("PR-1") && shown[1].contains("feat/x -> main"),
        "{table}"
    );
    assert!(
        shown[1].contains("0123456") && shown[1].ends_with("Add x"),
        "{table}"
    );
    let json = local.go(&["pr", "list", "--json"]);
    let list: Value = serde_json::from_str(ok(&json)).expect("json");
    let first = &list[0];
    assert_eq!(first["id"], "PR-1");
    for field in ["head", "base", "headCommit", "status", "checks"] {
        assert!(first.get(field).is_some(), "{field} missing in {first}");
    }
    assert_eq!(first["checks"], json!([]));
    assert_eq!(
        ok(&local.go(&["pr", "list", "--status", "merged"])).trim(),
        "no pull requests."
    );
}

#[test]
fn open_refuses_what_is_missing_or_wrong() {
    let local = Local::new();
    refused(
        &local.go(&["pr", "open", "--head", "feat/x"]),
        1,
        "give the title",
    );
    refused(
        &local.go(&["pr", "open", "T"]),
        1,
        "--head <branch> is required.",
    );
    refused(
        &local.open("T", "main"),
        1,
        "head and base are the same branch.",
    );
    refused(
        &local.open("T", "bad name"),
        1,
        "head must be a branch name",
    );
    ok(&local.open("T", "feat/x"));
    refused(&local.open("T again", "feat/x"), 1, "PR-1 is already open");
    refused(
        &local.go(&["pr", "frobnicate"]),
        1,
        "pr verb must be one of open, list, merge, close.",
    );
    refused(
        &local.go(&["pr", "close", "7"]),
        1,
        "give the pull request id",
    );
    refused(
        &local.go(&["pr", "close", "PR-7"]),
        1,
        "no pull request \"PR-7\".",
    );
}

#[test]
fn close_ends_a_pr_and_a_closed_pr_neither_closes_nor_merges() {
    let local = Local::new();
    ok(&local.open("T", "feat/x"));
    assert_eq!(
        ok(&local.go(&["pr", "close", "PR-1"])).trim(),
        "PR-1 closed"
    );
    assert_eq!(local.board()["prs"]["PR-1"]["status"], "closed");
    refused(&local.go(&["pr", "close", "PR-1"]), 1, "PR-1 is closed");
    refused(
        &local.go(&["pr", "merge", "PR-1"]),
        1,
        "only an open pull request merges",
    );
}
