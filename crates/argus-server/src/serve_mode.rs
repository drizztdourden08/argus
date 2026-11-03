//! How a server treats its home: the live dashboard, a shadow that writes nothing, or a preview
//! that writes its copied boards and nothing outside them.

/// The mode a server runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServeMode {
    /// The live dashboard.
    #[default]
    Live,
    /// No board writes and no outward effect.
    Shadow,
    /// A copied home: board writes, no outward effect. Refused on the live boards.
    Preview,
}

impl ServeMode {
    /// The mode named by the two command line flags; shadow outranks preview.
    #[must_use]
    pub fn of(shadow: bool, preview: bool) -> Self {
        match (shadow, preview) {
            (true, _) => Self::Shadow,
            (_, true) => Self::Preview,
            _ => Self::Live,
        }
    }

    /// Nothing reaches outside the home: worktrees, processes, launches, toasts.
    #[must_use]
    pub fn outward_off(self) -> bool {
        self != Self::Live
    }
}
