//! The Argus HTTP server: one listener on `127.0.0.1`, the Host, peer, Origin and content-type
//! checks of DESIGN 8, the db shim and action routes, the doc stream with frames written by hand
//! so the bytes match the Node server, the admin routes and the web build.
//!
//! Each slice adds its routes in its own module under `routes`, registered in
//! [`routes::register_all`].
#![deny(clippy::cognitive_complexity)]

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
pub use services::Services;
pub use web::{PageVars, WebAssets, render_page};
