//! A thread doc as every collection carries it: who made it, revision 1, last in the manual
//! order (`newThread` of `dispatch.mjs`, standing in for `createDoc`).

use argus_board::Board;
use argus_board::json::map_mut;
use argus_board::store::{rank_end, touch};
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Map, Value};

/// Adds the thread `name` to the board with `fields` after `id`, `name` and `role`, then the
/// bookkeeping. A field named `role` replaces the default in place.
pub fn new_thread(board: &mut Board, name: &str, fields: Vec<(&str, Value)>, actor: &str) {
    let at = now_iso();
    let rank = js::number(rank_end(board, "threads"));
    let mut thread = Map::new();
    thread.insert("id".into(), name.into());
    thread.insert("name".into(), name.into());
    thread.insert("role".into(), "thread".into());
    for (key, value) in fields {
        thread.insert(key.to_owned(), value);
    }
    thread.insert("registeredAt".into(), at.clone().into());
    thread.insert("createdBy".into(), actor.into());
    thread.insert("updatedAt".into(), at.into());
    thread.insert("updatedBy".into(), actor.into());
    thread.insert("rev".into(), 1.into());
    thread.insert("rank".into(), rank);
    map_mut(board, "threads").insert(name.to_owned(), Value::Object(thread));
    touch(board, "threads", name);
}
