//! `coord ask` and `coord decide` with `--image [N=]<file> "<caption>"`: the picture is kept next
//! to the board like a test proof and listed in the entry's `images`, tied to an option when it
//! says so. Temp home, no server.

mod common;
mod support;
mod work_board;

use std::fs;

use serde_json::{Value, json};
use work_board::Work;

const CTX: &str = "The login form can remember the email. It changes what is stored on the machine, so it is your call. I will store nothing if unanswered.";
const PNG: [u8; 12] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 1];

fn picture(work: &Work, name: &str) -> String {
    let file = work.cwd.join(name);
    fs::write(&file, PNG).expect("image");
    file.to_string_lossy().into_owned()
}

fn run(work: &Work, words: &[&str]) -> String {
    let ran = work.worker(words);
    assert!(ran.code == 0, "{words:?} failed: {} {}", ran.out, ran.err);
    ran.out
}

fn refused(work: &Work, words: &[&str], needle: &str) {
    let ran = work.worker(words);
    let said = ran.code == 1 && ran.err.contains(needle);
    assert!(said, "{words:?} wanted {needle:?}: {} {}", ran.out, ran.err);
}

fn entries(work: &Work) -> serde_json::Map<String, Value> {
    work.board()["entries"]
        .as_object()
        .cloned()
        .unwrap_or_default()
}

fn kept(work: &Work, name: &str) -> bool {
    work.home
        .coord_home()
        .join("alpha")
        .join("proofs")
        .join(name)
        .is_file()
}

#[test]
fn an_ask_keeps_each_image_with_its_caption_and_the_option_it_belongs_to() {
    let work = Work::new();
    let (a, b) = (picture(&work, "a.png"), picture(&work, "b.png"));
    let tied = format!("2={b}");
    let out = run(
        &work,
        &[
            "ask",
            "I-1",
            "Which look?",
            "--context",
            CTX,
            "--option",
            "Round",
            "--option",
            "Square",
            "--image",
            &a,
            "The current look",
            "--image",
            &tied,
            "Square corners",
        ],
    );
    assert!(out.contains("Q-1 asked on I-1 (decision request, shown in the feed) with 2 image(s)"));
    let all = entries(&work);
    let images = &all["Q-1"]["images"];
    assert_eq!(images[0]["n"], 1);
    assert_eq!(images[0]["caption"], "The current look");
    assert!(images[0].get("option").is_none());
    assert_eq!(images[1]["n"], 2);
    assert_eq!(images[1]["option"], json!(2));
    assert!(kept(&work, "Q-1-1.png") && kept(&work, "Q-1-2.png"));
    assert_eq!(fs::read(images[0]["file"].as_str().unwrap()).unwrap(), PNG);
}

#[test]
fn a_decision_keeps_images_for_the_whole_entry() {
    let work = Work::new();
    let a = picture(&work, "a.png");
    run(
        &work,
        &[
            "decide",
            "I-1",
            "Round corners",
            "--context",
            CTX,
            "--image",
            &a,
            "After",
        ],
    );
    let all = entries(&work);
    assert_eq!(all["D-1"]["images"][0]["caption"], "After");
    assert!(kept(&work, "D-1-1.png"));
}

#[test]
fn an_entry_without_images_has_no_images_field() {
    let work = Work::new();
    run(&work, &["decide", "I-1", "No pictures", "--context", CTX]);
    assert!(entries(&work)["D-1"].get("images").is_none());
}

#[test]
fn a_missing_caption_a_bad_type_a_missing_file_or_a_missing_option_is_refused_and_nothing_is_kept()
{
    let work = Work::new();
    let a = picture(&work, "a.png");
    fs::write(work.cwd.join("n.txt"), "x").expect("text file");
    let base = [
        "ask",
        "I-1",
        "Which?",
        "--context",
        CTX,
        "--option",
        "A",
        "--option",
        "B",
    ];
    let with = |extra: &[&str]| -> Vec<String> {
        base.iter().chain(extra).map(ToString::to_string).collect()
    };
    let go = |extra: &[&str], needle: &str| {
        let words = with(extra);
        let refs: Vec<&str> = words.iter().map(String::as_str).collect();
        refused(&work, &refs, needle);
    };
    go(&["--image", &a], "each image needs a caption");
    go(&["--image", "n.txt", "Text"], "an image must be one of");
    go(&["--image", "nope.png", "Gone"], "no image at");
    go(
        &["--image", &format!("3={a}"), "Third"],
        "option 1 to 2 only",
    );
    go(&["--image"], "--image needs a file");
    assert!(entries(&work).is_empty());
    assert!(!kept(&work, "Q-1-1.png"));
}

#[test]
fn an_image_tied_to_an_option_on_a_plain_decision_record_is_refused() {
    let work = Work::new();
    let a = picture(&work, "a.png");
    let tied = format!("1={a}");
    refused(
        &work,
        &[
            "decide",
            "I-1",
            "Done",
            "--context",
            CTX,
            "--image",
            &tied,
            "Look",
        ],
        "option 1 to 0 only",
    );
    assert!(entries(&work).is_empty());
}

#[test]
fn extra_words_without_an_image_flag_are_ignored_as_before() {
    let work = Work::new();
    run(
        &work,
        &["decide", "I-1", "Plain", "stray", "--context", CTX],
    );
    assert!(entries(&work)["D-1"].get("images").is_none());
}
