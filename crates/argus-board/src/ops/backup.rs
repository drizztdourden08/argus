//! The only hard delete: write a restorable backup under `<project>/purged/`, then remove what a
//! plan names.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use argus_core::clock::{file_stamp, now_iso};
use argus_core::js;
use serde_json::{Map, Value, json};

use super::announce::{audit, coordinator_ids, refresh_needs_human};
use super::plan::{COLLS, PurgePlan};
use super::who::Who;
use crate::Board;
use crate::error::BoardError;
use crate::inbox::queue;
use crate::json::{array_mut, display, doc, map_mut, set};
use crate::pause::release_all;
use crate::store::{SCHEMA, board_dir, remove_doc, touch};

/// What a purge did: the backup file and how much went.
#[derive(Debug, Clone)]
pub struct PurgeDone {
    /// The backup file.
    pub file: String,
    /// The counts of the plan.
    pub counts: Map<String, Value>,
}

/// `countLine`: the non-zero counts as `3 items, 2 tests`, or `nothing`.
#[must_use]
pub fn count_line(counts: &Map<String, Value>) -> String {
    let parts: Vec<String> = counts
        .iter()
        .filter(|(_, n)| js::truthy(n))
        .map(|(key, n)| format!("{} {key}", display(n)))
        .collect();
    if parts.is_empty() {
        "nothing".to_owned()
    } else {
        parts.join(", ")
    }
}

fn backup_name(dir: &Path) -> (String, PathBuf) {
    let stamp = file_stamp();
    let mut base = stamp.clone();
    let mut n = 2;
    while dir.join(format!("{base}.json")).exists() {
        base = format!("{stamp}-{n}");
        n += 1;
    }
    let file = dir.join(format!("{base}.json"));
    (base, file)
}

fn proof_files(tests: &Map<String, Value>, proof_dir: &Path) -> Vec<Value> {
    let mut found = Vec::new();
    for t in tests.values() {
        for p in t["proofs"].as_array().into_iter().flatten() {
            let Some(file) = p["file"]
                .as_str()
                .filter(|f| !f.is_empty() && Path::new(f).exists())
            else {
                continue;
            };
            let name = Path::new(file).file_name().unwrap_or_default();
            found.push(json!({
                "test": t["id"], "n": p["n"], "file": file,
                "kept": proof_dir.join(name).to_string_lossy(),
            }));
        }
    }
    found
}

fn records_of(board: &Board, plan: &PurgePlan, proof_dir: &Path) -> Map<String, Value> {
    let mut records = Map::new();
    for coll in COLLS {
        let docs: Map<String, Value> = plan
            .ids
            .list(coll)
            .iter()
            .filter_map(|id| doc(board, coll, id).map(|d| (id.clone(), d.clone())))
            .collect();
        records.insert(coll.to_owned(), Value::Object(docs));
    }
    let log: Map<String, Value> = plan
        .ids
        .log
        .iter()
        .map(|(subject, lines)| (subject.clone(), Value::Array(lines.clone())))
        .collect();
    records.insert("log".into(), Value::Object(log));
    let inbox: Map<String, Value> = plan
        .ids
        .inbox
        .iter()
        .map(|id| (id.clone(), board["inbox"][id.as_str()].clone()))
        .collect();
    records.insert("inbox".into(), Value::Object(inbox));
    let versions: Map<String, Value> = COLLS
        .iter()
        .flat_map(|c| plan.ids.list(c).iter().map(move |id| format!("{c}/{id}")))
        .filter_map(|key| board["versions"].get(&key).map(|v| (key, v.clone())))
        .collect();
    records.insert("versions".into(), Value::Object(versions));
    let tests = records["tests"].as_object().cloned().unwrap_or_default();
    records.insert(
        "proofFiles".into(),
        Value::Array(proof_files(&tests, proof_dir)),
    );
    records
}

fn write_backup(
    file: &Path,
    board: &Board,
    plan: &PurgePlan,
    by: &str,
    records: &Map<String, Value>,
) -> Result<(), BoardError> {
    let backup = json!({
        "schema": SCHEMA, "project": board["project"], "purgedAt": now_iso(), "by": by,
        "criteria": plan.criteria, "counts": plan.counts, "records": records,
    });
    let mut out = OpenOptions::new().write(true).create_new(true).open(file)?;
    out.write_all(format!("{}\n", serde_json::to_string_pretty(&backup)?).as_bytes())?;
    Ok(())
}

fn drop_log_lines(board: &mut Board, plan: &PurgePlan) {
    for (subject, lines) in &plan.ids.log {
        let Some(d) = board.get_mut("log").and_then(|l| l.get_mut(subject)) else {
            continue;
        };
        array_mut(d, "entries").retain(|l| !lines.contains(l));
        if d["entries"].as_array().is_some_and(Vec::is_empty) {
            remove_doc(board, "log", subject);
            continue;
        }
        set(d, "updatedAt", now_iso());
        touch(board, "log", subject);
    }
}

fn remove_selected(board: &mut Board, plan: &PurgePlan) {
    for id in &plan.ids.threads {
        release_all(board, id);
    }
    drop_log_lines(board, plan);
    for coll in COLLS {
        for id in plan.ids.list(coll) {
            remove_doc(board, coll, id);
        }
    }
    for id in &plan.ids.threads {
        map_mut(board, "inbox").shift_remove(id);
    }
    for b in map_mut(board, "builds").values_mut() {
        if b["thread"]
            .as_str()
            .is_some_and(|t| plan.ids.threads.iter().any(|p| p == t))
        {
            set(b, "thread", Value::Null);
        }
    }
}

fn announce_purge(board: &mut Board, by: &str, summary: &str, (file, name): (&str, &str)) {
    audit(board, by, &format!("purged {summary}; backup {name}"));
    let who = if by == "human" { "owner" } else { by };
    let project = display(&board["project"]);
    for to in coordinator_ids(board, true) {
        let text = format!(
            "{who} purged {summary}. Backup: {file}. Restore: coord restore --from {file} --confirm {project}"
        );
        queue(board, &to, by, "purge", &text, None);
    }
}

fn action_owners(records: &Map<String, Value>) -> Vec<String> {
    let mut owners: Vec<String> = Vec::new();
    for a in records["actions"]
        .as_object()
        .into_iter()
        .flat_map(Map::values)
    {
        let owner = display(&a["thread"]);
        if !owners.contains(&owner) {
            owners.push(owner);
        }
    }
    owners
}

/// Write the backup, then remove what the plan names. The backup is
/// `<home>/<project>/purged/<timestamp>.json` (records keyed by id, the work-log lines per
/// subject, the inbox per thread, the doc versions) and the proof images move to
/// `purged/<timestamp>/proofs/`. Counters are untouched: ids are never reused. Locks held by a
/// purged thread pass to the next in the queue. The coordinators are told where the backup is.
///
/// # Errors
/// A backup or proof file cannot be written, copied or removed.
pub fn apply_purge(
    home: &Path,
    board: &mut Board,
    plan: &PurgePlan,
    who: &Who,
) -> Result<PurgeDone, BoardError> {
    let by = who.actor_of();
    let dir = board_dir(home, &display(&board["project"])).join("purged");
    fs::create_dir_all(&dir)?;
    let (base, file) = backup_name(&dir);
    let proof_dir = dir.join(&base).join("proofs");
    let records = records_of(board, plan, &proof_dir);
    let proofs = records["proofFiles"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !proofs.is_empty() {
        fs::create_dir_all(&proof_dir)?;
    }
    for p in &proofs {
        fs::copy(display(&p["file"]), display(&p["kept"]))?;
    }
    let owners = action_owners(&records);
    write_backup(&file, board, plan, &by, &records)?;
    remove_selected(board, plan);
    for owner in &owners {
        refresh_needs_human(board, owner);
    }
    for p in &proofs {
        fs::remove_file(display(&p["file"]))?;
    }
    let file = file.to_string_lossy().into_owned();
    announce_purge(
        board,
        &by,
        &count_line(&plan.counts),
        (&file, &format!("{base}.json")),
    );
    Ok(PurgeDone {
        file,
        counts: plan.counts.clone(),
    })
}
