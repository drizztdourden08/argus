//! Attachments: the magic bytes decide, size and run existence are checked.

use crate::common::{Env, EnvOptions, PNG};

async fn finished_run(env: &Env) {
    env.define("alpha", vec![env.def("quick", "quick")]);
    env.start("alpha", "quick");
    env.host.exit(env.pid(0), Some(0)).await;
    env.finished("alpha", "R-1").await;
}

#[tokio::test(start_paused = true)]
async fn an_image_is_saved_listed_and_served_by_its_url() {
    let env = Env::new();
    finished_run(&env).await;
    let ok = env
        .svc
        .attach("alpha", "R-1", &PNG, "image/png", "a\ncaption", "tester")
        .unwrap();
    assert!(ok.id.starts_with("ra-") && ok.id.len() == 3 + 16);
    assert!(ok.id[3..].chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(ok.url, format!("/files/alpha/a/{}", ok.id));
    let list = env.svc.attachments("alpha", "R-1").unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!((list[0].caption.as_str(), list[0].n), ("a caption", 1));
    assert_eq!((list[0].mime.as_str(), list[0].bytes), ("image/png", 12));
    let second = env
        .svc
        .attach("alpha", "R-1", &PNG, "", "", "tester")
        .unwrap();
    assert_eq!(env.svc.attachments("alpha", "R-1").unwrap()[1].n, 2);
    assert_eq!(second.caption, "");
}

#[tokio::test(start_paused = true)]
async fn the_bytes_decide_not_the_claimed_type_and_the_run_must_exist() {
    let env = Env::new();
    finished_run(&env).await;
    let html = env
        .svc
        .attach("alpha", "R-1", b"<html>", "image/png", "", "x");
    assert_eq!(html.unwrap_err().code, "unsupported_media");
    let jpeg = env.svc.attach("alpha", "R-1", &PNG, "image/jpeg", "", "x");
    assert_eq!(jpeg.unwrap_err().code, "unsupported_media");
    let empty = env.svc.attach("alpha", "R-1", &[], "image/png", "", "x");
    assert_eq!(empty.unwrap_err().code, "bad_request");
    let ghost = env.svc.attach("alpha", "R-9", &PNG, "image/png", "", "x");
    assert_eq!(ghost.unwrap_err().code, "unknown_run");
    assert_eq!(
        env.svc.attachments("alpha", "R-9").unwrap_err().code,
        "unknown_run"
    );
}

#[tokio::test(start_paused = true)]
async fn an_attachment_over_the_limit_is_too_large() {
    let env = Env::build(EnvOptions {
        attach_max_bytes: 10,
        ..EnvOptions::default()
    });
    finished_run(&env).await;
    let big = env
        .svc
        .attach("alpha", "R-1", &PNG, "image/png", "", "x")
        .unwrap_err();
    assert_eq!((big.status, big.code), (413, "too_large"));
}
