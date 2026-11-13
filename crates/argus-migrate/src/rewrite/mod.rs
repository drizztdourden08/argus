//! The whitelist rewrites: only the fields named here change, in a board and in the portfolio.

pub mod board;
pub mod portfolio;
pub mod strings;
mod walk;

pub use walk::rekey;
