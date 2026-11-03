//! `--image <file> "<caption>"` on `coord item add` and `coord item set`: pictures the owner
//! opens in the item's pane. The captions are the words after the title (add) or after the item
//! id (set), one per `--image` in the order given. The board keeps the file (png, jpg, gif or
//! webp, 10 MiB at most, 12 per item, the same bytes once) and lists it in the item's `images`.

use std::path::PathBuf;

use argus_board::Board;
use argus_board::json::doc_mut;
use argus_board::proofs::add_item_image;
use argus_board::store::stamp;

use crate::cli::resolve::resolve;
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

const SYNTAX: &str = "--image <file> \"<caption>\"";

/// One picture to keep once the item exists.
pub(super) struct Pending {
    path: PathBuf,
    caption: String,
}

/// Every `--image` of the call with its caption, the captions being the arguments from
/// `first` on. Nothing is read yet.
///
/// # Errors
/// Refused for an `--image` with no file, a missing file, or captions that do not match the
/// images one to one.
pub(super) fn picked(r: &Run<'_>, first: usize) -> Res<Vec<Pending>> {
    if r.options.flag("image") {
        return Err(bad(format!("--image needs a file: {SYNTAX}.")));
    }
    let files = r.options.many("image");
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let captions = r.args.get(first..).unwrap_or_default();
    if files.len() != captions.len() {
        return Err(bad(format!(
            "each image needs a caption in quotes after the {}, in the same order: {SYNTAX}. Got {} image(s) and {} caption(s).",
            if first == 1 { "title or id" } else { "words" },
            files.len(),
            captions.len()
        )));
    }
    files
        .iter()
        .zip(captions)
        .map(|(file, caption)| {
            let crate::cli::Val::Text(name) = file else {
                return Err(bad(format!("--image needs a file: {SYNTAX}.")));
            };
            let path = resolve(&r.inv.cwd, std::path::Path::new(name));
            if !path.exists() {
                return Err(bad(format!("no image at {}", path.display())));
            }
            Ok(Pending {
                path,
                caption: caption.clone(),
            })
        })
        .collect()
}

/// Keeps the pictures on the item and stamps it.
///
/// # Errors
/// Refused as the board refuses a picture: the kind, the size, the count, the caption.
pub(super) fn keep_all(
    board: &mut Board,
    r: &Run<'_>,
    (id, who): (&str, &Caller),
    pictures: &[Pending],
) -> Res<()> {
    if pictures.is_empty() {
        return Ok(());
    }
    let Some(item) = doc_mut(board, "items", id) else {
        return Err(bad(format!("no item \"{id}\".")));
    };
    for pic in pictures {
        add_item_image(
            &r.home,
            &r.project,
            item,
            (pic.path.as_path(), pic.caption.as_str(), who.actor.as_str()),
        )?;
    }
    stamp(board, "items", id, &who.actor)?;
    Ok(())
}
