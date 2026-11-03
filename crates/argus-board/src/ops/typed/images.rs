//! The `images` list of an item: its shape, and the numbers a rich text may name.

use serde_json::{Value, json};

use crate::content::{Context, check as content_check};
use crate::error::{BoardError, refuse};
use crate::json::set;

/// The numbers of the images of a record.
pub(crate) fn numbers(images: &Value) -> Vec<u32> {
    images
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| i["n"].as_u64())
        .filter_map(|n| u32::try_from(n).ok())
        .collect()
}

/// The list checked: each image has a number, a file, a caption and a sha256, and no number
/// repeats. A blank value is an empty list.
pub(crate) fn check(v: &Value) -> Result<Value, BoardError> {
    if v.is_null() {
        return Ok(json!([]));
    }
    let Some(list) = v.as_array() else {
        return Err(refuse("images must be a list."));
    };
    let mut seen: Vec<u64> = Vec::new();
    let mut out: Vec<Value> = Vec::with_capacity(list.len());
    for image in list {
        out.push(one(image, &mut seen)?);
    }
    Ok(Value::Array(out))
}

fn one(image: &Value, seen: &mut Vec<u64>) -> Result<Value, BoardError> {
    let n = image["n"]
        .as_u64()
        .filter(|n| (1..=u64::from(u32::MAX)).contains(n))
        .ok_or_else(|| refuse("an image needs a number n above 0."))?;
    if seen.contains(&n) {
        return Err(refuse(format!("image {n} is listed twice.")));
    }
    seen.push(n);
    if !image["file"].as_str().is_some_and(|f| !f.trim().is_empty()) {
        return Err(refuse(format!("image {n} needs its file.")));
    }
    if !is_digest(&image["sha256"]) {
        return Err(refuse(format!(
            "image {n} needs its sha256 (64 hex digits)."
        )));
    }
    let caption = content_check("caption", &image["caption"], &Context::default())?;
    if caption.is_null() {
        return Err(refuse(format!("image {n} needs a caption.")));
    }
    let mut kept = image.clone();
    set(&mut kept, "caption", caption);
    Ok(kept)
}

fn is_digest(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()))
}
