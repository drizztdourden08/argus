//! JSON text as `JSON.stringify` prints it: a float with no fraction has none, so
//! `meta.leftoverOld.days` is `1` and not `1.0`.

use argus_core::json_text::{to_string, to_vec};
use serde::Serialize;
use serde_json::{Number, Value, json};

fn float(n: f64) -> Value {
    Value::Number(Number::from_f64(n).expect("a finite number"))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Old {
    count: u32,
    days: f64,
    size_mb: f64,
}

#[test]
fn whole_floats_print_without_a_fraction() {
    for (n, text) in [
        (1.0, "1"),
        (3.0, "3"),
        (-2.0, "-2"),
        (0.0, "0"),
        (-0.0, "0"),
        (1e20, "100000000000000000000"),
        (1024.0, "1024"),
    ] {
        assert_eq!(to_string(&float(n)).unwrap(), text, "{n}");
    }
}

#[test]
fn fractions_and_integers_are_left_alone() {
    assert_eq!(to_string(&float(1.5)).unwrap(), "1.5");
    assert_eq!(to_string(&float(0.25)).unwrap(), "0.25");
    assert_eq!(to_string(&json!(7)).unwrap(), "7");
    assert_eq!(to_string(&json!("1.0")).unwrap(), "\"1.0\"");
}

#[test]
fn a_float_field_of_a_struct_and_a_nested_value_are_both_covered() {
    let old = Old {
        count: 2,
        days: 1.0,
        size_mb: 12.5,
    };
    assert_eq!(
        to_string(&old).unwrap(),
        r#"{"count":2,"days":1,"sizeMb":12.5}"#
    );
    let nested = json!({ "meta": { "leftoverOld": serde_json::to_value(&old).unwrap() }, "list": [float(2.0)] });
    assert_eq!(
        String::from_utf8(to_vec(&nested).unwrap()).unwrap(),
        r#"{"meta":{"leftoverOld":{"count":2,"days":1,"sizeMb":12.5}},"list":[2]}"#
    );
}

#[test]
fn the_text_reads_back_as_the_same_number() {
    let back: Value = serde_json::from_str(&to_string(&float(4.0)).unwrap()).unwrap();
    assert_eq!(back.as_i64(), Some(4));
}
