//! `path.resolve` of Node: an absolute path with `.` and `..` folded away, lexically, without
//! touching the disk.

use std::path::{Component, Path, PathBuf};

/// `p` made absolute against `base`, with no `.` or `..` segments and no trailing separator.
#[must_use]
pub fn resolve(base: &Path, p: &Path) -> PathBuf {
    let joined = base.join(p);
    let mut out = PathBuf::new();
    for part in joined.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if out.parent().is_some() {
                    out.pop();
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
