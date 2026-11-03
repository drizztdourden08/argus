//! The links of an agent from the options of `coord agent start` and `coord agent set` (R21, R5):
//! `--item` (repeat it), `--add-item`, `--worktree <path|auto>` or `--master`, and `--blocked-by`
//! (repeat it, ids of the board). A start needs an item and a place; a set changes what it is given.

use argus_board::agent_links::items_of;
use argus_board::ops::{checked_blocked_by, checked_items};
use argus_board::{Board, BoardError};
use serde_json::{Value, json};

use super::fields::Fields;
use crate::cli::exit::CoordError;
use crate::cli::{Opt, Options, Val};

const NEEDS_ITEM: &str =
    "an agent works on an item: add --item I-n (repeat --item for more than one).";
const NEEDS_PLACE: &str = "an agent runs somewhere: add --worktree <path> (or --worktree auto for its own agent-<claudeId> folder) or --master for the main checkout.";
const EXCLUDE: &str = "--worktree and --master exclude each other: an agent runs in one place.";

/// Where the options put an agent.
#[derive(Debug, PartialEq, Eq)]
pub enum Place {
    /// A folder, as given.
    Worktree(String),
    /// Its own `agent-<claudeId>` folder, found once the scan sees it.
    Auto,
    /// The main checkout.
    Master,
}

fn refuse(e: BoardError) -> CoordError {
    CoordError::refuse(e.to_string())
}

/// Every text an option was given, split at commas: `--item I-1 --item I-2,I-3` is three. A bare
/// flag is a refusal. `None` when the option is absent.
fn texts(options: &Options, key: &str) -> Result<Option<Vec<String>>, CoordError> {
    let vals: Vec<&Val> = match options.get(key) {
        None => return Ok(None),
        Some(Opt::One(v)) => vec![v],
        Some(Opt::Many(list)) => list.iter().collect(),
    };
    let mut out = Vec::new();
    for val in vals {
        let Val::Text(text) = val else {
            return Err(CoordError::refuse(format!("--{key} needs a value.")));
        };
        out.extend(
            text.split(',')
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(str::to_owned),
        );
    }
    Ok(Some(out))
}

/// `--master`: a bare flag, true. A value is a refusal (it would have eaten the next word).
fn master_flag(options: &Options) -> Result<bool, CoordError> {
    match options.get("master") {
        None => Ok(false),
        Some(Opt::One(Val::Flag)) => Ok(true),
        Some(_) => Err(CoordError::refuse(
            "--master takes no value: put it after the name and the task.",
        )),
    }
}

/// `--worktree` and `--master`: where the agent runs, `None` when neither was given.
///
/// # Errors
/// Both given, a repeated or bare `--worktree`, or a `--master` with a value.
pub fn place_of(options: &Options) -> Result<Option<Place>, CoordError> {
    let master = master_flag(options)?;
    let worktree = match options.get("worktree") {
        None => None,
        Some(Opt::One(Val::Text(t))) => Some(t.trim().to_owned()),
        Some(_) => return Err(CoordError::refuse("--worktree needs a value.")),
    };
    match (master, worktree) {
        (true, Some(_)) => Err(CoordError::refuse(EXCLUDE)),
        (true, None) => Ok(Some(Place::Master)),
        (false, Some(w)) if w.eq_ignore_ascii_case("auto") => Ok(Some(Place::Auto)),
        (false, Some(w)) => Ok(Some(Place::Worktree(w))),
        (false, None) => Ok(None),
    }
}

fn blocked_by(board: &Board, options: &Options) -> Result<Option<Vec<String>>, CoordError> {
    texts(options, "blocked-by")?
        .map(|ids| checked_blocked_by(board, &json!(ids)).map_err(refuse))
        .transpose()
}

/// The fields of the links: `items` and `item` (its first), then the place.
fn put_items(fields: &mut Fields, items: &[String]) {
    fields.insert("items".to_owned(), json!(items));
    fields.insert("item".to_owned(), json!(items.first()));
}

fn put_place(fields: &mut Fields, place: &Place) {
    let (worktree, master) = match place {
        Place::Worktree(path) if !path.is_empty() => (json!(path), false),
        Place::Worktree(_) | Place::Auto => (Value::Null, false),
        Place::Master => (Value::Null, true),
    };
    fields.insert("worktree".to_owned(), worktree);
    fields.insert("master".to_owned(), json!(master));
}

/// The links a start needs: at least one item, a place, and the ids it waits on.
///
/// # Errors
/// No `--item`, neither `--worktree` nor `--master`, both, an item or blocker the board does not have.
pub fn for_start(board: &Board, options: &Options) -> Result<Fields, CoordError> {
    let given = texts(options, "item")?.unwrap_or_default();
    if given.is_empty() {
        return Err(CoordError::refuse(NEEDS_ITEM));
    }
    let items = checked_items(board, &json!(given)).map_err(refuse)?;
    let place = place_of(options)?.ok_or_else(|| CoordError::refuse(NEEDS_PLACE))?;
    let mut fields = Fields::new();
    put_items(&mut fields, &items);
    put_place(&mut fields, &place);
    let waits = blocked_by(board, options)?.unwrap_or_default();
    fields.insert("blockedBy".to_owned(), json!(waits));
    Ok(fields)
}

/// The items after `--item` (replaces the list; `""` clears it) and `--add-item` (appends).
fn items_after(
    board: &Board,
    options: &Options,
    agent: &Value,
) -> Result<Option<Vec<String>>, CoordError> {
    let replaced = texts(options, "item")?;
    let added = texts(options, "add-item")?;
    if replaced.is_none() && added.is_none() {
        return Ok(None);
    }
    let mut wanted = replaced.unwrap_or_else(|| items_of(agent));
    wanted.extend(added.unwrap_or_default());
    checked_items(board, &json!(wanted))
        .map(Some)
        .map_err(refuse)
}

/// The links a set changes: only what the options name.
///
/// # Errors
/// An item or blocker the board does not have, both `--worktree` and `--master`.
pub fn for_set(board: &Board, options: &Options, agent: &Value) -> Result<Fields, CoordError> {
    let mut fields = Fields::new();
    if let Some(items) = items_after(board, options, agent)? {
        put_items(&mut fields, &items);
    }
    if let Some(place) = place_of(options)? {
        put_place(&mut fields, &place);
    }
    if let Some(waits) = blocked_by(board, options)? {
        fields.insert("blockedBy".to_owned(), json!(waits));
    }
    Ok(fields)
}

/// `" for I-1, I-2"` after a start or bind line, or nothing when the agent has no item.
#[must_use]
pub fn for_items(agent: &Value) -> String {
    let items = items_of(agent);
    if items.is_empty() {
        return String::new();
    }
    format!(
        " for {}",
        items
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::parse_args;

    fn options(args: &[&str]) -> Options {
        let words: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
        parse_args(&words).options
    }

    fn board() -> Board {
        json!({"items": {"I-1": {"id": "I-1"}, "I-2": {"id": "I-2"}}, "agents": {"A-1": {"id": "A-1"}}})
    }

    #[test]
    fn a_start_needs_an_item_and_a_place_and_not_both_places() {
        let b = board();
        let no_item = for_start(&b, &options(&["--master"])).expect_err("refused");
        assert_eq!(no_item.to_string(), NEEDS_ITEM);
        let no_place = for_start(&b, &options(&["--item", "I-1"])).expect_err("refused");
        assert_eq!(no_place.to_string(), NEEDS_PLACE);
        let both = for_start(
            &b,
            &options(&["--item", "I-1", "--master", "--worktree", "w"]),
        );
        assert_eq!(both.expect_err("refused").to_string(), EXCLUDE);
    }

    #[test]
    fn a_start_with_two_items_and_auto_writes_the_links() {
        let b = board();
        let args = [
            "--item",
            "I-1",
            "--item",
            "I-2",
            "--worktree",
            "auto",
            "--blocked-by",
            "A-1",
        ];
        let fields = for_start(&b, &options(&args)).expect("fields");
        assert_eq!(fields["items"], json!(["I-1", "I-2"]));
        assert_eq!(fields["item"], "I-1");
        assert_eq!(fields["worktree"], Value::Null);
        assert_eq!(fields["master"], false);
        assert_eq!(fields["blockedBy"], json!(["A-1"]));
        let master = for_start(&b, &options(&["--item", "I-2", "--master"])).expect("fields");
        assert_eq!(master["master"], true);
    }

    #[test]
    fn an_item_or_a_blocker_the_board_does_not_have_is_refused() {
        let b = board();
        let item = for_start(&b, &options(&["--item", "I-9", "--master"])).expect_err("refused");
        assert_eq!(item.to_string(), "no item \"I-9\".");
        let text = for_start(
            &b,
            &options(&["--item", "I-1", "--master", "--blocked-by", "the key"]),
        );
        assert!(
            text.expect_err("refused")
                .to_string()
                .contains("is not an id")
        );
    }

    #[test]
    fn a_set_adds_an_item_to_the_list_and_clears_it_with_a_blank_value() {
        let b = board();
        let agent = json!({"item": "I-1", "items": ["I-1"]});
        let added = for_set(&b, &options(&["--add-item", "I-2"]), &agent).expect("fields");
        assert_eq!(added["items"], json!(["I-1", "I-2"]));
        let cleared = for_set(&b, &options(&["--item", ""]), &agent).expect("fields");
        assert_eq!(cleared["items"], json!([]));
        assert_eq!(cleared["item"], Value::Null);
        let nothing = for_set(&b, &options(&[]), &agent).expect("fields");
        assert_eq!(nothing.len(), 0);
    }
}
