//! The behaviour specs of the worktrees crate, ported from the Node dashboard's tests (`worktrees.scan`,
//! `worktrees.service`, `worktrees.leftover`, `worktrees.remove`, `worktrees.rules`, `worktrees.safety`,
//! `worktrees.fallback`). They run on temporary folders and repositories the tests create.

mod fallback;
mod holders;
mod hygiene;
mod leftover;
mod links;
mod remove;
mod requests;
mod review;
mod round2;
mod rules;
mod safety;
mod scan;
mod service;
mod slow;
mod stuck;
mod wire;
