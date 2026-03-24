//! Exhaustive integration tests for **Response direction + Direct binding**
//! covering both `stitch` (struct -> JSON, reverse) and `unstitch` (JSON -> struct, forward).
//!
//! Response suture JSON format:
//!   - Keys are JSON terminals (slash-separated, start with `/`)
//!   - Values are struct terminals (dot-separated, start with letter)
//!
//! For response direction:
//!   - `unstitch` is the natural/forward direction (JSON -> struct via trie)
//!   - `stitch` is the reverse direction (struct -> JSON)

use serde_json::json;
use sutures::Stitch;

// ===========================================================================
// Helpers
// ===========================================================================

fn parse_first(json: &str) -> sutures::v1::Suture {
    sutures::v1::parse(json)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .unwrap()
}

// ===========================================================================
// Test structs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SingleField {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct TwoFields {
    model: String,
    temperature: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FanOutStruct {
    model: String,
    model_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ConfigInner {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NestedTarget {
    #[seam(to_struct)]
    config: ConfigInner,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ConfigWithTemp {
    model: String,
    temp: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NestedConfigTarget {
    #[seam(to_struct)]
    config: ConfigWithTemp,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DeepC {
    c: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DeepB {
    #[seam(to_struct)]
    b: DeepC,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DeepA {
    #[seam(to_struct)]
    a: DeepB,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ThreeFields {
    model: String,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedFlat {
    model: String,
    name: String,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct WithOptional {
    model: String,
    nickname: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DeepValue {
    value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FiveFieldFlat {
    a: String,
    b: String,
    c: String,
    d: String,
    e: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnicodeStruct {
    greeting: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NumericStringStruct {
    count: String,
    code: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct BooleanNullStruct {
    flag: bool,
    label: Option<String>,
}

// Deep nesting structs (5+ levels)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level5 {
    leaf: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level4 {
    #[seam(to_struct)]
    e: Level5,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level3 {
    #[seam(to_struct)]
    d: Level4,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level2 {
    #[seam(to_struct)]
    c: Level3,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level1 {
    #[seam(to_struct)]
    b: Level2,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level0 {
    #[seam(to_struct)]
    a: Level1,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FanOutTarget {
    model: String,
    #[serde(default)]
    model_name: String,
    #[seam(to_struct)]
    config: ConfigInner,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedNestedFlat {
    name: String,
    #[seam(to_struct)]
    config: ConfigWithTemp,
}

// ===========================================================================
// 1. unstitch: single field
// ===========================================================================

#[test]
fn unstitch_single_field() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#,
    );

    let input = json!({ "model": "gpt-4" });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SingleField {
            model: "gpt-4".into()
        }
    );
}

// ===========================================================================
// 2. unstitch: multiple fields
// ===========================================================================

#[test]
fn unstitch_multiple_fields() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#,
    );

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

// ===========================================================================
// 3. unstitch: nested JSON source (deep JSON path to flat struct field)
// ===========================================================================

#[test]
fn unstitch_nested_json_source() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_src",
            "capture": "response",
            "sutures": [
                { "/config/model": "model" }
            ]
        }]
    }"#,
    );

    let input = json!({ "config": { "model": "gpt-4" } });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SingleField {
            model: "gpt-4".into()
        }
    );
}

// ===========================================================================
// 4. unstitch: very deep JSON source (4-level deep extraction)
// ===========================================================================

#[test]
fn unstitch_very_deep_json_source() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_src",
            "capture": "response",
            "sutures": [
                { "/a/b/c/d": "value" }
            ]
        }]
    }"#,
    );

    let input = json!({ "a": { "b": { "c": { "d": "deep_val" } } } });
    let result: DeepValue = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        DeepValue {
            value: "deep_val".into()
        }
    );
}

// ===========================================================================
// 5. unstitch: fan-out (one JSON value to multiple struct fields)
// ===========================================================================

#[test]
fn unstitch_fan_out() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "model_name"] }
            ]
        }]
    }"#,
    );

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

// ===========================================================================
// 6. unstitch: nested struct target with dot path
// ===========================================================================

#[test]
fn unstitch_nested_struct_target() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_target",
            "capture": "response",
            "sutures": [
                { "/model": "config.model" }
            ]
        }]
    }"#,
    );

    let input = json!({ "model": "gpt-4" });
    let result: NestedTarget = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NestedTarget {
            config: ConfigInner {
                model: "gpt-4".into()
            },
        }
    );
}

// ===========================================================================
// 7. unstitch: deep nested struct target (3-level nested)
// ===========================================================================

#[test]
fn unstitch_deep_nested_struct_target() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_target",
            "capture": "response",
            "sutures": [
                { "/x": "a.b.c" }
            ]
        }]
    }"#,
    );

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

// ===========================================================================
// 8. unstitch: nested object syntax (recursive suture objects)
// ===========================================================================

#[test]
fn unstitch_nested_object_syntax() {
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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

// ===========================================================================
// 9. unstitch: mixed flat and nested mappings in same suture
// ===========================================================================

#[test]
fn unstitch_mixed_flat_and_nested() {
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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

// ===========================================================================
// 10. unstitch: multiple suture objects in sutures array
// ===========================================================================

#[test]
fn unstitch_multiple_suture_objects() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj",
            "capture": "response",
            "sutures": [
                { "/model": "model" },
                { "/temperature": "temperature" }
            ]
        }]
    }"#,
    );

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

// ===========================================================================
// 11. unstitch: extra JSON fields not in mapping (ignored)
// ===========================================================================

#[test]
fn unstitch_extra_json_fields_ignored() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "extra",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#,
    );

    let input = json!({
        "model": "gpt-4",
        "extra_field": "should be ignored",
        "another_extra": 42
    });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SingleField {
            model: "gpt-4".into()
        }
    );
}

// ===========================================================================
// 12. unstitch: optional fields with null JSON value
// ===========================================================================

#[test]
fn unstitch_optional_field_null() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "optional",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#,
    );

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

// ===========================================================================
// 13. stitch: single field reverse
// ===========================================================================

#[test]
fn stitch_single_field_reverse() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#,
    );

    let input = SingleField {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

// ===========================================================================
// 14. stitch: multiple fields reverse
// ===========================================================================

#[test]
fn stitch_multiple_fields_reverse() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#,
    );

    let input = TwoFields {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "temp": 0.7 }));
}

// ===========================================================================
// 15. stitch: nested JSON target (struct field -> deep JSON path)
// ===========================================================================

#[test]
fn stitch_nested_json_target() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_target",
            "capture": "response",
            "sutures": [
                { "/config/model": "model" }
            ]
        }]
    }"#,
    );

    let input = SingleField {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "config": { "model": "gpt-4" } }));
}

// ===========================================================================
// 16. stitch: fan-out reverse
// ===========================================================================

#[test]
fn stitch_fan_out_reverse() {
    // Fan-out in response: one JSON key maps to multiple struct fields.
    // In reverse (stitch), we read from struct targets, write to JSON trie.
    // Both struct fields map to the same JSON location; last write wins.
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "model_name"] }
            ]
        }]
    }"#,
    );

    let input = FanOutStruct {
        model: "gpt-4".into(),
        model_name: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    // Both targets read into the same JSON location /model
    assert_eq!(result["model"], json!("gpt-4"));
}

// ===========================================================================
// 17. stitch: nested struct source reverse
// ===========================================================================

#[test]
fn stitch_nested_struct_source_reverse() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_src",
            "capture": "response",
            "sutures": [
                { "/model": "config.model" }
            ]
        }]
    }"#,
    );

    let input = NestedTarget {
        config: ConfigInner {
            model: "gpt-4".into(),
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

// ===========================================================================
// 18. stitch: deep nested reverse
// ===========================================================================

#[test]
fn stitch_deep_nested_reverse() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "response",
            "sutures": [
                { "/x": "a.b.c" }
            ]
        }]
    }"#,
    );

    let input = DeepA {
        a: DeepB {
            b: DeepC { c: "hello".into() },
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "x": "hello" }));
}

// ===========================================================================
// 19. stitch: nested object syntax reverse
// ===========================================================================

#[test]
fn stitch_nested_object_syntax_reverse() {
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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
// 20. stitch: mixed flat and nested reverse
// ===========================================================================

#[test]
fn stitch_mixed_reverse() {
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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
// 21. Round-trip: unstitch then stitch = identity for flat struct
// ===========================================================================

#[test]
fn roundtrip_unstitch_then_stitch_flat() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_flat",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#,
    );

    let original_json = json!({ "model": "gpt-4", "temp": 0.7 });
    let intermediate: TwoFields = suture.unstitch(&original_json).unwrap();
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

// ===========================================================================
// 22. Round-trip: unstitch then stitch = identity for nested struct
// ===========================================================================

#[test]
fn roundtrip_unstitch_then_stitch_nested() {
    let suture = parse_first(
        r#"{
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
    }"#,
    );

    let original_json = json!({ "config": { "model": "gpt-4", "temp": 0.5 } });
    let intermediate: NestedConfigTarget = suture.unstitch(&original_json).unwrap();
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

// ===========================================================================
// 23. Round-trip: stitch then unstitch = identity for flat struct
// ===========================================================================

#[test]
fn roundtrip_stitch_then_unstitch_flat() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_flat2",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#,
    );

    let original = TwoFields {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let intermediate = suture.stitch(&original).unwrap();
    let roundtripped: TwoFields = suture.unstitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original);
}

// ===========================================================================
// 24. Round-trip: stitch then unstitch = identity for nested struct
// ===========================================================================

#[test]
fn roundtrip_stitch_then_unstitch_nested() {
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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

// ===========================================================================
// 25. Round-trip: with multiple fields
// ===========================================================================

#[test]
fn roundtrip_multiple_fields() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt_multi",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temperature": "temperature", "/max_tokens": "max_tokens" }
            ]
        }]
    }"#,
    );

    // unstitch -> stitch roundtrip
    let original_json = json!({ "model": "gpt-4", "temperature": 0.7, "max_tokens": 100 });
    let intermediate: ThreeFields = suture.unstitch(&original_json).unwrap();
    let roundtripped_json = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped_json, original_json);

    // stitch -> unstitch roundtrip
    let original_struct = ThreeFields {
        model: "claude-3".into(),
        temperature: 0.5,
        max_tokens: 200,
    };
    let intermediate_json = suture.stitch(&original_struct).unwrap();
    let roundtripped_struct: ThreeFields = suture.unstitch(&intermediate_json).unwrap();
    assert_eq!(roundtripped_struct, original_struct);
}

// ===========================================================================
// 26. Comparison: same logical mapping as Request vs Response
// ===========================================================================

#[test]
fn comparison_request_vs_response_same_mapping() {
    // Response: JSON keys -> struct values
    let response_suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature" }
            ]
        }]
    }"#,
    );

    // Request: struct keys -> JSON values
    let request_suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req",
            "capture": "request",
            "sutures": [
                { "model": "/model", "temperature": "/temp" }
            ]
        }]
    }"#,
    );

    let the_struct = TwoFields {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let the_json = json!({ "model": "gpt-4", "temp": 0.7 });

    // Response stitch (struct -> JSON) should produce same as Request stitch
    let response_stitch_result = response_suture.stitch(&the_struct).unwrap();
    let request_stitch_result = request_suture.stitch(&the_struct).unwrap();
    assert_eq!(response_stitch_result, request_stitch_result);

    // Response unstitch (JSON -> struct) should produce same as Request unstitch
    let response_unstitch_result: TwoFields = response_suture.unstitch(&the_json).unwrap();
    let request_unstitch_result: TwoFields = request_suture.unstitch(&the_json).unwrap();
    assert_eq!(response_unstitch_result, request_unstitch_result);
}

// ===========================================================================
// 27. Edge case: empty sutures array
// ===========================================================================

#[test]
fn unstitch_empty_sutures_array() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty",
            "capture": "response",
            "sutures": []
        }]
    }"#,
    );

    // unstitch with no mappings produces an empty object, which serde
    // will attempt to deserialize. For a struct with all-optional fields
    // this can work, but for SingleField it will fail at deserialization.
    // The suture itself compiles fine though.
    assert!(suture.is_response());

    // stitch with empty mapping should produce an empty JSON object
    let input = SingleField {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({}));
}

// ===========================================================================
// 28. Edge case: unicode values in JSON
// ===========================================================================

#[test]
fn unstitch_unicode_values() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "unicode",
            "capture": "response",
            "sutures": [
                { "/greeting": "greeting", "/name": "name" }
            ]
        }]
    }"#,
    );

    let input = json!({ "greeting": "こんにちは", "name": "世界" });
    let result: UnicodeStruct = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        UnicodeStruct {
            greeting: "こんにちは".into(),
            name: "世界".into(),
        }
    );
}

#[test]
fn stitch_unicode_values() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "unicode",
            "capture": "response",
            "sutures": [
                { "/greeting": "greeting", "/name": "name" }
            ]
        }]
    }"#,
    );

    let input = UnicodeStruct {
        greeting: "Привет".into(),
        name: "мир".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "greeting": "Привет", "name": "мир" }));
}

// ===========================================================================
// 29. Edge case: numeric string fields
// ===========================================================================

#[test]
fn unstitch_numeric_string_fields() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "numstr",
            "capture": "response",
            "sutures": [
                { "/count": "count", "/code": "code" }
            ]
        }]
    }"#,
    );

    let input = json!({ "count": "42", "code": "007" });
    let result: NumericStringStruct = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NumericStringStruct {
            count: "42".into(),
            code: "007".into(),
        }
    );
}

#[test]
fn stitch_numeric_string_fields() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "numstr",
            "capture": "response",
            "sutures": [
                { "/count": "count", "/code": "code" }
            ]
        }]
    }"#,
    );

    let input = NumericStringStruct {
        count: "100".into(),
        code: "404".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "count": "100", "code": "404" }));
}

// ===========================================================================
// 30. Edge case: boolean and null values in JSON
// ===========================================================================

#[test]
fn unstitch_boolean_and_null_values() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "boolnull",
            "capture": "response",
            "sutures": [
                { "/flag": "flag", "/label": "label" }
            ]
        }]
    }"#,
    );

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
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "boolnull",
            "capture": "response",
            "sutures": [
                { "/flag": "flag", "/label": "label" }
            ]
        }]
    }"#,
    );

    let input = BooleanNullStruct {
        flag: false,
        label: None,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "flag": false, "label": null }));
}

// ===========================================================================
// 31. Edge case: deep nesting (5+ levels in JSON and struct)
// ===========================================================================

#[test]
fn unstitch_deep_nesting_5_levels_json() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep5",
            "capture": "response",
            "sutures": [
                { "/l1/l2/l3/l4/l5": "a.b.c.d.e.leaf" }
            ]
        }]
    }"#,
    );

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
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep5",
            "capture": "response",
            "sutures": [
                { "/l1/l2/l3/l4/l5": "a.b.c.d.e.leaf" }
            ]
        }]
    }"#,
    );

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
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep5_rt",
            "capture": "response",
            "sutures": [
                { "/l1/l2/l3/l4/l5": "a.b.c.d.e.leaf" }
            ]
        }]
    }"#,
    );

    let original_json = json!({
        "l1": { "l2": { "l3": { "l4": { "l5": "round_trip_value" } } } }
    });
    let intermediate: Level0 = suture.unstitch(&original_json).unwrap();
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);

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
// Additional unstitch edge cases
// ===========================================================================

#[test]
fn unstitch_fan_out_with_nested_target() {
    // Fan-out where one target is flat and another is nested
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout_mixed",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "config.model"] }
            ]
        }]
    }"#,
    );

    let input = json!({ "model": "gpt-4" });
    let result: FanOutTarget = suture.unstitch(&input).unwrap();
    assert_eq!(result.model, "gpt-4");
    assert_eq!(result.config.model, "gpt-4");
}

#[test]
fn unstitch_nested_json_multiple_children() {
    // Nested JSON object with multiple children at the same level
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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

#[test]
fn unstitch_preserves_numeric_types() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "types",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/temp": "temperature", "/tokens": "max_tokens" }
            ]
        }]
    }"#,
    );

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

#[test]
fn stitch_very_deep_json_target() {
    // Struct field maps to 4-level deep JSON path
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_tgt",
            "capture": "response",
            "sutures": [
                { "/a/b/c/d": "value" }
            ]
        }]
    }"#,
    );

    let input = DeepValue {
        value: "deep_val".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "a": { "b": { "c": { "d": "deep_val" } } } })
    );
}

#[test]
fn roundtrip_very_deep_json() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_rt",
            "capture": "response",
            "sutures": [
                { "/a/b/c/d": "value" }
            ]
        }]
    }"#,
    );

    let original_json = json!({ "a": { "b": { "c": { "d": "val" } } } });
    let intermediate: DeepValue = suture.unstitch(&original_json).unwrap();
    assert_eq!(intermediate.value, "val");
    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

#[test]
fn roundtrip_unicode() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "uni_rt",
            "capture": "response",
            "sutures": [
                { "/greeting": "greeting", "/name": "name" }
            ]
        }]
    }"#,
    );

    let original = UnicodeStruct {
        greeting: "你好".into(),
        name: "世界🌍".into(),
    };
    let json_val = suture.stitch(&original).unwrap();
    let roundtripped: UnicodeStruct = suture.unstitch(&json_val).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn roundtrip_boolean_and_null() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bn_rt",
            "capture": "response",
            "sutures": [
                { "/flag": "flag", "/label": "label" }
            ]
        }]
    }"#,
    );

    let original = BooleanNullStruct {
        flag: true,
        label: Some("present".into()),
    };
    let json_val = suture.stitch(&original).unwrap();
    let roundtripped: BooleanNullStruct = suture.unstitch(&json_val).unwrap();
    assert_eq!(roundtripped, original);

    // Also test with None
    let original_none = BooleanNullStruct {
        flag: false,
        label: None,
    };
    let json_val_none = suture.stitch(&original_none).unwrap();
    let roundtripped_none: BooleanNullStruct = suture.unstitch(&json_val_none).unwrap();
    assert_eq!(roundtripped_none, original_none);
}

#[test]
fn suture_metadata_is_correct() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "my_set",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#,
    );

    assert!(suture.is_response());
    assert!(!suture.is_request());
    assert_eq!(suture.name(), "my_set");
}

#[test]
fn multiple_suture_sets_parse_all() {
    let sutures: Vec<_> = sutures::v1::parse(
        r#"{
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
    }"#,
    )
    .unwrap()
    .into_iter()
    .map(|r| r.unwrap())
    .collect();

    assert_eq!(sutures.len(), 2);
    assert_eq!(sutures[0].name(), "first");
    assert_eq!(sutures[1].name(), "second");

    // First suture: /model -> model
    let input1 = json!({ "model": "gpt-4" });
    let result1: SingleField = sutures[0].unstitch(&input1).unwrap();
    assert_eq!(result1.model, "gpt-4");
}

#[test]
fn unstitch_optional_field_missing_from_json() {
    // When the JSON simply does not contain the mapped key, the struct field
    // should remain at its default (null for Option).
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "opt_missing",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#,
    );

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
fn stitch_optional_field_some() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "opt_some",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#,
    );

    let input = WithOptional {
        model: "gpt-4".into(),
        nickname: Some("fast_one".into()),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "nickname": "fast_one" }));
}

#[test]
fn roundtrip_optional_field_some() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "opt_rt",
            "capture": "response",
            "sutures": [
                { "/model": "model", "/nickname": "nickname" }
            ]
        }]
    }"#,
    );

    let original = WithOptional {
        model: "gpt-4".into(),
        nickname: Some("test_name".into()),
    };
    let json_val = suture.stitch(&original).unwrap();
    let roundtripped: WithOptional = suture.unstitch(&json_val).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn roundtrip_fan_out() {
    // Fan-out unstitch duplicates a value. Stitch reads from one of the targets.
    // The round-trip from JSON -> struct -> JSON should preserve the JSON value.
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout_rt",
            "capture": "response",
            "sutures": [
                { "/model": ["model", "model_name"] }
            ]
        }]
    }"#,
    );

    let original_json = json!({ "model": "gpt-4" });
    let intermediate: FanOutStruct = suture.unstitch(&original_json).unwrap();
    assert_eq!(intermediate.model, "gpt-4");
    assert_eq!(intermediate.model_name, "gpt-4");

    let roundtripped = suture.stitch(&intermediate).unwrap();
    assert_eq!(roundtripped, original_json);
}

#[test]
fn unstitch_empty_string_value() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#,
    );

    let input = json!({ "model": "" });
    let result: SingleField = suture.unstitch(&input).unwrap();
    assert_eq!(result, SingleField { model: "".into() });
}

#[test]
fn stitch_empty_string_value() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str",
            "capture": "response",
            "sutures": [
                { "/model": "model" }
            ]
        }]
    }"#,
    );

    let input = SingleField { model: "".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "" }));
}

#[test]
fn roundtrip_mixed_nested_and_flat_multiple_objects() {
    // Multiple suture objects in array, mixing flat and nested
    let suture = parse_first(
        r#"{
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
    }"#,
    );

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
