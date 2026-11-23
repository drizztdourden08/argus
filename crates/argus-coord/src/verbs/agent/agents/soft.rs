//! `coord agent archive|unarchive|rm|restore <A-id|name>... [--reason "..."]`: nothing is
//! hard-deleted here. The change goes through the shared op of the board.

use argus_board::json::display;
use argus_board::limits::optional_text;
use serde_json::{Map, Value};

use super::find::{Want, find};
use super::holds::release;
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::apply::{apply, op};
use crate::verbs::agent::base::opt::json_or_null;
use crate::verbs::agent::base::write;

/// The soft verbs: the op kind, how it reads in the output, and which agents a name may mean.
#[derive(Clone, Copy)]
pub enum Soft {
    /// `archive`.
    Archive,
    /// `unarchive`.
    Unarchive,
    /// `rm`.
    Remove,
    /// `restore`.
    Restore,
}

impl Soft {
    fn verb(self) -> &'static str {
        match self {
            Soft::Archive => "archive",
            Soft::Unarchive => "unarchive",
            Soft::Remove => "rm",
            Soft::Restore => "restore",
        }
    }

    fn kind(self) -> &'static str {
        match self {
            Soft::Remove => "delete",
            other => other.verb(),
        }
    }

    fn past(self) -> &'static str {
        match self {
            Soft::Archive => "archived",
            Soft::Unarchive => "unarchived",
            Soft::Remove => "moved to the trash",
            Soft::Restore => "restored",
        }
    }

    fn want(self) -> Want {
        match self {
            Soft::Archive | Soft::Remove => Want::Open,
            Soft::Unarchive => Want::Archived,
            Soft::Restore => Want::Deleted,
        }
    }
}

/// `coord agent archive`, `unarchive`, `rm` and `restore`.
///
/// # Errors
/// No agent named, a reason that is too long, no such agent, a refusal of the op.
pub fn run(call: &mut Call<'_>, soft: Soft) -> Outcome {
    if call.positional.is_empty() {
        return Err(CoordError::refuse(format!(
            "name the agent to {0}: coord agent {0} <A-id|name>.",
            soft.verb()
        )));
    }
    let reason = optional_text("reason", &json_or_null(&call.options, "reason"))?;
    write(call, |board, who, run| {
        for reference in run.positional {
            let id = find(board, Some(who), Some(reference), run.options, soft.want())?;
            let mut rest = Map::new();
            if let Some(why) = &reason {
                rest.insert("reason".to_owned(), Value::from(why.as_str()));
            }
            if matches!(soft, Soft::Remove) {
                release(board, who, &id)?;
            }
            let operation = op(soft.kind(), "agents", &id, rest, run.options)?;
            apply(board, who, &operation)?;
            let because = reason
                .as_ref()
                .map_or_else(String::new, |r| format!(" ({r})"));
            run.say(&format!(
                "{}: {}{because}",
                display(&Value::from(id)),
                soft.past()
            ));
        }
        Ok(())
    })
}
