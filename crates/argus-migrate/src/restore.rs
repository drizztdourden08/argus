//! `--restore <backup>`: checks the backup against its manifest, copies it into a sibling folder
//! and checks that copy, then, under the marker, renames the current home aside (never deletes
//! it) and renames the copy into its place. A hook never sees a half-copied home. Writes made
//! since the backup are in the aside folder, not in the new home.

use std::fs;
use std::path::Path;

use crate::error::{Result, stop};
use crate::options::Options;
use crate::{backup, marker, precheck, relocate};

fn is_not_restored(rel: &str) -> bool {
    rel == backup::MANIFEST
        || rel == "dashboard/dashboard.db-wal"
        || rel == "dashboard/dashboard.db-shm"
}

fn refuse_inside(home: &Path, from: &Path) -> Result<()> {
    let (home, from) = (fs::canonicalize(home)?, fs::canonicalize(from)?);
    if from.starts_with(&home) {
        return stop("the backup lies inside the home that would be set aside: move it out first");
    }
    Ok(())
}

fn swap(home: &Path, staged: &Path, aside: &Path) -> Result<()> {
    relocate::rename_retry(home, aside)?;
    if let Err(e) = relocate::rename_retry(staged, home) {
        relocate::rename_retry(aside, home).ok();
        return Err(e);
    }
    Ok(())
}

fn report(manifest: &backup::Manifest, from: &Path, aside: &Path) -> Vec<String> {
    let mut lines = vec![
        format!(
            "restored {} files from {}",
            manifest.files.len(),
            from.display()
        ),
        format!("the previous home is kept at {}", aside.display()),
    ];
    if !manifest.links.is_empty() {
        lines.push(format!(
            "{} links or junctions were not in the backup, recreate them by hand:",
            manifest.links.len()
        ));
        lines.extend(
            manifest
                .links
                .iter()
                .map(|l| format!("  {} -> {}", l.path, l.target)),
        );
    }
    lines
}

/// Puts the backup back in place of the home.
pub fn run(opts: &Options, from: &Path) -> Result<Vec<String>> {
    let home = &opts.home;
    precheck::server_down(opts)?;
    refuse_inside(home, from)?;
    backup::check_folder(from)?;
    let manifest = backup::read_manifest(from)?;
    let staged = backup::sibling(home, "restoring");
    backup::copy_tree(from, &staged, &is_not_restored)?;
    backup::against(&staged, &manifest)?;
    let aside = backup::sibling(home, "aside");
    let keep = marker::exists(home);
    marker::take(opts)?;
    marker::put(&staged)?;
    let swapped = precheck::server_down(opts).and_then(|()| swap(home, &staged, &aside));
    if let Err(e) = swapped {
        return Err(marker::release(home, keep, e));
    }
    marker::clear(home);
    Ok(report(&manifest, from, &aside))
}
