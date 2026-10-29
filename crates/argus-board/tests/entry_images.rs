//! Pictures of a question or a decision: kept next to the board, numbered per entry, tied to an
//! option when asked, and shown to the page with a `url` instead of the local path.

mod common;

use argus_board::proofs::{Image, keep_entry_image};
use argus_board::sync::export_docs;
use argus_core::Coll;
use common::{alpha, doc};
use serde_json::{Value, json};

const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

fn question(options: &[&str]) -> Value {
    let mut b = alpha();
    b["entries"]["Q-2"] = doc(
        "Q-2",
        "ui",
        json!({
            "kind": "question", "item": "I-1", "thread": "ui", "author": "ui", "text": "Which look?",
            "needs": "decision", "options": options, "status": "open", "replyTo": null,
        }),
        3072,
    );
    b
}

fn image(caption: &str) -> Image<'_> {
    Image {
        bytes: &PNG,
        ext: ".PNG",
        caption,
    }
}

#[test]
fn images_are_numbered_per_entry_and_may_belong_to_an_option() {
    let home = tempfile::tempdir().expect("home");
    let mut b = question(&["Round", "Square"]);
    let q = &mut b["entries"]["Q-2"];
    let whole = keep_entry_image(home.path(), "alpha", q, (image("Whole"), None));
    assert_eq!(whole.ok(), Some(1));
    let tied = keep_entry_image(home.path(), "alpha", q, (image("Square"), Some(2)));
    assert_eq!(tied.ok(), Some(2));
    assert!(home.path().join("alpha/proofs/Q-2-1.png").is_file());
    assert!(home.path().join("alpha/proofs/Q-2-2.png").is_file());
    assert!(q["images"][0].get("option").is_none());
    assert_eq!(q["images"][1]["option"], 2);
}

#[test]
fn an_option_the_entry_lacks_a_bad_caption_or_a_bad_type_is_refused() {
    let home = tempfile::tempdir().expect("home");
    let mut b = question(&["Round", "Square"]);
    let q = &mut b["entries"]["Q-2"];
    assert!(keep_entry_image(home.path(), "alpha", q, (image("x"), Some(3))).is_err());
    assert!(keep_entry_image(home.path(), "alpha", q, (image("x"), Some(0))).is_err());
    assert!(keep_entry_image(home.path(), "alpha", q, (image("  "), None)).is_err());
    let text = Image {
        bytes: b"x",
        ext: ".txt",
        caption: "x",
    };
    assert!(keep_entry_image(home.path(), "alpha", q, (text, None)).is_err());
    assert!(q.get("images").is_none());
}

fn exported(board: &Value, id: &str) -> Value {
    export_docs(board, &|p| format!("/files/{}", p["n"]))
        .into_iter()
        .find(|d| d.coll == Coll::Entries && d.id == id)
        .and_then(|d| d.data)
        .unwrap_or(Value::Null)
}

#[test]
fn the_page_gets_a_url_and_the_option_but_not_the_local_path() {
    let home = tempfile::tempdir().expect("home");
    let mut b = question(&["Round", "Square"]);
    let q = &mut b["entries"]["Q-2"];
    keep_entry_image(home.path(), "alpha", q, (image("Square"), Some(2))).expect("kept");
    let shown = exported(&b, "Q-2")["images"][0].clone();
    assert_eq!(shown["url"], "/files/1");
    assert_eq!(shown["option"], 2);
    assert_eq!(shown["caption"], "Square");
    assert!(shown.get("file").is_none());
}

#[test]
fn an_entry_without_images_is_exported_as_it_is() {
    let b = question(&["Round", "Square"]);
    assert_eq!(exported(&b, "Q-2"), b["entries"]["Q-2"]);
}
