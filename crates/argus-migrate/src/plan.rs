//! The plan, `<home>/board-ids.plan.json`: which id every key gets and the name it shows. A rerun
//! reads the file, so an id is never drawn twice.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use argus_board::names::display_name;
use argus_core::boards::{self, BoardEntry, Boards, Role};
use argus_core::ids::{PORTFOLIO, is_key};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::journal::Journal;
use crate::jsonio;

const NAME_MAX: usize = 30;

/// The id, name and role planned for one key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Planned {
    /// The new id.
    pub id: String,
    /// The display name seeded into the table: what is shown today.
    pub name: String,
    /// The role, if any.
    pub role: Option<Role>,
}

/// The plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// Format version, 1.
    pub version: u32,
    /// Old key to its id, name and role.
    pub boards: BTreeMap<String, Planned>,
    /// By file, the exact-key strings a rewrite leaves, as path shapes with counts (for review).
    #[serde(default)]
    pub remaining: BTreeMap<String, BTreeMap<String, usize>>,
}

/// The plan file of a home.
pub fn path(home: &Path) -> PathBuf {
    home.join("board-ids.plan.json")
}

/// The board file of a folder.
pub fn board_file(home: &Path, folder: &str) -> PathBuf {
    home.join(folder).join("board.json")
}

fn load(home: &Path) -> Result<Option<Plan>> {
    match fs::read_to_string(path(home)) {
        Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn folders_with_boards(home: &Path) -> Result<Vec<String>> {
    let mut names: Vec<String> = fs::read_dir(home)?
        .filter_map(std::result::Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| is_key(n) && n != "dashboard" && board_file(home, n).is_file())
        .collect();
    names.sort();
    Ok(names)
}

fn role_of(key: &str) -> Option<Role> {
    match key {
        "portfolio" => Some(Role::Overlord),
        "argus" => Some(Role::Argus),
        _ => None,
    }
}

fn seed_name(home: &Path, key: &str, registry: &Value) -> Result<String> {
    let board = jsonio::read(&board_file(home, key))?;
    let shown = display_name(
        home,
        key,
        board["name"].as_str(),
        registry["projects"][key]["name"].as_str(),
    );
    let name: String = shown.trim().chars().take(NAME_MAX).collect();
    Ok(if name.is_empty() { key.to_owned() } else { name })
}

fn registry(home: &Path) -> Value {
    jsonio::read(&board_file(home, PORTFOLIO)).unwrap_or(Value::Null)
}

fn taken(plan: &Plan) -> Boards {
    let mut table = Boards::default();
    for (key, p) in &plan.boards {
        table.boards.insert(
            p.id.clone(),
            BoardEntry {
                name: p.name.clone(),
                aliases: vec![key.clone()],
                role: p.role,
                created_at: String::new(),
                renamed_at: None,
            },
        );
    }
    table
}

fn name_of(home: &Path, id: &str, key: &str) -> String {
    boards::load(home)
        .ok()
        .and_then(|table| table.boards.get(id).map(|b| b.name.clone()))
        .unwrap_or_else(|| key.to_owned())
}

/// The journal wins: a key it holds keeps the id it recorded, even when the plan file is lost or
/// says otherwise.
fn seed_from_journal(home: &Path, plan: &mut Plan, journal: &Journal) {
    for e in &journal.boards {
        let name = name_of(home, &e.id, &e.key);
        let planned = plan.boards.entry(e.key.clone()).or_insert_with(|| Planned {
            id: e.id.clone(),
            name,
            role: role_of(&e.key),
        });
        planned.id.clone_from(&e.id);
    }
}

/// The plan of a home: the saved one, the ids of the journal of a run, plus an id for every board
/// folder it does not know yet.
pub fn build(home: &Path) -> Result<Plan> {
    let mut plan = load(home)?.unwrap_or(Plan {
        version: 1,
        boards: BTreeMap::new(),
        remaining: BTreeMap::new(),
    });
    if let Some(journal) = Journal::load(home)? {
        seed_from_journal(home, &mut plan, &journal);
    }
    let registry = registry(home);
    for folder in folders_with_boards(home)? {
        let known = plan.boards.contains_key(&folder)
            || plan.boards.values().any(|p| p.id == folder);
        if known {
            continue;
        }
        let id = boards::fresh_id(home, &taken(&plan))?;
        let name = seed_name(home, &folder, &registry)?;
        let role = role_of(&folder);
        plan.boards.insert(folder, Planned { id, name, role });
    }
    Ok(plan)
}

/// Writes the plan when it differs from the file.
pub fn save(home: &Path, plan: &Plan) -> Result<()> {
    if load(home)?.as_ref() != Some(plan) {
        jsonio::write(&path(home), &serde_json::to_value(plan)?)?;
    }
    Ok(())
}
