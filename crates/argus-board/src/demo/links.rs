//! The requests between boards: from demo items to the Tessera, Brock and Archipelia boards (open
//! and done), and one from Brock to the demo board. The verb makes them with the dependency
//! contract of this crate, inside the same preview home as the demo board.

use super::{Step, on};

/// The other boards the requests go to. Each gets a board and a lead when the home has none.
pub const TARGETS: [&str; 3] = ["tessera", "brock", "archipelia"];

/// One request: the requester's project (empty is the demo board) and item (a bound name), the
/// project asked (empty is the demo board), and whether the asked side then closes it as done.
#[derive(Debug, Clone, Copy)]
pub struct DepSpec {
    /// The project of the requester item; empty is the demo board.
    pub from: &'static str,
    /// The name the requester item is bound to.
    pub item: &'static str,
    /// The project asked; empty is the demo board.
    pub target: &'static str,
    /// The title of the request.
    pub title: &'static str,
    /// The acceptance line of the request.
    pub ac: &'static str,
    /// Whether the asked side closes it as done.
    pub done: bool,
}

const fn spec(
    item: &'static str,
    target: &'static str,
    title: &'static str,
    ac: &'static str,
    done: bool,
) -> DepSpec {
    DepSpec {
        from: "",
        item,
        target,
        title,
        ac,
        done,
    }
}

/// The requests that start on the demo board.
pub const OUTGOING: [DepSpec; 6] = [
    spec(
        "s3",
        "tessera",
        "Avatar tile for the lobby seats",
        "The tile takes a name and a colour",
        false,
    ),
    spec(
        "s4",
        "tessera",
        "Chat bubble primitive",
        "One bubble per message",
        true,
    ),
    spec(
        "t2",
        "brock",
        "Lint rule for lobby ids",
        "The rule flags a raw lobby id",
        false,
    ),
    spec(
        "t3",
        "brock",
        "A gate for the seat table",
        "The gate runs the seat tests",
        true,
    ),
    spec(
        "s2",
        "archipelia",
        "Shared lobby fixtures",
        "Two lobbies with three players each",
        false,
    ),
    spec(
        "s1",
        "archipelia",
        "Seed data for the lobby list",
        "Ten lobbies in the seed",
        true,
    ),
];

/// The request that comes in: a Brock item asks the demo board.
pub const INCOMING: DepSpec = DepSpec {
    from: "brock",
    item: "brock_task",
    target: "",
    title: "Lobby API for the Brock tests",
    ac: "A read only list of lobbies",
    done: false,
};

/// The Brock item the incoming request starts from, with the epic it needs.
pub(super) const INCOMING_ITEM: &[Step] = &[
    on(
        "brock",
        "lead",
        "brock_epic",
        &[
            "item",
            "add",
            "Demo requests",
            "--type",
            "epic",
            "--stage",
            "active",
        ],
    ),
    on(
        "brock",
        "lead",
        "brock_task",
        &[
            "item",
            "add",
            "Test against the demo lobbies",
            "--type",
            "task",
            "--parent",
            "{brock_epic}",
            "--stage",
            "todo",
        ],
    ),
];
