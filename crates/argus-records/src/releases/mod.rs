//! The release store and its update feed.

mod feed;
mod kind;
pub mod range;
mod record;
pub mod semver;
mod shipped;
mod store;
mod view;
mod yml;

pub use feed::{MANIFESTS, Served};
pub use kind::content_type;
pub use record::{Asset, Package, Place, Release, parse};
pub use shipped::{Merged, Shipped};
pub use store::ReleaseStore;
