//! The item type registry: the one file (`registry/item-types.json`) that names the item types,
//! their stages and lanes, fields, content kinds, limits, tabs and people, and the code, icon,
//! colour, tabs and links of every kind of record the app shows ([`Kind`]). It is compiled into
//! the binary and parsed once. The item ops, the verbs and the web read their rules from here
//! ([`registry`] gives the shared copy); nothing else in the code spells a stage or a limit.

#[macro_use]
mod shown;

mod kinds;
mod load;
mod lookup;
mod model;
mod verify;

#[cfg(test)]
mod schema;

pub use kinds::{Family, Kind, Look};
pub use load::{RegistryError, parse, registry};
pub use model::{
    BlocksWhile, Card, Collection, Content, ContentType, DoneRule, Field, Forbidden, Gate, Group,
    ItemType, Lead, LineKind, Link, Mode, Owner, Pattern, PatternKind, People, Plain, Progress,
    Record, RefKind, Registry, RichKind, Stage, Store, SubItem, View,
};
