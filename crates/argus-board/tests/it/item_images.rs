//! Pictures of an item: kept next to the board as `proofs/<I-n>-<n><ext>`, numbered per item, one
//! copy of the same bytes, 10 MB and 12 pictures at most. I-70 P3b.

// test-tag: critical

use std::fs;
use std::path::PathBuf;

use crate::common::alpha;
use argus_board::proofs::{
    MAX_BYTES, MAX_PER_ITEM, add_item_image, add_item_image_bytes, keep_image,
};
use serde_json::{Value, json};

const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

fn picture(dir: &tempfile::TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, bytes).expect("picture");
    path
}

fn item() -> Value {
    alpha()["items"]["I-1"].clone()
}

fn count(item: &Value) -> usize {
    item["images"].as_array().map_or(0, Vec::len)
}

#[test]
fn bytes_from_an_upload_are_kept_listed_and_deduplicated() {
    let home = tempfile::tempdir().expect("home");
    let mut it = item();
    let up = (&PNG[..], ".PNG");
    let first = add_item_image_bytes(home.path(), "alpha", &mut it, up, ("Shot", "human"));
    assert_eq!(first.expect("first").n, 1);
    let same = add_item_image_bytes(home.path(), "alpha", &mut it, up, ("Twice", "human"));
    let same = same.expect("same");
    assert_eq!((same.n, same.duplicate), (1, true));
    assert_eq!(count(&it), 1);
    assert!(home.path().join("alpha/proofs/I-1-1.png").is_file());
    let text = (&PNG[..], ".txt");
    let refused = add_item_image_bytes(home.path(), "alpha", &mut it, text, ("No", "human"));
    assert!(refused.is_err());
}

#[test]
fn a_picture_is_kept_under_the_item_with_its_hash() {
    let home = tempfile::tempdir().expect("home");
    let src = picture(&home, "shot.PNG", &PNG);
    let mut it = item();
    let kept = add_item_image(home.path(), "alpha", &mut it, (&src, "The form", "ui"));
    let kept = kept.expect("kept");
    assert_eq!(kept.n, 1);
    assert!(!kept.duplicate);
    assert_eq!(kept.sha256.len(), 64);
    assert!(home.path().join("alpha/proofs/I-1-1.png").is_file());
    let expected = home.path().join("alpha").join("proofs").join("I-1-1.png");
    assert_eq!(PathBuf::from(&kept.file), expected);
    assert_eq!(count(&it), 1);
    let record = &it["images"][0];
    assert_eq!(record["n"], 1);
    assert_eq!(record["caption"], "The form");
    assert_eq!(record["by"], "ui");
    assert_eq!(record["sha256"], json!(kept.sha256));
    assert_eq!(record["file"], json!(kept.file));
}

#[test]
fn the_same_picture_twice_is_kept_once() {
    let home = tempfile::tempdir().expect("home");
    let mut it = item();
    let first = picture(&home, "a.png", &PNG);
    let copy = picture(&home, "b.jpg", &PNG);
    let one = add_item_image(home.path(), "alpha", &mut it, (&first, "One", "ui")).expect("one");
    let two = add_item_image(home.path(), "alpha", &mut it, (&copy, "Again", "ui")).expect("two");
    assert!(two.duplicate);
    assert_eq!(two.n, one.n);
    assert_eq!(two.file, one.file);
    assert_eq!(count(&it), 1);
    let files = fs::read_dir(home.path().join("alpha/proofs")).expect("proofs");
    assert_eq!(files.count(), 1);
}

#[test]
fn numbers_only_grow_so_a_removed_picture_leaves_a_gap() {
    let home = tempfile::tempdir().expect("home");
    let mut it = item();
    for (n, name) in [(1, "a.png"), (2, "b.png")] {
        let bytes = [PNG.as_slice(), &[n]].concat();
        let src = picture(&home, name, &bytes);
        let kept =
            add_item_image(home.path(), "alpha", &mut it, (&src, "Shot", "ui")).expect("kept");
        assert_eq!(kept.n, i64::from(n));
    }
    it["images"].as_array_mut().expect("images").remove(0);
    let bytes = [PNG.as_slice(), &[3]].concat();
    let src = picture(&home, "c.png", &bytes);
    let third = keep_image(home.path(), "alpha", &it, &src).expect("third");
    assert_eq!(third.n, 3);
    assert!(home.path().join("alpha/proofs/I-1-3.png").is_file());
}

#[test]
fn a_bad_file_or_caption_is_refused_and_nothing_is_written() {
    let home = tempfile::tempdir().expect("home");
    let mut it = item();
    let text = picture(&home, "notes.txt", b"words");
    let empty = picture(&home, "empty.png", b"");
    let big = picture(
        &home,
        "big.png",
        &vec![1_u8; usize::try_from(MAX_BYTES).expect("size") + 1],
    );
    let fine = picture(&home, "fine.png", &PNG);
    let missing = home.path().join("missing.png");
    for src in [&text, &empty, &big, &missing] {
        assert!(
            keep_image(home.path(), "alpha", &it, src).is_err(),
            "{src:?}"
        );
    }
    let blank = add_item_image(home.path(), "alpha", &mut it, (&fine, "  ", "ui"));
    assert!(blank.is_err());
    let long = "x".repeat(121);
    assert!(add_item_image(home.path(), "alpha", &mut it, (&fine, &long, "ui")).is_err());
    assert_eq!(count(&it), 0);
    assert!(!home.path().join("alpha/proofs").exists());
}

#[test]
fn an_item_keeps_at_most_twelve_pictures() {
    let home = tempfile::tempdir().expect("home");
    let mut it = item();
    for n in 0..MAX_PER_ITEM {
        let bytes = [PNG.as_slice(), &[u8::try_from(n).expect("n")]].concat();
        let src = picture(&home, &format!("{n}.png"), &bytes);
        add_item_image(home.path(), "alpha", &mut it, (&src, "Shot", "ui")).expect("kept");
    }
    let over = picture(&home, "over.png", &[PNG.as_slice(), &[99]].concat());
    let refused = add_item_image(home.path(), "alpha", &mut it, (&over, "Shot", "ui"));
    assert!(refused.is_err());
    assert_eq!(count(&it), MAX_PER_ITEM);
}
