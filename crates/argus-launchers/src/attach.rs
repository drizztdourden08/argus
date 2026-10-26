//! Run attachments: a screenshot or image the owner (page) or an agent (MCP) attaches to a
//! run. The bytes are checked by magic number, never by the claimed type, and stored under
//! `<dir>/<scope>/<id>.<ext>`; the store keeps the row with the absolute path.

use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::RunError;

const CAPTION_MAX: usize = 120;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentRow {
    pub id: String,
    pub n: u64,
    pub path: String,
    pub mime: String,
    pub bytes: u64,
    pub sha256: String,
    pub caption: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

/// What a request to save an attachment carries.
pub struct SaveRequest<'a> {
    pub dir: &'a Path,
    pub scope: &'a str,
    pub bytes: &'a [u8],
    pub mime: &'a str,
    pub max_bytes: usize,
    pub n: u64,
    pub caption: &'a str,
    pub by: &'a str,
    pub at: &'a str,
}

/// `(mime, extension)` for png, jpeg, gif and webp bytes.
pub fn sniff_image(b: &[u8]) -> Option<(&'static str, &'static str)> {
    if b.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some(("image/png", ".png"))
    } else if b.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(("image/jpeg", ".jpg"))
    } else if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        Some(("image/gif", ".gif"))
    } else if b.len() >= 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        Some(("image/webp", ".webp"))
    } else {
        None
    }
}

fn clean_caption(caption: &str) -> Option<String> {
    let mut out = String::new();
    let mut in_control = false;
    for c in caption.chars() {
        if c.is_control() {
            if !in_control {
                out.push(' ');
            }
            in_control = true;
        } else {
            out.push(c);
            in_control = false;
        }
    }
    let clipped: String = out.trim().chars().take(CAPTION_MAX).collect();
    (!clipped.is_empty()).then_some(clipped)
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn check_claim(claimed: &str, found: &str) -> Result<(), RunError> {
    let claimed = claimed
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let claimed = claimed.replace("image/jpg", "image/jpeg");
    if claimed.is_empty() || claimed == "application/octet-stream" || claimed == found {
        Ok(())
    } else {
        Err(RunError::unsupported_media(format!(
            "the bytes are {found}, not {claimed}"
        )))
    }
}

fn new_id() -> String {
    let mut random = [0_u8; 8];
    if getrandom::fill(&mut random).is_err() {
        random = argus_core::clock::now_ms().to_le_bytes();
    }
    format!("ra-{}", hex(&random))
}

/// Checks and writes the file, returning the row the caller stores.
pub fn save_attachment(req: &SaveRequest<'_>) -> Result<AttachmentRow, RunError> {
    if req.bytes.is_empty() {
        return Err(RunError::bad_request("the attachment is empty"));
    }
    if req.bytes.len() > req.max_bytes {
        return Err(RunError::too_large(format!(
            "the attachment is {} bytes, the limit is {}",
            req.bytes.len(),
            req.max_bytes
        )));
    }
    let (found, ext) = sniff_image(req.bytes).ok_or_else(|| {
        RunError::unsupported_media("only png, jpeg, gif and webp images can be attached")
    })?;
    check_claim(req.mime, found)?;
    let id = new_id();
    let folder = req.dir.join(req.scope);
    let path = folder.join(format!("{id}{ext}"));
    std::fs::create_dir_all(&folder)
        .and_then(|()| argus_core::fs::write_atomic(&path, req.bytes))
        .map_err(|e| {
            RunError::new(
                500,
                "internal",
                format!("the attachment was not saved: {e}"),
            )
        })?;
    Ok(AttachmentRow {
        id,
        n: req.n,
        path: path.to_string_lossy().into_owned(),
        mime: found.to_owned(),
        bytes: u64::try_from(req.bytes.len()).unwrap_or(u64::MAX),
        sha256: hex(&Sha256::digest(req.bytes)),
        caption: clean_caption(req.caption),
        created_by: req.by.to_owned(),
        created_at: req.at.to_owned(),
    })
}

/// Best effort: a missing file must not stop a retention sweep.
pub(crate) fn remove_file(path: &str) {
    let _ = std::fs::remove_file(path);
}

pub(crate) fn url_of(scope: &str, id: &str) -> String {
    format!("/files/{scope}/a/{id}")
}
