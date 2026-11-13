//! K1: the shortcut rows and the launch.json merge.

// test-tag: critical

use std::fs;
use std::path::Path;

use crate::common::Home;
use argus_coord::shortcut::{
    alias_url, is_overlord, launch_file, needs_shortcut, read_launch, shortcut_config,
    shortcut_configs, write_shortcuts,
};
use serde_json::{Value, json};

/// A home with no identity table: the folder names stand for the roles.
const NO_HOME: &str = "no-such-home";

fn read(root: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(launch_file(root)).unwrap()).unwrap()
}

fn row(name: &str, port: u16, url: &str) -> Value {
    json!({ "name": name, "port": port, "url": url })
}

#[test]
fn the_lead_is_the_project_alias_and_a_thread_is_thread_dot_project() {
    assert_eq!(
        alias_url("alpha", "coordinator", 51999).unwrap(),
        "http://alpha.localhost:51999"
    );
    assert_eq!(
        alias_url("alpha", "worker", 51999).unwrap(),
        "http://worker.alpha.localhost:51999"
    );
    let lead = shortcut_config("Alpha", "alpha", "coordinator", 51999).unwrap();
    assert_eq!(
        lead,
        row("Alpha board", 51999, "http://alpha.localhost:51999")
    );
    let thread = shortcut_config("Alpha", "alpha", "worker", 51999).unwrap();
    assert_eq!(
        thread,
        row(
            "Alpha board: worker",
            51999,
            "http://worker.alpha.localhost:51999"
        )
    );
    assert_eq!(
        thread.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["name", "port", "url"]
    );
}

#[test]
fn a_thread_id_a_host_name_cannot_carry_gets_no_shortcut() {
    let err = alias_url("alpha", "dot.x", 1).unwrap_err().to_string();
    assert!(err.contains("cannot be a host name"), "{err}");
    assert!(alias_url("alpha", "UPPER", 1).is_err());
    assert!(alias_url("alpha", &"a".repeat(64), 1).is_err());
    assert!(alias_url("alpha", &"a".repeat(63), 1).is_ok());
}

#[test]
fn a_project_key_a_host_name_cannot_carry_gets_no_shortcut() {
    for key in ["", "-lead", "Has.Dot", "under_score", &"k".repeat(64)] {
        let err = alias_url(key, "coordinator", 1).unwrap_err().to_string();
        assert!(err.contains("cannot be a host name"), "{key}: {err}");
    }
    assert!(alias_url(&"k".repeat(63), "coordinator", 1).is_ok());
}

#[test]
fn a_reserved_project_key_gets_no_shortcut_and_the_app_project_is_argus_app() {
    let err = alias_url("overlord", "coordinator", 1)
        .unwrap_err()
        .to_string();
    assert!(err.contains("reserved host name"), "{err}");
    assert_eq!(
        alias_url("argus", "coordinator", 5).unwrap(),
        "http://argus-app.localhost:5"
    );
    assert_eq!(
        alias_url("argus", "ui", 5).unwrap(),
        "http://ui.argus-app.localhost:5"
    );
}

#[test]
fn the_overlord_gets_two_rows_and_nobody_else_does() {
    assert!(is_overlord(Path::new(NO_HOME), "portfolio", "coordinator"));
    assert!(!is_overlord(Path::new(NO_HOME), "portfolio", "helper"));
    assert!(!is_overlord(Path::new(NO_HOME), "alpha", "coordinator"));
    let rows = shortcut_configs(
        Path::new(NO_HOME),
        "Portfolio",
        "portfolio",
        "coordinator",
        51999,
    )
    .unwrap();
    assert_eq!(
        rows,
        [
            row("Argus", 51999, "http://argus.localhost:51999"),
            row("Overlord", 51999, "http://overlord.localhost:51999"),
        ]
    );
    let helper = shortcut_configs(
        Path::new(NO_HOME),
        "Portfolio",
        "portfolio",
        "helper",
        51999,
    )
    .unwrap();
    assert_eq!(
        helper,
        [row(
            "Portfolio board: helper",
            51999,
            "http://helper.portfolio.localhost:51999"
        )]
    );
}

#[test]
fn a_new_file_gets_version_and_one_row_as_two_space_json() {
    let home = Home::new();
    let root = home.folder("lead");
    let rows =
        shortcut_configs(Path::new(NO_HOME), "Alpha", "alpha", "coordinator", 51999).unwrap();
    let written = write_shortcuts(&root, &rows).unwrap();
    assert!(written.changed);
    assert_eq!(written.wrote, rows);
    assert_eq!(
        read(&root),
        json!({ "version": "0.0.1", "configurations": rows })
    );
    let text = fs::read_to_string(launch_file(&root)).unwrap();
    assert!(text.starts_with(
        "{\n  \"version\": \"0.0.1\",\n  \"configurations\": [\n    {\n      \"name\""
    ));
    assert!(text.ends_with("}\n"));
    assert!(!text.contains("runtimeExecutable") && !text.contains("runtimeArgs"));
}

#[test]
fn the_same_call_again_changes_nothing() {
    let home = Home::new();
    let root = home.folder("lead");
    let rows =
        shortcut_configs(Path::new(NO_HOME), "Alpha", "alpha", "coordinator", 51999).unwrap();
    write_shortcuts(&root, &rows).unwrap();
    let first = fs::read_to_string(launch_file(&root)).unwrap();
    let again = write_shortcuts(&root, &rows).unwrap();
    assert!(!again.changed);
    assert_eq!(again.wrote, Vec::<Value>::new());
    assert_eq!(fs::read_to_string(launch_file(&root)).unwrap(), first);
}

#[test]
fn other_rows_and_top_level_keys_are_kept_in_order_and_the_same_name_is_replaced_in_place() {
    let home = Home::new();
    let root = home.folder("work");
    let mine = json!({"name": "Dev server", "port": 3000, "runtimeExecutable": "npm", "runtimeArgs": ["run", "dev"]});
    let other = json!({"name": "Docs", "port": 4000, "url": "http://docs.localhost:4000"});
    fs::create_dir_all(root.join(".claude")).unwrap();
    let start =
        json!({"version": "0.0.1", "note": {"keep": [1, 2]}, "configurations": [mine, other]});
    fs::write(
        launch_file(&root),
        serde_json::to_string_pretty(&start).unwrap(),
    )
    .unwrap();

    let one = shortcut_configs(Path::new(NO_HOME), "Alpha", "alpha", "worker", 51999).unwrap();
    write_shortcuts(&root, &one).unwrap();
    assert_eq!(read(&root)["configurations"], json!([mine, other, one[0]]));
    assert_eq!(read(&root)["note"], json!({"keep": [1, 2]}));
    assert_eq!(
        read(&root).as_object().unwrap().keys().collect::<Vec<_>>(),
        ["version", "note", "configurations"]
    );

    let moved = shortcut_configs(Path::new(NO_HOME), "Alpha", "alpha", "worker", 52111).unwrap();
    let written = write_shortcuts(&root, &moved).unwrap();
    assert!(written.changed);
    let list = read(&root)["configurations"].clone();
    let names: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Dev server", "Docs", "Alpha board: worker"]);
    assert_eq!(list[2], moved[0]);
    assert_eq!(list[0]["runtimeExecutable"], "npm");
}

#[test]
fn with_one_overlord_row_missing_only_that_one_is_written() {
    let home = Home::new();
    let root = home.folder("overlord");
    let rows = shortcut_configs(
        Path::new(NO_HOME),
        "Portfolio",
        "portfolio",
        "coordinator",
        51999,
    )
    .unwrap();
    write_shortcuts(&root, &rows).unwrap();
    let mut json = read(&root);
    json["configurations"] = json!([rows[0]]);
    fs::write(launch_file(&root), json.to_string()).unwrap();
    let written = write_shortcuts(&root, &rows).unwrap();
    assert_eq!(written.wrote, [rows[1].clone()]);
    let names: Vec<String> = read(&root)["configurations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["Argus", "Overlord"]);
}

#[test]
fn a_launch_json_it_cannot_read_is_refused_and_never_touched() {
    let home = Home::new();
    let root = home.folder("broken");
    fs::create_dir_all(root.join(".claude")).unwrap();
    let rows = shortcut_configs(Path::new(NO_HOME), "Alpha", "alpha", "worker", 1).unwrap();
    for (text, why) in [
        (
            "{ \"version\": \"0.0.1\", \"configurations\": [ oops ",
            "not valid JSON",
        ),
        ("[1, 2]", "not a JSON object"),
        ("{\"configurations\": {\"a\": 1}}", "not a list"),
        ("{\"configurations\": null}", "not a list"),
    ] {
        fs::write(launch_file(&root), text).unwrap();
        let err = write_shortcuts(&root, &rows).unwrap_err().to_string();
        assert!(err.contains(why), "{text}: {err}");
        assert!(
            err.ends_with("fix it by hand, it was not touched."),
            "{err}"
        );
        assert_eq!(fs::read_to_string(launch_file(&root)).unwrap(), text);
    }
}

#[test]
fn a_byte_order_mark_is_stripped_before_reading() {
    let home = Home::new();
    let root = home.folder("bom");
    fs::create_dir_all(root.join(".claude")).unwrap();
    fs::write(launch_file(&root), "\u{feff}{\"configurations\": []}").unwrap();
    assert!(read_launch(&launch_file(&root)).unwrap().is_some());
    assert!(read_launch(&root.join("absent.json")).unwrap().is_none());
}

#[test]
fn a_thread_is_told_to_run_the_shortcut_until_its_url_is_listed_on_any_port() {
    let home = Home::new();
    let root = home.folder("work");
    assert!(
        needs_shortcut(Path::new(NO_HOME), &root, "alpha", "worker"),
        "no file yet"
    );
    let rows = shortcut_configs(Path::new(NO_HOME), "Alpha", "alpha", "worker", 51999).unwrap();
    write_shortcuts(&root, &rows).unwrap();
    assert!(!needs_shortcut(
        Path::new(NO_HOME),
        &root,
        "alpha",
        "worker"
    ));
    // a listed alias on another port still counts
    let moved = shortcut_configs(Path::new(NO_HOME), "Alpha", "alpha", "worker", 40000).unwrap();
    write_shortcuts(&root, &moved).unwrap();
    assert!(!needs_shortcut(
        Path::new(NO_HOME),
        &root,
        "alpha",
        "worker"
    ));
    assert!(needs_shortcut(Path::new(NO_HOME), &root, "alpha", "other"));
}

#[test]
fn the_overlord_is_told_while_either_row_is_missing() {
    let home = Home::new();
    let root = home.folder("overlord");
    assert!(needs_shortcut(
        Path::new(NO_HOME),
        &root,
        "portfolio",
        "coordinator"
    ));
    let rows = shortcut_configs(
        Path::new(NO_HOME),
        "Portfolio",
        "portfolio",
        "coordinator",
        51999,
    )
    .unwrap();
    write_shortcuts(&root, &rows[..1]).unwrap();
    assert!(needs_shortcut(
        Path::new(NO_HOME),
        &root,
        "portfolio",
        "coordinator"
    ));
    write_shortcuts(&root, &rows).unwrap();
    assert!(!needs_shortcut(
        Path::new(NO_HOME),
        &root,
        "portfolio",
        "coordinator"
    ));
}

#[test]
fn no_shortcut_is_asked_for_a_thread_that_cannot_have_one_or_a_file_that_cannot_be_read() {
    let home = Home::new();
    let root = home.folder("work");
    assert!(!needs_shortcut(Path::new(NO_HOME), &root, "alpha", "dot.x"));
    fs::create_dir_all(root.join(".claude")).unwrap();
    fs::write(launch_file(&root), "not json").unwrap();
    assert!(!needs_shortcut(
        Path::new(NO_HOME),
        &root,
        "alpha",
        "worker"
    ));
}
