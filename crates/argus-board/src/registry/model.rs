//! The typed shape of `registry/item-types.json`. Every struct refuses unknown keys, so a typo in
//! the file fails the load; the cross-reference rules live in [`super::verify`].

use std::collections::BTreeMap;

use regex::Regex;
use serde::{Deserialize, Deserializer};

/// A regular expression that is compiled while the registry is parsed, so a bad pattern fails
/// the load and never a later check.
#[derive(Debug, Clone)]
pub struct Pattern(pub Regex);

impl<'de> Deserialize<'de> for Pattern {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let text = String::deserialize(de)?;
        Regex::new(&text)
            .map(Pattern)
            .map_err(|e| serde::de::Error::custom(format!("pattern {text}: {e}")))
    }
}

/// A content kind that has no settings of its own.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plain {}

/// The registry: item types, fields, content kinds, sub-items, people and the text limits.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Registry {
    /// The editor hint that points at the schema file.
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    /// The version of the file's structure.
    pub version: u32,
    /// The board lanes, in column order.
    pub lanes: Vec<String>,
    /// The settings of every content kind.
    pub content: Content,
    /// The fields every work item has.
    pub base_fields: Vec<String>,
    /// Every field that has a content kind and a limit, by id.
    pub fields: BTreeMap<String, Field>,
    /// The item types, in display order.
    pub types: Vec<ItemType>,
    /// Questions, decisions, to-dos and notes.
    pub sub_items: Vec<SubItem>,
    /// The owner and the leads, with their initials and colours.
    pub people: People,
    /// The length limit of every kind of free text on the board.
    pub limits: BTreeMap<String, usize>,
}

/// The settings of the content kinds.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    /// One line of plain text.
    pub line: LineKind,
    /// Lower case words joined by hyphens.
    pub slug: PatternKind,
    /// Bullets and images (rule rich-v1).
    pub rich: RichKind,
    /// One of a list of values.
    pub r#enum: Plain,
    /// One id of a kind.
    #[serde(rename = "ref")]
    pub reference: RefKind,
    /// A list of ids of a kind.
    pub refs: Plain,
    /// A `YYYY-MM-DD` date.
    pub date: PatternKind,
    /// A thread, the owner or a lead.
    pub person: PatternKind,
    /// A list of structured records kept by an op.
    pub records: Plain,
    /// A true or false switch.
    pub flag: Plain,
}

/// The `line` kind.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineKind {
    /// Whether Markdown block syntax is allowed.
    pub markup: bool,
    /// Whether a line break is allowed.
    pub newlines: bool,
    /// Block syntax: a heading, table, quote, fence or list marker at the start.
    pub block: Pattern,
}

/// A kind that is one pattern.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatternKind {
    /// The pattern the whole text must match.
    pub pattern: Pattern,
}

/// The `ref` kind: the id shape of every kind a field may point to.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefKind {
    /// Kind name to id pattern.
    pub kinds: BTreeMap<String, Pattern>,
}

/// The `rich` kind: the numbers and the per-line patterns of rule rich-v1.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RichKind {
    /// The rule's name.
    pub rule: String,
    /// The most bullets a text holds.
    pub bullets: usize,
    /// The most characters a bullet holds.
    pub bullet_max: usize,
    /// The most characters the lead line holds.
    pub lead_max: usize,
    /// The longest single line accepted as it is, with no bullet.
    pub short_max: usize,
    /// A bullet line.
    pub bullet: Pattern,
    /// An image line; group 1 is the image number.
    pub image: Pattern,
    /// Syntax refused on any line.
    pub forbidden: Vec<Forbidden>,
}

/// A syntax that rich text refuses.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forbidden {
    /// What it is called in the refusal.
    pub name: String,
    /// The pattern, tried on each line.
    pub pattern: Pattern,
}

/// The content kind of a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// One line of plain text.
    Line,
    /// A slug.
    Slug,
    /// Bullets and images.
    Rich,
    /// One of a list of values.
    Enum,
    /// One id.
    Ref,
    /// A list of ids.
    Refs,
    /// A date.
    Date,
    /// A thread, the owner or a lead.
    Person,
    /// A list of structured records.
    Records,
    /// A true or false switch.
    Flag,
}

/// A field: its content kind and its limits.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    /// The content kind.
    pub content: ContentType,
    /// The most characters (the text of a rich field, without markup).
    #[serde(default)]
    pub max: Option<usize>,
    /// Whether a value is required.
    #[serde(default)]
    pub required: bool,
    /// The most entries when the field is a list.
    #[serde(default)]
    pub list: Option<usize>,
    /// The most images a rich field holds.
    #[serde(default)]
    pub images: Option<usize>,
    /// The allowed values of an enum.
    #[serde(default)]
    pub values: Vec<String>,
    /// The labels of the values, in the same order.
    #[serde(default)]
    pub labels: Vec<String>,
    /// The id kinds a ref may name.
    #[serde(default)]
    pub to: Vec<String>,
    /// Whether a person may be a lead of another board (`lead:<key>`).
    #[serde(default)]
    pub leads: bool,
}

/// Where the records of a type live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Collection {
    /// The `items` collection (`I-n`).
    Items,
    /// The `tests` collection (`T-n`).
    Tests,
    /// The `launchers` collection, keyed by name: the builds.
    Launchers,
    /// The worktrees the scanner reports; never written from the board.
    Worktrees,
}

/// How strict a rule is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// The pane and the CLI say what is open; nothing is refused.
    Warn,
    /// The change is refused unless forced.
    Refuse,
}

/// What a card shows as its progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Progress {
    /// Criteria checked over criteria.
    Ac,
    /// Children done over children.
    Children,
    /// Children first, then criteria.
    ChildrenAc,
    /// The last plan run.
    Run,
    /// No bar.
    None,
}

/// An item type.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ItemType {
    /// The type id (`bug`).
    pub id: String,
    /// The name shown to the owner.
    pub label: String,
    /// A Tessera icon name.
    pub icon: String,
    /// A Tessera tag category colour.
    pub color: String,
    /// Where its records live.
    pub collection: Collection,
    /// The key of its records when it is not `id` (a build is its launcher `name`).
    #[serde(default)]
    pub id_key: Option<String>,
    /// The types it may sit under; empty means a root only.
    pub parents: Vec<String>,
    /// The item types a build may serve; empty for every other type.
    #[serde(default)]
    pub serves: Vec<String>,
    /// Whether the board only shows its records and never writes them.
    #[serde(default)]
    pub read_only: bool,
    /// Its stages, in flow order (empty for a type with statuses).
    pub stages: Vec<Stage>,
    /// Its statuses, for a type that has no lane.
    #[serde(default)]
    pub statuses: Vec<String>,
    /// Its own fields, beyond the base fields.
    pub fields: Vec<String>,
    /// How the overview groups its fields.
    #[serde(default)]
    pub groups: Vec<Group>,
    /// Its pane tabs, in order.
    pub tabs: Vec<String>,
    /// What its card shows.
    pub card: Card,
    /// The rule that must hold before it is done.
    #[serde(default)]
    pub done: Option<DoneRule>,
}

/// A stage of a type, mapped to one lane.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    /// The stage id.
    pub id: String,
    /// The name shown to the owner.
    pub label: String,
    /// The lane the stage sits in.
    pub lane: String,
    /// The stages it may move to; absent means any move.
    #[serde(default)]
    pub next: Option<Vec<String>>,
    /// Fields that must be set to enter the stage.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Rules that must hold to enter the stage.
    #[serde(default)]
    pub gates: Vec<Gate>,
}

/// A rule on entering a stage.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gate {
    /// The rule's name.
    pub need: String,
    /// Whether it refuses or only warns.
    pub mode: Mode,
}

/// The rule that must hold before an item is done.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoneRule {
    /// The names of the conditions.
    pub needs: Vec<String>,
    /// Whether a missing condition refuses or only warns.
    pub mode: Mode,
}

/// A group of fields on the overview tab.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// The group id.
    pub id: String,
    /// The heading.
    pub label: String,
    /// Its fields.
    pub fields: Vec<String>,
}

/// What the card of a type shows beyond the common rows.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    /// The progress bar.
    pub progress: Progress,
    /// The fields shown as chips.
    pub chips: Vec<String>,
}

/// Where a sub-item is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Store {
    /// The `entries` collection.
    Entries,
    /// The `actions` collection.
    Actions,
}

/// When a sub-item blocks its item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlocksWhile {
    /// While it is open.
    Open,
    /// While it is pending.
    Pending,
}

/// A small record under an item: a question, decision, to-do or note.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SubItem {
    /// The kind name.
    pub id: String,
    /// Where it is stored.
    pub store: Store,
    /// The letter of its ids (`Q` for `Q-3`).
    pub prefix: String,
    /// A Tessera icon name.
    pub icon: String,
    /// A tone name.
    pub tone: String,
    /// When it blocks its item; absent means never.
    #[serde(default)]
    pub blocks_while: Option<BlocksWhile>,
    /// The registry field that holds its text.
    pub field: String,
}

/// The owner and the leads.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct People {
    /// The owner.
    pub owner: Owner,
    /// The lead of each board, by project key.
    pub leads: BTreeMap<String, Lead>,
}

/// How the owner is drawn.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Owner {
    /// The name shown.
    pub label: String,
    /// The two letters.
    pub initials: String,
    /// The colour.
    pub color: String,
}

/// How a lead is drawn.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lead {
    /// The two letters.
    pub initials: String,
    /// The colour.
    pub color: String,
}
