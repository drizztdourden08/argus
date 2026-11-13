//! Who is calling and for which project (`project.mjs`): the project lookup, the caller of a
//! verb, the session registry scan and the folder lookups.

pub mod board_ref;
pub mod caller;
pub mod migrating;
pub mod path;
pub mod project;
pub mod session;

pub use board_ref::{board_ref, shown};
pub use caller::{Caller, caller, me, resolve_thread};
pub use migrating::{migrating, refuse_while_migrating};
pub use path::git_root;
pub use project::find_project;
pub use session::{ENDED, is_ended, project_of_session};
