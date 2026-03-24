//! Exhaustive integration tests for the `Knit` trait (streaming/byte layer).
//!
//! Tests cover both request (struct -> bytes) and response (JSON -> struct) directions,
//! round-trip identity, cross-layer consistency with Stitch, byte-level correctness,
//! and edge cases.

use serde_json::json;
use sutures::Knit;
use sutures::Seam;
use sutures::Stitch;

// ============================================================================
// Helper
// ============================================================================

fn parse_first(json: &str) -> sutures::v1::Suture {
    sutures::v1::parse(json)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .unwrap()
}

/// Build a request suture JSON string from a list of mappings.
/// Each mapping is a (struct_terminal, json_terminal) pair.
fn request_json(name: &str, mappings: &[(&str, serde_json::Value)]) -> String {
    let mut obj = serde_json::Map::new();
    for (k, v) in mappings {
        obj.insert(k.to_string(), v.clone());
    }
    json!({
        "name": name,
        "suture_sets": [{
            "name": name,
            "capture": "request",
            "sutures": [obj]
        }]
    })
    .to_string()
}

/// Build a response suture JSON string from a list of mappings.
/// Each mapping is a (json_terminal, struct_terminal) pair.
fn response_json(name: &str, mappings: &[(&str, serde_json::Value)]) -> String {
    let mut obj = serde_json::Map::new();
    for (k, v) in mappings {
        obj.insert(k.to_string(), v.clone());
    }
    json!({
        "name": name,
        "suture_sets": [{
            "name": name,
            "capture": "response",
            "sutures": [obj]
        }]
    })
    .to_string()
}

// ============================================================================
// Test structs
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct Simple {
    name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct MultiField {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct Inner {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct Nested {
    label: String,
    #[seam(to_struct)]
    pos: Inner,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct WithArray {
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct WithNumbers {
    items: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct FanOut {
    value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct FanOutTarget {
    first: String,
    second: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct DeepInner {
    value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct DeepMiddle {
    tag: String,
    #[seam(to_struct)]
    deep: DeepInner,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct DeepOuter {
    name: String,
    #[seam(to_struct)]
    middle: DeepMiddle,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct EmptyStruct {}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct LargeStruct {
    f01: String,
    f02: String,
    f03: String,
    f04: String,
    f05: String,
    f06: String,
    f07: String,
    f08: String,
    f09: String,
    f10: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct UnicodeStruct {
    text: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct TypedValues {
    int_val: i64,
    float_val: f64,
    bool_val: bool,
    null_val: Option<String>,
    str_val: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct WithEmptyArray {
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct SingleIndex {
    first: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct SliceTarget {
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Seam)]
struct NestedJson {
    content: String,
}

// ============================================================================
// 1–8: Request direction — knit (struct -> bytes)
// ============================================================================

#[test]
fn t01_request_knit_simple_single_field() {
    let suture = parse_first(&request_json("t01", &[("name", json!("/user_name"))]));
    let input = Simple {
        name: "Alice".into(),
    };
    let bytes = suture.knit(&input).unwrap();

    // Output must be valid JSON bytes
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, json!({"user_name": "Alice"}));
}

#[test]
fn t02_request_knit_multiple_fields() {
    let suture = parse_first(&request_json(
        "t02",
        &[
            ("name", json!("/user_name")),
            ("age", json!("/user_age")),
            ("active", json!("/is_active")),
        ],
    ));
    let input = MultiField {
        name: "Bob".into(),
        age: 30,
        active: true,
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["user_name"], json!("Bob"));
    assert_eq!(parsed["user_age"], json!(30));
    assert_eq!(parsed["is_active"], json!(true));
}

#[test]
fn t03_request_knit_nested_struct() {
    let suture = parse_first(&request_json(
        "t03",
        &[
            ("label", json!("/label")),
            ("pos.x", json!("/coordinates/px")),
            ("pos.y", json!("/coordinates/py")),
        ],
    ));
    let input = Nested {
        label: "origin".into(),
        pos: Inner { x: 1.5, y: 2.5 },
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["label"], json!("origin"));
    assert_eq!(parsed["coordinates"]["px"], json!(1.5));
    assert_eq!(parsed["coordinates"]["py"], json!(2.5));
}

#[test]
fn t04_request_knit_array_iteration() {
    let suture = parse_first(&request_json("t04", &[("items[:]", json!("/data[:]"))]));
    let input = WithArray {
        items: vec!["a".into(), "b".into(), "c".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["data"], json!(["a", "b", "c"]));
}

#[test]
fn t05_request_knit_single_index() {
    let suture = parse_first(&request_json("t05", &[("items[0]", json!("/first_item"))]));
    let input = WithArray {
        items: vec!["alpha".into(), "beta".into(), "gamma".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["first_item"], json!("alpha"));
}

#[test]
fn t06_request_knit_slice() {
    let suture = parse_first(&request_json("t06", &[("items[1:3]", json!("/subset[:]"))]));
    let input = WithArray {
        items: vec!["a".into(), "b".into(), "c".into(), "d".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["subset"], json!(["b", "c"]));
}

#[test]
fn t07_request_knit_fan_out() {
    let suture = parse_first(&request_json(
        "t07",
        &[("value", json!(["/first", "/second"]))],
    ));
    let input = FanOut {
        value: "shared".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["first"], json!("shared"));
    assert_eq!(parsed["second"], json!("shared"));
}

#[test]
fn t08_request_knit_nested_object_syntax() {
    // Use nested object syntax in suture definition
    let suture_json = json!({
        "name": "t08",
        "suture_sets": [{
            "name": "t08",
            "capture": "request",
            "sutures": [{
                "pos": {
                    "x": "/coordinates/px",
                    "y": "/coordinates/py"
                },
                "label": "/label"
            }]
        }]
    })
    .to_string();

    let suture = parse_first(&suture_json);
    let input = Nested {
        label: "point".into(),
        pos: Inner { x: 10.0, y: 20.0 },
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["label"], json!("point"));
    assert_eq!(parsed["coordinates"]["px"], json!(10.0));
    assert_eq!(parsed["coordinates"]["py"], json!(20.0));
}

// ============================================================================
// 9–13: Request direction — unknit (bytes -> struct)
// ============================================================================

#[test]
fn t09_request_unknit_simple() {
    let suture = parse_first(&request_json("t09", &[("name", json!("/user_name"))]));
    let json_bytes = serde_json::to_vec(&json!({"user_name": "Alice"})).unwrap();
    let result: Simple = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        Simple {
            name: "Alice".into()
        }
    );
}

#[test]
fn t10_request_unknit_multiple_fields() {
    let suture = parse_first(&request_json(
        "t10",
        &[
            ("name", json!("/user_name")),
            ("age", json!("/user_age")),
            ("active", json!("/is_active")),
        ],
    ));
    let json_bytes = serde_json::to_vec(&json!({
        "user_name": "Charlie",
        "user_age": 25,
        "is_active": false
    }))
    .unwrap();
    let result: MultiField = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        MultiField {
            name: "Charlie".into(),
            age: 25,
            active: false,
        }
    );
}

#[test]
fn t11_request_unknit_nested_json() {
    let suture = parse_first(&request_json(
        "t11",
        &[
            ("label", json!("/label")),
            ("pos.x", json!("/coordinates/px")),
            ("pos.y", json!("/coordinates/py")),
        ],
    ));
    let json_bytes = serde_json::to_vec(&json!({
        "label": "center",
        "coordinates": {"px": 3.0, "py": 4.0}
    }))
    .unwrap();
    let result: Nested = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        Nested {
            label: "center".into(),
            pos: Inner { x: 3.0, y: 4.0 },
        }
    );
}

#[test]
fn t12_request_unknit_array() {
    let suture = parse_first(&request_json("t12", &[("items[:]", json!("/data[:]"))]));
    let json_bytes = serde_json::to_vec(&json!({"data": ["x", "y", "z"]})).unwrap();
    let result: WithArray = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        WithArray {
            items: vec!["x".into(), "y".into(), "z".into()],
        }
    );
}

#[test]
fn t13_request_unknit_slice() {
    let suture = parse_first(&request_json("t13", &[("items[1:3]", json!("/subset[:]"))]));
    let json_bytes = serde_json::to_vec(&json!({"subset": ["b", "c"]})).unwrap();
    let result: WithArray = suture.unknit(&json_bytes).unwrap();
    // The unknit reverse walk uses enumeration indices (0-based dense),
    // so items[1:3] produces items[0] = "b" and items[1] = "c".
    assert_eq!(result.items[0], "b");
    assert_eq!(result.items[1], "c");
}

// ============================================================================
// 14–17: Response direction — knit (struct -> bytes)
// ============================================================================

#[test]
fn t14_response_knit_simple() {
    let suture = parse_first(&response_json("t14", &[("/user_name", json!("name"))]));
    let input = Simple {
        name: "Diana".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["user_name"], json!("Diana"));
}

#[test]
fn t15_response_knit_multiple_fields() {
    let suture = parse_first(&response_json(
        "t15",
        &[
            ("/user_name", json!("name")),
            ("/user_age", json!("age")),
            ("/is_active", json!("active")),
        ],
    ));
    let input = MultiField {
        name: "Eve".into(),
        age: 28,
        active: true,
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["user_name"], json!("Eve"));
    assert_eq!(parsed["user_age"], json!(28));
    assert_eq!(parsed["is_active"], json!(true));
}

#[test]
fn t16_response_knit_nested_struct() {
    let suture = parse_first(&response_json(
        "t16",
        &[
            ("/label", json!("label")),
            ("/coordinates/px", json!("pos.x")),
            ("/coordinates/py", json!("pos.y")),
        ],
    ));
    let input = Nested {
        label: "here".into(),
        pos: Inner { x: 5.0, y: 6.0 },
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["label"], json!("here"));
    assert_eq!(parsed["coordinates"]["px"], json!(5.0));
    assert_eq!(parsed["coordinates"]["py"], json!(6.0));
}

#[test]
fn t17_response_knit_array_fields() {
    let suture = parse_first(&response_json("t17", &[("/data[:]", json!("items[:]"))]));
    let input = WithArray {
        items: vec!["one".into(), "two".into(), "three".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["data"], json!(["one", "two", "three"]));
}

// ============================================================================
// 18–21: Response direction — unknit (bytes -> struct)
// ============================================================================

#[test]
fn t18_response_unknit_simple() {
    let suture = parse_first(&response_json("t18", &[("/user_name", json!("name"))]));
    let json_bytes = serde_json::to_vec(&json!({"user_name": "Frank"})).unwrap();
    let result: Simple = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        Simple {
            name: "Frank".into()
        }
    );
}

#[test]
fn t19_response_unknit_nested() {
    let suture = parse_first(&response_json(
        "t19",
        &[
            ("/label", json!("label")),
            ("/coordinates/px", json!("pos.x")),
            ("/coordinates/py", json!("pos.y")),
        ],
    ));
    let json_bytes = serde_json::to_vec(&json!({
        "label": "there",
        "coordinates": {"px": 7.0, "py": 8.0}
    }))
    .unwrap();
    let result: Nested = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        Nested {
            label: "there".into(),
            pos: Inner { x: 7.0, y: 8.0 },
        }
    );
}

#[test]
fn t20_response_unknit_array() {
    let suture = parse_first(&response_json("t20", &[("/data[:]", json!("items[:]"))]));
    let json_bytes = serde_json::to_vec(&json!({"data": ["p", "q", "r"]})).unwrap();
    let result: WithArray = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        WithArray {
            items: vec!["p".into(), "q".into(), "r".into()],
        }
    );
}

#[test]
fn t21_response_unknit_slice() {
    let suture = parse_first(&response_json("t21", &[("/data[1:3]", json!("items[:]"))]));
    let json_bytes = serde_json::to_vec(&json!({"data": ["a", "b", "c", "d"]})).unwrap();
    let result: WithArray = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        WithArray {
            items: vec!["b".into(), "c".into()],
        }
    );
}

// ============================================================================
// 22–28: knit/unknit round-trips
// ============================================================================

#[test]
fn t22_request_round_trip_simple() {
    let suture = parse_first(&request_json("t22", &[("name", json!("/user_name"))]));
    let original = Simple {
        name: "roundtrip".into(),
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: Simple = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn t23_request_round_trip_nested() {
    let suture = parse_first(&request_json(
        "t23",
        &[
            ("label", json!("/label")),
            ("pos.x", json!("/cx")),
            ("pos.y", json!("/cy")),
        ],
    ));
    let original = Nested {
        label: "round".into(),
        pos: Inner { x: 11.0, y: 22.0 },
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: Nested = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn t24_request_round_trip_arrays() {
    let suture = parse_first(&request_json("t24", &[("items[:]", json!("/list[:]"))]));
    let original = WithArray {
        items: vec!["x".into(), "y".into(), "z".into()],
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: WithArray = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn t25_response_round_trip_simple() {
    let suture = parse_first(&response_json("t25", &[("/user_name", json!("name"))]));
    let original = Simple {
        name: "resp_rt".into(),
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: Simple = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn t26_response_round_trip_nested() {
    let suture = parse_first(&response_json(
        "t26",
        &[
            ("/label", json!("label")),
            ("/cx", json!("pos.x")),
            ("/cy", json!("pos.y")),
        ],
    ));
    let original = Nested {
        label: "resp_nested".into(),
        pos: Inner { x: 33.0, y: 44.0 },
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: Nested = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn t27_response_round_trip_arrays() {
    let suture = parse_first(&response_json("t27", &[("/list[:]", json!("items[:]"))]));
    let original = WithArray {
        items: vec!["m".into(), "n".into(), "o".into()],
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: WithArray = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn t28_unknit_then_knit_produces_equivalent_json() {
    let suture = parse_first(&request_json(
        "t28",
        &[
            ("name", json!("/user_name")),
            ("age", json!("/user_age")),
            ("active", json!("/is_active")),
        ],
    ));
    let json_bytes = serde_json::to_vec(&json!({
        "user_name": "test",
        "user_age": 42,
        "is_active": true
    }))
    .unwrap();

    // unknit: bytes -> struct
    let intermediate: MultiField = suture.unknit(&json_bytes).unwrap();
    // knit: struct -> bytes
    let re_knit_bytes = suture.knit(&intermediate).unwrap();

    // Parse both and compare as Values (order-independent)
    let original_val: serde_json::Value = serde_json::from_slice(&json_bytes).unwrap();
    let re_knit_val: serde_json::Value = serde_json::from_slice(&re_knit_bytes).unwrap();
    assert_eq!(original_val, re_knit_val);
}

// ============================================================================
// 29–33: Cross-layer consistency (knit vs stitch)
// ============================================================================

#[test]
fn t29_request_knit_equals_stitch_simple() {
    let suture = parse_first(&request_json("t29", &[("name", json!("/user_name"))]));
    let input = Simple {
        name: "cross".into(),
    };

    let knit_bytes = suture.knit(&input).unwrap();
    let stitch_value = suture.stitch(&input).unwrap();
    let stitch_bytes = serde_json::to_vec(&stitch_value).unwrap();

    // Parse both to compare as Values (byte ordering may differ)
    let knit_val: serde_json::Value = serde_json::from_slice(&knit_bytes).unwrap();
    let stitch_val: serde_json::Value = serde_json::from_slice(&stitch_bytes).unwrap();
    assert_eq!(knit_val, stitch_val);
}

#[test]
fn t30_request_knit_equals_stitch_nested() {
    let suture = parse_first(&request_json(
        "t30",
        &[
            ("label", json!("/label")),
            ("pos.x", json!("/cx")),
            ("pos.y", json!("/cy")),
        ],
    ));
    let input = Nested {
        label: "cross_nested".into(),
        pos: Inner { x: 9.0, y: 10.0 },
    };

    let knit_bytes = suture.knit(&input).unwrap();
    let stitch_value = suture.stitch(&input).unwrap();
    let stitch_bytes = serde_json::to_vec(&stitch_value).unwrap();

    let knit_val: serde_json::Value = serde_json::from_slice(&knit_bytes).unwrap();
    let stitch_val: serde_json::Value = serde_json::from_slice(&stitch_bytes).unwrap();
    assert_eq!(knit_val, stitch_val);
}

#[test]
fn t31_request_knit_equals_stitch_arrays() {
    let suture = parse_first(&request_json("t31", &[("items[:]", json!("/arr[:]"))]));
    let input = WithArray {
        items: vec!["a".into(), "b".into()],
    };

    let knit_bytes = suture.knit(&input).unwrap();
    let stitch_value = suture.stitch(&input).unwrap();
    let stitch_bytes = serde_json::to_vec(&stitch_value).unwrap();

    let knit_val: serde_json::Value = serde_json::from_slice(&knit_bytes).unwrap();
    let stitch_val: serde_json::Value = serde_json::from_slice(&stitch_bytes).unwrap();
    assert_eq!(knit_val, stitch_val);
}

#[test]
fn t32_response_knit_equals_stitch() {
    let suture = parse_first(&response_json(
        "t32",
        &[
            ("/label", json!("label")),
            ("/cx", json!("pos.x")),
            ("/cy", json!("pos.y")),
        ],
    ));
    let input = Nested {
        label: "resp_cross".into(),
        pos: Inner { x: 100.0, y: 200.0 },
    };

    let knit_bytes = suture.knit(&input).unwrap();
    let stitch_value = suture.stitch(&input).unwrap();
    let stitch_bytes = serde_json::to_vec(&stitch_value).unwrap();

    let knit_val: serde_json::Value = serde_json::from_slice(&knit_bytes).unwrap();
    let stitch_val: serde_json::Value = serde_json::from_slice(&stitch_bytes).unwrap();
    assert_eq!(knit_val, stitch_val);
}

#[test]
fn t33_unknit_equals_unstitch() {
    let suture = parse_first(&request_json(
        "t33",
        &[
            ("name", json!("/user_name")),
            ("age", json!("/user_age")),
            ("active", json!("/is_active")),
        ],
    ));
    let json_bytes = serde_json::to_vec(&json!({
        "user_name": "check",
        "user_age": 99,
        "is_active": false
    }))
    .unwrap();
    let json_value: serde_json::Value = serde_json::from_slice(&json_bytes).unwrap();

    let unknit_result: MultiField = suture.unknit(&json_bytes).unwrap();
    let unstitch_result: MultiField = suture.unstitch(&json_value).unwrap();
    assert_eq!(unknit_result, unstitch_result);
}

// ============================================================================
// 34–40: Byte-level checks
// ============================================================================

#[test]
fn t34_knit_output_is_valid_utf8() {
    let suture = parse_first(&request_json("t34", &[("name", json!("/n"))]));
    let input = Simple {
        name: "utf8test".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    assert!(std::str::from_utf8(&bytes).is_ok());
}

#[test]
fn t35_knit_output_parses_as_valid_json() {
    let suture = parse_first(&request_json(
        "t35",
        &[
            ("name", json!("/n")),
            ("age", json!("/a")),
            ("active", json!("/b")),
        ],
    ));
    let input = MultiField {
        name: "json_valid".into(),
        age: 1,
        active: true,
    };
    let bytes = suture.knit(&input).unwrap();
    let result: Result<serde_json::Value, _> = serde_json::from_slice(&bytes);
    assert!(result.is_ok(), "knit output must be valid JSON");
}

#[test]
fn t36_knit_preserves_numeric_types() {
    let suture = parse_first(&request_json(
        "t36",
        &[
            ("int_val", json!("/iv")),
            ("float_val", json!("/fv")),
            ("bool_val", json!("/bv")),
            ("null_val", json!("/nv")),
            ("str_val", json!("/sv")),
        ],
    ));
    let input = TypedValues {
        int_val: 42,
        float_val: 3.14,
        bool_val: true,
        null_val: None,
        str_val: "hello".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    // Integer stays integer
    assert!(parsed["iv"].is_i64() || parsed["iv"].is_u64());
    assert_eq!(parsed["iv"].as_i64().unwrap(), 42);

    // Float stays float
    assert!(parsed["fv"].is_f64());
    assert!((parsed["fv"].as_f64().unwrap() - 3.14).abs() < 1e-10);
}

#[test]
fn t37_knit_null_values() {
    let suture = parse_first(&request_json(
        "t37",
        &[
            ("int_val", json!("/iv")),
            ("float_val", json!("/fv")),
            ("bool_val", json!("/bv")),
            ("null_val", json!("/nv")),
            ("str_val", json!("/sv")),
        ],
    ));
    let input = TypedValues {
        int_val: 0,
        float_val: 0.0,
        bool_val: false,
        null_val: None,
        str_val: "".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(parsed["nv"].is_null());
}

#[test]
fn t38_knit_boolean_values() {
    let suture = parse_first(&request_json(
        "t38",
        &[
            ("int_val", json!("/iv")),
            ("float_val", json!("/fv")),
            ("bool_val", json!("/bv")),
            ("null_val", json!("/nv")),
            ("str_val", json!("/sv")),
        ],
    ));
    let input_true = TypedValues {
        int_val: 0,
        float_val: 0.0,
        bool_val: true,
        null_val: None,
        str_val: "".into(),
    };
    let bytes_true = suture.knit(&input_true).unwrap();
    let parsed_true: serde_json::Value = serde_json::from_slice(&bytes_true).unwrap();
    assert_eq!(parsed_true["bv"], json!(true));

    let input_false = TypedValues {
        int_val: 0,
        float_val: 0.0,
        bool_val: false,
        null_val: None,
        str_val: "".into(),
    };
    let bytes_false = suture.knit(&input_false).unwrap();
    let parsed_false: serde_json::Value = serde_json::from_slice(&bytes_false).unwrap();
    assert_eq!(parsed_false["bv"], json!(false));
}

#[test]
fn t39_knit_empty_arrays() {
    let suture = parse_first(&request_json("t39", &[("items[:]", json!("/data[:]"))]));
    let input = WithEmptyArray { items: vec![] };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    // With no items to iterate, the output should not contain the key at all,
    // or it should be an empty object.
    // The forward walk produces nothing for an empty array, so dst stays as {}.
    assert!(parsed.is_object());
}

#[test]
fn t40_knit_empty_object_output() {
    // A suture that maps nothing visible (e.g. empty struct with no mappings)
    // should produce a valid empty JSON object.
    let suture_json = json!({
        "name": "t40",
        "suture_sets": [{
            "name": "t40",
            "capture": "request",
            "sutures": [{}]
        }]
    })
    .to_string();

    let suture = parse_first(&suture_json);
    let input = EmptyStruct {};
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed, json!({}));
}

// ============================================================================
// 41–45: Edge cases
// ============================================================================

#[test]
fn t41_empty_struct_minimal_json() {
    let suture_json = json!({
        "name": "t41",
        "suture_sets": [{
            "name": "t41",
            "capture": "request",
            "sutures": [{}]
        }]
    })
    .to_string();

    let suture = parse_first(&suture_json);
    let input = EmptyStruct {};
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, json!({}));
}

#[test]
fn t42_large_struct_many_fields() {
    let suture = parse_first(&request_json(
        "t42",
        &[
            ("f01", json!("/o01")),
            ("f02", json!("/o02")),
            ("f03", json!("/o03")),
            ("f04", json!("/o04")),
            ("f05", json!("/o05")),
            ("f06", json!("/o06")),
            ("f07", json!("/o07")),
            ("f08", json!("/o08")),
            ("f09", json!("/o09")),
            ("f10", json!("/o10")),
        ],
    ));
    let input = LargeStruct {
        f01: "v01".into(),
        f02: "v02".into(),
        f03: "v03".into(),
        f04: "v04".into(),
        f05: "v05".into(),
        f06: "v06".into(),
        f07: "v07".into(),
        f08: "v08".into(),
        f09: "v09".into(),
        f10: "v10".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    for i in 1..=10 {
        let key = format!("o{:02}", i);
        let expected = format!("v{:02}", i);
        assert_eq!(parsed[&key], json!(expected), "mismatch at field {key}");
    }

    // Round-trip
    let recovered: LargeStruct = suture.unknit(&bytes).unwrap();
    assert_eq!(input, recovered);
}

#[test]
fn t43_deeply_nested_struct() {
    let suture = parse_first(&request_json(
        "t43",
        &[
            ("name", json!("/name")),
            ("middle.tag", json!("/mid/tag")),
            ("middle.deep.value", json!("/mid/deep/val")),
        ],
    ));
    let input = DeepOuter {
        name: "outer".into(),
        middle: DeepMiddle {
            tag: "mid".into(),
            deep: DeepInner { value: 777 },
        },
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["name"], json!("outer"));
    assert_eq!(parsed["mid"]["tag"], json!("mid"));
    assert_eq!(parsed["mid"]["deep"]["val"], json!(777));

    // Round-trip
    let recovered: DeepOuter = suture.unknit(&bytes).unwrap();
    assert_eq!(input, recovered);
}

#[test]
fn t44_unicode_string_values() {
    let suture = parse_first(&request_json("t44", &[("text", json!("/content"))]));
    let input = UnicodeStruct {
        text: "Hello \u{1F600} \u{00E9}\u{00E8}\u{00EA} \u{4E16}\u{754C} \u{0410}\u{0411}\u{0412}"
            .into(),
    };
    let bytes = suture.knit(&input).unwrap();

    // Must be valid UTF-8
    let utf8_str = std::str::from_utf8(&bytes).unwrap();

    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(utf8_str).unwrap();
    assert_eq!(parsed["content"].as_str().unwrap(), input.text);

    // Round-trip
    let recovered: UnicodeStruct = suture.unknit(&bytes).unwrap();
    assert_eq!(input, recovered);
}

#[test]
fn t45_special_json_characters() {
    let suture = parse_first(&request_json("t45", &[("text", json!("/content"))]));
    let input = UnicodeStruct {
        text: r#"She said "hello\" and then \n left"#.into(),
    };
    let bytes = suture.knit(&input).unwrap();

    // Must be valid UTF-8
    assert!(std::str::from_utf8(&bytes).is_ok());

    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["content"].as_str().unwrap(), input.text);

    // Round-trip
    let recovered: UnicodeStruct = suture.unknit(&bytes).unwrap();
    assert_eq!(input, recovered);
}
