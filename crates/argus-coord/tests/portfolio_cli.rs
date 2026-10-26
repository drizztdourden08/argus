//! Ports the CLI cases of `portfolio.longsend.test.mjs` and the registry verbs of `coord
//! portfolio` against temp boards: `init`, `add`, `remove`, `send` (a note, a decision, `all`, a
//! project still waiting for its board, the text limits), `tick`, `show` and the retired verbs.

mod common;
mod remote;

use std::fs;

use common::{Home, Ran, coord};
use remote::{lead_board, make_checkout, ok, read_board_at, refused};
use serde_json::{Value, json};

/// Boards `alpha` and `beta` with a lead each, and an Argus registry that names `alpha`.
fn world() -> Home {
    let home = Home::new();
    for key in ["alpha", "beta"] {
        let checkout = make_checkout(home.path(), key);
        argus_testkit::write_board(&home.coord_home(), key, &lead_board(key, &checkout));
    }
    portfolio(&home, &["init"]);
    portfolio(&home, &["add", "alpha", "--name", "Alpha"]);
    home
}

fn portfolio(home: &Home, words: &[&str]) -> Ran {
    let mut argv = vec!["portfolio"];
    argv.extend(words);
    coord(&argv, home.env(), home.path())
}

fn board(home: &Home, key: &str) -> Value {
    read_board_at(&home.coord_home(), key)
}

#[test]
fn init_says_where_the_portfolio_is_and_add_and_remove_keep_the_registry() {
    let home = Home::new();
    let init = portfolio(&home, &["init"]);
    let out = ok(&init);
    assert!(
        out.starts_with("portfolio (Argus) at ")
            && out.contains("board.json (page: http://127.0.0.1:47615/)"),
        "{out}"
    );
    assert_eq!(board(&home, "_portfolio")["name"], "Argus");
    let added = portfolio(
        &home,
        &["add", "rotp", "--name", "RotP", "--root", "checkouts/rotp"],
    );
    assert_eq!(ok(&added), "rotp listed as \"RotP\"\n");
    let record = board(&home, "_portfolio")["projects"]["rotp"].clone();
    assert_eq!(
        json!([record["key"], record["name"]]),
        json!(["rotp", "RotP"])
    );
    assert!(
        record["root"]
            .as_str()
            .is_some_and(|r| r.replace('\\', "/").ends_with("checkouts/rotp"))
    );
    ok(&portfolio(&home, &["add", "rotp", "--name", "RotP again"]));
    assert!(
        board(&home, "_portfolio")["projects"]["rotp"]["root"].is_string(),
        "the root is kept"
    );
    let gone = portfolio(&home, &["remove", "rotp"]);
    assert_eq!(
        ok(&gone),
        "rotp retired from the portfolio (its board folder is kept).\n"
    );
    assert_eq!(
        board(&home, "_portfolio")["projects"]["rotp"]["retired"],
        true
    );
    let cases: [(&[&str], &str); 5] = [
        (
            &["add", "Bad Key", "--name", "x"],
            "pass the project key (the ai-config key, e.g. rotp).",
        ),
        (&["add", "ok"], "name text is required."),
        (
            &["remove", "Bad"],
            "pass the project key to retire (e.g. foundation).",
        ),
        (
            &["frobnicate"],
            "portfolio verb must be one of init, add, remove, send, tick, show, page, url, pushed.",
        ),
        (&[], "portfolio verb must be one of"),
    ];
    for (words, text) in cases {
        refused(&portfolio(&home, words), 1, text);
    }
}

#[test]
fn without_a_portfolio_the_verbs_say_so() {
    let home = Home::new();
    refused(
        &portfolio(&home, &["show"]),
        1,
        "no portfolio yet: coord portfolio init.",
    );
    refused(
        &portfolio(&home, &["add", "x", "--name", "X"]),
        1,
        "no board for project \"_portfolio\".",
    );
}

#[test]
fn send_hands_a_note_or_a_decision_down_to_the_lead_of_a_project_or_of_all() {
    let home = world();
    let note = portfolio(&home, &["send", "alpha", "please look at the lint rule"]);
    assert_eq!(ok(&note), "S-1 followup for alpha: delivered to alpha\n");
    let message = board(&home, "alpha")["inbox"]["lead"][0].clone();
    assert_eq!(message["from"], "the Overlord");
    assert!(message["text"].as_str().is_some_and(|t| t.contains("followup from the Overlord: please look at the lint rule")));
    let all = portfolio(&home, &["send", "all", "freeze the config", "--decision"]);
    assert_eq!(ok(&all), "S-2 decision for all: delivered to alpha, beta\n");
    let hint = board(&home, "beta")["inbox"]["lead"][0]["text"].clone();
    assert!(
        hint.as_str()
            .is_some_and(|t| t.ends_with("Run coord entry pick P-1 when you start on it.")),
        "{hint}"
    );
    let stored = board(&home, "_portfolio")["entries"]["S-2"].clone();
    assert_eq!(
        json!([stored["status"], stored["deliveredTo"]]),
        json!(["delivered", ["alpha", "beta"]])
    );
    refused(
        &portfolio(&home, &["send", "nowhere", "hello"]),
        1,
        "unknown project \"nowhere\"; coord portfolio add it first.",
    );
}

#[test]
fn a_project_with_no_board_waits_and_the_tick_delivers_when_the_board_comes() {
    let home = world();
    ok(&portfolio(&home, &["add", "ghost", "--name", "Ghost"]));
    let waiting = portfolio(&home, &["send", "ghost", "anyone home"]);
    assert_eq!(
        ok(&waiting),
        "S-1 followup for ghost: delivered to none yet; waiting for a board: ghost\n"
    );
    let checkout = make_checkout(home.path(), "ghost");
    argus_testkit::write_board(&home.coord_home(), "ghost", &lead_board("ghost", &checkout));
    let tick = portfolio(&home, &["tick"]);
    let report: Value = serde_json::from_str(ok(&tick)).expect("json");
    assert!(
        report["attention"].is_array() && report["open"].is_number(),
        "{report}"
    );
    let entry = board(&home, "_portfolio")["entries"]["S-1"].clone();
    assert_eq!(
        json!([entry["status"], entry["deliveredTo"], entry["pendingFor"]]),
        json!(["delivered", ["ghost"], []])
    );
    assert_eq!(
        board(&home, "ghost")["inbox"]["lead"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    let hashes = board(&home, "_portfolio")["pushedHash"].clone();
    assert!(
        hashes["alpha"].is_string() && hashes["ghost"].is_string(),
        "{hashes}"
    );
    let again: Value = serde_json::from_str(ok(&portfolio(&home, &["tick"]))).expect("json");
    assert_eq!(
        again["attention"],
        json!([]),
        "only what is new is reported"
    );
    assert!(
        portfolio(&home, &["tick", "--dir", "x"])
            .err
            .contains("coord: --dir is ignored; the dashboard syncs the portfolio itself.")
    );
}

#[test]
fn the_cli_side_keeps_its_limits_240_characters_and_200_for_a_decision() {
    let home = world();
    let over = portfolio(&home, &["send", "alpha", &"z".repeat(241)]);
    refused(&over, 1, "241 chars, the limit is 240");
    let decision = portfolio(&home, &["send", "alpha", &"z".repeat(201), "--decision"]);
    refused(&decision, 1, "201 chars, the limit is 200");
    ok(&portfolio(&home, &["send", "alpha", &"z".repeat(240)]));
    assert_eq!(
        board(&home, "_portfolio")["entries"]
            .as_object()
            .map(serde_json::Map::len),
        Some(1),
        "nothing was stored for a refusal"
    );
}

#[test]
fn show_lists_the_cards_as_a_table_or_json_and_the_old_page_verbs_are_retired() {
    let home = world();
    let table = ok(&portfolio(&home, &["show"])).to_owned();
    assert!(
        table.starts_with("# Argus (http://127.0.0.1:47615/)\n\n"),
        "{table}"
    );
    let header: Vec<&str> = table
        .lines()
        .nth(2)
        .expect("header")
        .split_whitespace()
        .collect();
    assert_eq!(header[..4], ["project", "state", "coordinator", "threads"]);
    assert!(table.lines().any(|l| l.starts_with("Alpha ")), "{table}");
    let cards: Value =
        serde_json::from_str(ok(&portfolio(&home, &["show", "--json"]))).expect("json");
    let keys: Vec<&Value> = cards
        .as_array()
        .expect("cards")
        .iter()
        .map(|c| &c["key"])
        .collect();
    assert_eq!(keys, [&json!("alpha"), &json!("beta")]);
    for verb in ["page", "url", "pushed"] {
        let said = portfolio(&home, &[verb]);
        assert_eq!(
            ok(&said),
            "retired: the dashboard serves this page at http://127.0.0.1:47615/\n"
        );
    }
    fs::write(
        home.dashboard_home().join("config.json"),
        json!({ "port": 51999 }).to_string(),
    )
    .expect("config");
    assert!(
        ok(&portfolio(&home, &["page"])).contains(":51999/"),
        "the port comes from config.json once it ran"
    );
}
