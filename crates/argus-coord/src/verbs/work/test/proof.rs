//! `coord test proof <T> <image> "<caption>" | rm <n>`. Proof images are kept next to the board
//! (`proofs/<T>-<n><ext>`) so they outlive the worktree they came from; the dashboard serves them
//! from there. Numbers only grow: a removed proof leaves a gap.

use std::fs;
use std::path::Path;

use argus_board::json::{array_mut, display, to_number};
use argus_board::proofs::{IMAGE, keep_proof_bytes};
use serde_json::{Value, json};

use super::edit::edit;
use crate::cli::Outcome;
use crate::cli::resolve::resolve;
use crate::verbs::work::run::{Res, Run, bad};

fn extension(src: &Path) -> String {
    src.extension().map_or_else(String::new, |e| {
        format!(".{}", e.to_string_lossy().to_lowercase())
    })
}

/// Keeps the image at `file` as proof number `n` of the test. `caption` is checked by the board.
pub(super) fn keep_proof(
    r: &Run<'_>,
    test: &mut Value,
    (file, caption): (&Value, Option<&str>),
) -> Res<i64> {
    let Some(path) = file.as_str() else {
        return Err(bad(format!(
            "The \"paths[0]\" argument must be of type string. Received {}",
            if file.is_null() {
                "undefined".to_owned()
            } else {
                format!("type boolean ({})", display(file))
            }
        )));
    };
    let src = resolve(&r.inv.cwd, Path::new(path));
    if !src.exists() {
        return Err(bad(format!("no image at {}", src.display())));
    }
    let ext = extension(&src);
    if !IMAGE.contains(&ext.as_str()) {
        return Err(bad(format!(
            "proof must be an image ({}).",
            IMAGE.join(" ")
        )));
    }
    let bytes = fs::read(&src).map_err(|e| bad(e.to_string()))?;
    keep_proof_bytes(
        &r.home,
        &r.project,
        test,
        &bytes,
        &ext,
        caption.unwrap_or_default(),
    )
}

fn remove_proof(test: &mut Value, spec: &Value) -> Res<String> {
    let n = to_number(Some(spec));
    let proofs = array_mut(test, "proofs");
    let before = proofs.len();
    proofs.retain(|p| p["n"].as_f64() != Some(n));
    let id = display(&test["id"]);
    if array_mut(test, "proofs").len() == before {
        let named = if spec.is_null() {
            String::new()
        } else {
            display(spec)
        };
        return Err(bad(format!("{id} has no proof {named}.")));
    }
    Ok(format!("{id}: proof {} removed.", display(&json!(n))))
}

/// `coord test proof`.
pub(super) fn proof(r: &mut Run<'_>) -> Outcome {
    edit(r, |r, test, _| {
        let line = if r.arg(1) == Some("rm") {
            remove_proof(test, &r.arg_json(2))?
        } else {
            let n = keep_proof(r, test, (&r.arg_json(1), r.arg(2)))?;
            format!(
                "{} proof {n} kept; it shows on the dashboard.",
                display(&test["id"])
            )
        };
        r.say(&[&line]);
        Ok(())
    })
}
