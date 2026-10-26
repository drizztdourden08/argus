//! WP3: the verbs of the threads and the board: `thread` (and `dispatch`, its `add`), `board`,
//! `purge` (with `restore` and `trash`), `check`, `tick` and `sync import`. Each verb is its own
//! module; the pieces they share are the call frame, the doc readers, the new-thread doc and the
//! lock hand-over lines.

mod board;
mod check;
mod dispatch;
mod docs;
mod frame;
mod newthread;
mod passed;
mod purge;
mod sync;
mod threads;
mod tick;

pub use board::run as run_board;
pub use check::run as run_check;
pub use dispatch::run as run_dispatch;
pub use purge::run as run_purge;
pub use sync::run as run_sync;
pub use threads::run as run_thread;
pub use tick::run as run_tick;
