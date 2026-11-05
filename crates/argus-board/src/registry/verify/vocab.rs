//! The words the registry may use for things the code and the web know by name.

use super::super::model::Collection;

/// Tessera icon names the registry may use (`gavel` and `signpost` come with the next Tessera).
pub(super) const ICONS: [&str; 23] = [
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
    "play",
    "git-branch",
    "robot",
    "git-fork",
    "tag",
    "link",
    "square-code",
    "mail",
    "bell",
    "send",
    "trash-2",
    "layout-dashboard",
];

/// Tessera tag category colours: the type marks and the people use these.
pub(super) const CATEGORY_COLORS: [&str; 10] = [
    "violet", "green", "blue", "rose", "amber", "cyan", "teal", "orange", "pink", "lime",
];

/// Colours a person may have beyond the category colours.
pub(super) const PERSON_COLORS: [&str; 2] = ["primary", "neutral"];

/// Tones of a kind that has no category colour.
pub(super) const TONES: [&str; 5] = ["danger", "info", "warning", "success", "neutral"];

/// The tabs of a record pane. `builds` is the older name of `work` and stays valid.
pub(super) const TABS: [&str; 19] = [
    "overview",
    "children",
    "cases",
    "run",
    "links",
    "discussion",
    "tests",
    "media",
    "builds",
    "activity",
    "work",
    "result",
    "runs",
    "log",
    "checks",
    "steps",
    "files",
    "transcript",
    "trail",
];

/// The words an `assignee` may be besides a property: the owner, or the lead of the board.
pub(super) const ASSIGNEE_WORDS: [&str; 2] = ["=owner", "=lead"];

/// The collections the item types live in; every other record has one of its own.
pub(super) const TYPE_COLLECTIONS: [Collection; 5] = [
    Collection::Items,
    Collection::Tests,
    Collection::Launchers,
    Collection::Worktrees,
    Collection::Boards,
];

/// The letter an item is stored with on disk (`I-12`); no kind takes it as a code.
pub(super) const ITEM_LETTER: &str = "I";

/// Conditions a done rule may name.
pub(super) const DONE_NEEDS: [&str; 5] = ["ac", "children", "tests", "verified", "plan-passed"];

/// Conditions a stage gate may name.
pub(super) const GATE_NEEDS: [&str; 2] = ["cases", "bundle-ready"];

/// The deepest chain of parents: epic, story, leaf.
pub(super) const MAX_DEPTH: usize = 3;
