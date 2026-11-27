//! The handover desk's ports in the server: the working leads read from the boards, wakes
//! through the wake engine, and the cycle kept in the kv store and shown as docs of
//! `_portfolio` (the `cycle` doc of `AccountsState`, one `Handovers` doc per lead).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use argus_accounts::handover::{AgentLoad, AgentStop, Cycle, Lead, READY_REASON, WakeKind};
use argus_accounts::{Clock, HandoverStore, Threads};
use argus_core::clock::{now_ms, parse_ms};
use argus_core::ids::PORTFOLIO;
use argus_core::{Coll, DocChange};
use argus_store::Db;
use argus_wake::board::{BoardView, LIVE_AGENT, ThreadView};

use crate::boards::BoardReader;
use crate::services::{WakeHook, WakeRequest};

const CYCLE_KEY: &str = "accounts.handover.cycle";
const COUNTER_PREFIX: &str = "accounts.handover.";
/// The file a lead writes its handover to, under its board's folder of the coord home.
const HANDOVER_FILE: &str = "handover.md";
/// How long after its handover file a lead's last sign of life may be: the stop hook of the
/// turn that wrote it comes a little later.
const SEEN_SLACK_MS: i64 = 3 * 60 * 1000;

/// The clock of the server.
pub struct SystemNow;

impl Clock for SystemNow {
    fn now_ms(&self) -> i64 {
        now_ms()
    }
}

/// The leads on the boards and the wakes to them.
pub struct BoardThreads {
    boards: BoardReader,
    wake: WakeHook,
    clock: Arc<dyn Clock>,
    coord_home: PathBuf,
}

impl BoardThreads {
    /// The leads of the boards the reader knows, woken through `wake`; handover files are looked
    /// for under `coord_home`.
    #[must_use]
    pub fn new(
        boards: BoardReader,
        wake: WakeHook,
        clock: Arc<dyn Clock>,
        coord_home: PathBuf,
    ) -> Self {
        BoardThreads {
            boards,
            wake,
            clock,
            coord_home,
        }
    }
}

/// A lead's handover file and when it was last written (epoch ms).
struct HandoverFile {
    path: String,
    mtime: i64,
}

/// The handover file of a board, when it exists.
fn handover_file(coord_home: &Path, board: &str) -> Option<HandoverFile> {
    let path = coord_home.join(board).join(HANDOVER_FILE);
    let modified = std::fs::metadata(&path).ok()?.modified().ok()?;
    let mtime = i64::try_from(modified.duration_since(UNIX_EPOCH).ok()?.as_millis()).ok()?;
    Some(HandoverFile {
        path: path.to_string_lossy().into_owned(),
        mtime,
    })
}

/// Whether a lead is idle with a current handover: not in a turn, and no sign of work (its last
/// sign of life less the slack, each live agent's last update, all in epoch ms) after the file.
fn handed_over(thread: &ThreadView, agent_updates: &[i64], file_mtime: i64, now: i64) -> bool {
    if thread.in_turn(now) {
        return false;
    }
    let seen = parse_ms(thread.last_seen.as_str()).unwrap_or(0);
    seen - SEEN_SLACK_MS <= file_mtime && agent_updates.iter().all(|at| *at <= file_mtime)
}

/// When each live agent of a lead last changed (epoch ms; unknown counts as long ago).
fn agent_updates(board: &BoardView, lead: &str) -> Vec<i64> {
    board
        .agents
        .values()
        .filter(|a| a.rec.is_live() && a.thread.is(lead) && a.status.is_any(&LIVE_AGENT))
        .map(|a| parse_ms(a.rec.updated_at.as_str()).unwrap_or(0))
        .collect()
}

/// The handover file of the lead of a board when the lead is idle with a current handover.
fn idle_handover(
    board: &BoardView,
    lead: (&str, &ThreadView),
    file: Option<&HandoverFile>,
    now: i64,
) -> Option<String> {
    let file = file?;
    handed_over(lead.1, &agent_updates(board, lead.0), file.mtime, now).then(|| file.path.clone())
}

/// The lead of a board when it is a live, unpaused Desktop session. A hosted one (run by
/// Argus) has no chat to hand over from and none to message.
fn live_lead(board: &BoardView) -> Option<(String, &ThreadView)> {
    let id = board.lead()?;
    let live = board
        .threads
        .get(&id)
        .filter(|t| t.is_live() && !t.status.is("paused") && !t.is_hosted())?;
    Some((id, live))
}

/// The live agent records of a lead, with the branch each works on.
fn live_agents(board: &BoardView, lead: &str) -> Vec<AgentStop> {
    board
        .agents
        .iter()
        .filter(|(_, a)| a.rec.is_live() && a.thread.is(lead) && a.status.is_any(&LIVE_AGENT))
        .map(|(key, a)| {
            let id = a.rec.id.some().unwrap_or(key).to_owned();
            let branch = board.json()["agents"][key]["branch"]
                .as_str()
                .map(str::to_owned);
            AgentStop {
                id,
                branch,
                stop: None,
            }
        })
        .collect()
}

/// The live agent records of a lead with the model each runs on. The boards do not know an
/// agent's context size, so it stays unknown and the estimator applies no long-context factor.
fn live_loads(board: &BoardView, lead: &str) -> Vec<AgentLoad> {
    board
        .agents
        .iter()
        .filter(|(_, a)| a.rec.is_live() && a.thread.is(lead) && a.status.is_any(&LIVE_AGENT))
        .map(|(key, _)| AgentLoad {
            model: board.json()["agents"][key]["model"]
                .as_str()
                .map(str::to_owned),
            context_tokens: None,
        })
        .collect()
}

/// The lead of a board when it is working at `now`: inside a turn, or with a live agent.
fn working_lead(
    key: &str,
    board: &BoardView,
    now: i64,
    file: Option<&HandoverFile>,
) -> Option<Lead> {
    let (id, thread) = live_lead(board)?;
    let agents = live_agents(board, &id);
    if !thread.in_turn(now) && agents.is_empty() {
        return None;
    }
    let raw = &board.json()["threads"][id.as_str()];
    Some(Lead {
        board: key.to_owned(),
        thread: id.clone(),
        session_id: raw["sessionId"].as_str().map(str::to_owned),
        link: raw["link"].as_str().map(str::to_owned),
        agents,
        handed_over: idle_handover(board, (&id, thread), file, now),
    })
}

impl Threads for BoardThreads {
    fn working_leads(&self) -> Vec<Lead> {
        let now = self.clock.now_ms();
        self.boards
            .projects()
            .into_iter()
            .filter(|p| p.has_board)
            .filter_map(|p| {
                let raw = self.boards.board(&p.key)?;
                let file = handover_file(&self.coord_home, &p.key);
                working_lead(&p.key, &BoardView::from_json(raw), now, file.as_ref())
            })
            .collect()
    }

    fn wake(
        &self,
        (board, thread): (&str, &str),
        kind: WakeKind,
        subject: &str,
        text: &str,
    ) -> Result<(), String> {
        let wake = WakeRequest::about(board, thread, kind.reason(), subject, text);
        self.wake.submit(&[wake])
    }

    fn agent_loads(&self, board: &str, thread: &str) -> Vec<AgentLoad> {
        self.boards
            .board(board)
            .map(|raw| live_loads(&BoardView::from_json(raw), thread))
            .unwrap_or_default()
    }

    fn project_name(&self, board: &str) -> Option<String> {
        self.boards
            .projects()
            .into_iter()
            .find(|p| p.key == board)
            .map(|p| p.name)
            .filter(|n| !n.is_empty())
    }

    fn handed_over(&self, board: &str, thread: &str) -> Option<String> {
        let view = BoardView::from_json(self.boards.board(board)?);
        let (id, lead) = live_lead(&view)?;
        if id != thread {
            return None;
        }
        let file = handover_file(&self.coord_home, board);
        idle_handover(&view, (&id, lead), file.as_ref(), self.clock.now_ms())
    }

    fn notice(&self, subject: &str, text: &str) -> Result<(), String> {
        let (Some(main), Some(overlord)) = (self.boards.overlord(), self.boards.overlord_thread())
        else {
            return Err("the Overlord's board has no live lead".to_owned());
        };
        let wake = WakeRequest::about(&main, &overlord, READY_REASON, subject, text);
        self.wake.submit(&[wake])
    }
}

/// The cycle in the kv store and the docs.
pub struct HandoverDocs {
    db: Arc<Db>,
}

impl HandoverDocs {
    /// The cycle of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        HandoverDocs { db }
    }
}

fn changes_of(cycle: &Cycle) -> Vec<DocChange> {
    let mut changes = vec![DocChange::set(Coll::AccountsState, "cycle", cycle.to_doc())];
    changes.extend(
        cycle
            .leads
            .iter()
            .map(|h| DocChange::set(Coll::Handovers, h.id.clone(), h.to_doc())),
    );
    changes
}

impl HandoverStore for HandoverDocs {
    fn next_number(&self, counter: &str) -> Result<u64, String> {
        let key = format!("{COUNTER_PREFIX}{counter}");
        let n = self
            .db
            .tx(|tx| tx.next_counter(&key))
            .map_err(|e| e.to_string())?;
        u64::try_from(n).map_err(|e| e.to_string())
    }

    fn load(&self) -> Result<Option<Cycle>, String> {
        let Some(text) = self.db.kv_get(CYCLE_KEY).map_err(|e| e.to_string())? else {
            return Ok(None);
        };
        serde_json::from_str(&text)
            .map_err(|e| format!("the kept handover cycle does not read: {e}"))
    }

    fn save(&self, cycle: &Cycle) -> Result<(), String> {
        let text = serde_json::to_string(cycle).map_err(|e| e.to_string())?;
        let changes = changes_of(cycle);
        self.db
            .tx(|tx| {
                tx.kv_set(CYCLE_KEY, &text)?;
                tx.apply(PORTFOLIO, &changes)?;
                Ok(())
            })
            .map_err(|e| e.to_string())
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use argus_core::clock::iso_from_ms;
    use argus_wake::board::BoardView;
    use serde_json::json;

    use super::{HandoverFile, handed_over, working_lead};

    const NOW: i64 = 1_762_855_200_000;

    fn board(activity: &str, seen_ago_ms: i64, agent_status: &str) -> BoardView {
        BoardView::from_json(json!({
            "threads": { "lead": {
                "id": "lead", "role": "lead", "status": "active",
                "activity": activity, "lastSeen": iso_from_ms(NOW - seen_ago_ms),
                "sessionId": "s-1", "link": "claude://claude.ai/epitaxy/local_1"
            }},
            "agents": { "A-1": {
                "id": "A-1", "thread": "lead", "status": agent_status, "branch": "wip/a"
            }}
        }))
    }

    #[test]
    fn a_lead_inside_a_turn_is_working() {
        let lead = working_lead("rotp", &board("working", 60_000, "done"), NOW, None).unwrap();
        assert_eq!(lead.session_id.as_deref(), Some("s-1"));
        assert_eq!(lead.agents.len(), 0);
    }

    #[test]
    fn a_lead_with_a_live_agent_is_working_and_lists_it() {
        let lead = working_lead("rotp", &board("idle", 3_600_000, "working"), NOW, None).unwrap();
        assert_eq!(lead.agents[0].id, "A-1");
        assert_eq!(lead.agents[0].branch.as_deref(), Some("wip/a"));
    }

    #[test]
    fn an_idle_lead_is_left_out() {
        assert!(working_lead("rotp", &board("idle", 60_000, "done"), NOW, None).is_none());
        assert!(
            working_lead("rotp", &board("working", 600_000, "done"), NOW, None).is_none(),
            "a working flag with no sign of life for ten minutes is not a turn"
        );
    }

    fn thread(activity: &str, seen_ago_ms: i64) -> argus_wake::board::ThreadView {
        let board = board(activity, seen_ago_ms, "done");
        board.threads["lead"].clone()
    }

    #[test]
    fn an_idle_lead_with_a_file_after_its_work_is_handed_over() {
        let idle = thread("idle", 60_000);
        assert!(handed_over(&idle, &[], NOW - 30_000, NOW));
        assert!(handed_over(&idle, &[NOW - 40_000], NOW - 30_000, NOW));
    }

    #[test]
    fn a_lead_in_a_turn_or_an_agent_updated_after_the_file_is_not_handed_over() {
        let working = thread("working", 60_000);
        assert!(!handed_over(&working, &[], NOW, NOW));
        let idle = thread("idle", 60_000);
        assert!(!handed_over(&idle, &[NOW - 10_000], NOW - 30_000, NOW));
        let busy_since = thread("idle", 10_000);
        assert!(!handed_over(&busy_since, &[], NOW - 600_000, NOW));
    }

    #[test]
    fn the_working_lead_carries_the_file_only_when_it_is_handed_over() {
        let file = HandoverFile {
            path: "h.md".to_owned(),
            mtime: NOW - 30_000,
        };
        let idle = board("idle", 60_000, "working");
        let lead = working_lead("rotp", &idle, NOW, Some(&file)).unwrap();
        assert_eq!(lead.handed_over.as_deref(), Some("h.md"));
        let none = working_lead("rotp", &idle, NOW, None).unwrap();
        assert_eq!(none.handed_over, None);
    }
}
