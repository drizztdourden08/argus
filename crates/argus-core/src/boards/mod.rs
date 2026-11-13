//! The board identity table, `<home>/boards.json`: one id per board, its display name, its frozen
//! old keys (aliases) and its role. Code asks the table, never a key.

mod error;
mod lock;
mod mint;
mod model;
mod rename;
mod resolve;
mod role;
mod store;

pub use error::BoardsError;
pub use mint::mint;
pub use model::{BoardEntry, BoardId, Boards, Role};
pub use rename::rename;
pub use resolve::{display_name, resolve};
pub use role::{by_role, role};
pub use store::{load, save};

#[cfg(test)]
mod specs;
