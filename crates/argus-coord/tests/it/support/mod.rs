//! Test support shared by the coord test binaries: the differential harness against the Node
//! CLI and the latency timing helpers. The `it` binary declares it once in `main.rs` and the
//! latency bench pulls it in by path; each uses only part of it, so the dead code and unused
//! import lints are off for this module tree.
#![allow(dead_code, unused_imports)]

pub mod differ;
pub mod latency;
