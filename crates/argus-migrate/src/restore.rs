//! `--restore <backup>`: checks the backup against its manifest, renames the current home aside
//! (never deletes it), copies the backup in its place and checks the copy. Writes made since
//! the backup are in the aside folder, not in the new home.

use std::path::Path;

use crate::error::Result;
use crate::options::Options;
use crate::{backup, precheck, relocate};

fn is_not_restored(rel: &str) -> bool {
    rel == backup::MANIFEST
        || rel == "dashboard/dashboard.db-wal"
        || rel == "dashboard/dashboard.db-shm"
}

/// Puts the backup back in place of the home.
pub fn run(opts: &Options, from: &Path) -> Result<Vec<String>> {
    precheck::server_down(opts)?;
    backup::check_folder(from)?;
    let manifest = backup::read_manifest(from)?;
    let aside = backup::sibling(&opts.home, "aside");
    relocate::rename_retry(&opts.home, &aside)?;
    backup::copy_tree(from, &opts.home, &is_not_restored)?;
    backup::against(&opts.home, &manifest)?;
    Ok(vec![
        format!("restored {} files from {}", manifest.files.len(), from.display()),
        format!("the previous home is kept at {}", aside.display()),
    ])
}
