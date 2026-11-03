//! The item a worktree belongs to, resolved in this order: an explicit link (`coord worktree link`), the item of the
//! agent record that owns it, an `I-n` in its branch name, an open item whose folder, branch or text names it.
//!
//! The link is read from the board on every look (it is not stored): a link made after the item closed, a restart and a
//! missed scan all converge on the same answer.

use crate::approval::{mentions, names_folder};
use crate::board::{AgentFact, BoardFacts};
use crate::fact::Fact;
use crate::paths::base_name;
use crate::service::Service;
use serde::Serialize;

/// A worktree with no item this long after Argus first saw it is flagged once.
pub(crate) const UNLINKED_AFTER_MS: i64 = 3_600_000;

/// Where a worktree's link to its item comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkSource {
    /// `coord worktree link`.
    Manual,
    /// The item of the agent record that owns the worktree.
    Agent,
    /// An `I-n` in the branch name.
    Branch,
    /// An open item whose folder, branch or text names the worktree.
    Text,
}

/// The item a worktree belongs to and how Argus knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ItemLink {
    pub item: String,
    pub source: LinkSource,
}

fn linked(item: &str, source: LinkSource) -> ItemLink {
    ItemLink {
        item: item.to_string(),
        source,
    }
}

/// `i-78` in a branch name (`feat/I-78-close`, `worktree-i-78`) as the item id, when the board has that item.
fn branch_item(board: &BoardFacts, branch: &str) -> Option<String> {
    let lower = branch.to_ascii_lowercase();
    lower.match_indices("i-").find_map(|(at, _)| {
        let fresh = lower[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        let digits: String = lower[at + 2..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let id = format!("I-{digits}");
        (fresh && !digits.is_empty() && board.items.contains_key(&id)).then_some(id)
    })
}

/// Could the text name the worktree at all: its folder name or its branch occurs in it (a cheap look before the exact
/// one, which folds the case of the whole text).
fn may_name(text: &str, name: &str, branch: Option<&str>) -> bool {
    let lower = text.to_lowercase();
    lower.contains(&name.to_lowercase())
        || branch.is_some_and(|b| lower.contains(&b.to_lowercase()))
}

/// The first open item whose text names the worktree: its folder, its folder name or its branch.
fn text_item(board: &BoardFacts, fact: &Fact, ci: bool) -> Option<String> {
    let name = base_name(&fact.path);
    let branch = fact.branch.as_deref();
    board
        .items
        .values()
        .filter(|item| !item.closed() && may_name(&item.text, &name, branch))
        .find(|item| {
            names_folder(&item.text, &fact.path, ci)
                || mentions(&item.text, &name)
                || branch.is_some_and(|b| mentions(&item.text, b))
        })
        .map(|item| item.id.clone())
}

/// The item the last agent to own the worktree works for.
pub(crate) fn agent_item<'a>(owners: &[&'a AgentFact]) -> Option<&'a str> {
    owners.iter().rev().find_map(|a| a.item.as_deref())
}

/// The item of a worktree, by the order above. A link to an item the board does not have is passed over.
pub(crate) fn resolve(
    board: &BoardFacts,
    fact: &Fact,
    agent: Option<&str>,
    ci: bool,
) -> Option<ItemLink> {
    let known = |id: &str| board.items.contains_key(id);
    board
        .worktree_links
        .get(&fact.id)
        .filter(|l| known(&l.item))
        .map(|l| linked(&l.item, LinkSource::Manual))
        .or_else(|| {
            agent
                .filter(|i| known(i))
                .map(|i| linked(i, LinkSource::Agent))
        })
        .or_else(|| {
            fact.branch
                .as_deref()
                .and_then(|b| branch_item(board, b))
                .map(|i| linked(&i, LinkSource::Branch))
        })
        .or_else(|| text_item(board, fact, ci).map(|i| linked(&i, LinkSource::Text)))
}

impl Service {
    /// The item of a worktree, or none.
    pub(crate) fn link_of(&self, board: &BoardFacts, fact: &Fact) -> Option<ItemLink> {
        let owners = self.owners(board, fact);
        resolve(board, fact, agent_item(&owners), self.ci())
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{ItemFact, WorktreeLink};
    use crate::fact::blank_fact;
    use crate::rule::Rule;

    fn fact() -> Fact {
        let rule = Rule::folder("W-1", "X:/p/.claude/worktrees", "t");
        let mut fact = blank_fact("X:/p/.claude/worktrees/agent-a12", &rule, true);
        fact.id = "w1".into();
        fact.branch = Some("feat/I-3-close".into());
        fact
    }

    fn board() -> BoardFacts {
        let mut board = BoardFacts::default();
        for (id, lane, text) in [
            ("I-1", "done", "land agent-a12"),
            ("I-2", "doing", "land agent-a12 first"),
            ("I-3", "todo", ""),
            ("I-4", "todo", ""),
            ("I-5", "todo", ""),
        ] {
            board.items.insert(
                id.into(),
                ItemFact {
                    id: id.into(),
                    lane: lane.into(),
                    trashed: false,
                    text: text.into(),
                },
            );
        }
        board
    }

    fn source(board: &BoardFacts, agent: Option<&str>) -> Option<(String, LinkSource)> {
        resolve(board, &fact(), agent, true).map(|l| (l.item, l.source))
    }

    #[test]
    fn a_manual_link_wins_over_the_agent_over_the_branch_over_the_text() {
        let mut board = board();
        assert_eq!(
            source(&board, Some("I-4")),
            Some(("I-4".into(), LinkSource::Agent))
        );
        board.worktree_links.insert(
            "w1".into(),
            WorktreeLink {
                item: "I-5".into(),
                by: None,
                at_ms: None,
            },
        );
        assert_eq!(
            source(&board, Some("I-4")),
            Some(("I-5".into(), LinkSource::Manual))
        );
        board.worktree_links.clear();
        assert_eq!(
            source(&board, None),
            Some(("I-3".into(), LinkSource::Branch))
        );
        board.items.remove("I-3");
        assert_eq!(
            source(&board, None),
            Some(("I-2".into(), LinkSource::Text)),
            "the done item's text names it too, but only an open item counts"
        );
        board.items.remove("I-2");
        assert_eq!(source(&board, None), None);
    }

    #[test]
    fn a_link_to_an_item_the_board_lacks_is_passed_over() {
        let board = board();
        assert_eq!(
            source(&board, Some("I-99")),
            Some(("I-3".into(), LinkSource::Branch))
        );
    }

    #[test]
    fn the_branch_names_an_item_only_as_a_whole_id() {
        let board = board();
        for (branch, want) in [
            ("worktree-i-4", Some("I-4")),
            ("I-5", Some("I-5")),
            ("ci-3", None),
            ("feat/I-30", None),
            ("agent-a12", None),
        ] {
            assert_eq!(branch_item(&board, branch).as_deref(), want, "{branch}");
        }
    }
}
