//! The behaviour specs of the worktrees crate, ported from the Node dashboard's tests (`worktrees.scan`,
//! `worktrees.service`, `worktrees.leftover`, `worktrees.remove`, `worktrees.rules`, `worktrees.safety`,
//! `worktrees.fallback`). They run on temporary folders and repositories the tests create.
//!
//! Every file carries one `test-tag` line. The `critical` and `standard` files run on temporary folders, in-memory facts
//! and the fake git (`crate::fakegit`, through `testkit::Fixture::fake`). The `e2e` files run the real git through the
//! service harness and are ignored by a plain `cargo test`.

mod auto_rules;
mod exec_real;
mod fallback;
mod holders;
mod hygiene;
mod hygiene_approve;
mod hygiene_busy;
mod hygiene_ask;
mod hygiene_gone;
mod hygiene_kit;
mod hygiene_rule_error;
mod leftover;
mod leftover_restart;
mod links;
mod links_fs;
mod probe;
mod registered;
mod release_probe;
mod remove;
mod remove_links;
mod remove_real;
mod request_kit;
mod requests;
mod review;
mod round2;
mod rules;
mod rules_walk;
mod safety;
mod scan;
mod scan_git;
mod scan_kit;
mod scratch;
mod service;
mod service_status;
mod slow;
mod stale_lock;
mod stuck;
mod submodules;
mod turns;
mod wire;
mod wire_answer;
mod worktree_class;
