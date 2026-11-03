//! The words the registry may use for things the code and the web know by name.

/// Tessera icon names the registry may use (`gavel` and `signpost` come with the next Tessera).
pub(super) const ICONS: [&str; 11] = [
    "zap",
    "bookmark",
    "circle-check",
    "bug",
    "wrench",
    "scroll-text",
    "flask",
    "circle-help",
    "gavel",
    "list-checks",
    "message-square",
];

/// Tessera tag category colours: the type marks and the people use these.
pub(super) const CATEGORY_COLORS: [&str; 10] = [
    "violet", "green", "blue", "rose", "amber", "cyan", "teal", "orange", "pink", "lime",
];

/// Colours a person may have beyond the category colours.
pub(super) const PERSON_COLORS: [&str; 2] = ["primary", "neutral"];

/// Tones of a sub-item.
pub(super) const TONES: [&str; 4] = ["danger", "info", "warning", "neutral"];

/// The tabs of the item pane.
pub(super) const TABS: [&str; 9] = [
    "overview",
    "children",
    "cases",
    "run",
    "links",
    "discussion",
    "tests",
    "media",
    "activity",
];

/// Conditions a done rule may name.
pub(super) const DONE_NEEDS: [&str; 5] = ["ac", "children", "tests", "verified", "plan-passed"];

/// Conditions a stage gate may name.
pub(super) const GATE_NEEDS: [&str; 2] = ["cases", "bundle-ready"];

/// The deepest chain of parents: epic, story, leaf.
pub(super) const MAX_DEPTH: usize = 3;
