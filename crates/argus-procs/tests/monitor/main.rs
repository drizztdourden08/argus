//! The process monitor over a fake machine: rules, status, protection, docs, alerts, kills, a
//! thread's own kills and the reaper.

mod alerts;
mod harness;
mod kill;
mod reaper;
mod rules;
mod scope;
mod self_kill;
mod status;
