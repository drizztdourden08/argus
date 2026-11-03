//! What the content checker may look at on a board: the ids, the threads and the image numbers
//! of the record being written.

use crate::Board;
use crate::content::Context;
use crate::json::{ids, section, truthy};

/// The ids, threads and image numbers a value is checked against.
pub(crate) struct Known {
    ids: Vec<String>,
    threads: Vec<String>,
    images: Vec<u32>,
}

impl Known {
    /// The live ids and threads of `board`, and the image numbers of the record.
    pub(crate) fn of(board: &Board, images: Vec<u32>) -> Known {
        Known {
            ids: live_ids(board),
            threads: ids(board, "threads"),
            images,
        }
    }

    /// The context the content checker takes.
    pub(crate) fn context(&self) -> Context<'_> {
        Context {
            images: self.images.as_slice(),
            ids: Some(self.ids.as_slice()),
            threads: Some(self.threads.as_slice()),
        }
    }
}

/// The items and tests that are not in the trash, and the build tags.
fn live_ids(board: &Board) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for coll in ["items", "tests"] {
        let live = section(board, coll)
            .iter()
            .filter(|(_, d)| !truthy(d, "deletedAt"))
            .map(|(id, _)| id.clone());
        all.extend(live);
    }
    all.extend(ids(board, "builds"));
    all
}
