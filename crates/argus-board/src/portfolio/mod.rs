//! The portfolio (shown as Argus, the overview of every project): one level above every project
//! board. Its store is `<home>/_portfolio/board.json`, board-shaped so locking is shared. It
//! keeps the registry of projects and the entries the owner posted on the Argus page; the
//! project cards are computed from every project's board.

mod cards;
mod deliver;
mod registry;

pub use cards::summarize;
pub use deliver::{PortfolioImport, dispatch_entry, import_portfolio_entries};
pub use registry::{project_keys, shape};
