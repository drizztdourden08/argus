//! The Argus HTTP server: one listener on `127.0.0.1`, the Host, peer, Origin and content-type
//! checks of DESIGN 8, the db shim and action routes, the doc stream with frames written by hand
//! so the bytes match the Node server, the admin routes and the web build.
//!
//! Each service has one module in `services` (build, start, stop), one in `adapters` (its crate's
//! traits over the stores), one in `routes` (registered in [`routes::register_all`]) and one
//! test file; `docs/architecture.md` has the layout.
#![deny(clippy::cognitive_complexity)]

pub mod adapters;
pub mod boards;
mod body;
mod ctx;
mod gate;
pub mod instance;
mod pathing;
mod reply;
mod router;
pub mod routes;
pub mod security;
mod server;
mod services;
pub mod sse;
mod web;

pub use ctx::{AuthKind, Ctx};
pub use reply::{html_page, json_reply};
pub use router::{AuthMode, Handler, RouteMatch, RouteOpts, Router};
pub use server::{ServeError, ServeOptions, ServerHandle, serve};
pub use services::{ProcsOptions, RunsOptions, Services, WorktreesOptions};
pub use web::{PageVars, WebAssets, render_page};
