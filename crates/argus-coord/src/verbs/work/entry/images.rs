//! `--image [N=]<file> "<caption>"` on `coord ask` and `coord decide`: pictures the owner opens
//! and zooms in the question card. The captions are the words after the question, one per
//! `--image` in the order given. `N=` ties the picture to option N (counted from 1); without it
//! the picture belongs to the whole entry. The files are kept like test proofs
//! (`proofs/<Q>-<n><ext>`) and listed in the entry's `images`.

use std::fs;
use std::path::Path;

use argus_board::Board;
use argus_board::json::doc_mut;
use argus_board::proofs::{Image, keep_entry_image};

use crate::cli::Val;
use crate::cli::resolve::resolve;
use crate::verbs::work::run::{Res, Run, bad};

const IMAGE: [&str; 5] = [".png", ".jpg", ".jpeg", ".gif", ".webp"];
const MAX_BYTES: u64 = 10_485_760;
const SYNTAX: &str = "--image [N=]<file> \"<caption>\"";

/// One picture read from disk, ready to be kept once the entry has its id.
pub(super) struct Picked {
    bytes: Vec<u8>,
    ext: String,
    caption: String,
    option: Option<i64>,
}

/// `2=shot.png` is option 2 and `shot.png`; a Windows path such as `C:\shot.png` has no number
/// before its `=`, so it is never cut.
fn split_option(spec: &str) -> (Option<i64>, &str) {
    match spec.split_once('=') {
        Some((n, file)) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => {
            (n.parse().ok(), file)
        }
        _ => (None, spec),
    }
}

fn read_one(r: &Run<'_>, spec: &Val, caption: &str) -> Res<Picked> {
    let Val::Text(spec) = spec else {
        return Err(bad(format!("--image needs a file: {SYNTAX}.")));
    };
    let (option, file) = split_option(spec);
    let src = resolve(&r.inv.cwd, Path::new(file));
    if !src.exists() {
        return Err(bad(format!("no image at {}", src.display())));
    }
    let ext = src.extension().map_or_else(String::new, |e| {
        format!(".{}", e.to_string_lossy().to_lowercase())
    });
    if !IMAGE.contains(&ext.as_str()) {
        return Err(bad(format!("an image must be one of {}.", IMAGE.join(" "))));
    }
    let bytes = fs::read(&src).map_err(|e| bad(e.to_string()))?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_BYTES {
        return Err(bad(format!(
            "{} is {} bytes, an image is 1 to {MAX_BYTES} bytes.",
            src.display(),
            bytes.len()
        )));
    }
    Ok(Picked {
        bytes,
        ext,
        caption: caption.to_owned(),
        option,
    })
}

/// Every `--image` of the call, read and checked before the board is touched. The captions are
/// the arguments after the question, one per image. `choices` is how many options the entry has,
/// so a picture tied to a missing option is refused before any file is written.
pub(super) fn picked(r: &Run<'_>, choices: usize) -> Res<Vec<Picked>> {
    if r.options.flag("image") {
        return Err(bad(format!("--image needs a file: {SYNTAX}.")));
    }
    let specs = r.options.many("image");
    if specs.is_empty() {
        return Ok(Vec::new());
    }
    let captions = r.args.get(2..).unwrap_or_default();
    if captions.len() != specs.len() {
        return Err(bad(format!(
            "each image needs a caption in quotes after the question, in the same order: {SYNTAX}. Got {} image(s) and {} caption(s).",
            specs.len(),
            captions.len()
        )));
    }
    let pics: Vec<Picked> = specs
        .iter()
        .zip(captions)
        .map(|(spec, caption)| read_one(r, spec, caption))
        .collect::<Res<_>>()?;
    for pic in &pics {
        let fits = |n: i64| usize::try_from(n).is_ok_and(|n| (1..=choices).contains(&n));
        if pic.option.is_some_and(|n| !fits(n)) {
            return Err(bad(format!(
                "an image can belong to option 1 to {choices} only (this entry has {choices} option(s))."
            )));
        }
    }
    Ok(pics)
}

/// Keeps the pictures on the entry just created.
pub(super) fn keep_all(board: &mut Board, r: &Run<'_>, id: &str, pics: &[Picked]) -> Res<()> {
    let Some(entry) = doc_mut(board, "entries", id) else {
        return Ok(());
    };
    for pic in pics {
        let image = Image {
            bytes: &pic.bytes,
            ext: &pic.ext,
            caption: &pic.caption,
        };
        keep_entry_image(&r.home, &r.project, entry, (image, pic.option))?;
    }
    Ok(())
}

/// The note appended to the confirmation line.
pub(super) fn said(pics: &[Picked]) -> String {
    match pics.len() {
        0 => String::new(),
        n => format!(" with {n} image(s)"),
    }
}
