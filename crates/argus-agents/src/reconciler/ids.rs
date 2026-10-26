//! Step 1: the ids of every open or recently closed record agree with each other and with the
//! name of the transcript. Ids that disagree are flagged, and repaired from the file when exactly
//! one of them has a transcript. A record with an id and no transcript path gets the path.

use super::pass::Pass;
use super::session::{Session, agent_id_of_path};
use crate::record::{Patch, Record, claude_id_of, filled, is_set, text};

fn distinct_ids(agent: &Record) -> Vec<String> {
    let named = [
        filled(agent, "claudeId").map(str::to_owned),
        filled(agent, "hookId").map(str::to_owned),
        filled(agent, "transcriptFile").and_then(agent_id_of_path),
    ];
    let mut ids: Vec<String> = Vec::new();
    for id in named.into_iter().flatten() {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

impl<'a> Pass<'a> {
    pub(super) fn verify_ids(&mut self) {
        let targets: Vec<&'a Record> = self
            .all
            .iter()
            .copied()
            .filter(|a| Self::is_open(a) || self.recently_closed(a))
            .collect();
        for agent in targets {
            let session = self.session(agent);
            let ids = distinct_ids(agent);
            if ids.len() == 1 && !is_set(agent, "claudeId") {
                self.patch(agent, Patch::new().with("claudeId", ids[0].as_str()));
            } else if ids.len() > 1 {
                self.repair_ids(agent, &session, &ids);
            }
            self.restore_path(agent, &session);
        }
    }

    fn repair_ids(&mut self, agent: &Record, session: &Session, ids: &[String]) {
        let real: Vec<(&String, String)> = ids
            .iter()
            .filter_map(|id| {
                let file = session.find_file(id)?;
                Some((id, file.to_string_lossy().into_owned()))
            })
            .collect();
        let mut fix = Patch::new().with("idMismatch", true);
        if let [(id, file)] = real.as_slice() {
            fix = fix
                .with("claudeId", id.as_str())
                .with("transcriptFile", file.as_str());
            if is_set(agent, "hookId") {
                fix = fix.with("hookId", id.as_str());
            }
        }
        let changes = fix
            .entries()
            .iter()
            .any(|(k, v)| agent.get(k) != v.as_ref());
        if changes {
            self.patch(agent, fix);
        }
    }

    fn restore_path(&mut self, agent: &Record, session: &Session) {
        let current = self.cur(agent);
        let Some(id) = claude_id_of(&current) else {
            return;
        };
        if self.exists(text(&current, "transcriptFile")) {
            return;
        }
        let Some(file) = session.find_file(&id) else {
            return;
        };
        let path = file.to_string_lossy().into_owned();
        if text(agent, "transcriptFile") != Some(path.as_str()) {
            self.patch(agent, Patch::new().with("transcriptFile", path));
        }
    }
}
