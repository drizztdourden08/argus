//! Put a backup's records back (`restoreFrom` of `ops.mjs`): only those whose id is absent
//! (nothing is overwritten), optionally only the named threads and what hangs off them. Proof
//! images return to their old paths. Everything restored is marked dirty.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::announce::{audit, refresh_needs_human};
use super::backup::count_line;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{
    array_mut, display, doc, map_mut, nullish, obj, section, set, shown, slot, strict_eq,
    to_number,
};
use crate::store::touch;

const COUNTED: [&str; 10] = [
    "threads", "items", "tests", "entries", "actions", "agents", "logLines", "inbox", "proofs",
    "skipped",
];
const DOC_COLLS: [&str; 6] = ["threads", "items", "tests", "entries", "actions", "agents"];

/// The records of a backup, narrowed to the threads asked for.
struct Scope<'a> {
    rec: &'a Value,
    only: Option<HashSet<String>>,
    items: HashSet<String>,
}

impl<'a> Scope<'a> {
    fn new(rec: &'a Value, threads: &[String]) -> Result<Self, BoardError> {
        for id in threads {
            if !js::truthy(&rec["threads"][id.as_str()]) {
                return Err(refuse(format!("the backup holds no thread \"{id}\".")));
            }
        }
        let only = (!threads.is_empty()).then(|| threads.iter().cloned().collect());
        let mut scope = Scope {
            rec,
            only,
            items: HashSet::new(),
        };
        scope.items = section(rec, "items")
            .values()
            .filter(|i| scope.owns(&i["thread"]))
            .map(|i| display(&i["id"]))
            .collect();
        Ok(scope)
    }

    fn owns(&self, thread: &Value) -> bool {
        self.only
            .as_ref()
            .is_none_or(|o| thread.as_str().is_some_and(|t| o.contains(t)))
    }

    fn wanted(&self, d: &Value) -> bool {
        self.only.is_none()
            || self.owns(&d["thread"])
            || d["item"].as_str().is_some_and(|i| self.items.contains(i))
    }

    fn picks(&self, coll: &str, d: &Value) -> bool {
        match coll {
            "threads" => self.only.is_none() || self.owns(&d["id"]),
            "items" => self.items.contains(&display(&d["id"])),
            "agents" => self.only.is_none() || self.owns(&d["thread"]),
            _ => self.wanted(d),
        }
    }
}

fn bump(counts: &mut Map<String, Value>, key: &str, n: usize) {
    let total = counts.get(key).and_then(Value::as_u64).unwrap_or(0) + n as u64;
    counts.insert(key.to_owned(), json!(total));
}

fn restore_docs(
    board: &mut Board,
    scope: &Scope,
    counts: &mut Map<String, Value>,
) -> HashSet<String> {
    let mut restored = HashSet::new();
    for coll in DOC_COLLS {
        for d in section(scope.rec, coll)
            .values()
            .filter(|d| scope.picks(coll, d))
        {
            let id = display(&d["id"]);
            if doc(board, coll, &id).is_some() {
                bump(counts, "skipped", 1);
                continue;
            }
            map_mut(board, coll).insert(id.clone(), d.clone());
            touch(board, coll, &id);
            restored.insert(format!("{coll}/{id}"));
            bump(counts, coll, 1);
        }
    }
    restored
}

fn seq_or_zero(line: &Value) -> f64 {
    line.get("seq")
        .filter(|s| !s.is_null())
        .map_or(0.0, |s| to_number(Some(s)))
}

fn merge_log(board: &mut Board, subject: &str, mine: &[&Value]) -> usize {
    let entry = slot(slot(board, "log"), subject);
    if entry.is_null() {
        *entry = json!({"subject": subject, "entries": [], "updatedAt": now_iso()});
    }
    let have: Vec<f64> = entry["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|l| l["seq"].as_f64())
        .collect();
    let back: Vec<Value> = mine
        .iter()
        .filter(|l| l.get("seq").is_none_or(|s| s.as_f64().is_none_or(|n| !have.contains(&n))))
        .map(|l| (*l).clone())
        .collect();
    if back.is_empty() {
        return 0;
    }
    let restored = back.len();
    let list = array_mut(entry, "entries");
    list.extend(back);
    list.sort_by(|a, b| {
        seq_or_zero(a)
            .total_cmp(&seq_or_zero(b))
            .then_with(|| shown(a.get("ts")).cmp(&shown(b.get("ts"))))
    });
    let top = list.iter().map(seq_or_zero).fold(f64::NEG_INFINITY, f64::max);
    set(entry, "updatedAt", now_iso());
    touch(board, "log", subject);
    let key = format!("log:{subject}");
    let kept = board["counters"][key.as_str()].as_f64().unwrap_or(0.0);
    map_mut(board, "counters").insert(key, js::number(kept.max(top)));
    restored
}

fn restore_log(board: &mut Board, scope: &Scope, counts: &mut Map<String, Value>) {
    for (subject, lines) in obj(&scope.rec["log"]) {
        let mine: Vec<&Value> = lines
            .as_array()
            .into_iter()
            .flatten()
            .filter(|l| scope.wanted(l))
            .collect();
        let back = merge_log(board, subject, &mine);
        bump(counts, "logLines", back);
    }
}

fn restore_inbox(board: &mut Board, scope: &Scope, counts: &mut Map<String, Value>) {
    for (thread, messages) in obj(&scope.rec["inbox"]) {
        if scope.only.as_ref().is_some_and(|o| !o.contains(thread)) {
            continue;
        }
        let have: Vec<String> = board["inbox"][thread.as_str()]
            .as_array()
            .into_iter()
            .flatten()
            .map(|m| display(&m["id"]))
            .collect();
        let back: Vec<Value> = messages
            .as_array()
            .into_iter()
            .flatten()
            .filter(|m| !have.contains(&display(&m["id"])))
            .cloned()
            .collect();
        bump(counts, "inbox", back.len());
        array_mut(slot(board, "inbox"), thread).extend(back);
    }
}

fn restore_proofs(rec: &Value, restored: &HashSet<String>) -> Result<usize, BoardError> {
    let mut back = 0;
    for p in rec["proofFiles"].as_array().into_iter().flatten() {
        let (Some(kept), Some(file)) = (p["kept"].as_str(), p["file"].as_str()) else {
            continue;
        };
        let wanted = restored.contains(&format!("tests/{}", display(&p["test"])));
        if !wanted || !Path::new(kept).exists() || Path::new(file).exists() {
            continue;
        }
        if let Some(dir) = Path::new(file).parent() {
            fs::create_dir_all(dir)?;
        }
        fs::copy(kept, file)?;
        fs::remove_file(kept)?;
        back += 1;
    }
    Ok(back)
}

fn refresh_owners(board: &mut Board, rec: &Value) {
    let mut owners: Vec<String> = Vec::new();
    for owner in section(rec, "actions")
        .values()
        .filter_map(|a| nullish(a, "thread"))
    {
        if !owners.contains(&owner) {
            owners.push(owner);
        }
    }
    for owner in owners {
        refresh_needs_human(board, &owner);
    }
}

fn audit_restore(board: &mut Board, counts: &Map<String, Value>, file: &Path) {
    let changed = counts
        .iter()
        .any(|(k, n)| k != "skipped" && js::truthy(n));
    if !changed {
        return;
    }
    let mut shown_counts = counts.clone();
    shown_counts.insert("skipped".into(), json!(0));
    let name = file.file_name().map(|n| n.to_string_lossy().into_owned());
    let line = format!(
        "restored {} from {}",
        count_line(&shown_counts),
        name.unwrap_or_default()
    );
    audit(board, "coordinator", &line);
}

/// Put a backup's records back: only those whose id is absent (nothing is overwritten),
/// optionally only the named `threads` and what hangs off them. Proof images return to their
/// old paths. Everything restored is marked dirty. Returns the counts restored (`threads`,
/// `items`, `tests`, `entries`, `actions`, `agents`, `logLines`, `inbox`, `proofs`), plus
/// `skipped` for records already on the board. The caller checks that the caller is the
/// coordinator.
///
/// # Errors
/// The file cannot be read, is not JSON, is not a purge backup, belongs to another project, or
/// holds none of a named thread; a proof file cannot be copied.
pub fn restore_from(
    board: &mut Board,
    file: &Path,
    threads: &[String],
) -> Result<Map<String, Value>, BoardError> {
    let backup: Value = serde_json::from_str(&fs::read_to_string(file)?)?;
    let rec = &backup["records"];
    if !js::truthy(rec) {
        return Err(refuse(format!(
            "{} is not a coord purge backup.",
            file.display()
        )));
    }
    if !strict_eq(&backup["project"], &board["project"]) {
        return Err(refuse(format!(
            "{} is a backup of \"{}\", this board is \"{}\".",
            file.display(),
            shown(backup.get("project")),
            display(&board["project"])
        )));
    }
    let scope = Scope::new(rec, threads)?;
    let mut counts: Map<String, Value> = COUNTED.iter().map(|k| ((*k).to_owned(), json!(0))).collect();
    let restored = restore_docs(board, &scope, &mut counts);
    restore_log(board, &scope, &mut counts);
    restore_inbox(board, &scope, &mut counts);
    bump(&mut counts, "proofs", restore_proofs(rec, &restored)?);
    refresh_owners(board, rec);
    audit_restore(board, &counts, file);
    Ok(counts)
}
