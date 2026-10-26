//! The display name of a project, from the boards the sync service knows.

use argus_notify::ProjectNames;

use crate::boards::BoardReader;

/// The names the boards give their projects.
pub struct BoardNames {
    boards: BoardReader,
}

impl BoardNames {
    /// The names of the projects the boards list.
    #[must_use]
    pub fn new(boards: BoardReader) -> Self {
        BoardNames { boards }
    }
}

impl ProjectNames for BoardNames {
    fn name(&self, key: &str) -> Option<String> {
        self.boards
            .projects()
            .into_iter()
            .find(|p| p.key == key)
            .map(|p| p.name)
    }
}
