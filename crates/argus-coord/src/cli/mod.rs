//! The command line layer: arguments, the router, the usage text, output helpers and the
//! environment of a call.

pub mod args;
pub mod capture;
pub mod env;
pub mod exit;
pub mod invocation;
pub mod out;
pub mod resolve;
pub mod route;
pub mod usage;

pub use args::{Args, Opt, Options, Val, parse_args};
pub use capture::Capture;
pub use env::Env;
pub use exit::{CoordError, NOT_PORTED, Outcome};
pub use invocation::Invocation;
pub use route::{Call, Entry, Handler, dispatch, module_of};
pub use usage::USAGE;
