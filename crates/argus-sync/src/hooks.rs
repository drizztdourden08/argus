//! What other services hang on the sync service: a look at each applied owner post, at each
//! project board just exported, and a say in the docs of a project export before they are
//! stored (the live view of sub-agents joins the `agents` docs this way). A hook never fails
//! the post or the export it hears about.

use std::sync::{Arc, PoisonError, RwLock};

use argus_board::Board;
use argus_core::DocChange;
use serde_json::Value;

/// One owner post the importer applied and exported.
#[derive(Debug, Clone)]
pub struct AppliedPost {
    /// The project key, or `_portfolio`.
    pub scope: String,
    /// The entry as the board stored it.
    pub entry: Option<Value>,
    /// The post status.
    pub status: String,
    /// The project board after the post (none for a portfolio post).
    pub board: Option<Board>,
    /// The project board as the last export left it, before the post.
    pub before: Option<Board>,
}

type OnApplied = Arc<dyn Fn(&AppliedPost) + Send + Sync>;
type OnBoard = Arc<dyn Fn(&str, &Board) + Send + Sync>;
type Overlay = Arc<dyn Fn(&str, &mut [DocChange]) + Send + Sync>;

/// The hooks of one sync service. Each is added once, at boot, and lives as long as the service.
#[derive(Default)]
pub struct Hooks {
    applied: RwLock<Vec<OnApplied>>,
    board: RwLock<Vec<OnBoard>>,
    overlay: RwLock<Vec<Overlay>>,
}

fn snapshot<T: Clone>(list: &RwLock<Vec<T>>) -> Vec<T> {
    list.read().unwrap_or_else(PoisonError::into_inner).clone()
}

fn push<T>(list: &RwLock<Vec<T>>, item: T) {
    list.write()
        .unwrap_or_else(PoisonError::into_inner)
        .push(item);
}

impl Hooks {
    /// Hears every applied owner post, after its export.
    pub fn on_applied(&self, f: impl Fn(&AppliedPost) + Send + Sync + 'static) {
        push(&self.applied, Arc::new(f));
    }

    /// Hears every project board after its export, with the board as exported.
    pub fn on_board(&self, f: impl Fn(&str, &Board) + Send + Sync + 'static) {
        push(&self.board, Arc::new(f));
    }

    /// Changes the docs of a project export before they are stored.
    pub fn on_overlay(&self, f: impl Fn(&str, &mut [DocChange]) + Send + Sync + 'static) {
        push(&self.overlay, Arc::new(f));
    }

    pub(crate) fn applied(&self, post: &AppliedPost) {
        for f in snapshot(&self.applied) {
            f(post);
        }
    }

    pub(crate) fn exported(&self, key: &str, board: &Board) {
        for f in snapshot(&self.board) {
            f(key, board);
        }
    }

    pub(crate) fn overlaid(&self, key: &str, docs: &mut [DocChange]) {
        for f in snapshot(&self.overlay) {
            f(key, docs);
        }
    }
}
