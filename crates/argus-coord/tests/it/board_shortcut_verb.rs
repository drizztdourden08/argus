//! Ports the verb cases of `board.shortcut.test.mjs`: `coord board shortcut` creates
//! `.claude/launch.json` with one configuration (a url on the board alias host, no command),
//! merges and keeps every other configuration and top-level key, replaces itself by name, names a
//! thread's shortcut after the thread, gives the Overlord two rows, and refuses (touching nothing)
//! a `launch.json` that is not valid JSON, a thread id a host name cannot carry, an unknown
//! thread and a thread with no folder.

use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::thread_support::Cli;
use serde_json::{Value, json};

struct Fixture {
    cli: Cli,
    lead_root: PathBuf,
    work_root: PathBuf,
}

impl Deref for Fixture {
    type Target = Cli;

    fn deref(&self) -> &Cli {
        &self.cli
    }
}

impl Fixture {
    fn new() -> Self {
        let cli = Cli::new("", &[]);
        let (lead_root, work_root) = (cli.home.folder("lead"), cli.home.folder("work"));
        let fx = Fixture {
            cli,
            lead_root,
            work_root,
        };
        fx.set_port(51999);
        fx.ok(&["board", "init", "--project", "alpha", "--name", "Alpha"]);
        fx.register("coordinator", &["--role", "coordinator"], &fx.lead_root);
        fx.register("worker", &["--subject", "work"], &fx.work_root);
        fx.register("dot.x", &["--subject", "dots"], &fx.work_root);
        fx.register(
            "ghost",
            &["--subject", "gone"],
            &fx.home.path().join("does-not-exist"),
        );
        fx
    }

    fn register(&self, name: &str, extra: &[&str], root: &Path) {
        let root = root.to_string_lossy().into_owned();
        let mut words = vec![
            "thread",
            "register",
            name,
            "--project",
            "alpha",
            "--root",
            &root,
        ];
        words.extend(extra);
        self.ok(&words);
    }

    fn set_port(&self, port: u16) {
        let file = self.home.dashboard_home().join("config.json");
        fs::write(file, json!({ "port": port }).to_string()).unwrap();
    }
}

fn launch(root: &Path) -> PathBuf {
    root.join(".claude").join("launch.json")
}

fn read(root: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(launch(root)).unwrap()).unwrap()
}

const A: [&str; 2] = ["--project", "alpha"];

fn with<'a>(words: &[&'a str], tail: &[&'a str]) -> Vec<&'a str> {
    let mut all = words.to_vec();
    all.extend(tail);
    all
}

#[test]
fn the_lead_gets_one_configuration_on_the_project_alias_and_no_command() {
    let fx = Fixture::new();
    assert!(!launch(&fx.lead_root).exists());
    let ran = fx.ok(&with(&["board", "shortcut", "--as", "coordinator"], &A));
    assert_eq!(
        read(&fx.lead_root),
        json!({"version": "0.0.1", "configurations": [{"name": "Alpha board", "port": 51999, "url": "http://alpha.localhost:51999"}]})
    );
    let text = fs::read_to_string(launch(&fx.lead_root)).unwrap();
    assert!(text.starts_with(
        "{\n  \"version\": \"0.0.1\",\n  \"configurations\": [\n    {\n      \"name\""
    ));
    assert!(text.ends_with("}\n"));
    assert!(ran.out.contains("wrote "), "{}", ran.out);
    assert!(
        ran.out.contains(
            "{\"name\":\"Alpha board\",\"port\":51999,\"url\":\"http://alpha.localhost:51999\"}"
        ),
        "{}",
        ran.out
    );
    assert!(ran.out.contains("In the Browser pane, a new tab now lists 'Alpha board': Play opens the board without starting anything."), "{}", ran.out);
}

#[test]
fn the_same_call_again_changes_nothing() {
    let fx = Fixture::new();
    fx.ok(&with(&["board", "shortcut", "--as", "coordinator"], &A));
    let before = fs::read_to_string(launch(&fx.lead_root)).unwrap();
    let again = fx.ok(&with(&["board", "shortcut", "--as", "coordinator"], &A));
    assert_eq!(fs::read_to_string(launch(&fx.lead_root)).unwrap(), before);
    assert!(
        again.out.contains("already lists 'Alpha board'"),
        "{}",
        again.out
    );
    assert!(!again.out.contains("Browser pane"));
}

#[test]
fn a_thread_keeps_every_other_row_and_key_and_replaces_itself_in_place() {
    let fx = Fixture::new();
    let mine = json!({"name": "Dev server", "port": 3000, "runtimeExecutable": "npm", "runtimeArgs": ["run", "dev"]});
    let other = json!({"name": "Docs", "port": 4000, "url": "http://docs.localhost:4000"});
    fs::create_dir_all(fx.work_root.join(".claude")).unwrap();
    let before =
        json!({"version": "0.0.1", "note": {"keep": [1, 2]}, "configurations": [mine, other]});
    fs::write(
        launch(&fx.work_root),
        serde_json::to_string_pretty(&before).unwrap(),
    )
    .unwrap();
    let ran = fx.ok(&with(&["board", "shortcut", "--thread", "worker"], &A));
    let row = json!({"name": "Alpha board: worker", "port": 51999, "url": "http://worker.alpha.localhost:51999"});
    assert_eq!(
        read(&fx.work_root),
        json!({"version": "0.0.1", "note": {"keep": [1, 2]}, "configurations": [mine, other, row]})
    );
    assert!(
        ran.out.contains("lists 'Alpha board: worker'"),
        "{}",
        ran.out
    );
    assert!(
        !launch(&fx.lead_root).exists(),
        "another checkout is untouched"
    );
    fx.set_port(52111);
    fx.ok(&with(&["board", "shortcut", "--thread", "worker"], &A));
    let list = read(&fx.work_root)["configurations"].clone();
    let names = json!(
        list.as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(names, json!(["Dev server", "Docs", "Alpha board: worker"]));
    assert_eq!(list[2]["port"], 52111);
}

#[test]
fn without_thread_the_caller_is_the_thread_as_names_and_root_decides_the_folder() {
    let fx = Fixture::new();
    let other = fx.home.folder("other");
    let root = other.to_string_lossy().into_owned();
    fx.ok(&with(
        &["board", "shortcut", "--as", "worker", "--root", &root],
        &A,
    ));
    assert_eq!(
        read(&other)["configurations"][0]["url"],
        "http://worker.alpha.localhost:51999"
    );
    fx.refused(
        &with(&["board", "shortcut"], &A),
        "no thread registered for this session",
    );
}

#[test]
fn refusals_touch_nothing() {
    let fx = Fixture::new();
    let broken = fx.home.folder("broken");
    fs::create_dir_all(broken.join(".claude")).unwrap();
    let root = broken.to_string_lossy().into_owned();
    let words = with(
        &["board", "shortcut", "--thread", "worker", "--root", &root],
        &A,
    );
    for (content, text) in [
        (
            "{ \"version\": \"0.0.1\", \"configurations\": [ oops ",
            "not valid JSON",
        ),
        ("[1, 2]", "not a JSON object"),
        ("{\"configurations\": {\"a\": 1}}", "not a list"),
    ] {
        fs::write(launch(&broken), content).unwrap();
        fx.refused(&words, text);
        assert_eq!(fs::read_to_string(launch(&broken)).unwrap(), content);
    }
    fx.refused(
        &with(&["board", "shortcut", "--thread", "dot.x"], &A),
        "cannot be a host name",
    );
    fx.refused(
        &with(&["board", "shortcut", "--thread", "nobody"], &A),
        "no thread named \"nobody\"",
    );
    fx.refused(
        &with(&["board", "shortcut", "--thread", "ghost"], &A),
        "no project folder",
    );
}

#[test]
fn the_overlord_gets_two_rows_and_other_projects_one() {
    let fx = Fixture::new();
    let root = fx.home.folder("overlord");
    let path = root.to_string_lossy().into_owned();
    let p = ["--project", "portfolio"];
    fx.ok(&with(&["board", "init", "--name", "Portfolio"], &p));
    fx.ok(&with(
        &[
            "thread",
            "register",
            "coordinator",
            "--role",
            "coordinator",
            "--root",
            &path,
        ],
        &p,
    ));
    let ran = fx.ok(&with(&["board", "shortcut", "--as", "coordinator"], &p));
    let rows = |port: u16| {
        json!([
            {"name": "Argus", "port": port, "url": format!("http://argus.localhost:{port}")},
            {"name": "Overlord", "port": port, "url": format!("http://overlord.localhost:{port}")}
        ])
    };
    assert_eq!(read(&root)["configurations"], rows(51999));
    assert!(
        ran.out.contains(
            "a new tab now lists 'Argus' and 'Overlord': Play opens them without starting anything."
        ),
        "{}",
        ran.out
    );
    let again = fx.ok(&with(&["board", "shortcut", "--as", "coordinator"], &p));
    assert!(
        again.out.contains("already lists 'Argus' and 'Overlord'"),
        "{}",
        again.out
    );
    let mut json = read(&root);
    json["configurations"] = json!([json["configurations"][0].clone()]);
    fs::write(launch(&root), json.to_string()).unwrap();
    let one = fx.ok(&with(&["board", "shortcut", "--as", "coordinator"], &p));
    assert!(
        one.out.contains("lists 'Overlord': Play opens the board"),
        "{}",
        one.out
    );
    assert!(!one.out.contains("\"name\":\"Argus\""));
}

#[test]
fn a_reserved_key_gets_no_shortcut_and_the_argus_project_is_argus_app() {
    let fx = Fixture::new();
    let root = fx.home.folder("reserved");
    let path = root.to_string_lossy().into_owned();
    let o = ["--project", "overlord"];
    fx.ok(&with(&["board", "init", "--name", "overlord"], &o));
    fx.ok(&with(
        &[
            "thread",
            "register",
            "coordinator",
            "--role",
            "coordinator",
            "--root",
            &path,
        ],
        &o,
    ));
    fx.refused(
        &with(&["board", "shortcut", "--as", "coordinator"], &o),
        "reserved host name",
    );
    assert!(!launch(&root).exists());
    let app = fx.home.folder("app");
    let app_path = app.to_string_lossy().into_owned();
    let a = ["--project", "argus"];
    fx.ok(&with(&["board", "init", "--name", "Argus"], &a));
    fx.ok(&with(
        &[
            "thread",
            "register",
            "coordinator",
            "--role",
            "coordinator",
            "--root",
            &app_path,
        ],
        &a,
    ));
    fx.ok(&with(&["board", "shortcut", "--as", "coordinator"], &a));
    assert_eq!(
        read(&app)["configurations"][0]["url"],
        "http://argus-app.localhost:51999"
    );
}
