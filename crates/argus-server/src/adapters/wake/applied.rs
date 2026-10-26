//! An applied owner post, from the sync service's shape to the wake logic's.

use argus_wake::board::EntryView;
use argus_wake::service::AppliedPost;
use serde::Deserialize;

use super::boards::view_of;

/// The post the wake policy asks about.
#[must_use]
pub fn applied_of(post: &argus_sync::AppliedPost) -> AppliedPost {
    AppliedPost {
        scope: post.scope.clone(),
        entry: post
            .entry
            .as_ref()
            .and_then(|entry| EntryView::deserialize(entry).ok()),
        status: Some(post.status.clone()),
        board: post.board.as_ref().map(view_of),
        before: post.before.as_ref().map(view_of),
    }
}
