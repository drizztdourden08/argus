//! Proof images of tests: kept next to the board so they outlive the worktree they came from.

use std::fs;
use std::path::Path;

use argus_core::clock::now_iso;
use serde_json::{Value, json};

use crate::error::{BoardError, refuse};
use crate::json::{array_mut, display};
use crate::limits::require_str;
use crate::store::board_dir;

const IMAGE: [&str; 5] = [".png", ".jpg", ".jpeg", ".gif", ".webp"];

/// Keep the bytes of a proof image as `<home>/<key>/proofs/<T>-<n><ext>` and add the proof to the
/// test. Numbers only grow (a removed proof leaves a gap) so each file name keeps pointing at its
/// own entry. `ext` includes the dot. The caller stamps the test and saves the board. Returns the
/// proof number.
///
/// # Errors
/// Refused when the extension is not an image or the caption is empty or too long; the file
/// cannot be written.
pub fn keep_proof_bytes(
    home: &Path,
    key: &str,
    test: &mut Value,
    bytes: &[u8],
    ext: &str,
    caption: &str,
) -> Result<i64, BoardError> {
    let suffix = ext.to_lowercase();
    if !IMAGE.contains(&suffix.as_str()) {
        return Err(refuse(format!(
            "proof must be an image ({}).",
            IMAGE.join(" ")
        )));
    }
    let note = require_str("caption", caption)?;
    let latest = array_mut(test, "proofs")
        .iter()
        .filter_map(|p| p["n"].as_i64())
        .fold(0, i64::max);
    let n = latest + 1;
    let dir = board_dir(home, key).join("proofs");
    fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{}-{n}{suffix}", display(&test["id"])));
    fs::write(&dest, bytes)?;
    array_mut(test, "proofs").push(json!({
        "n": n, "file": dest.to_string_lossy(), "caption": note, "url": null, "at": now_iso(),
    }));
    Ok(n)
}
