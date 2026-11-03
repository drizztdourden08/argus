//! Proof images of tests and pictures of questions, decisions and items: kept next to the board so
//! they outlive the worktree they came from.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use argus_core::clock::now_iso;
use argus_core::fs::write_atomic;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::error::{BoardError, refuse};
use crate::json::{array_mut, display};
use crate::limits::require_str;
use crate::store::board_dir;

/// The extensions a kept image may have, dot included.
pub const IMAGE: [&str; 5] = [".png", ".jpg", ".jpeg", ".gif", ".webp"];

/// The most bytes a kept image may have (10 MiB, as `attach.maxBytes`).
pub const MAX_BYTES: u64 = 10_485_760;

/// The most images one item keeps.
pub const MAX_PER_ITEM: usize = 12;

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
    append_image(home, key, (test, "proofs"), (image, None))
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
    append_image(home, key, (entry, "images"), (image, option))
}

fn append_image(
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

/// Where an image of an item was kept: its number on the item, its file and its hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRef {
    /// The number on the item (`img:<n>` in its rich text); numbers only grow.
    pub n: i64,
    /// The file, `<home>/<key>/proofs/<I-n>-<n><ext>`.
    pub file: String,
    /// The sha256 of the bytes, 64 hex digits.
    pub sha256: String,
    /// The item already held this picture: nothing was written and `n` is the earlier one.
    pub duplicate: bool,
}

impl ImageRef {
    /// The record an item lists under `images`: `{n, file, caption, sha256, at, by}`.
    #[must_use]
    pub fn record(&self, caption: &str, by: &str) -> Value {
        json!({
            "n": self.n, "file": self.file, "caption": caption, "sha256": self.sha256,
            "at": now_iso(), "by": by,
        })
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn extension_of(path: &Path) -> String {
    path.extension().map_or_else(String::new, |e| {
        format!(".{}", e.to_string_lossy().to_lowercase())
    })
}

fn read_checked(path: &Path) -> Result<(Vec<u8>, String), BoardError> {
    let ext = extension_of(path);
    if !IMAGE.contains(&ext.as_str()) {
        return Err(refuse(format!(
            "an image must be one of {}.",
            IMAGE.join(" ")
        )));
    }
    if !path.is_file() {
        return Err(refuse(format!("no image at {}", path.display())));
    }
    let bytes = fs::read(path)?;
    check_size(&bytes, &path.display().to_string())?;
    Ok((bytes, ext))
}

fn check_size(bytes: &[u8], what: &str) -> Result<(), BoardError> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_BYTES {
        return Err(refuse(format!(
            "{what} is {} bytes, an image is 1 to {MAX_BYTES} bytes.",
            bytes.len()
        )));
    }
    Ok(())
}

fn listed(item: &Value) -> &[Value] {
    item["images"].as_array().map_or(&[][..], Vec::as_slice)
}

/// Keep the file at `path` as an image of `item` (a doc of `items`):
/// `<home>/<key>/proofs/<I-n>-<n><ext>`, numbered after the item's highest image (a removed one
/// leaves a gap). The same bytes already on the item are not kept again: the earlier image comes
/// back with `duplicate` set and nothing is written. The item is not changed; the caller adds
/// [`ImageRef::record`] to its `images` (unless `duplicate`), stamps the item and saves the board.
///
/// # Errors
/// Refused when the file is missing, is not png, jpg, jpeg, gif or webp, is empty or over 10 MiB,
/// or the item already holds 12 images; the file cannot be written.
pub fn keep_image(
    home: &Path,
    key: &str,
    item: &Value,
    path: &Path,
) -> Result<ImageRef, BoardError> {
    let (bytes, ext) = read_checked(path)?;
    keep_image_bytes(home, key, item, (&bytes, &ext))
}

/// [`keep_image`] for bytes already in hand, for example an upload: `ext` includes the dot.
///
/// # Errors
/// Refused as [`keep_image`] refuses, for the bytes and the extension given.
pub fn keep_image_bytes(
    home: &Path,
    key: &str,
    item: &Value,
    (bytes, ext): (&[u8], &str),
) -> Result<ImageRef, BoardError> {
    let ext = ext.to_lowercase();
    if !IMAGE.contains(&ext.as_str()) {
        return Err(refuse(format!(
            "an image must be one of {}.",
            IMAGE.join(" ")
        )));
    }
    check_size(bytes, "the image")?;
    let sha256 = hex(&Sha256::digest(bytes));
    let kept = listed(item);
    if let Some(same) = kept.iter().find(|i| i["sha256"].as_str() == Some(&sha256)) {
        return Ok(ImageRef {
            n: same["n"].as_i64().unwrap_or_default(),
            file: display(&same["file"]),
            sha256,
            duplicate: true,
        });
    }
    if kept.len() >= MAX_PER_ITEM {
        return Err(refuse(format!(
            "{} already has {MAX_PER_ITEM} images; remove one first.",
            display(&item["id"])
        )));
    }
    let latest = kept
        .iter()
        .filter_map(|i| i["n"].as_i64())
        .fold(0, i64::max);
    let n = latest + 1;
    let dest = board_dir(home, key)
        .join("proofs")
        .join(format!("{}-{n}{ext}", display(&item["id"])));
    write_atomic(&dest, bytes)?;
    Ok(ImageRef {
        n,
        file: dest.to_string_lossy().into_owned(),
        sha256,
        duplicate: false,
    })
}

/// [`keep_image`] and the item's `images` entry in one step: the image is listed with its caption
/// and the actor, unless the item held it already. The caller stamps the item and saves the board.
///
/// # Errors
/// Refused as [`keep_image`] refuses, and when the caption is empty or too long.
pub fn add_item_image(
    home: &Path,
    key: &str,
    item: &mut Value,
    (path, caption, by): (&Path, &str, &str),
) -> Result<ImageRef, BoardError> {
    let note = require_str("caption", caption)?;
    let kept = keep_image(home, key, item, path)?;
    if !kept.duplicate {
        array_mut(item, "images").push(kept.record(&note, by));
    }
    Ok(kept)
}

/// [`add_item_image`] for bytes already in hand, for example an upload: `ext` includes the dot.
///
/// # Errors
/// Refused as [`add_item_image`] refuses, for the bytes and the extension given.
pub fn add_item_image_bytes(
    home: &Path,
    key: &str,
    item: &mut Value,
    (bytes, ext): (&[u8], &str),
    (caption, by): (&str, &str),
) -> Result<ImageRef, BoardError> {
    let note = require_str("caption", caption)?;
    let kept = keep_image_bytes(home, key, item, (bytes, ext))?;
    if !kept.duplicate {
        array_mut(item, "images").push(kept.record(&note, by));
    }
    Ok(kept)
}
