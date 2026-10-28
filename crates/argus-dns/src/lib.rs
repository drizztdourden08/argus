//! The tiny DNS responder behind the local names of Argus. It answers A queries for the suffix
//! and everything under it with the one loopback address of Argus, AAAA and other types of those
//! names with an empty answer, and refuses every other name: no recursion, no forwarding. The
//! wire format is hand-parsed for exactly one question (`wire`); `zone` decides the answer;
//! `responder` owns the UDP and TCP sockets; `client` asks one question, for `argus dns status`
//! and the tests.
#![deny(clippy::cognitive_complexity)]

mod client;
mod responder;
mod wire;
mod zone;

pub use client::{ask, resolve};
pub use responder::Responder;
pub use wire::{Answer, TYPE_A, TYPE_AAAA};
pub use zone::Zone;
