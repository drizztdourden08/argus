//! The release store and its update feed.

mod feed;
pub mod range;
mod record;
pub mod semver;
mod store;
mod view;
mod yml;

pub use feed::{MANIFESTS, Served};
pub use record::{Asset, Package, Place, Release, parse};
pub use store::ReleaseStore;
