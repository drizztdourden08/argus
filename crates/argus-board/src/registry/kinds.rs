//! One view over every kind of record the app shows. The registry keeps item types, sub-items and
//! other records in three lists; a [`Kind`] reads the same facts from any of them, so a tab, a
//! card or a link parser asks "what is this kind" once and never names a kind.

use super::model::{Card, Collection, ItemType, Link, Record, Registry, Store, SubItem, View};

/// Which list of the registry a kind comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// An item type (`types`).
    Type,
    /// A question, decision, to-do or note (`subItems`).
    SubItem,
    /// Any other record (`records`).
    Record,
}

/// How a kind is coloured: a Tessera category colour or a tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look<'a> {
    /// A category colour; only the ten work and execution kinds have one.
    Color(&'a str),
    /// A tone, told apart from the others by its icon.
    Tone(&'a str),
}

/// The facts every kind has, whichever list of the registry it is in.
#[derive(Debug, Clone, Copy)]
pub struct Kind<'a> {
    /// The list it comes from.
    pub family: Family,
    /// The kind name (`bug`, `change`).
    pub id: &'a str,
    /// The name shown to the owner.
    pub label: &'a str,
    /// The letters its ids are shown with.
    pub code: &'a str,
    /// The kind whose code and counter it shares.
    pub shares: Option<&'a str>,
    /// Older letters that still parse.
    pub aliases: &'a [String],
    /// The letters its ids carry on disk when they differ from the code.
    pub stored: Option<&'a str>,
    /// A Tessera icon name.
    pub icon: &'a str,
    /// Its colour or tone.
    pub look: Look<'a>,
    /// Where its records live.
    pub collection: Collection,
    /// The key of its records when it is not `id`.
    pub id_key: Option<&'a str>,
    /// Whether a list draws it as a card or a row.
    pub view: View,
    /// The property shown as its title.
    pub title: &'a str,
    /// Where its avatar comes from: a property, `=owner` or `=lead`.
    pub assignee: &'a str,
    /// Its pane tabs, in order.
    pub tabs: &'a [String],
    /// What its card shows.
    pub card: &'a Card,
    /// The links it carries.
    pub links: &'a [Link],
    /// The providers of the signal strip of its card.
    pub signals: &'a [String],
    /// The link field a list groups it by.
    pub group_by: Option<&'a str>,
    /// The primary action of its card.
    pub action: Option<&'a str>,
}

/// The look a record names: its colour, else its tone, else the neutral tone.
fn look_of(rec: &Record) -> Look<'_> {
    match (&rec.color, &rec.tone) {
        (Some(color), _) => Look::Color(color),
        (None, Some(tone)) => Look::Tone(tone),
        (None, None) => Look::Tone("neutral"),
    }
}

fn from_type(t: &ItemType) -> Kind<'_> {
    Kind {
        family: Family::Type,
        id: &t.id,
        label: &t.label,
        code: &t.code,
        shares: None,
        aliases: &[],
        stored: None,
        icon: &t.icon,
        look: Look::Color(&t.color),
        collection: t.collection,
        id_key: t.id_key.as_deref(),
        view: t.view,
        title: &t.title,
        assignee: &t.assignee,
        tabs: &t.tabs,
        card: &t.card,
        links: &t.links,
        signals: &t.signals,
        group_by: t.group_by.as_deref(),
        action: t.action.as_deref(),
    }
}

fn from_sub_item(s: &SubItem) -> Kind<'_> {
    Kind {
        family: Family::SubItem,
        id: &s.id,
        label: &s.label,
        code: &s.code,
        shares: None,
        aliases: &[],
        stored: None,
        icon: &s.icon,
        look: Look::Tone(&s.tone),
        collection: match s.store {
            Store::Entries => Collection::Entries,
            Store::Actions => Collection::Actions,
        },
        id_key: None,
        view: s.view,
        title: &s.field,
        assignee: &s.assignee,
        tabs: &s.tabs,
        card: &s.card,
        links: &s.links,
        signals: &s.signals,
        group_by: s.group_by.as_deref(),
        action: s.action.as_deref(),
    }
}

fn from_record(r: &Record) -> Kind<'_> {
    Kind {
        family: Family::Record,
        id: &r.id,
        label: &r.label,
        code: &r.code,
        shares: r.shares.as_deref(),
        aliases: &r.aliases,
        stored: r.stored.as_deref(),
        icon: &r.icon,
        look: look_of(r),
        collection: r.collection,
        id_key: r.id_key.as_deref(),
        view: r.view,
        title: &r.title,
        assignee: &r.assignee,
        tabs: &r.tabs,
        card: &r.card,
        links: &r.links,
        signals: &r.signals,
        group_by: r.group_by.as_deref(),
        action: r.action.as_deref(),
    }
}

impl Kind<'_> {
    /// Whether it is a work item type: one that lives in `items` and shows as `I-n` on disk.
    #[must_use]
    pub fn is_work_item(&self) -> bool {
        self.family == Family::Type && self.collection == Collection::Items
    }

    /// Whether `letters` is its code or one of its older letters.
    #[must_use]
    pub fn answers_to(&self, letters: &str) -> bool {
        self.code == letters || self.aliases.iter().any(|a| a == letters)
    }
}

impl Registry {
    /// Every kind, item types first, then sub-items, then the other records, each in file order.
    #[must_use]
    pub fn kinds(&self) -> Vec<Kind<'_>> {
        let types = self.types.iter().map(from_type);
        let subs = self.sub_items.iter().map(from_sub_item);
        let records = self.records.iter().map(from_record);
        types.chain(subs).chain(records).collect()
    }

    /// The kind with this id.
    #[must_use]
    pub fn kind(&self, id: &str) -> Option<Kind<'_>> {
        self.kinds().into_iter().find(|k| k.id == id)
    }

    /// The kind whose ids are written with these letters. A kind that shares a code with another
    /// (a notice with the notes) never answers: the kind it shares with does.
    #[must_use]
    pub fn kind_by_code(&self, letters: &str) -> Option<Kind<'_>> {
        self.kinds()
            .into_iter()
            .find(|k| k.shares.is_none() && k.answers_to(letters))
    }

    /// The letters ids are parsed with and the kind each names, the longest letters first (so
    /// `TP` is read before `T`), then in alphabetical order. Older letters are in the table.
    #[must_use]
    pub fn code_table(&self) -> Vec<(&str, &str)> {
        let mut table: Vec<(&str, &str)> = Vec::new();
        for kind in self.kinds().iter().filter(|k| k.shares.is_none()) {
            table.push((kind.code, kind.id));
            table.extend(kind.aliases.iter().map(|a| (a.as_str(), kind.id)));
        }
        table.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(b.0)));
        table
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{Family, Look};
    use crate::registry::{Collection, View, registry};

    const WORK_CODES: [(&str, &str); 6] = [
        ("epic", "E"),
        ("story", "S"),
        ("task", "K"),
        ("bug", "B"),
        ("debt", "X"),
        ("test-plan", "TP"),
    ];

    #[test]
    fn the_work_item_types_carry_the_letters_of_q12() {
        for (id, code) in WORK_CODES {
            let kind = registry().kind(id).unwrap();
            assert_eq!(kind.code, code, "{id}");
            assert!(kind.is_work_item(), "{id}");
        }
        assert_eq!(registry().kind("test-case").unwrap().code, "T");
        assert_eq!(registry().kind("test-case").unwrap().view, View::Row);
    }

    #[test]
    fn every_kind_of_the_standard_is_there_in_its_list() {
        let count = |family: Family| {
            registry()
                .kinds()
                .iter()
                .filter(|k| k.family == family)
                .count()
        };
        assert_eq!(count(Family::Type), 9);
        assert_eq!(count(Family::SubItem), 4);
        assert_eq!(count(Family::Record), 9);
        let code = |id: &str| registry().kind(id).unwrap().code;
        let want = [
            ("build", "BL"),
            ("worktree", "WT"),
            ("agent", "A"),
            ("change", "PR"),
            ("release", "V"),
            ("dependency", "DP"),
            ("run", "R"),
            ("check", "C"),
            ("question", "Q"),
            ("decision", "D"),
            ("todo", "H"),
            ("note", "N"),
            ("notice", "N"),
            ("followup", "P"),
            ("message", "M"),
        ];
        for (id, letters) in want {
            assert_eq!(code(id), letters, "{id}");
        }
    }

    #[test]
    fn the_ten_category_colours_go_to_ten_kinds_once_each() {
        let mut colors: Vec<&str> = registry()
            .kinds()
            .iter()
            .filter_map(|k| match k.look {
                Look::Color(c) => Some(c),
                Look::Tone(_) => None,
            })
            .collect();
        assert_eq!(colors.len(), 10);
        colors.sort_unstable();
        colors.dedup();
        assert_eq!(colors.len(), 10);
        let build = registry().kind("build").unwrap();
        assert_eq!(build.look, Look::Color("lime"));
        let agent = registry().kind("agent").unwrap();
        assert_eq!(agent.look, Look::Color("pink"));
    }

    #[test]
    fn the_code_table_reads_the_longest_letters_first_and_keeps_the_older_ones() {
        let table = registry().code_table();
        let at = |letters: &str| table.iter().position(|(l, _)| *l == letters).unwrap();
        assert!(at("TP") < at("T"));
        assert!(at("PR") < at("P"));
        assert!(at("WT") < at("A"));
        assert!(at("BL") < at("B"));
        assert!(table.contains(&("F", "followup")));
        assert_eq!(table.iter().filter(|(l, _)| *l == "N").count(), 1);
    }

    #[test]
    fn a_letter_names_one_kind_and_a_notice_leaves_the_n_to_the_notes() {
        let reg = registry();
        assert_eq!(reg.kind_by_code("K").unwrap().id, "task");
        assert_eq!(reg.kind_by_code("TP").unwrap().id, "test-plan");
        assert_eq!(reg.kind_by_code("T").unwrap().id, "test-case");
        assert_eq!(reg.kind_by_code("N").unwrap().id, "note");
        assert_eq!(reg.kind_by_code("F").unwrap().id, "followup");
        assert!(reg.kind_by_code("Z").is_none());
        assert!(reg.kind_by_code("I").is_none());
    }

    #[test]
    fn a_dependency_is_shown_dp_and_stored_d_and_lives_in_the_ledger() {
        let dep = registry().kind("dependency").unwrap();
        assert_eq!(dep.code, "DP");
        assert_eq!(dep.stored, Some("D"));
        assert_eq!(dep.collection, Collection::Deps);
        assert!(!dep.is_work_item());
    }

    #[test]
    fn a_sub_item_reads_its_title_from_its_field_and_a_record_has_no_stages() {
        let q = registry().kind("question").unwrap();
        assert_eq!(q.title, "question");
        assert_eq!(q.collection, Collection::Entries);
        assert_eq!(q.assignee, "=owner");
        assert_eq!(
            registry().kind("todo").unwrap().collection,
            Collection::Actions
        );
        assert_eq!(registry().records.len(), 9);
        assert_eq!(registry().kind("agent").unwrap().tabs[1], "work");
    }

    #[test]
    fn the_links_are_listed_with_their_other_end() {
        let bug = registry().kind("bug").unwrap();
        let dup = bug.links.iter().find(|l| l.field == "duplicateOf").unwrap();
        assert_eq!(dup.to, ["bug"]);
        assert_eq!(dup.inverse.as_deref(), Some("Duplicates"));
        assert_eq!(bug.group_by, Some("parent"));
        let agent = registry().kind("agent").unwrap();
        assert_eq!(agent.group_by, Some("items"));
        assert!(agent.links.iter().any(|l| l.to == ["worktree"]));
    }
}
