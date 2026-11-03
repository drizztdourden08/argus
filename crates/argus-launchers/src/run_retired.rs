//! A build that closed with its item has no Play definition any more (the board write that
//! retires it drops `launcher-<name>`). A hidden run of that Play is stopped here; a visible one
//! is the owner's window and stays, as it does at shutdown.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::host::ProcessHost;
use crate::launcher::DEF_PREFIX;
use crate::run_service::{RunService, run_key};

/// How often the live runs are compared with the definitions.
pub(crate) const CLOSED_PLAYS_EVERY: Duration = Duration::from_secs(30);

/// Who is named as the stopper in the log and the row.
const STOPPED_BY: &str = "build closed";

/// A live hidden run of a Play: where it runs, its id and the definition name it came from.
struct PlayRun {
    scope: String,
    rid: String,
    name: String,
}

impl PlayRun {
    fn key(&self) -> String {
        run_key(&self.scope, &self.rid)
    }
}

impl<H: ProcessHost> RunService<H> {
    fn hidden_plays(&self) -> Vec<PlayRun> {
        self.state()
            .live
            .values()
            .filter(|run| !run.visible && run.name.starts_with(DEF_PREFIX))
            .map(|run| PlayRun {
                scope: run.scope.clone(),
                rid: run.rid.clone(),
                name: run.name.clone(),
            })
            .collect()
    }

    /// The hidden Play runs whose definition is gone from the board.
    fn without_definition(&self) -> Vec<PlayRun> {
        let mut known: HashMap<String, HashSet<String>> = HashMap::new();
        let mut gone = Vec::new();
        for run in self.hidden_plays() {
            let names = known.entry(run.scope.clone()).or_insert_with(|| {
                self.inner
                    .deps
                    .defs
                    .defs(&run.scope)
                    .into_iter()
                    .map(|def| def.name)
                    .collect()
            });
            if !names.contains(&run.name) {
                gone.push(run);
            }
        }
        gone
    }

    /// Stops the hidden runs whose Play definition has been missing on two checks in a row (one
    /// check can catch a board file half written). Returns `scope:rid` of each stopped run.
    pub async fn stop_closed_plays(&self) -> Vec<String> {
        let gone = self.without_definition();
        let now: HashSet<String> = gone.iter().map(PlayRun::key).collect();
        let confirmed: Vec<PlayRun> = {
            let mut state = self.state();
            let before = std::mem::replace(&mut state.missing_plays, now);
            gone.into_iter()
                .filter(|run| before.contains(&run.key()))
                .collect()
        };
        let mut stopped = Vec::new();
        for run in confirmed {
            if self.stop(&run.scope, &run.rid, STOPPED_BY).await.is_ok() {
                stopped.push(run.key());
            }
        }
        stopped
    }
}
