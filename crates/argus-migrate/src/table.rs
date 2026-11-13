//! Step 3 and the end of step 8: the identity table `boards.json`, written `migrating` before
//! the first board moves and `active` once the run verified.

use std::path::Path;

use argus_core::boards::{self, BoardEntry};
use argus_core::clock::now_iso;

use crate::error::Result;
use crate::plan::Plan;

/// Adds the boards of the plan the table does not hold, with the state `migrating`.
pub fn seed(home: &Path, plan: &Plan) -> Result<()> {
    let mut table = boards::load(home)?;
    let mut changed = false;
    for (key, p) in &plan.boards {
        if table.boards.contains_key(&p.id) {
            continue;
        }
        table.boards.insert(
            p.id.clone(),
            BoardEntry {
                name: p.name.clone(),
                aliases: vec![key.clone()],
                role: p.role,
                created_at: now_iso(),
                renamed_at: None,
            },
        );
        changed = true;
    }
    if changed {
        "migrating".clone_into(&mut table.state);
        boards::save(home, &table)?;
    }
    Ok(())
}

/// Sets the state `active`.
pub fn activate(home: &Path) -> Result<()> {
    let mut table = boards::load(home)?;
    if table.state != "active" {
        "active".clone_into(&mut table.state);
        boards::save(home, &table)?;
    }
    Ok(())
}
