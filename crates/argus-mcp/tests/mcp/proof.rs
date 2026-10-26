//! `attach_proof`: base64 and path sources, the magic bytes, the allowed folders and the cap.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::config::AttachConfig;
use argus_mcp::MemoryBackend;
use base64::Engine;
use serde_json::{Map, Value, json};

use crate::support::{Rig, fixture, png, png_base64, run_lines, run_rows};

fn rig_with(roots: Vec<PathBuf>, max_bytes: u64, extra_roots: Vec<String>) -> Rig {
    let attach = AttachConfig {
        max_bytes,
        extra_roots,
        extra: Map::new(),
    };
    let backend = MemoryBackend::new(fixture()).with_runs(run_rows(), run_lines());
    Rig::over(backend.with_roots(roots).with_attach(attach))
}

fn proof_args(extra: &Value) -> Value {
    let mut args = json!({ "project": "alpha", "test": "T-1", "caption": "c" });
    args.as_object_mut()
        .expect("args")
        .extend(extra.as_object().expect("extra").clone());
    args
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[tokio::test]
async fn base64_goes_to_a_test_and_a_run_with_absolute_urls_and_magic_bytes_checked() {
    let rig = &Rig::new();
    let b64 = png_base64();
    let t = rig
        .ok(
            "attach_proof",
            proof_args(&json!({ "caption": "altar", "base64": b64 })),
        )
        .await;
    assert_eq!(
        t,
        json!({ "n": 2, "url": "http://127.0.0.1:47615/files/alpha/proofs/T-1-2.png" })
    );
    let want = json!({ "key": "alpha", "testId": "T-1", "bytes": png().len(), "mime": "image/png", "caption": "altar", "by": "mcp:tester" });
    assert_eq!(rig.backend.calls().proof[0], want);
    let run = json!({ "project": "alpha", "run": "R-1", "caption": "screen", "base64": b64, "mime": "image/png" });
    let r = rig.ok("attach_proof", run).await;
    assert_eq!(
        r,
        json!({ "id": "att1", "url": "http://127.0.0.1:47615/files/alpha/a/att1" })
    );
    assert_eq!(rig.backend.calls().run_attach.len(), 1);
    let not_image = base64::engine::general_purpose::STANDARD.encode("not an image at all");
    rig.err(
        "attach_proof",
        proof_args(&json!({ "base64": not_image })),
        "unsupported_media",
    )
    .await;
    rig.err(
        "attach_proof",
        proof_args(&json!({ "base64": b64, "mime": "image/gif" })),
        "bad_request",
    )
    .await;
    let neither = json!({ "project": "alpha", "caption": "x", "base64": b64 });
    rig.err("attach_proof", neither, "bad_request").await;
    let both =
        json!({ "project": "alpha", "test": "T-1", "run": "R-1", "caption": "x", "base64": b64 });
    rig.err("attach_proof", both, "bad_request").await;
    rig.err("attach_proof", proof_args(&json!({})), "bad_request")
        .await;
    rig.err(
        "attach_proof",
        proof_args(&json!({ "base64": b64, "path": "C:\\x.png" })),
        "bad_request",
    )
    .await;
    rig.err(
        "attach_proof",
        proof_args(&json!({ "base64": "###" })),
        "bad_request",
    )
    .await;
    assert_eq!(rig.backend.calls().proof.len(), 1);
}

#[tokio::test]
async fn the_error_messages_are_the_node_ones() {
    let rig = &Rig::new();
    let b64 = png_base64();
    let message = |args: Value, code: &'static str| async move {
        rig.err("attach_proof", args, code).await["message"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    };
    assert_eq!(
        message(proof_args(&json!({ "base64": "!!" })), "bad_request").await,
        "base64 is not valid base64"
    );
    assert_eq!(
        message(proof_args(&json!({})), "bad_request").await,
        "give exactly one of path or base64"
    );
    assert_eq!(
        message(
            json!({ "project": "alpha", "caption": "c", "base64": b64 }),
            "bad_request"
        )
        .await,
        "give exactly one of test or run"
    );
    let plain = base64::engine::general_purpose::STANDARD.encode("plain");
    assert_eq!(
        message(proof_args(&json!({ "base64": plain })), "unsupported_media").await,
        "the file is not a png, jpeg, gif or webp image"
    );
    assert_eq!(
        message(
            proof_args(&json!({ "base64": b64, "mime": "image/webp" })),
            "bad_request"
        )
        .await,
        "mime says image/webp but the bytes are image/png"
    );
}

#[tokio::test]
async fn webp_gif_and_jpeg_are_told_by_their_first_bytes() {
    let rig = &Rig::new();
    let webp: Vec<u8> = [b"RIFF".as_slice(), &[0, 0, 0, 0], b"WEBPVP8 "].concat();
    let gif = b"GIF89a\x01\x00".to_vec();
    let jpeg = vec![0xff, 0xd8, 0xff, 0xe0, 0, 0];
    for (bytes, mime) in [
        (webp, "image/webp"),
        (gif, "image/gif"),
        (jpeg, "image/jpeg"),
    ] {
        let data = base64::engine::general_purpose::STANDARD.encode(bytes);
        rig.ok(
            "attach_proof",
            proof_args(&json!({ "base64": data, "mime": mime })),
        )
        .await;
    }
    assert_eq!(rig.backend.calls().proof.len(), 3);
}

#[tokio::test]
async fn a_path_inside_an_allowed_root_is_read_sniffed_and_passed_on_as_bytes() {
    let scratch = tempfile::tempdir().expect("a temp dir");
    let root = scratch.path().join("allowed");
    fs::create_dir_all(&root).expect("root");
    fs::write(root.join("shot.png"), png()).expect("png");
    fs::write(root.join("notes.txt"), "plain text").expect("txt");
    let rig = &rig_with(vec![root.clone()], 4096, Vec::new());
    let ok = rig
        .ok(
            "attach_proof",
            proof_args(&json!({ "path": text(&root.join("shot.png")) })),
        )
        .await;
    assert_eq!(ok["n"], 2);
    assert_eq!(rig.backend.calls().proof[0]["bytes"], png().len());
    let path = |name: &str| json!({ "path": text(&root.join(name)) });
    rig.err(
        "attach_proof",
        proof_args(&path("notes.txt")),
        "unsupported_media",
    )
    .await;
    rig.err(
        "attach_proof",
        proof_args(&path("missing.png")),
        "not_found",
    )
    .await;
    rig.err(
        "attach_proof",
        proof_args(&json!({ "path": "relative/shot.png" })),
        "bad_request",
    )
    .await;
}

#[tokio::test]
async fn a_file_outside_every_root_is_forbidden() {
    let rig = &rig_with(Vec::new(), 1024 * 1024, Vec::new());
    let outside = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let e = rig
        .err(
            "attach_proof",
            proof_args(&json!({ "path": text(&outside) })),
            "forbidden",
        )
        .await;
    assert!(
        e["message"]
            .as_str()
            .is_some_and(|m| m.contains("outside the allowed folders"))
    );
    assert_eq!(rig.backend.calls().proof.len(), 0);
}

#[cfg(unix)]
fn link(src: &Path, dst: &Path) -> bool {
    std::os::unix::fs::symlink(src, dst).is_ok()
}

#[cfg(windows)]
fn link(src: &Path, dst: &Path) -> bool {
    std::os::windows::fs::symlink_file(src, dst).is_ok()
}

#[tokio::test]
async fn extra_roots_widen_the_list_a_link_out_is_followed_and_refused_and_the_size_cap_holds() {
    let scratch = tempfile::tempdir().expect("a temp dir");
    let root = scratch.path().join("linkroot");
    fs::create_dir_all(&root).expect("root");
    fs::write(root.join("big.png"), [png(), vec![0; 5000]].concat()).expect("big");
    let exe = std::env::current_exe().expect("the test binary");
    let extra = vec![text(exe.parent().expect("a folder"))];
    let wide = rig_with(vec![root.clone()], 4096, extra);
    let big = proof_args(&json!({ "path": text(&root.join("big.png")) }));
    wide.err("attach_proof", big, "too_large").await;
    let exe_path = proof_args(&json!({ "path": text(&exe) }));
    wide.err("attach_proof", exe_path, "too_large").await;
    let strict = rig_with(vec![root.clone()], 1024 * 1024 * 1024, Vec::new());
    if link(&exe, &root.join("link.png")) {
        let linked = proof_args(&json!({ "path": text(&root.join("link.png")) }));
        strict.err("attach_proof", linked, "forbidden").await;
    }
}
