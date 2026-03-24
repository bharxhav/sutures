//! Exhaustive integration tests for `sutures_comptime::parse!()` + `Knit` trait.
//!
//! Tests cover both Request (struct -> bytes) and Response (JSON -> struct) directions,
//! round-trip identity, cross-layer consistency with Stitch, byte-level correctness,
//! and edge cases.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Knit, Seam, Stitch};

// ============================================================================
// Test structs
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Simple {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MultiField {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Inner {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Nested {
    label: String,
    #[seam(to_struct)]
    pos: Inner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct WithArray {
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct WithNumbers {
    items: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOut {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutTarget {
    first: String,
    second: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepInner {
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepMiddle {
    tag: String,
    #[seam(to_struct)]
    deep: DeepInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepOuter {
    name: String,
    #[seam(to_struct)]
    middle: DeepMiddle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct EmptyStruct {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct UnicodeStruct {
    text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TypedValues {
    int_val: i64,
    float_val: f64,
    bool_val: bool,
    null_val: Option<String>,
    str_val: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct WithEmptyArray {
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SingleIndex {
    first: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SliceTarget {
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedJson {
    content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedObjOuter {
    label: String,
    #[seam(to_struct)]
    pos: Inner,
}

// ============================================================================
// 1–8: Request direction — knit (struct -> bytes)
// ============================================================================

// 1. Simple single field
#[test]
fn t01_request_knit_simple_single_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t01",
        "suture_sets": [{
            "name": "t01",
            "capture": "request",
            "sutures": [{ "name": "/user_name" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = Simple { name: "Alice".into() };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, json!({"user_name": "Alice"}));
}

// 2. Multiple fields
#[test]
fn t02_request_knit_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t02",
        "suture_sets": [{
            "name": "t02",
            "capture": "request",
            "sutures": [{
                "name": "/user_name",
                "age": "/user_age",
                "active": "/is_active"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = MultiField {
        name: "Bob".into(),
        age: 30,
        active: true,
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["user_name"], json!("Bob"));
    assert_eq!(parsed["user_age"], json!(30));
    assert_eq!(parsed["is_active"], json!(true));
}

// 3. Nested struct
#[test]
fn t03_request_knit_nested_struct() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t03",
        "suture_sets": [{
            "name": "t03",
            "capture": "request",
            "sutures": [{
                "label": "/label",
                "pos.x": "/coordinates/px",
                "pos.y": "/coordinates/py"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = Nested {
        label: "origin".into(),
        pos: Inner { x: 1.5, y: 2.5 },
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["label"], json!("origin"));
    assert_eq!(parsed["coordinates"]["px"], json!(1.5));
    assert_eq!(parsed["coordinates"]["py"], json!(2.5));
}

// 4. Array iteration [:]
#[test]
fn t04_request_knit_array_iteration() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t04",
        "suture_sets": [{
            "name": "t04",
            "capture": "request",
            "sutures": [{ "items[:]": "/data[:]" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = WithArray {
        items: vec!["a".into(), "b".into(), "c".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["data"], json!(["a", "b", "c"]));
}

// 5. Single index [0]
#[test]
fn t05_request_knit_single_index() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t05",
        "suture_sets": [{
            "name": "t05",
            "capture": "request",
            "sutures": [{ "items[0]": "/first_item" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = WithArray {
        items: vec!["alpha".into(), "beta".into(), "gamma".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["first_item"], json!("alpha"));
}

// 6. Slice [1:3]
#[test]
fn t06_request_knit_slice() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t06",
        "suture_sets": [{
            "name": "t06",
            "capture": "request",
            "sutures": [{ "items[1:3]": "/subset[:]" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = WithArray {
        items: vec!["a".into(), "b".into(), "c".into(), "d".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["subset"], json!(["b", "c"]));
}

// 7. Fan-out
#[test]
fn t07_request_knit_fan_out() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t07",
        "suture_sets": [{
            "name": "t07",
            "capture": "request",
            "sutures": [{ "value": ["/first", "/second"] }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = FanOut { value: "shared".into() };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["first"], json!("shared"));
    assert_eq!(parsed["second"], json!("shared"));
}

// 8. Nested object syntax
#[test]
fn t08_request_knit_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(r#"{
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
    }"#);
    let suture = &sutures[0];
    let input = Nested {
        label: "point".into(),
        pos: Inner { x: 10.0, y: 20.0 },
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["label"], json!("point"));
    assert_eq!(parsed["coordinates"]["px"], json!(10.0));
    assert_eq!(parsed["coordinates"]["py"], json!(20.0));
}

// ============================================================================
// 9–12: Request direction — unknit (bytes -> struct)
// ============================================================================

// 9. Simple JSON bytes
#[test]
fn t09_request_unknit_simple() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t09",
        "suture_sets": [{
            "name": "t09",
            "capture": "request",
            "sutures": [{ "name": "/user_name" }]
        }]
    }"#);
    let suture = &sutures[0];
    let json_bytes = serde_json::to_vec(&json!({"user_name": "Alice"})).unwrap();
    let result: Simple = suture.unknit(&json_bytes).unwrap();
    assert_eq!(result, Simple { name: "Alice".into() });
}

// 10. Multiple fields
#[test]
fn t10_request_unknit_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t10",
        "suture_sets": [{
            "name": "t10",
            "capture": "request",
            "sutures": [{
                "name": "/user_name",
                "age": "/user_age",
                "active": "/is_active"
            }]
        }]
    }"#);
    let suture = &sutures[0];
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

// 11. Nested JSON
#[test]
fn t11_request_unknit_nested_json() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t11",
        "suture_sets": [{
            "name": "t11",
            "capture": "request",
            "sutures": [{
                "label": "/label",
                "pos.x": "/coordinates/px",
                "pos.y": "/coordinates/py"
            }]
        }]
    }"#);
    let suture = &sutures[0];
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

// 12. Array in JSON
#[test]
fn t12_request_unknit_array() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t12",
        "suture_sets": [{
            "name": "t12",
            "capture": "request",
            "sutures": [{ "items[:]": "/data[:]" }]
        }]
    }"#);
    let suture = &sutures[0];
    let json_bytes = serde_json::to_vec(&json!({"data": ["x", "y", "z"]})).unwrap();
    let result: WithArray = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        WithArray {
            items: vec!["x".into(), "y".into(), "z".into()],
        }
    );
}

// ============================================================================
// 13–15: Response direction — knit (struct -> bytes)
// ============================================================================

// 13. Simple field
#[test]
fn t13_response_knit_simple() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t13",
        "suture_sets": [{
            "name": "t13",
            "capture": "response",
            "sutures": [{ "/user_name": "name" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = Simple { name: "Diana".into() };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["user_name"], json!("Diana"));
}

// 14. Multiple fields
#[test]
fn t14_response_knit_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t14",
        "suture_sets": [{
            "name": "t14",
            "capture": "response",
            "sutures": [{
                "/user_name": "name",
                "/user_age": "age",
                "/is_active": "active"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = MultiField {
        name: "Eve".into(),
        age: 28,
        active: true,
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["user_name"], json!("Eve"));
    assert_eq!(parsed["user_age"], json!(28));
    assert_eq!(parsed["is_active"], json!(true));
}

// 15. Array fields
#[test]
fn t15_response_knit_array_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t15",
        "suture_sets": [{
            "name": "t15",
            "capture": "response",
            "sutures": [{ "/data[:]": "items[:]" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = WithArray {
        items: vec!["one".into(), "two".into(), "three".into()],
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["data"], json!(["one", "two", "three"]));
}

// ============================================================================
// 16–18: Response direction — unknit (bytes -> struct)
// ============================================================================

// 16. Simple bytes -> struct
#[test]
fn t16_response_unknit_simple() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t16",
        "suture_sets": [{
            "name": "t16",
            "capture": "response",
            "sutures": [{ "/user_name": "name" }]
        }]
    }"#);
    let suture = &sutures[0];
    let json_bytes = serde_json::to_vec(&json!({"user_name": "Frank"})).unwrap();
    let result: Simple = suture.unknit(&json_bytes).unwrap();
    assert_eq!(result, Simple { name: "Frank".into() });
}

// 17. Nested JSON -> struct
#[test]
fn t17_response_unknit_nested() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t17",
        "suture_sets": [{
            "name": "t17",
            "capture": "response",
            "sutures": [{
                "/label": "label",
                "/coordinates/px": "pos.x",
                "/coordinates/py": "pos.y"
            }]
        }]
    }"#);
    let suture = &sutures[0];
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

// 18. Array extraction
#[test]
fn t18_response_unknit_array() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t18",
        "suture_sets": [{
            "name": "t18",
            "capture": "response",
            "sutures": [{ "/data[:]": "items[:]" }]
        }]
    }"#);
    let suture = &sutures[0];
    let json_bytes = serde_json::to_vec(&json!({"data": ["p", "q", "r"]})).unwrap();
    let result: WithArray = suture.unknit(&json_bytes).unwrap();
    assert_eq!(
        result,
        WithArray {
            items: vec!["p".into(), "q".into(), "r".into()],
        }
    );
}

// ============================================================================
// 19–23: Round-trips (knit -> unknit = identity)
// ============================================================================

// 19. Request: knit -> unknit = identity (simple)
#[test]
fn t19_request_round_trip_simple() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t19",
        "suture_sets": [{
            "name": "t19",
            "capture": "request",
            "sutures": [{ "name": "/user_name" }]
        }]
    }"#);
    let suture = &sutures[0];
    let original = Simple { name: "roundtrip".into() };
    let bytes = suture.knit(&original).unwrap();
    let recovered: Simple = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

// 20. Request: knit -> unknit = identity (arrays)
#[test]
fn t20_request_round_trip_arrays() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t20",
        "suture_sets": [{
            "name": "t20",
            "capture": "request",
            "sutures": [{ "items[:]": "/list[:]" }]
        }]
    }"#);
    let suture = &sutures[0];
    let original = WithArray {
        items: vec!["x".into(), "y".into(), "z".into()],
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: WithArray = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

// 21. Response: knit -> unknit = identity (simple)
#[test]
fn t21_response_round_trip_simple() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t21",
        "suture_sets": [{
            "name": "t21",
            "capture": "response",
            "sutures": [{ "/user_name": "name" }]
        }]
    }"#);
    let suture = &sutures[0];
    let original = Simple { name: "resp_rt".into() };
    let bytes = suture.knit(&original).unwrap();
    let recovered: Simple = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

// 22. Response: knit -> unknit = identity (arrays)
#[test]
fn t22_response_round_trip_arrays() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t22",
        "suture_sets": [{
            "name": "t22",
            "capture": "response",
            "sutures": [{ "/list[:]": "items[:]" }]
        }]
    }"#);
    let suture = &sutures[0];
    let original = WithArray {
        items: vec!["m".into(), "n".into(), "o".into()],
    };
    let bytes = suture.knit(&original).unwrap();
    let recovered: WithArray = suture.unknit(&bytes).unwrap();
    assert_eq!(original, recovered);
}

// 23. unknit -> knit produces equivalent JSON
#[test]
fn t23_unknit_knit_produces_equivalent_json() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t23",
        "suture_sets": [{
            "name": "t23",
            "capture": "request",
            "sutures": [{
                "name": "/user_name",
                "age": "/user_age",
                "active": "/is_active"
            }]
        }]
    }"#);
    let suture = &sutures[0];
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
    let original_val: Value = serde_json::from_slice(&json_bytes).unwrap();
    let re_knit_val: Value = serde_json::from_slice(&re_knit_bytes).unwrap();
    assert_eq!(original_val, re_knit_val);
}

// ============================================================================
// 24–26: Cross-layer (knit vs stitch)
// ============================================================================

// 24. Request: knit output == serde_json::to_vec(stitch output)
#[test]
fn t24_request_knit_equals_stitch() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t24",
        "suture_sets": [{
            "name": "t24",
            "capture": "request",
            "sutures": [{
                "label": "/label",
                "pos.x": "/cx",
                "pos.y": "/cy"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = Nested {
        label: "cross".into(),
        pos: Inner { x: 9.0, y: 10.0 },
    };

    let knit_bytes = suture.knit(&input).unwrap();
    let stitch_value = suture.stitch(&input).unwrap();
    let stitch_bytes = serde_json::to_vec(&stitch_value).unwrap();

    // Parse both to compare as Values (byte ordering may differ)
    let knit_val: Value = serde_json::from_slice(&knit_bytes).unwrap();
    let stitch_val: Value = serde_json::from_slice(&stitch_bytes).unwrap();
    assert_eq!(knit_val, stitch_val);
}

// 25. Response: knit output == serde_json::to_vec(stitch output)
#[test]
fn t25_response_knit_equals_stitch() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t25",
        "suture_sets": [{
            "name": "t25",
            "capture": "response",
            "sutures": [{
                "/label": "label",
                "/cx": "pos.x",
                "/cy": "pos.y"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = Nested {
        label: "resp_cross".into(),
        pos: Inner { x: 100.0, y: 200.0 },
    };

    let knit_bytes = suture.knit(&input).unwrap();
    let stitch_value = suture.stitch(&input).unwrap();
    let stitch_bytes = serde_json::to_vec(&stitch_value).unwrap();

    let knit_val: Value = serde_json::from_slice(&knit_bytes).unwrap();
    let stitch_val: Value = serde_json::from_slice(&stitch_bytes).unwrap();
    assert_eq!(knit_val, stitch_val);
}

// 26. unknit result == unstitch result
#[test]
fn t26_unknit_equals_unstitch() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t26",
        "suture_sets": [{
            "name": "t26",
            "capture": "request",
            "sutures": [{
                "name": "/user_name",
                "age": "/user_age",
                "active": "/is_active"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let json_bytes = serde_json::to_vec(&json!({
        "user_name": "check",
        "user_age": 99,
        "is_active": false
    }))
    .unwrap();
    let json_value: Value = serde_json::from_slice(&json_bytes).unwrap();

    let unknit_result: MultiField = suture.unknit(&json_bytes).unwrap();
    let unstitch_result: MultiField = suture.unstitch(&json_value).unwrap();
    assert_eq!(unknit_result, unstitch_result);
}

// ============================================================================
// 27–30: Byte-level checks
// ============================================================================

// 27. Valid UTF-8
#[test]
fn t27_knit_output_is_valid_utf8() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t27",
        "suture_sets": [{
            "name": "t27",
            "capture": "request",
            "sutures": [{ "name": "/n" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = Simple { name: "utf8test".into() };
    let bytes = suture.knit(&input).unwrap();
    assert!(std::str::from_utf8(&bytes).is_ok());
}

// 28. Valid JSON
#[test]
fn t28_knit_output_is_valid_json() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t28",
        "suture_sets": [{
            "name": "t28",
            "capture": "request",
            "sutures": [{
                "name": "/n",
                "age": "/a",
                "active": "/b"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = MultiField {
        name: "json_valid".into(),
        age: 1,
        active: true,
    };
    let bytes = suture.knit(&input).unwrap();
    let result: Result<Value, _> = serde_json::from_slice(&bytes);
    assert!(result.is_ok(), "knit output must be valid JSON");
}

// 29. Numeric type preservation
#[test]
fn t29_knit_preserves_numeric_types() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t29",
        "suture_sets": [{
            "name": "t29",
            "capture": "request",
            "sutures": [{
                "int_val": "/iv",
                "float_val": "/fv",
                "bool_val": "/bv",
                "null_val": "/nv",
                "str_val": "/sv"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = TypedValues {
        int_val: 42,
        float_val: 3.14,
        bool_val: true,
        null_val: None,
        str_val: "hello".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();

    // Integer stays integer
    assert!(parsed["iv"].is_i64() || parsed["iv"].is_u64());
    assert_eq!(parsed["iv"].as_i64().unwrap(), 42);

    // Float stays float
    assert!(parsed["fv"].is_f64());
    assert!((parsed["fv"].as_f64().unwrap() - 3.14).abs() < 1e-10);
}

// 30. Null, boolean, empty array handling
#[test]
fn t30_knit_null_boolean_empty_array() {
    // Null and boolean
    let sutures_typed = sutures_comptime::parse!(r#"{
        "name": "t30a",
        "suture_sets": [{
            "name": "t30a",
            "capture": "request",
            "sutures": [{
                "int_val": "/iv",
                "float_val": "/fv",
                "bool_val": "/bv",
                "null_val": "/nv",
                "str_val": "/sv"
            }]
        }]
    }"#);
    let suture = &sutures_typed[0];
    let input = TypedValues {
        int_val: 0,
        float_val: 0.0,
        bool_val: false,
        null_val: None,
        str_val: "".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(parsed["nv"].is_null());
    assert_eq!(parsed["bv"], json!(false));

    // True boolean
    let input_true = TypedValues {
        int_val: 0,
        float_val: 0.0,
        bool_val: true,
        null_val: None,
        str_val: "".into(),
    };
    let bytes_true = suture.knit(&input_true).unwrap();
    let parsed_true: Value = serde_json::from_slice(&bytes_true).unwrap();
    assert_eq!(parsed_true["bv"], json!(true));

    // Empty array
    let sutures_arr = sutures_comptime::parse!(r#"{
        "name": "t30b",
        "suture_sets": [{
            "name": "t30b",
            "capture": "request",
            "sutures": [{ "items[:]": "/data[:]" }]
        }]
    }"#);
    let suture_arr = &sutures_arr[0];
    let input_empty = WithEmptyArray { items: vec![] };
    let bytes_empty = suture_arr.knit(&input_empty).unwrap();
    let parsed_empty: Value = serde_json::from_slice(&bytes_empty).unwrap();
    // Forward walk produces nothing for an empty array, so dst stays as {}
    assert!(parsed_empty.is_object());
}

// ============================================================================
// 31–34: Edge cases
// ============================================================================

// 31. Empty struct
#[test]
fn t31_empty_struct() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t31",
        "suture_sets": [{
            "name": "t31",
            "capture": "request",
            "sutures": [{}]
        }]
    }"#);
    let suture = &sutures[0];
    let input = EmptyStruct {};
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, json!({}));
}

// 32. Unicode strings
#[test]
fn t32_unicode_strings() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t32",
        "suture_sets": [{
            "name": "t32",
            "capture": "request",
            "sutures": [{ "text": "/content" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = UnicodeStruct {
        text: "Hello \u{1F600} \u{00E9}\u{00E8}\u{00EA} \u{4E16}\u{754C} \u{0410}\u{0411}\u{0412}".into(),
    };
    let bytes = suture.knit(&input).unwrap();

    // Must be valid UTF-8
    let utf8_str = std::str::from_utf8(&bytes).unwrap();

    // Must be valid JSON
    let parsed: Value = serde_json::from_str(utf8_str).unwrap();
    assert_eq!(parsed["content"].as_str().unwrap(), input.text);

    // Round-trip
    let recovered: UnicodeStruct = suture.unknit(&bytes).unwrap();
    assert_eq!(input, recovered);
}

// 33. Special JSON chars
#[test]
fn t33_special_json_chars() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t33",
        "suture_sets": [{
            "name": "t33",
            "capture": "request",
            "sutures": [{ "text": "/content" }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = UnicodeStruct {
        text: r#"She said "hello\" and then \n left"#.into(),
    };
    let bytes = suture.knit(&input).unwrap();

    // Must be valid UTF-8
    assert!(std::str::from_utf8(&bytes).is_ok());

    // Must be valid JSON
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["content"].as_str().unwrap(), input.text);

    // Round-trip
    let recovered: UnicodeStruct = suture.unknit(&bytes).unwrap();
    assert_eq!(input, recovered);
}

// 34. Deeply nested struct
#[test]
fn t34_deeply_nested_struct() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "t34",
        "suture_sets": [{
            "name": "t34",
            "capture": "request",
            "sutures": [{
                "name": "/name",
                "middle.tag": "/mid/tag",
                "middle.deep.value": "/mid/deep/val"
            }]
        }]
    }"#);
    let suture = &sutures[0];
    let input = DeepOuter {
        name: "outer".into(),
        middle: DeepMiddle {
            tag: "mid".into(),
            deep: DeepInner { value: 777 },
        },
    };
    let bytes = suture.knit(&input).unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["name"], json!("outer"));
    assert_eq!(parsed["mid"]["tag"], json!("mid"));
    assert_eq!(parsed["mid"]["deep"]["val"], json!(777));

    // Round-trip
    let recovered: DeepOuter = suture.unknit(&bytes).unwrap();
    assert_eq!(input, recovered);
}
