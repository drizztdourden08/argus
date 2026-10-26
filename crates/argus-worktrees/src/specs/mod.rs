//! The behaviour specs of the worktrees crate, ported from the Node dashboard's tests (`worktrees.scan`,
//! `worktrees.service`, `worktrees.leftover`, `worktrees.remove`, `worktrees.rules`, `worktrees.safety`,
//! `worktrees.fallback`). They run on temporary folders and repositories the tests create.

mod fallback;
mod leftover;
mod remove;
mod rules;
mod safety;
mod scan;
mod service;
