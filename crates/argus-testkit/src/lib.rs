//! One harness for the Argus integration tests: the fixture boards of the Node
//! `test/fixtures/make-home.mjs`, a server on an ephemeral port with throwaway homes, a small
//! HTTP client that can send any `Host`, and an SSE reader.
#![deny(clippy::cognitive_complexity)]

mod client;
mod coord_cli;
mod fixture;
mod png;
mod server;
mod stream;

pub use client::{Reply, Request, request, request_at};
pub use coord_cli::{CliOut, CoordCli, argus_exe};
pub use fixture::{ago, alpha_board, beta_board, make_home, portfolio_board, write_board};
pub use png::png;
pub use server::{TestOptions, TestServer, Tweak};
pub use stream::{Event, SseReader};
