//! The board icon: a brand, a curated name or a small png or svg, refused when it is anything
//! else or an svg that could run code, carried by the export and by the project card.
//! K1, K3: the owner op, the export and the card.

// test-tag: critical

use crate::common::{alpha, exported, post};
use argus_board::ops::{ICON_NAMES, icon_bytes};
use argus_board::portfolio::summarize;
use argus_board::store::save_board;
use argus_core::Coll;
use serde_json::{Value, json};

const DIGITS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PNG: &str = "data:image/png;base64,iVBORw0KGgo=";

fn encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0_u32, |acc, (i, b)| acc | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..=chunk.len() {
            out.push(char::from(DIGITS[((n >> (18 - 6 * i)) & 63) as usize]));
        }
        out.push_str(&"=".repeat(3 - chunk.len()));
    }
    out
}

fn svg_url(svg: &str) -> String {
    format!("data:image/svg+xml;base64,{}", encode(svg.as_bytes()))
}

fn set_icon(b: &mut Value, icon: &Value) -> usize {
    let (_, res) = post(b, json!({"kind": "icon", "icon": icon}));
    res.ops.refused
}

#[test]
fn the_icon_takes_a_brand_a_curated_name_a_png_or_a_safe_svg() {
    let mut b = alpha();
    let clean = svg_url("<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"4\"/></svg>");
    let mut valid = vec![
        json!("brand:argus"),
        json!("icon:globe"),
        json!(PNG),
        json!(clean),
    ];
    valid.extend(ICON_NAMES.iter().map(|n| json!(format!("icon:{n}"))));
    for icon in valid {
        assert_eq!(set_icon(&mut b, &icon), 0, "{icon}");
        assert_eq!(b["icon"], icon);
    }
    assert_eq!(set_icon(&mut b, &Value::Null), 0);
    assert_eq!(b["icon"], Value::Null);
}

#[test]
fn the_icon_refuses_a_bad_brand_name_size_or_encoding() {
    let mut b = alpha();
    set_icon(&mut b, &json!("icon:globe"));
    let big_png = format!("data:image/png;base64,{}", "A".repeat(66_000));
    let big_svg = svg_url(&format!("<svg>{}</svg>", " ".repeat(40_000)));
    for bad in [
        json!("brand:nope"),
        json!("icon:nope"),
        json!("icon:"),
        json!("rocket"),
        json!(7),
        json!(big_png),
        json!(big_svg),
        json!("data:image/png;base64,not base64!"),
        json!("data:image/png;base64,abc"),
        json!("data:image/svg+xml;base64,"),
        json!("data:image/jpeg;base64,iVBORw0KGgo="),
    ] {
        assert_eq!(set_icon(&mut b, &bad), 1, "{bad}");
    }
    assert_eq!(b["icon"], "icon:globe");
}

#[test]
fn the_icon_refuses_an_svg_that_could_run_code_or_load_from_outside() {
    let mut b = alpha();
    for svg in [
        "<svg><script>alert(1)</script></svg>",
        "<svg><SCRIPT src=\"x\"></SCRIPT></svg>",
        "<svg><foreignObject><div/></foreignObject></svg>",
        "<svg onload=\"alert(1)\"></svg>",
        "<svg><rect onclick = 'x()'/></svg>",
        "<svg><a href=\"javascript:alert(1)\"><rect/></a></svg>",
        "<svg><image href=\"https://example.com/a.png\"/></svg>",
        "<svg><image xlink:href=\"http://example.com/a.png\"/></svg>",
        "<svg><image href=\"//example.com/a.png\"/></svg>",
    ] {
        assert_eq!(set_icon(&mut b, &json!(svg_url(svg))), 1, "{svg}");
    }
    assert!(b.get("icon").is_none_or(Value::is_null));
}

#[test]
fn an_image_icon_decodes_to_its_bytes_and_type() {
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
    assert_eq!(
        icon_bytes(&svg_url(svg)),
        Some(("image/svg+xml", svg.as_bytes().to_vec()))
    );
    let (kind, bytes) = icon_bytes(PNG).expect("png");
    assert_eq!((kind, bytes.len()), ("image/png", 8));
    assert_eq!(icon_bytes("icon:globe"), None);
}

#[test]
fn the_export_and_the_card_carry_the_icon() {
    let mut b = alpha();
    set_icon(&mut b, &json!("icon:globe"));
    assert_eq!(exported(&b, Coll::Meta, "board")["icon"], "icon:globe");
    let home = tempfile::tempdir().expect("temp dir");
    save_board(home.path(), &b).expect("save");
    let reg = json!({"projects": {"alpha": {"key": "alpha", "name": "Alpha", "root": null}}});
    assert_eq!(
        summarize(home.path(), "alpha", &reg, None)["icon"],
        "icon:globe"
    );
    set_icon(&mut b, &json!(PNG));
    assert_eq!(exported(&b, Coll::Meta, "board")["icon"], PNG);
    save_board(home.path(), &b).expect("save");
    let url = summarize(home.path(), "alpha", &reg, None)["icon"].clone();
    let url = url.as_str().expect("text").to_owned();
    let (path, hash) = url.split_once("?v=").expect("versioned");
    assert_eq!(path, "/api/p/alpha/icon");
    assert!(hash.len() == 8 && hash.bytes().all(|c| c.is_ascii_hexdigit()));
    set_icon(&mut b, &Value::Null);
    save_board(home.path(), &b).expect("save");
    assert!(
        summarize(home.path(), "alpha", &reg, None)
            .get("icon")
            .is_none()
    );
}
