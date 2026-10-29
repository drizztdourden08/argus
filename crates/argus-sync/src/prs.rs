//! Local pull requests: an op on the `prs` collection (`pr-open`, `pr-update`, `pr-merge`,
//! `pr-close`, `pr-checks`) applied under the board lock as the caller, and the board exported
//! before the answer. The caller is not an owner post: nothing lands in `entries`.

use std::sync::Arc;

use argus_board::ops::{Role, Status, Who, apply_op};
use argus_board::store::{LockWait, board_path, with_board_async};
use argus_board::{Board, BoardError};
use argus_core::ApiError;
use argus_core::ids::is_key;
use serde_json::Value;

use crate::failure::board_error;
use crate::state::{State, clear_marks};

/// What an applied pull request op leaves.
#[derive(Debug, Clone, PartialEq)]
pub struct PrApplied {
    /// The sentence the board gives (`PR-3 opened`).
    pub outcome: String,
    /// The pull request.
    pub id: String,
    /// The record as the board holds it now.
    pub pr: Value,
    /// The seq of the export that carried it.
    pub seq: i64,
}

fn caller(by: &str) -> Who {
    Who {
        thread: None,
        role: Role::Human,
        actor: Some(by.to_owned()),
    }
}

fn id_of(op: &Value, created: Option<&Value>) -> String {
    let named = created.map_or(&op["target"]["id"], |c| &c["id"]);
    named.as_str().unwrap_or_default().to_owned()
}

fn run_op(board: &mut Board, op: &Value, who: &Who) -> Result<(String, String), BoardError> {
    let res = apply_op(board, op, who, None);
    if res.status != Status::Applied {
        return Err(BoardError::Refused(res.outcome));
    }
    clear_marks(board);
    Ok((res.outcome, id_of(op, res.extra.get("created"))))
}

impl State {
    /// Applies one pull request op as `by` and exports the board.
    pub async fn apply_pr(
        self: &Arc<Self>,
        key: &str,
        op: Value,
        by: &str,
    ) -> Result<PrApplied, ApiError> {
        if !is_key(key) {
            return Err(ApiError::bad_request(format!(
                "invalid project key \"{key}\""
            )));
        }
        if !board_path(self.coord_home(), key).exists() {
            return Err(ApiError::not_found(format!(
                "no board for project \"{key}\""
            )));
        }
        let (me, who) = (Arc::clone(self), caller(by));
        let key_owned = key.to_owned();
        self.opts
            .queue
            .run(key, async move { me.apply_pr_now(&key_owned, &op, &who).await })
            .await
    }

    async fn apply_pr_now(
        self: &Arc<Self>,
        key: &str,
        op: &Value,
        who: &Who,
    ) -> Result<PrApplied, ApiError> {
        let ((outcome, id), saved) =
            with_board_async(self.coord_home(), key, LockWait::default(), |board| {
                let done = run_op(board, op, who)?;
                Ok((done, board.clone()))
            })
            .await
            .map_err(board_error)?;
        let pr = saved["prs"][id.as_str()].clone();
        let out = self.export_board(key, saved)?;
        Ok(PrApplied {
            outcome,
            id,
            pr,
            seq: out.seq,
        })
    }
}
