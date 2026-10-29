//! Proof images of tests and pictures of questions and decisions: kept next to the board so they
//! outlive the worktree they came from.

use std::path::Path;

use argus_core::clock::now_iso;
use argus_core::fs::write_atomic;
use serde_json::{Value, json};

use crate::error::{BoardError, refuse};
use crate::json::{array_mut, display};
use crate::limits::require_str;
use crate::store::board_dir;

const IMAGE: [&str; 5] = [".png", ".jpg", ".jpeg", ".gif", ".webp"];

/// One image to keep: its bytes, its extension (dot included) and its caption.
#[derive(Debug, Clone, Copy)]
pub struct Image<'a> {
    /// The file bytes.
    pub bytes: &'a [u8],
    /// The extension, dot included.
    pub ext: &'a str,
    /// What the picture shows.
    pub caption: &'a str,
}

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
    let image = Image {
        bytes,
        ext,
        caption,
    };
    keep_image(home, key, (test, "proofs"), (image, None))
}

/// Keep an image of a question or a decision entry as `<home>/<key>/proofs/<Q>-<n><ext>`, next
/// to the test proofs, and add it to the entry's `images`. `option` is the 1-based choice the
/// picture belongs to, `None` when it belongs to the entry as a whole. The caller stamps the
/// entry and saves the board. Returns the image number.
///
/// # Errors
/// Refused as [`keep_proof_bytes`] refuses, and when `option` names a choice the entry lacks.
pub fn keep_entry_image(
    home: &Path,
    key: &str,
    entry: &mut Value,
    (image, option): (Image<'_>, Option<i64>),
) -> Result<i64, BoardError> {
    if let Some(n) = option {
        let choices = entry["options"].as_array().map_or(0, Vec::len);
        if !usize::try_from(n).is_ok_and(|n| (1..=choices).contains(&n)) {
            return Err(refuse(format!(
                "option {n} does not exist: {} has {choices} option(s), numbered from 1.",
                display(&entry["id"])
            )));
        }
    }
    keep_image(home, key, (entry, "images"), (image, option))
}

fn keep_image(
    home: &Path,
    key: &str,
    (doc, field): (&mut Value, &str),
    (image, option): (Image<'_>, Option<i64>),
) -> Result<i64, BoardError> {
    let suffix = image.ext.to_lowercase();
    if !IMAGE.contains(&suffix.as_str()) {
        return Err(refuse(format!(
            "proof must be an image ({}).",
            IMAGE.join(" ")
        )));
    }
    let note = require_str("caption", image.caption)?;
    let latest = array_mut(doc, field)
        .iter()
        .filter_map(|p| p["n"].as_i64())
        .fold(0, i64::max);
    let n = latest + 1;
    let dir = board_dir(home, key).join("proofs");
    let dest = dir.join(format!("{}-{n}{suffix}", display(&doc["id"])));
    write_atomic(&dest, image.bytes)?;
    let mut record = json!({
        "n": n, "file": dest.to_string_lossy(), "caption": note, "url": null, "at": now_iso(),
    });
    if let Some(o) = option {
        record["option"] = json!(o);
    }
    array_mut(doc, field).push(record);
    Ok(n)
}
