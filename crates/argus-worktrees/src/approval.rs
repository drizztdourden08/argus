//! The owner's say for discarding work: which board entry counts, and what names a folder.

use crate::board::{BoardFacts, OwnerEntry};
use crate::clock::DAY_MS;
use crate::fact::Fact;
use crate::paths::{base_name, path_key};

/// An owner entry that asks for a deletion counts for this many days.
pub const APPROVAL_DAYS: i64 = 14;

fn word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn find_all<'a>(text: &'a str, needle: &'a str) -> impl Iterator<Item = usize> + 'a {
    let mut from = 0;
    std::iter::from_fn(move || {
        let at = from + text.get(from..)?.find(needle)?;
        from = at + needle.chars().next().map_or(1, char::len_utf8);
        Some(at)
    })
}

/// Does the text name `word` as a whole word (a branch, a folder name)? Case is ignored.
pub fn mentions(text: &str, word: &str) -> bool {
    let (text, word) = (text.to_lowercase(), word.trim().to_lowercase());
    if word.is_empty() {
        return false;
    }
    find_all(&text, &word).any(|at| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !before.is_some_and(word_char) && !after.is_some_and(word_char)
    })
}

/// Does a command line, executable path or working directory name this folder (or something inside it)?
pub fn names_folder(text: &str, path: &str, case_insensitive: bool) -> bool {
    let slashed = text.replace('\\', "/");
    let haystack = if case_insensitive {
        slashed.to_lowercase()
    } else {
        slashed
    };
    let key = path_key(path, case_insensitive);
    if key.is_empty() {
        return false;
    }
    find_all(&haystack, &key).any(|at| match haystack[at + key.len()..].chars().next() {
        None => true,
        Some(c) => c == '/' || c == '"' || c == '\'' || c.is_whitespace(),
    })
}

/// What a written note must say to count as the owner's go: a delete word.
const DELETE_WORDS: [&str; 7] = [
    "delete",
    "remove",
    "discard",
    "get rid of",
    "clean up",
    "drop",
    "can go",
];

/// Words that turn a delete request around: such a note is not the owner's go.
const NEGATION_WORDS: [&str; 10] = [
    "don't", "dont", "do not", "never", "not", "keep", "leave", "stop", "wait", "no",
];

fn says_delete(text: &str) -> bool {
    DELETE_WORDS.iter().any(|w| word_alone(text, w))
}

/// Does the text say no, keep, leave, stop, wait or never?
fn negated(text: &str) -> bool {
    NEGATION_WORDS.iter().any(|w| word_alone(text, w))
}

pub(crate) fn word_alone(text: &str, word: &str) -> bool {
    let lower = text.to_lowercase();
    find_all(&lower, word).any(|at| {
        let before = lower[..at].chars().next_back();
        let after = lower[at + word.len()..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
            && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

fn joined<'a>(parts: impl Iterator<Item = &'a str>) -> String {
    parts
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The entry when it can be the owner's say (a decision or a note he posted from the page, young enough), else the reason it cannot.
fn entry_problem<'a>(
    board: &'a BoardFacts,
    id: &str,
    now_ms: i64,
) -> Result<&'a OwnerEntry, String> {
    let Some(entry) = board.entries.get(id).filter(|e| !e.deleted) else {
        return Err(format!("there is no entry {id} on this board"));
    };
    if entry.author != "human" || entry.via.as_deref() != Some("page") {
        return Err(format!("{id} was not posted by the owner from the page"));
    }
    if entry.kind != "decision" && entry.kind != "followup" {
        let kind = if entry.kind.is_empty() {
            "record"
        } else {
            entry.kind.as_str()
        };
        return Err(format!(
            "{id} is a {kind}, not the owner's answer to a decision or a note"
        ));
    }
    if entry
        .created_ms
        .is_none_or(|at| now_ms - at > APPROVAL_DAYS * DAY_MS)
    {
        return Err(format!(
            "{id} is older than {APPROVAL_DAYS} days: ask the owner again"
        ));
    }
    Ok(entry)
}

/// The words of the entry itself, and of the question it answers.
fn entry_words(board: &BoardFacts, entry: &OwnerEntry) -> (String, String) {
    let wt = entry.worktree.as_ref();
    let own = [
        Some(entry.text.as_str()),
        wt.map(|w| w.path.as_str()),
        wt.and_then(|w| w.message.as_deref()),
    ];
    let asked = entry
        .reply_to
        .as_ref()
        .and_then(|r| board.entries.get(r))
        .map(|q| {
            let parts = [Some(q.text.as_str()), q.context.as_deref()]
                .into_iter()
                .flatten()
                .chain(q.options.iter().map(String::as_str));
            joined(parts)
        });
    (joined(own.into_iter().flatten()), asked.unwrap_or_default())
}

/// The owner's say for discarding the work of one worktree: none when `entry_id` gives it, else the plain reason it does not.
pub fn approval_of(board: &BoardFacts, entry_id: &str, fact: &Fact, now_ms: i64) -> Option<String> {
    let id = entry_id.trim();
    if id.is_empty() {
        return Some("no owner entry was given (--owner-entry <id>: the owner's answer or note that asks for it)".to_string());
    }
    let entry = match entry_problem(board, id, now_ms) {
        Ok(entry) => entry,
        Err(why) => return Some(why),
    };
    let name = base_name(&fact.path);
    let (own, asked) = entry_words(board, entry);
    let requested = entry
        .worktree
        .as_ref()
        .is_some_and(|w| w.id == fact.id && w.kind == "delete");
    let names = |text: &str| {
        names_folder(text, &fact.path, true)
            || mentions(text, &name)
            || fact.branch.as_deref().is_some_and(|b| mentions(text, b))
    };
    if requested || (says_delete(&own) && !negated(&own) && names(&format!("{own} {asked}"))) {
        return None;
    }
    let branch = fact.branch.as_deref().unwrap_or("(detached)");
    Some(format!(
        "{id} does not name this worktree (its branch {branch}, its folder {name} or its path)"
    ))
}

/// The say for discarding work when the worktree's item closed (`Caller::ItemClosed`): the owner's I-78 request, decision
/// D-42, while `worktrees.closeWithItem` is on and the item is closed on the board now. Switched off, the removal needs
/// the owner's entry exactly as a thread's does (I-67), and says so in the same words.
pub fn item_closed_say(
    board: &BoardFacts,
    (item, on): (&str, bool),
    fact: &Fact,
    now_ms: i64,
) -> Option<String> {
    if !on {
        return approval_of(board, "", fact, now_ms);
    }
    match board.items.get(item) {
        Some(found) if found.closed() => None,
        Some(_) => Some(format!("item {item} is not closed")),
        None => Some(format!("there is no item {item} on this board")),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{EntryWorktree, OwnerEntry};
    use crate::fact::blank_fact;
    use crate::rule::Rule;
    use crate::testkit::owner_entry;

    const NOW: i64 = 1_800_000_000_000;

    fn owner(id: &str, text: &str) -> OwnerEntry {
        owner_entry(id, text, NOW)
    }

    fn board_of(entries: Vec<OwnerEntry>) -> BoardFacts {
        let mut board = BoardFacts::default();
        for e in entries {
            board.entries.insert(e.id.clone(), e);
        }
        board
    }

    fn fact() -> Fact {
        let rule = Rule::folder("W-1", "X:/t/.claude/worktrees", "t");
        let mut fact = blank_fact("X:/t/.claude/worktrees/feat-login", &rule, true);
        fact.id = "abc".into();
        fact.branch = Some("claude/feat-login".into());
        fact
    }

    #[test]
    fn the_approval_check_on_its_own() {
        let fact = fact();
        let mut request = owner("O-1", "[worktree delete] x");
        request.worktree = Some(EntryWorktree {
            id: "abc".into(),
            path: fact.path.clone(),
            kind: "delete".into(),
            message: Some(String::new()),
        });
        assert_eq!(
            approval_of(&board_of(vec![request]), "O-1", &fact, NOW),
            None
        );
        assert_eq!(
            approval_of(
                &board_of(vec![owner(
                    "O-2",
                    "remove X:\\t\\.claude\\worktrees\\feat-login please"
                )]),
                "O-2",
                &fact,
                NOW
            ),
            None
        );
        assert_eq!(
            approval_of(
                &board_of(vec![owner("O-3", "delete claude/feat-login")]),
                "O-3",
                &fact,
                NOW
            ),
            None
        );
        assert_eq!(
            approval_of(
                &board_of(vec![owner("O-3b", "claude/feat-login can go")]),
                "O-3b",
                &fact,
                NOW
            ),
            None
        );
        let longer = approval_of(
            &board_of(vec![owner("O-4", "feat-login-2 can go")]),
            "O-4",
            &fact,
            NOW,
        )
        .unwrap();
        assert!(
            longer.contains("does not name"),
            "a longer name is another folder"
        );
    }

    #[test]
    fn a_note_that_only_names_the_worktree_or_says_keep_or_not_is_not_the_owners_go() {
        let fact = fact();
        for text in [
            "claude/feat-login looks fine to me",
            "keep claude/feat-login, still working",
            "do not delete claude/feat-login yet",
            "don't remove claude/feat-login",
            "keep feat-login, remove the others",
            "delete the fresh one",
        ] {
            let board = board_of(vec![owner("O-1", text)]);
            let why = approval_of(&board, "O-1", &fact, NOW);
            assert!(
                why.is_some_and(|w| w.contains("does not name this worktree")),
                "{text}"
            );
        }
        let go = board_of(vec![owner("O-2", "please delete claude/feat-login")]);
        assert_eq!(approval_of(&go, "O-2", &fact, NOW), None);
    }

    #[test]
    fn every_way_an_entry_can_fail_to_count() {
        let fact = fact();
        let mut not_page = owner("O-b", "delete feat-login");
        not_page.via = Some("mcp".into());
        let mut by_lead = owner("O-a", "delete feat-login");
        by_lead.author = "lead".into();
        let mut wrong_kind = owner("O-c", "delete feat-login");
        wrong_kind.kind = "pause".into();
        let mut old = owner("O-d", "delete feat-login");
        old.created_ms = Some(NOW - 15 * DAY_MS);
        let board = board_of(vec![
            by_lead,
            not_page,
            wrong_kind,
            old,
            owner("O-e", "delete the fresh one"),
        ]);
        let reason = |id: &str| approval_of(&board, id, &fact, NOW).unwrap();
        assert!(reason("").contains("no owner entry was given"));
        assert!(reason("O-zz").contains("there is no entry O-zz on this board"));
        assert!(reason("O-a").contains("O-a was not posted by the owner from the page"));
        assert!(reason("O-b").contains("O-b was not posted by the owner from the page"));
        assert!(
            reason("O-c")
                .contains("O-c is a pause, not the owner's answer to a decision or a note")
        );
        assert!(reason("O-d").contains("O-d is older than 14 days"));
        assert!(reason("O-e").contains("O-e does not name this worktree (its branch claude/feat-login, its folder feat-login or its path)"));
    }

    #[test]
    fn a_decision_answer_names_the_folder_through_the_question_it_answers() {
        let fact = fact();
        let mut question = owner("E-1", "Delete the two old worktrees feat-login and other?");
        question.kind = "question".into();
        question.options = vec!["Delete both".into(), "Keep them".into()];
        let mut answer = owner("O-ok", "Delete both");
        answer.kind = "decision".into();
        answer.reply_to = Some("E-1".into());
        let mut orphan = owner("O-f", "Delete both");
        orphan.reply_to = Some("E-9".into());
        let board = board_of(vec![question, answer, orphan]);
        assert_eq!(approval_of(&board, "O-ok", &fact, NOW), None);
        assert!(
            approval_of(&board, "O-f", &fact, NOW)
                .unwrap()
                .contains("does not name this worktree")
        );
    }

    #[test]
    fn folders_are_named_as_whole_paths_only() {
        let path = "X:/t/.claude/worktrees/feat-login";
        assert!(names_folder(
            "node X:\\t\\.claude\\worktrees\\feat-login\\x.js",
            path,
            true
        ));
        assert!(!names_folder(
            "node X:/t/.claude/worktrees/feat-login-2/x.js",
            path,
            true
        ));
        assert!(names_folder(
            "bash -c \"coord worktree remove X:/t/.claude/worktrees/feat-login\"",
            path,
            true
        ));
        assert!(mentions("remove **old-unmerged** now", "old-unmerged"));
        assert!(!mentions("feat-login-2 can go", "feat-login"));
    }
}
