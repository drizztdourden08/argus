//! The resume after an account switch: one one-off scheduled task of the Claude Desktop app that
//! fires a few minutes after Claude starts on the other account. Its prompt makes the Overlord's
//! session wake each lead that was handed over. The task is a `SKILL.md` under the Claude
//! scheduled-tasks folder plus an entry in the `scheduled-tasks.json` of the target scope; every
//! other task and field of that file stays as it is.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::now_ms;
use serde_json::{Map, Value, json};

use crate::catalog::read::write_atomic;
use crate::handover::HandoverState;
use crate::sync::lock_running;
use crate::types::Scope;

/// The id of a task starts with this; the rest is the `SW-n` of its cycle.
const ID_PREFIX: &str = "argus-resume-";
/// The folder of a task's scope holds this file.
const TASKS_FILE: &str = "scheduled-tasks.json";
/// The folder of the Desktop app that holds the chat records and task files by scope.
const SESSIONS: &str = "claude-code-sessions";
/// Where the task runs: the Overlord's folder.
pub const TASK_CWD: &str = "X:\\ai-config";
/// How long after the first message the task checks the wake listeners.
const CHECK_MINUTES: u32 = 3;
const COORD: &str = "node X:/ai-config/shared/tools/coord/coord.mjs";

/// One lead to wake after the switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeLead {
    /// The board key.
    pub board: String,
    /// The lead's thread id.
    pub thread: String,
    /// The Claude Code session of the lead's thread.
    pub session_id: String,
    /// The handover file the lead wrote, when it did.
    pub file: Option<String>,
    /// `Ready` or `CutOff`.
    pub state: HandoverState,
}

/// The scheduled task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeTask {
    /// The task id, also the name of its folder.
    pub id: String,
    /// When it fires, milliseconds since the Unix epoch.
    pub fire_at_ms: i64,
    /// The folder it runs in.
    pub cwd: String,
    /// The session told when it ran, when known.
    pub notify_session_id: Option<String>,
    /// What it does.
    pub prompt: String,
}

fn lead_line(lead: &ResumeLead) -> String {
    let file = lead
        .file
        .as_deref()
        .unwrap_or("none written, it was cut off");
    format!(
        "- board {}, thread {}, session {}, handover file: {file}\n",
        lead.board, lead.thread, lead.session_id
    )
}

fn resume_text(port: u16) -> String {
    format!(
        "Overlord, owner order: the Claude account was switched; resume exactly where you stopped. \
Read your handover file (the board's handover.md when none is named), run coord inbox, check your \
sub-agents by their real id (coord agent list) and restart the cut-off ones from the branches named \
in the handover, reopen your board tab http://127.0.0.1:{port}/p/<board>/?thread=<thread>, and carry \
on with the next steps in order, unattended. Ask the owner only the important things."
    )
}

/// The prompt of the resume task for `leads`; Argus listens on `port`.
#[must_use]
pub fn build_prompt(leads: &[ResumeLead], port: u16) -> String {
    let list: String = leads.iter().map(lead_line).collect();
    let text = resume_text(port);
    format!(
        "You act as the Overlord's resume routine on the owner's machine, on his standing order: \
Claude was just restarted on another account and these project leads must carry on from their \
handover. Do exactly this and nothing more. Never kill a process, never push, commit nothing.\n\
\n\
1. Check http://127.0.0.1:{port}/api/health answers. Check GET http://127.0.0.1:{port}/api/accounts \
shows an account marked active that is not the account being left (GET /api/accounts/cycle names \
it). If Argus does not answer or the wrong account is active, toast the owner (step 5) saying so \
and stop.\n\
2. Load mcp__ccd_session_mgmt__send_message with ToolSearch. For each lead below, send its session \
the resume text.\n\
{list}\n\
Resume text (put the lead's own board and thread in the tab address):\n\
{text}\n\
\n\
3. After {CHECK_MINUTES} minutes check each board's wake listener with GET \
http://127.0.0.1:{port}/api/p/<board>/wakes/listeners and message again any lead that did not \
start.\n\
4. Report each lead to Argus with POST http://127.0.0.1:{port}/api/accounts/handover and the JSON \
body {{\"action\":\"resumed\",\"board\":\"<board>\",\"thread\":\"<thread>\"}} for a lead that \
started, or \"not_resumed\" for one that did not start after the second message (curl.exe or \
Invoke-RestMethod, Content-Type application/json).\n\
5. Toast the owner once, saying which leads resumed and which did not: {COORD} ping \"<text>\" \
--title \"Argus\" --level ping --project portfolio --as coordinator\n\
Then finish with a one line report."
    )
}

fn tasks_path(app: &Path, scope: &Scope) -> PathBuf {
    app.join(SESSIONS)
        .join(&scope.account)
        .join(&scope.org)
        .join(TASKS_FILE)
}

fn read_tasks(path: &Path) -> Result<Value, String> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({ "scheduledTasks": [] })),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn list_of(root: &mut Value) -> Result<&mut Vec<Value>, String> {
    root.as_object_mut()
        .ok_or("scheduled-tasks.json is not an object")?
        .entry("scheduledTasks")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| "scheduledTasks is not a list".to_owned())
}

fn entry_of(task: &ResumeTask, skill: &Path) -> Value {
    let mut entry = Map::new();
    entry.insert("id".into(), json!(task.id));
    entry.insert(
        "displayName".into(),
        json!(format!(
            "Resume leads after the account switch ({})",
            task.id
        )),
    );
    entry.insert("fireAt".into(), json!(task.fire_at_ms));
    entry.insert("enabled".into(), json!(true));
    entry.insert("filePath".into(), json!(skill.display().to_string()));
    entry.insert("createdAt".into(), json!(now_ms()));
    entry.insert("cwd".into(), json!(task.cwd));
    if let Some(session) = &task.notify_session_id {
        entry.insert("notifySessionId".into(), json!(session));
    }
    Value::Object(entry)
}

fn skill_text(task: &ResumeTask) -> String {
    format!(
        "---\nname: {}\ndescription: Wake the project leads that were handed over before the Claude account switch\n---\n\n{}\n",
        task.id, task.prompt
    )
}

fn save_tasks(path: &Path, root: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(root).map_err(|e| e.to_string())?;
    write_atomic(path, text.as_bytes())
}

/// The app reads and rewrites `scheduled-tasks.json` while it runs, so it is only written with
/// Claude closed.
fn ensure_closed(app: &Path) -> Result<(), String> {
    if lock_running(app) {
        Err(
            "Claude Desktop is running: the scheduled tasks are only changed while it is closed"
                .into(),
        )
    } else {
        Ok(())
    }
}

/// Copies the current file to `scheduled-tasks.json.bak-<sw-n>` beside it, when there is one.
fn backup_tasks(path: &Path, task_id: &str) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let cycle = task_id.strip_prefix(ID_PREFIX).unwrap_or(task_id);
    let copy = path.with_file_name(format!("{TASKS_FILE}.bak-{cycle}"));
    fs::copy(path, &copy)
        .map(|_| ())
        .map_err(|e| format!("{}: {e}", copy.display()))
}

/// Writes the task's `SKILL.md` under `skill_root` and adds or replaces its entry in the scope's
/// `scheduled-tasks.json` (created when missing), after a backup copy of that file. Every other
/// task and field stays.
///
/// # Errors
/// Claude Desktop is running, or a file cannot be read, parsed or written.
pub fn write_task(
    app: &Path,
    scope: &Scope,
    task: &ResumeTask,
    skill_root: &Path,
) -> Result<(), String> {
    ensure_closed(app)?;
    let path = tasks_path(app, scope);
    let mut root = read_tasks(&path)?;
    backup_tasks(&path, &task.id)?;
    let skill = skill_root.join(&task.id).join("SKILL.md");
    write_atomic(&skill, skill_text(task).as_bytes())?;
    let list = list_of(&mut root)?;
    let entry = entry_of(task, &skill);
    match list.iter().position(|t| t["id"] == json!(task.id)) {
        Some(at) => list[at] = entry,
        None => list.push(entry),
    }
    save_tasks(&path, &root)
}

/// Removes the task's entry and its skill folder. A missing task is not an error.
///
/// # Errors
/// Claude Desktop is running, or a file cannot be read, parsed or written.
pub fn remove_task(app: &Path, scope: &Scope, id: &str, skill_root: &Path) -> Result<(), String> {
    ensure_closed(app)?;
    let path = tasks_path(app, scope);
    if path.exists() {
        let mut root = read_tasks(&path)?;
        let list = list_of(&mut root)?;
        let before = list.len();
        list.retain(|t| t["id"] != json!(id));
        if list.len() != before {
            save_tasks(&path, &root)?;
        }
    }
    let dir = skill_root.join(id);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    Ok(())
}

/// Whether the app ran the task: its entry has a `lastRunAt`.
///
/// # Errors
/// The file cannot be read or parsed.
pub fn has_run(app: &Path, scope: &Scope, id: &str) -> Result<bool, String> {
    let root = read_tasks(&tasks_path(app, scope))?;
    Ok(root["scheduledTasks"]
        .as_array()
        .and_then(|l| l.iter().find(|t| t["id"] == json!(id)))
        .is_some_and(|t| t.get("lastRunAt").is_some_and(|v| !v.is_null())))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::{Value, json};

    use super::{ResumeLead, ResumeTask, build_prompt, has_run, remove_task, write_task};
    use crate::handover::HandoverState;
    use crate::types::Scope;

    const ID: &str = "argus-resume-sw-3";

    fn scope() -> Scope {
        Scope {
            account: "acct".into(),
            org: "org".into(),
        }
    }

    fn task() -> ResumeTask {
        ResumeTask {
            id: ID.into(),
            fire_at_ms: 1_793_178_180_000,
            cwd: "X:\\ai-config".into(),
            notify_session_id: Some("local_overlord".into()),
            prompt: "do the thing".into(),
        }
    }

    fn lead(board: &str, file: Option<&str>) -> ResumeLead {
        ResumeLead {
            board: board.into(),
            thread: "coordinator".into(),
            session_id: format!("local_{board}"),
            file: file.map(str::to_owned),
            state: HandoverState::Ready,
        }
    }

    fn tasks_file(app: &Path) -> PathBuf {
        app.join("claude-code-sessions/acct/org/scheduled-tasks.json")
    }

    /// A copy of the real file shape: a cron task, a finished one-off and the extra keys.
    fn fixture(app: &Path) -> PathBuf {
        let path = tasks_file(app);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = json!({
            "scheduledTasks": [
                {"id": "sweep", "displayName": "Sweep", "cronExpression": "0 7 * * *",
                 "enabled": true, "filePath": "C:\\x\\SKILL.md", "createdAt": 1,
                 "lastRunAt": "2026-11-11T14:12:49.422Z", "cwd": "X:\\mail",
                 "approvedPermissions": [{"toolName": "Read"}]},
                {"id": "old", "fireAt": 5, "enabled": false,
                 "lastRunAt": "2026-10-28T09:03:27.728Z"}
            ],
            "recordedSkips": {},
            "sundayAliasBoundaryStamped": true
        });
        fs::write(&path, text.to_string()).unwrap();
        path
    }

    fn read(path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn the_prompt_names_every_lead_and_the_port_and_has_no_dashes() {
        let prompt = build_prompt(&[lead("rotp", Some("h.md")), lead("brock", None)], 47615);
        assert!(prompt.contains("http://127.0.0.1:47615/api/health"));
        assert!(prompt.contains("board rotp, thread coordinator, session local_rotp"));
        assert!(prompt.contains("handover file: h.md"));
        assert!(prompt.contains("none written, it was cut off"));
        assert!(prompt.contains("\"action\":\"resumed\""));
        assert!(prompt.contains("wakes/listeners"));
        assert!(!prompt.contains('\u{2014}') && !prompt.contains('\u{2013}'));
    }

    /// A temp folder with the app folder holding the fixture file, and a skills folder.
    fn world() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let (app, skills) = (temp.path().join("app"), temp.path().join("skills"));
        let path = fixture(&app);
        (temp, app, skills, path)
    }

    #[test]
    fn a_task_is_added_beside_the_others_without_touching_them() {
        let (_temp, app, skills, path) = world();
        write_task(&app, &scope(), &task(), &skills).unwrap();
        let root = read(&path);
        let list = root["scheduledTasks"].as_array().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(
            list[0]["approvedPermissions"],
            json!([{"toolName": "Read"}])
        );
        assert_eq!(list[0]["cronExpression"], json!("0 7 * * *"));
        assert_eq!(root["sundayAliasBoundaryStamped"], json!(true));
        let mine = &list[2];
        assert_eq!(mine["fireAt"], json!(1_793_178_180_000_i64));
        assert_eq!(mine["enabled"], json!(true));
        assert_eq!(mine["cwd"], json!("X:\\ai-config"));
        assert_eq!(mine["notifySessionId"], json!("local_overlord"));
        assert!(mine.get("cronExpression").is_none());
        let skill = fs::read_to_string(skills.join(ID).join("SKILL.md")).unwrap();
        assert!(skill.starts_with("---\nname: argus-resume-sw-3\ndescription: "));
        assert!(skill.contains("do the thing"));
    }

    #[test]
    fn writing_again_replaces_the_entry_and_a_missing_file_is_created() {
        let temp = tempfile::tempdir().unwrap();
        let (app, skills) = (temp.path().join("app"), temp.path().join("skills"));
        let mut t = task();
        t.notify_session_id = None;
        write_task(&app, &scope(), &t, &skills).unwrap();
        t.fire_at_ms = 7;
        write_task(&app, &scope(), &t, &skills).unwrap();
        let root = read(&tasks_file(&app));
        let list = root["scheduledTasks"].as_array().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["fireAt"], json!(7));
        assert!(list[0].get("notifySessionId").is_none());
    }

    #[test]
    fn removing_drops_the_entry_and_the_folder_only() {
        let (_temp, app, skills, path) = world();
        write_task(&app, &scope(), &task(), &skills).unwrap();
        remove_task(&app, &scope(), ID, &skills).unwrap();
        let ids: Vec<Value> = read(&path)["scheduledTasks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["id"].clone())
            .collect();
        assert_eq!(ids, vec![json!("sweep"), json!("old")]);
        assert!(!skills.join(ID).exists());
        remove_task(&app, &scope(), ID, &skills).unwrap();
    }

    #[test]
    fn a_task_has_run_once_its_last_run_is_set() {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let path = fixture(&app);
        assert!(has_run(&app, &scope(), "old").unwrap());
        assert!(!has_run(&app, &scope(), ID).unwrap());
        let mut root = read(&path);
        root["scheduledTasks"][0]["id"] = json!(ID);
        fs::write(&path, root.to_string()).unwrap();
        assert!(has_run(&app, &scope(), ID).unwrap());
    }

    #[test]
    fn a_write_keeps_a_backup_named_after_the_cycle() {
        let (_temp, app, skills, path) = world();
        let before = fs::read_to_string(&path).unwrap();
        write_task(&app, &scope(), &task(), &skills).unwrap();
        let backup = path.with_file_name("scheduled-tasks.json.bak-sw-3");
        assert_eq!(fs::read_to_string(backup).unwrap(), before);
    }

    #[cfg(windows)]
    fn hold_lock(app: &Path) -> fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = app.join("Local Storage").join("leveldb");
        fs::create_dir_all(&dir).unwrap();
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .open(dir.join("LOCK"))
            .unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn nothing_is_written_or_removed_while_claude_runs() {
        let (_temp, app, skills, path) = world();
        let before = fs::read_to_string(&path).unwrap();
        let _lock = hold_lock(&app);
        assert!(write_task(&app, &scope(), &task(), &skills).is_err());
        assert!(remove_task(&app, &scope(), "old", &skills).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert!(!skills.join(ID).exists());
        assert!(
            !path
                .with_file_name("scheduled-tasks.json.bak-sw-3")
                .exists()
        );
    }
}
