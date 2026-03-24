//! Exhaustive integration tests for **Response direction + Direct binding**
//! using `sutures_comptime::parse!()` (compile-time suture compilation)
//! covering both `unstitch` (JSON -> struct, forward) and `stitch` (struct -> JSON, reverse).
//!
//! Response suture JSON format:
//!   - Keys are JSON terminals (slash-separated, start with `/`)
//!   - Values are struct terminals (dot-separated, start with letter)
//!
//! For response direction:
//!   - `unstitch` is the natural/forward direction (JSON -> struct via trie)
//!   - `stitch` is the reverse direction (struct -> JSON)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Seam, Stitch};

// ===========================================================================
// Test structs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SingleField {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TwoFields {
    model: String,
    temperature: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ThreeFields {
    model: String,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutStruct {
    model: String,
    model_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ConfigInner {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedTarget {
    #[seam(to_struct)]
    config: ConfigInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ConfigWithTemp {
    model: String,
    temp: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedConfigTarget {
    #[seam(to_struct)]
    config: ConfigWithTemp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepC {
    c: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepB {
    #[seam(to_struct)]
    b: DeepC,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepA {
    #[seam(to_struct)]
    a: DeepB,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepValue {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedFlat {
    model: String,
    name: String,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedNestedFlat {
    name: String,
    #[seam(to_struct)]
    config: ConfigWithTemp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct WithOptional {
    model: String,
    nickname: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct UnicodeStruct {
    greeting: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct BooleanNullStruct {
    flag: bool,
    label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutTarget {
    model: String,
    #[serde(default)]
    model_name: String,
    #[seam(to_struct)]
    config: ConfigInner,
}

// Deep nesting structs (5+ levels)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Level5 {
    leaf: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Level4 {
    #[seam(to_struct)]
    e: Level5,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Level3 {
    #[seam(to_struct)]
    d: Level4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Level2 {
    #[seam(to_struct)]
    c: Level3,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Level1 {
    #[seam(to_struct)]
    b: Level2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Level0 {
    #[seam(to_struct)]
    a: Level1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FiveFieldFlat {
    a: String,
    b: String,
    c: String,
    d: String,
    e: String,
}

// ===========================================================================
// 1. Single field: unstitch and stitch
// ===========================================================================

#[test]
fn unstitch_single_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4" });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(result, SingleField { model: "gpt-4".into() });
}

#[test]
fn stitch_single_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = SingleField { model: "gpt-4".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

// ===========================================================================
// 2. Multiple fields: unstitch and stitch
// ===========================================================================

#[test]
fn unstitch_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4", "temp": 0.7 });
    let result: TwoFields = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        TwoFields {
            model: "gpt-4".into(),
            temperature: 0.7,
        }
    );
}

#[test]
fn stitch_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = TwoFields {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "temp": 0.7 }));
}

// ===========================================================================
// 3. Nested JSON source: deep JSON extraction
// ===========================================================================

#[test]
fn unstitch_nested_json_source() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_src",
            "capture": "response",
            "sutures": [
                { "/config/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "config": { "model": "gpt-4" } });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(result, SingleField { model: "gpt-4".into() });
}

#[test]
fn stitch_nested_json_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_tgt",
            "capture": "response",
            "sutures": [
                { "/config/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = SingleField { model: "gpt-4".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "config": { "model": "gpt-4" } }));
}

// ===========================================================================
// 4. Very deep JSON source: 4-level extraction
// ===========================================================================

#[test]
fn unstitch_very_deep_json_source() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_src",
            "capture": "response",
            "sutures": [
                { "/a/b/c/d": "value" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "a": { "b": { "c": { "d": "deep_val" } } } });
    let result: DeepValue = suture.unstitch(&input).unwrap();
    assert_eq!(result, DeepValue { value: "deep_val".into() });
}

#[test]
fn stitch_very_deep_json_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_tgt",
            "capture": "response",
            "sutures": [
                { "/a/b/c/d": "value" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = DeepValue { value: "deep_val".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "a": { "b": { "c": { "d": "deep_val" } } } })
    );
}

// ===========================================================================
// 5. Fan-out: one JSON key to multiple struct fields
// ===========================================================================

#[test]
fn unstitch_fan_out() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "model_name"] }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4" });
    let result: FanOutStruct = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        FanOutStruct {
            model: "gpt-4".into(),
            model_name: "gpt-4".into(),
        }
    );
}

#[test]
fn stitch_fan_out_reverse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "model_name"] }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = FanOutStruct {
        model: "gpt-4".into(),
        model_name: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    // Both struct targets map back to the same JSON key /model; last write wins.
    assert_eq!(result["model"], json!("gpt-4"));
}

// ===========================================================================
// 6. Nested struct target with #[seam(to_struct)]
// ===========================================================================

#[test]
fn unstitch_nested_struct_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_target",
            "capture": "response",
            "sutures": [
                { "/model": "config.model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4" });
    let result: NestedTarget = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NestedTarget {
            config: ConfigInner { model: "gpt-4".into() },
        }
    );
}

#[test]
fn stitch_nested_struct_source_reverse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_src",
            "capture": "response",
            "sutures": [
                { "/model": "config.model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = NestedTarget {
        config: ConfigInner { model: "gpt-4".into() },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

// ===========================================================================
// 7. Deep nested struct target: 3-level dot path
// ===========================================================================

#[test]
fn unstitch_deep_nested_struct_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_target",
            "capture": "response",
            "sutures": [
                { "/x": "a.b.c" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "x": "hello" });
    let result: DeepA = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        DeepA {
            a: DeepB {
                b: DeepC { c: "hello".into() },
            },
        }
    );
}

#[test]
fn stitch_deep_nested_struct_reverse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "response",
            "sutures": [
                { "/x": "a.b.c" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = DeepA {
        a: DeepB {
            b: DeepC { c: "hello".into() },
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "x": "hello" }));
}

// ===========================================================================
// 8. Nested object syntax: recursive suture objects
// ===========================================================================

#[test]
fn unstitch_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_obj",
            "capture": "response",
            "sutures": [
                {
                    "/config": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "config": { "model": "gpt-4", "temp": 0.5 } });
    let result: NestedConfigTarget = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NestedConfigTarget {
            config: ConfigWithTemp {
                model: "gpt-4".into(),
                temp: 0.5,
            },
        }
    );
}

#[test]
fn stitch_nested_object_syntax_reverse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_obj",
            "capture": "response",
            "sutures": [
                {
                    "/config": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = NestedConfigTarget {
        config: ConfigWithTemp {
            model: "gpt-4".into(),
            temp: 0.5,
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "config": { "model": "gpt-4", "temp": 0.5 } })
    );
}

// ===========================================================================
// 9. Mixed flat and nested mappings in the same suture
// ===========================================================================

#[test]
fn unstitch_mixed_flat_and_nested() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "response",
            "sutures": [
                {
                    "/name": "name",
                    "/settings": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "name": "my_config",
        "settings": { "model": "gpt-4", "temp": 0.9 }
    });
    let result: MixedNestedFlat = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        MixedNestedFlat {
            name: "my_config".into(),
            config: ConfigWithTemp {
                model: "gpt-4".into(),
                temp: 0.9,
            },
        }
    );
}

#[test]
fn stitch_mixed_flat_and_nested_reverse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "response",
            "sutures": [
                {
                    "/name": "name",
                    "/settings": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = MixedNestedFlat {
        name: "my_config".into(),
        config: ConfigWithTemp {
            model: "gpt-4".into(),
            temp: 0.9,
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "name": "my_config",
            "settings": { "model": "gpt-4", "temp": 0.9 }
        })
    );
}

// ===========================================================================
// 10. Multiple suture objects in the sutures array
// ===========================================================================

#[test]
fn unstitch_multiple_suture_objects() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj",
            "capture": "response",
            "sutures": [
                { "/model": "model" },
                { "/temperature": "temperature" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4", "temperature": 0.7 });
    let result: TwoFields = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        TwoFields {
            model: "gpt-4".into(),
            temperature: 0.7,
        }
    );
}

#[test]
fn stitch_multiple_suture_objects_reverse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj",
            "capture": "response",
            "sutures": [
                { "/model": "model" },
                { "/temperature": "temperature" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = TwoFields {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "temperature": 0.7 }));
}

// ===========================================================================
// 11. Round-trip: unstitch -> stitch = identity
// ===========================================================================

#[test]
fn roundtrip_unstitch_then_stitch_flat() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_flat",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original_json = json!({ "model": "gpt-4", "temp": 0.7 });
    let intermediate: TwoFields = suture.unstitch(&original_json).unwrap();
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

#[test]
fn roundtrip_unstitch_then_stitch_nested() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_nested",
            "capture": "response",
            "sutures": [
                {
                    "/config": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original_json = json!({ "config": { "model": "gpt-4", "temp": 0.5 } });
    let intermediate: NestedConfigTarget = suture.unstitch(&original_json).unwrap();
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

#[test]
fn roundtrip_unstitch_then_stitch_deep_json() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_deep",
            "capture": "response",
            "sutures": [
                { "/a/b/c/d": "value" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original_json = json!({ "a": { "b": { "c": { "d": "val" } } } });
    let intermediate: DeepValue = suture.unstitch(&original_json).unwrap();
    assert_eq!(intermediate.value, "val");
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

#[test]
fn roundtrip_unstitch_then_stitch_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_multi",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temperature": "temperature", "/max_tokens": "max_tokens" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original_json = json!({ "model": "gpt-4", "temperature": 0.7, "max_tokens": 100 });
    let intermediate: ThreeFields = suture.unstitch(&original_json).unwrap();
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

// ===========================================================================
// 12. Round-trip: stitch -> unstitch = identity
// ===========================================================================

#[test]
fn roundtrip_stitch_then_unstitch_flat() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_flat2",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original = TwoFields {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let intermediate = suture.stitch(&original).unwrap();
    let roundtripped: TwoFields = suture.unstitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn roundtrip_stitch_then_unstitch_nested() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_nested2",
            "capture": "response",
            "sutures": [
                {
                    "/config": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original = NestedConfigTarget {
        config: ConfigWithTemp {
            model: "gpt-4".into(),
            temp: 0.5,
        },
    };
    let intermediate = suture.stitch(&original).unwrap();
    let roundtripped: NestedConfigTarget = suture.unstitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn roundtrip_stitch_then_unstitch_deep_struct() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_deep_struct",
            "capture": "response",
            "sutures": [
                { "/x": "a.b.c" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original = DeepA {
        a: DeepB {
            b: DeepC { c: "round_trip".into() },
        },
    };
    let intermediate = suture.stitch(&original).unwrap();
    let roundtripped: DeepA = suture.unstitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn roundtrip_stitch_then_unstitch_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_multi2",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temperature": "temperature", "/max_tokens": "max_tokens" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original = ThreeFields {
        model: "claude-3".into(),
        temperature: 0.5,
        max_tokens: 200,
    };
    let intermediate = suture.stitch(&original).unwrap();
    let roundtripped: ThreeFields = suture.unstitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original);
}

// ===========================================================================
// 13. Extra JSON fields are ignored during unstitch
// ===========================================================================

#[test]
fn unstitch_extra_json_fields_ignored() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "extra",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "model": "gpt-4",
        "extra_field": "should be ignored",
        "another_extra": 42,
        "nested_extra": { "deep": true }
    });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(result, SingleField { model: "gpt-4".into() });
}

#[test]
fn unstitch_extra_json_fields_at_nested_level_ignored() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "extra_nested",
            "capture": "response",
            "sutures": [
                { "/config/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "config": {
            "model": "gpt-4",
            "extra": "ignored"
        },
        "top_level_extra": "also ignored"
    });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(result, SingleField { model: "gpt-4".into() });
}

// ===========================================================================
// 14. Optional fields with null
// ===========================================================================

#[test]
fn unstitch_optional_field_null() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "optional",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4", "nickname": null });
    let result: WithOptional = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        WithOptional {
            model: "gpt-4".into(),
            nickname: None,
        }
    );
}

#[test]
fn unstitch_optional_field_missing_from_json() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "opt_missing",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4" });
    let result: WithOptional = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        WithOptional {
            model: "gpt-4".into(),
            nickname: None,
        }
    );
}

#[test]
fn stitch_optional_field_none() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "opt_none",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = WithOptional {
        model: "gpt-4".into(),
        nickname: None,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "nickname": null }));
}

#[test]
fn stitch_optional_field_some() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "opt_some",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = WithOptional {
        model: "gpt-4".into(),
        nickname: Some("fast_one".into()),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "nickname": "fast_one" }));
}

#[test]
fn roundtrip_optional_field_some() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "opt_rt",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original = WithOptional {
        model: "gpt-4".into(),
        nickname: Some("test_name".into()),
    };
    let json_val = suture.stitch(&original).unwrap();
    let roundtripped: WithOptional = suture.unstitch(&json_val).unwrap();
    assert_eq!(roundtripped, original);
}

// ===========================================================================
// 15. Unicode values
// ===========================================================================

#[test]
fn unstitch_unicode_values() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "unicode",
            "capture": "response",
            "sutures": [
                { "/greeting": "greeting", "/name": "name" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "greeting": "\u{3053}\u{3093}\u{306b}\u{3061}\u{306f}", "name": "\u{4e16}\u{754c}" });
    let result: UnicodeStruct = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        UnicodeStruct {
            greeting: "\u{3053}\u{3093}\u{306b}\u{3061}\u{306f}".into(),
            name: "\u{4e16}\u{754c}".into(),
        }
    );
}

#[test]
fn stitch_unicode_values() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "unicode",
            "capture": "response",
            "sutures": [
                { "/greeting": "greeting", "/name": "name" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = UnicodeStruct {
        greeting: "\u{041f}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}".into(),
        name: "\u{043c}\u{0438}\u{0440}".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "greeting": "\u{041f}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}", "name": "\u{043c}\u{0438}\u{0440}" })
    );
}

#[test]
fn roundtrip_unicode() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "uni_rt",
            "capture": "response",
            "sutures": [
                { "/greeting": "greeting", "/name": "name" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original = UnicodeStruct {
        greeting: "\u{4f60}\u{597d}".into(),
        name: "\u{4e16}\u{754c}\u{1f30d}".into(),
    };
    let json_val = suture.stitch(&original).unwrap();
    let roundtripped: UnicodeStruct = suture.unstitch(&json_val).unwrap();
    assert_eq!(roundtripped, original);
}

// ===========================================================================
// 16. Boolean and null values
// ===========================================================================

#[test]
fn unstitch_boolean_and_null_values() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "boolnull",
            "capture": "response",
            "sutures": [
                { "/flag": "flag", "/label": "label" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "flag": true, "label": null });
    let result: BooleanNullStruct = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        BooleanNullStruct {
            flag: true,
            label: None,
        }
    );
}

#[test]
fn stitch_boolean_and_null_values() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "boolnull",
            "capture": "response",
            "sutures": [
                { "/flag": "flag", "/label": "label" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = BooleanNullStruct {
        flag: false,
        label: None,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "flag": false, "label": null }));
}

#[test]
fn stitch_boolean_true_with_label() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "boolsome",
            "capture": "response",
            "sutures": [
                { "/flag": "flag", "/label": "label" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = BooleanNullStruct {
        flag: true,
        label: Some("present".into()),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "flag": true, "label": "present" }));
}

#[test]
fn roundtrip_boolean_and_null() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bn_rt",
            "capture": "response",
            "sutures": [
                { "/flag": "flag", "/label": "label" }
            ]
        }]
    }"#);

    let suture = &sutures[0];

    // With Some
    let original = BooleanNullStruct {
        flag: true,
        label: Some("present".into()),
    };
    let json_val = suture.stitch(&original).unwrap();
    let roundtripped: BooleanNullStruct = suture.unstitch(&json_val).unwrap();
    assert_eq!(roundtripped, original);

    // With None
    let original_none = BooleanNullStruct {
        flag: false,
        label: None,
    };
    let json_val_none = suture.stitch(&original_none).unwrap();
    let roundtripped_none: BooleanNullStruct = suture.unstitch(&json_val_none).unwrap();
    assert_eq!(roundtripped_none, original_none);
}

// ===========================================================================
// 17. Deep nesting (5+ levels in JSON and struct)
// ===========================================================================

#[test]
fn unstitch_deep_nesting_5_levels_json() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep5",
            "capture": "response",
            "sutures": [
                { "/l1/l2/l3/l4/l5": "a.b.c.d.e.leaf" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "l1": { "l2": { "l3": { "l4": { "l5": "deeply_nested" } } } }
    });
    let result: Level0 = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        Level0 {
            a: Level1 {
                b: Level2 {
                    c: Level3 {
                        d: Level4 {
                            e: Level5 {
                                leaf: "deeply_nested".into(),
                            },
                        },
                    },
                },
            },
        }
    );
}

#[test]
fn stitch_deep_nesting_5_levels() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep5",
            "capture": "response",
            "sutures": [
                { "/l1/l2/l3/l4/l5": "a.b.c.d.e.leaf" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = Level0 {
        a: Level1 {
            b: Level2 {
                c: Level3 {
                    d: Level4 {
                        e: Level5 {
                            leaf: "deeply_nested".into(),
                        },
                    },
                },
            },
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "l1": { "l2": { "l3": { "l4": { "l5": "deeply_nested" } } } }
        })
    );
}

#[test]
fn roundtrip_deep_nesting_5_levels() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep5_rt",
            "capture": "response",
            "sutures": [
                { "/l1/l2/l3/l4/l5": "a.b.c.d.e.leaf" }
            ]
        }]
    }"#);

    let suture = &sutures[0];

    // JSON -> struct -> JSON
    let original_json = json!({
        "l1": { "l2": { "l3": { "l4": { "l5": "round_trip_value" } } } }
    });
    let intermediate: Level0 = suture.unstitch(&original_json).unwrap();
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);

    // struct -> JSON -> struct
    let original_struct = Level0 {
        a: Level1 {
            b: Level2 {
                c: Level3 {
                    d: Level4 {
                        e: Level5 {
                            leaf: "struct_side".into(),
                        },
                    },
                },
            },
        },
    };
    let intermediate_json = suture.stitch(&original_struct).unwrap();
    let roundtripped_struct: Level0 = suture.unstitch(&intermediate_json).unwrap();
    assert_eq!(roundtripped_struct, original_struct);
}

// ===========================================================================
// Additional: fan-out with mixed flat + nested targets
// ===========================================================================

#[test]
fn unstitch_fan_out_with_nested_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout_mixed",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "config.model"] }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4" });
    let result: FanOutTarget = suture.unstitch(&input).unwrap();
    assert_eq!(result.model, "gpt-4");
    assert_eq!(result.config.model, "gpt-4");
}

// ===========================================================================
// Additional: nested JSON with multiple children at same level
// ===========================================================================

#[test]
fn unstitch_nested_json_multiple_children() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_child",
            "capture": "response",
            "sutures": [
                {
                    "/config/model": "model",
                    "/config/temp": "temperature"
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "config": { "model": "gpt-4", "temp": 0.7 } });
    let result: TwoFields = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        TwoFields {
            model: "gpt-4".into(),
            temperature: 0.7,
        }
    );
}

#[test]
fn stitch_nested_json_multiple_children() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_child",
            "capture": "response",
            "sutures": [
                {
                    "/config/model": "model",
                    "/config/temp": "temperature"
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = TwoFields {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "config": { "model": "gpt-4", "temp": 0.7 } })
    );
}

// ===========================================================================
// Additional: empty string values
// ===========================================================================

#[test]
fn unstitch_empty_string_value() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "" });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(result, SingleField { model: "".into() });
}

#[test]
fn stitch_empty_string_value() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = SingleField { model: "".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "" }));
}

// ===========================================================================
// Additional: numeric types preserved
// ===========================================================================

#[test]
fn unstitch_preserves_numeric_types() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "types",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature", "/tokens": "max_tokens" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({ "model": "gpt-4", "temp": 0.0, "tokens": 0 });
    let result: ThreeFields = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        ThreeFields {
            model: "gpt-4".into(),
            temperature: 0.0,
            max_tokens: 0,
        }
    );
}

// ===========================================================================
// Additional: suture metadata is correct with comptime parse
// ===========================================================================

#[test]
fn suture_metadata_is_correct() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "my_set",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    assert!(suture.is_response());
    assert!(!suture.is_request());
    assert_eq!(suture.name(), "my_set");
}

// ===========================================================================
// Additional: multiple suture sets in a single parse
// ===========================================================================

#[test]
fn multiple_suture_sets_parse_all() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "first",
                "capture": "response",
                "sutures": [
                    { "/model": "model" }
                ]
            },
            {
                "name": "second",
                "capture": "response",
                "sutures": [
                    { "/temp": "temperature" }
                ]
            }
        ]
    }"#);

    assert_eq!(sutures.len(), 2);
    assert_eq!(sutures[0].name(), "first");
    assert_eq!(sutures[1].name(), "second");

    // First suture: /model -> model
    let input1 = json!({ "model": "gpt-4" });
    let result1: SingleField = sutures[0].unstitch(&input1).unwrap();
    assert_eq!(result1.model, "gpt-4");
}

// ===========================================================================
// Additional: round-trip fan-out
// ===========================================================================

#[test]
fn roundtrip_fan_out() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout_rt",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "model_name"] }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original_json = json!({ "model": "gpt-4" });
    let intermediate: FanOutStruct = suture.unstitch(&original_json).unwrap();
    assert_eq!(intermediate.model, "gpt-4");
    assert_eq!(intermediate.model_name, "gpt-4");

    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

// ===========================================================================
// Additional: round-trip mixed nested and flat with multiple objects
// ===========================================================================

#[test]
fn roundtrip_mixed_nested_and_flat_multiple_objects() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj_mixed",
            "capture": "response",
            "sutures": [
                { "/name": "name" },
                {
                    "/settings": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    let suture = &sutures[0];
    let original = MixedNestedFlat {
        name: "test_cfg".into(),
        config: ConfigWithTemp {
            model: "claude".into(),
            temp: 0.3,
        },
    };
    let json_val = suture.stitch(&original).unwrap();
    assert_eq!(json_val["name"], json!("test_cfg"));
    assert_eq!(json_val["settings"]["model"], json!("claude"));
    assert_eq!(json_val["settings"]["temp"], json!(0.3));

    let roundtripped: MixedNestedFlat = suture.unstitch(&json_val).unwrap();
    assert_eq!(roundtripped, original);
}

// ===========================================================================
// Additional: comptime produces identical results to runtime parse
// ===========================================================================

#[test]
fn comptime_matches_runtime_single_field() {
    // Compile-time
    let ct_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#);

    // Runtime
    let rt_sutures = sutures::v1::parse(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#)
    .unwrap()
    .into_iter()
    .map(|r| r.unwrap())
    .collect::<Vec<_>>();

    let input = json!({ "model": "gpt-4" });

    let ct_result: SingleField = ct_sutures[0].unstitch(&input).unwrap();
    let rt_result: SingleField = rt_sutures[0].unstitch(&input).unwrap();
    assert_eq!(ct_result, rt_result);

    let struct_input = SingleField { model: "claude".into() };
    let ct_stitch = ct_sutures[0].stitch(&struct_input).unwrap();
    let rt_stitch = rt_sutures[0].stitch(&struct_input).unwrap();
    assert_eq!(ct_stitch, rt_stitch);
}

#[test]
fn comptime_matches_runtime_nested() {
    // Compile-time
    let ct_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested",
            "capture": "response",
            "sutures": [
                {
                    "/config": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#);

    // Runtime
    let rt_sutures = sutures::v1::parse(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested",
            "capture": "response",
            "sutures": [
                {
                    "/config": {
                        "/model": "config.model",
                        "/temp": "config.temp"
                    }
                }
            ]
        }]
    }"#)
    .unwrap()
    .into_iter()
    .map(|r| r.unwrap())
    .collect::<Vec<_>>();

    let input = json!({ "config": { "model": "gpt-4", "temp": 0.5 } });

    let ct_result: NestedConfigTarget = ct_sutures[0].unstitch(&input).unwrap();
    let rt_result: NestedConfigTarget = rt_sutures[0].unstitch(&input).unwrap();
    assert_eq!(ct_result, rt_result);

    let struct_input = NestedConfigTarget {
        config: ConfigWithTemp {
            model: "claude".into(),
            temp: 0.9,
        },
    };
    let ct_stitch = ct_sutures[0].stitch(&struct_input).unwrap();
    let rt_stitch = rt_sutures[0].stitch(&struct_input).unwrap();
    assert_eq!(ct_stitch, rt_stitch);
}

// ===========================================================================
// Additional: empty sutures array
// ===========================================================================

#[test]
fn empty_sutures_array() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty",
            "capture": "response",
            "sutures": []
        }]
    }"#);

    let suture = &sutures[0];
    assert!(suture.is_response());

    // stitch with empty mapping should produce an empty JSON object
    let input = SingleField { model: "gpt-4".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({}));
}
