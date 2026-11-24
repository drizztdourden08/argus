//! The id and role of a project's lead thread, and the legacy name boards and stamps still carry.

use std::borrow::Cow;

/// The thread id of a project's lead.
pub const LEAD: &str = "lead";
/// The role of a project's lead thread.
pub const LEAD_ROLE: &str = "lead";
/// The name the lead went by before the rename; read and migrated, never written.
pub const LEGACY: &str = "coordinator";

const LEGACY_HOSTED: &str = "coordinator-hosted";
const LEAD_HOSTED: &str = "lead-hosted";

/// The canonical form of a thread id: the legacy lead ids become the lead ids, anything else is
/// returned unchanged.
#[must_use]
pub fn canonical(id: &str) -> Cow<'_, str> {
    match id {
        LEGACY => Cow::Borrowed(LEAD),
        LEGACY_HOSTED => Cow::Borrowed(LEAD_HOSTED),
        LEGACY_HOSTED => Cow::Borrowed(LEAD_HOSTED),
        other => Cow::Borrowed(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_legacy_ids_map_to_the_lead_ids() {
        assert_eq!(canonical("coordinator"), "lead");
        assert_eq!(canonical("coordinator-hosted"), "lead-hosted");
    }

    #[test]
    fn every_other_id_is_unchanged() {
        for id in ["lead", "ui", "coordinators", "Coordinator", "", "owner"] {
            assert_eq!(canonical(id), id);
        }
    }
}
