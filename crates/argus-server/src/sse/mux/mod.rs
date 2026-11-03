//! The mux stream: one long-lived connection per browser that carries the doc scopes of every
//! tab. `GET /api/mux` opens it and its first frame is `hello {conn, v}`; `POST /api/mux/:conn`
//! takes a list of ops and answers one result per op, in order. The frames an op causes are
//! queued on the stream before the POST answers.
//!
//! A connection holds one fan-out client per scope (a `Handle`), all writing through the one
//! sender of the stream. When the stream body drops, the registry forgets the connection and
//! removes every client it held, gives back every presence claim and drops every run-log tail
//! (`tail` and `untail` ops, frames tagged with the op's `sub`).

mod conn;
mod ops;
mod parse;
mod presence;
mod registry;
mod tails;

pub use conn::Conn;
pub use registry::{MAX_CONNS, Registry as MuxRegistry};

/// The wire version sent in `hello`.
pub const VERSION: u32 = 1;
