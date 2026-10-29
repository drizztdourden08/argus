//! Who uses the cache: every project root Argus knows (the config's `projectRoots` and the registered projects) and every
//! git worktree the scanner found.

use std::collections::HashSet;

use argus_cache::{Place, PlaceKind};
use argus_worktrees::{Status, WorktreeDoc, path_key};

use crate::Services;

fn project_places(svc: &Services) -> Vec<Place> {
    let registered = svc
        .boards()
        .projects()
        .into_iter()
        .filter_map(|p| p.root.map(|root| (p.key, root)));
    let configured = svc.config().project_roots.into_iter();
    registered
        .chain(configured)
        .map(|(key, root)| Place {
            label: key,
            kind: PlaceKind::Project,
            root,
        })
        .collect()
}

/// A worktree is a place when it is a git worktree on disk. A leftover folder, a folder that is going, a gone one and a
/// folder that is no repository are not.
fn is_worktree(doc: &WorktreeDoc) -> bool {
    !matches!(
        doc.status,
        Status::Leftover | Status::Deleting | Status::Missing | Status::NotGit
    )
}

fn worktree_places(svc: &Services) -> Vec<Place> {
    let Some(service) = &svc.worktrees else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for key in service.deps.store.projects() {
        for doc in service.list(&key).into_iter().filter(is_worktree) {
            found.push(Place {
                label: format!("{key} / {}", doc.name),
                kind: PlaceKind::Worktree,
                root: doc.path,
            });
        }
    }
    found
}

/// The known places, a folder once: a project first, then its worktrees.
pub(super) fn known(svc: &Services) -> Vec<Place> {
    let mut seen = HashSet::new();
    project_places(svc)
        .into_iter()
        .chain(worktree_places(svc))
        .filter(|place| seen.insert(path_key(&place.root, cfg!(windows))))
        .collect()
}

/// Has the scanner looked at every project since the start? Without a scanner there is nothing to wait for.
pub(super) fn worktrees_known(svc: &Services) -> bool {
    svc.worktrees.as_ref().is_none_or(|service| {
        service
            .deps
            .store
            .projects()
            .iter()
            .all(|key| service.scanned(key))
    })
}
