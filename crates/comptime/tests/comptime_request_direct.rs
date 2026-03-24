//! Exhaustive integration tests for **Request direction + Direct binding**
//! using the `sutures_comptime::parse!()` proc macro.
//!
//! These mirror the runtime tests in `crates/core/tests/stitch_request_direct.rs`
//! but compile the suture manifests at **compile time** via `parse!`. The resulting
//! `Vec<sutures::v1::Suture>` uses `Cow::Borrowed` (zero allocation) and should
//! behave identically to the runtime-parsed equivalent.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Seam, Stitch};

// ===========================================================================
// STITCH (struct -> JSON) tests
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. Single field, single target
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SingleField {
    model: String,
}

#[test]
fn stitch_single_field_single_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = SingleField {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

#[test]
fn unstitch_single_field_single_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "model": "gpt-4" });
    let result: SingleField = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        SingleField {
            model: "gpt-4".into()
        }
    );
}

// ---------------------------------------------------------------------------
// 2. Multiple fields
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MultiField {
    model: String,
    temperature: f64,
}

#[test]
fn stitch_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi",
            "capture": "request",
            "sutures": [{ "model": "/model", "temperature": "/temperature" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = MultiField {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "model": "gpt-4", "temperature": 0.7 })
    );
}

#[test]
fn unstitch_multiple_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi",
            "capture": "request",
            "sutures": [{ "model": "/model", "temperature": "/temperature" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "model": "gpt-4", "temperature": 0.7 });
    let result: MultiField = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        MultiField {
            model: "gpt-4".into(),
            temperature: 0.7,
        }
    );
}

// ---------------------------------------------------------------------------
// 3. Nested JSON target (deep JSON path)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ModelOnly {
    model: String,
}

#[test]
fn stitch_nested_json_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_target",
            "capture": "request",
            "sutures": [{ "model": "/config/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = ModelOnly {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "config": { "model": "gpt-4" } }));
}

#[test]
fn unstitch_nested_json_target() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_target",
            "capture": "request",
            "sutures": [{ "model": "/config/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "config": { "model": "gpt-4" } });
    let result: ModelOnly = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        ModelOnly {
            model: "gpt-4".into()
        }
    );
}

// ---------------------------------------------------------------------------
// 4. Fan-out (one field -> multiple targets)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutStruct {
    model: String,
}

#[test]
fn stitch_fan_out() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{ "model": ["/model", "/config/model_name"] }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = FanOutStruct {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "model": "gpt-4", "config": { "model_name": "gpt-4" } })
    );
}

#[test]
fn unstitch_fan_out_both_targets() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{ "model": ["/model", "/config/model_name"] }]
        }]
    }"#);
    let suture = &sutures[0];

    // Both targets present — the second target's value overwrites.
    let json_input = json!({
        "model": "gpt-3",
        "config": { "model_name": "gpt-4" }
    });
    let result: FanOutStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        FanOutStruct {
            model: "gpt-4".into()
        }
    );
}

#[test]
fn unstitch_fan_out_first_target_only() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{ "model": ["/model", "/config/model_name"] }]
        }]
    }"#);
    let suture = &sutures[0];

    // Only the first target present.
    let json_input = json!({ "model": "gpt-4" });
    let result: FanOutStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        FanOutStruct {
            model: "gpt-4".into()
        }
    );
}

// ---------------------------------------------------------------------------
// 5. Nested struct with dot path
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct InnerConfig {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct OuterWithConfig {
    #[seam(to_struct)]
    config: InnerConfig,
}

#[test]
fn stitch_nested_struct_dot_path() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_struct",
            "capture": "request",
            "sutures": [{ "config.model": "/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = OuterWithConfig {
        config: InnerConfig {
            model: "gpt-4".into(),
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

#[test]
fn unstitch_nested_struct_dot_path() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_struct",
            "capture": "request",
            "sutures": [{ "config.model": "/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "model": "gpt-4" });
    let result: OuterWithConfig = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        OuterWithConfig {
            config: InnerConfig {
                model: "gpt-4".into(),
            },
        }
    );
}

// ---------------------------------------------------------------------------
// 6. Deep nested struct (a.b.c -> /x/y/z) — 3-level nesting
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct LevelC {
    c: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct LevelB {
    #[seam(to_struct)]
    b: LevelC,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct LevelA {
    #[seam(to_struct)]
    a: LevelB,
}

#[test]
fn stitch_deep_nested_struct() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "request",
            "sutures": [{ "a.b.c": "/x/y/z" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = LevelA {
        a: LevelB {
            b: LevelC {
                c: "deep_value".into(),
            },
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "x": { "y": { "z": "deep_value" } } }));
}

#[test]
fn unstitch_deep_nested_struct() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "request",
            "sutures": [{ "a.b.c": "/x/y/z" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "x": { "y": { "z": "deep_value" } } });
    let result: LevelA = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        LevelA {
            a: LevelB {
                b: LevelC {
                    c: "deep_value".into(),
                },
            },
        }
    );
}

// ---------------------------------------------------------------------------
// 7. Nested object syntax (recursive suture)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ConfigForObject {
    model: String,
    temp: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct WithConfigObject {
    #[seam(to_struct)]
    config: ConfigForObject,
}

#[test]
fn stitch_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "obj_syntax",
            "capture": "request",
            "sutures": [{
                "config": {
                    "model": "/config/model",
                    "temp": "/config/temp"
                }
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = WithConfigObject {
        config: ConfigForObject {
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

#[test]
fn unstitch_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "obj_syntax",
            "capture": "request",
            "sutures": [{
                "config": {
                    "model": "/config/model",
                    "temp": "/config/temp"
                }
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "config": { "model": "gpt-4", "temp": 0.5 } });
    let result: WithConfigObject = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        WithConfigObject {
            config: ConfigForObject {
                model: "gpt-4".into(),
                temp: 0.5,
            },
        }
    );
}

// ---------------------------------------------------------------------------
// 8. Mixed flat + nested + fan-out in one suture
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedInner {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedStruct {
    name: String,
    #[seam(to_struct)]
    settings: MixedInner,
    tag: String,
}

#[test]
fn stitch_mixed() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "request",
            "sutures": [{
                "name": "/name",
                "settings.model": ["/model", "/config/model"],
                "tag": "/metadata/tag"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = MixedStruct {
        name: "test-run".into(),
        settings: MixedInner {
            model: "gpt-4".into(),
        },
        tag: "v1".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "name": "test-run",
            "model": "gpt-4",
            "config": { "model": "gpt-4" },
            "metadata": { "tag": "v1" }
        })
    );
}

#[test]
fn unstitch_mixed() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "request",
            "sutures": [{
                "name": "/name",
                "settings.model": "/model",
                "tag": "/metadata/tag"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({
        "name": "test-run",
        "model": "gpt-4",
        "metadata": { "tag": "v1" }
    });
    let result: MixedStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        MixedStruct {
            name: "test-run".into(),
            settings: MixedInner {
                model: "gpt-4".into(),
            },
            tag: "v1".into(),
        }
    );
}

// ---------------------------------------------------------------------------
// 9. Multiple suture objects in the sutures array (merged mappings)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MultiSutureStruct {
    model: String,
    temperature: f64,
    stream: bool,
}

#[test]
fn stitch_multiple_suture_objects() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "merged",
            "capture": "request",
            "sutures": [
                { "model": "/model" },
                { "temperature": "/temperature" },
                { "stream": "/stream" }
            ]
        }]
    }"#);
    let suture = &sutures[0];

    let input = MultiSutureStruct {
        model: "gpt-4".into(),
        temperature: 0.9,
        stream: true,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "model": "gpt-4", "temperature": 0.9, "stream": true })
    );
}

#[test]
fn unstitch_multiple_suture_objects() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "merged",
            "capture": "request",
            "sutures": [
                { "model": "/model" },
                { "temperature": "/temperature" },
                { "stream": "/stream" }
            ]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "model": "gpt-4", "temperature": 0.9, "stream": true });
    let result: MultiSutureStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        MultiSutureStruct {
            model: "gpt-4".into(),
            temperature: 0.9,
            stream: true,
        }
    );
}

// ===========================================================================
// ROUND-TRIP tests
// ===========================================================================

// ---------------------------------------------------------------------------
// 10. Round-trip: stitch then unstitch = identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RtSimple {
    model: String,
    temperature: f64,
}

#[test]
fn roundtrip_stitch_unstitch_simple() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt",
            "capture": "request",
            "sutures": [{ "model": "/model", "temperature": "/temperature" }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = RtSimple {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: RtSimple = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn roundtrip_stitch_unstitch_nested() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
    struct RtNInner {
        model: String,
        temp: f64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
    struct RtNOuter {
        name: String,
        #[seam(to_struct)]
        config: RtNInner,
    }

    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt",
            "capture": "request",
            "sutures": [{
                "name": "/name",
                "config.model": "/config/model",
                "config.temp": "/config/temp"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = RtNOuter {
        name: "run".into(),
        config: RtNInner {
            model: "gpt-4".into(),
            temp: 0.5,
        },
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: RtNOuter = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// 11. Round-trip: unstitch then stitch = identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RtJsonSimple {
    model: String,
    count: i64,
}

#[test]
fn roundtrip_unstitch_stitch_simple_json() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt",
            "capture": "request",
            "sutures": [{ "model": "/model", "count": "/count" }]
        }]
    }"#);
    let suture = &sutures[0];

    let original_json = json!({ "model": "gpt-4", "count": 5 });
    let intermediate: RtJsonSimple = suture.unstitch(&original_json).unwrap();
    let recovered_json = suture.stitch(&intermediate).unwrap();
    assert_eq!(original_json, recovered_json);
}

#[test]
fn roundtrip_unstitch_stitch_nested_json() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
    struct RtJsonInner {
        model: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
    struct RtJsonNested {
        #[seam(to_struct)]
        config: RtJsonInner,
    }

    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt",
            "capture": "request",
            "sutures": [{ "config.model": "/config/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let original_json = json!({ "config": { "model": "gpt-4" } });
    let intermediate: RtJsonNested = suture.unstitch(&original_json).unwrap();
    let recovered_json = suture.stitch(&intermediate).unwrap();
    assert_eq!(original_json, recovered_json);
}

// ---------------------------------------------------------------------------
// 12. Renamed fields with #[seam(rename = "...")]
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RenamedStruct {
    #[serde(rename = "model_type")]
    #[seam(rename = "model_type")]
    ty: String,
}

#[test]
fn stitch_renamed_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "renamed",
            "capture": "request",
            "sutures": [{ "model_type": "/type" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = RenamedStruct {
        ty: "chat".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "type": "chat" }));
}

#[test]
fn unstitch_renamed_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "renamed",
            "capture": "request",
            "sutures": [{ "model_type": "/type" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "type": "chat" });
    let result: RenamedStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        RenamedStruct {
            ty: "chat".into()
        }
    );
}

#[test]
fn roundtrip_renamed_field() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
    struct RenamedRt {
        #[serde(rename = "model_type")]
        #[seam(rename = "model_type")]
        ty: String,
        value: i64,
    }

    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rename_rt",
            "capture": "request",
            "sutures": [{
                "model_type": "/type",
                "value": "/value"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = RenamedRt {
        ty: "chat".into(),
        value: 99,
    };
    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "type": "chat", "value": 99 }));
    let recovered: RenamedRt = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// 13. Optional fields (Option<String>) with null
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct OptionalFields {
    name: String,
    nickname: Option<String>,
}

#[test]
fn stitch_optional_field_some() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "optional",
            "capture": "request",
            "sutures": [{ "name": "/name", "nickname": "/nickname" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = OptionalFields {
        name: "Alice".into(),
        nickname: Some("Ali".into()),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "name": "Alice", "nickname": "Ali" }));
}

#[test]
fn stitch_optional_field_none() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "optional",
            "capture": "request",
            "sutures": [{ "name": "/name", "nickname": "/nickname" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = OptionalFields {
        name: "Alice".into(),
        nickname: None,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "name": "Alice", "nickname": null }));
}

#[test]
fn unstitch_optional_field_null() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "optional",
            "capture": "request",
            "sutures": [{ "name": "/name", "nickname": "/nickname" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "name": "Alice", "nickname": null });
    let result: OptionalFields = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        OptionalFields {
            name: "Alice".into(),
            nickname: None,
        }
    );
}

#[test]
fn unstitch_optional_field_present() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "optional",
            "capture": "request",
            "sutures": [{ "name": "/name", "nickname": "/nickname" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "name": "Alice", "nickname": "Ali" });
    let result: OptionalFields = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        OptionalFields {
            name: "Alice".into(),
            nickname: Some("Ali".into()),
        }
    );
}

#[test]
fn roundtrip_null_optional() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
    struct NullRoundTrip {
        required: String,
        optional: Option<i64>,
    }

    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "null_rt",
            "capture": "request",
            "sutures": [{
                "required": "/required",
                "optional": "/optional"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    // With None
    let original_none = NullRoundTrip {
        required: "hello".into(),
        optional: None,
    };
    let stitched = suture.stitch(&original_none).unwrap();
    assert_eq!(
        stitched,
        json!({ "required": "hello", "optional": null })
    );
    let recovered: NullRoundTrip = suture.unstitch(&stitched).unwrap();
    assert_eq!(original_none, recovered);

    // With Some
    let original_some = NullRoundTrip {
        required: "hello".into(),
        optional: Some(42),
    };
    let stitched = suture.stitch(&original_some).unwrap();
    assert_eq!(
        stitched,
        json!({ "required": "hello", "optional": 42 })
    );
    let recovered: NullRoundTrip = suture.unstitch(&stitched).unwrap();
    assert_eq!(original_some, recovered);
}

// ---------------------------------------------------------------------------
// 14. Various value types (bool, int, float, string)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TypesStruct {
    flag: bool,
    count: i64,
    ratio: f64,
    label: String,
}

#[test]
fn stitch_various_types() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "types",
            "capture": "request",
            "sutures": [{
                "flag": "/flag",
                "count": "/count",
                "ratio": "/ratio",
                "label": "/label"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = TypesStruct {
        flag: true,
        count: 42,
        ratio: 2.718,
        label: "test".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "flag": true, "count": 42, "ratio": 2.718, "label": "test" })
    );
}

#[test]
fn unstitch_various_types() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "types",
            "capture": "request",
            "sutures": [{
                "flag": "/flag",
                "count": "/count",
                "ratio": "/ratio",
                "label": "/label"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "flag": true, "count": 42, "ratio": 2.718, "label": "test" });
    let result: TypesStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        TypesStruct {
            flag: true,
            count: 42,
            ratio: 2.718,
            label: "test".into(),
        }
    );
}

#[test]
fn roundtrip_various_types() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "types",
            "capture": "request",
            "sutures": [{
                "flag": "/flag",
                "count": "/count",
                "ratio": "/ratio",
                "label": "/label"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = TypesStruct {
        flag: false,
        count: -100,
        ratio: 0.0,
        label: "".into(),
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: TypesStruct = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// 15. Unicode values
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct UnicodeStruct {
    name: String,
}

#[test]
fn stitch_unicode_json_value() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "unicode",
            "capture": "request",
            "sutures": [{ "name": "/name" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = UnicodeStruct {
        name: "\u{1f600} Hello \u{4e16}\u{754c}".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "name": "\u{1f600} Hello \u{4e16}\u{754c}" })
    );
}

#[test]
fn unstitch_unicode_json_value() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "unicode",
            "capture": "request",
            "sutures": [{ "name": "/name" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "name": "\u{1f600} Hello \u{4e16}\u{754c}" });
    let result: UnicodeStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnicodeStruct {
            name: "\u{1f600} Hello \u{4e16}\u{754c}".into(),
        }
    );
}

#[test]
fn roundtrip_unicode() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "unicode",
            "capture": "request",
            "sutures": [{ "name": "/name" }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = UnicodeStruct {
        name: "\u{1f680}\u{2764}\u{fe0f}\u{1f30d}".into(),
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: UnicodeStruct = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// 16. Very deep JSON nesting (5+ levels)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct VeryDeepStruct {
    value: String,
}

#[test]
fn stitch_very_deep_json_nesting() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "request",
            "sutures": [{ "value": "/a/b/c/d/e/f/g" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = VeryDeepStruct {
        value: "deep!".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "a": { "b": { "c": { "d": { "e": { "f": { "g": "deep!" } } } } } } })
    );
}

#[test]
fn unstitch_very_deep_json_nesting() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "request",
            "sutures": [{ "value": "/a/b/c/d/e/f/g" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input =
        json!({ "a": { "b": { "c": { "d": { "e": { "f": { "g": "deep!" } } } } } } });
    let result: VeryDeepStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        VeryDeepStruct {
            value: "deep!".into()
        }
    );
}

#[test]
fn roundtrip_very_deep_nesting() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "request",
            "sutures": [{ "value": "/a/b/c/d/e/f/g" }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = VeryDeepStruct {
        value: "round-trip".into(),
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: VeryDeepStruct = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// 17. Empty sutures array (no mappings)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct EmptyMappingStruct {}

#[test]
fn stitch_empty_sutures_array() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty",
            "capture": "request",
            "sutures": [{}]
        }]
    }"#);
    let suture = &sutures[0];

    let input = EmptyMappingStruct {};
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({}));
}

#[test]
fn unstitch_empty_sutures_array() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty",
            "capture": "request",
            "sutures": [{}]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({});
    let result: EmptyMappingStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(result, EmptyMappingStruct {});
}

// ---------------------------------------------------------------------------
// 18. Multiple suture_sets producing multiple sutures in the Vec
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SetAStruct {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SetBStruct {
    temperature: f64,
}

#[test]
fn multiple_suture_sets_vec_length() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "set_a",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "set_b",
                "capture": "request",
                "sutures": [{ "temperature": "/temperature" }]
            }
        ]
    }"#);

    assert_eq!(sutures.len(), 2);
    assert_eq!(sutures[0].name(), "set_a");
    assert_eq!(sutures[1].name(), "set_b");
}

#[test]
fn multiple_suture_sets_stitch_independently() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "set_a",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "set_b",
                "capture": "request",
                "sutures": [{ "temperature": "/temperature" }]
            }
        ]
    }"#);

    let input_a = SetAStruct {
        model: "gpt-4".into(),
    };
    let result_a = sutures[0].stitch(&input_a).unwrap();
    assert_eq!(result_a, json!({ "model": "gpt-4" }));

    let input_b = SetBStruct {
        temperature: 0.9,
    };
    let result_b = sutures[1].stitch(&input_b).unwrap();
    assert_eq!(result_b, json!({ "temperature": 0.9 }));
}

#[test]
fn multiple_suture_sets_unstitch_independently() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "set_a",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "set_b",
                "capture": "request",
                "sutures": [{ "temperature": "/temperature" }]
            }
        ]
    }"#);

    let json_a = json!({ "model": "gpt-4" });
    let result_a: SetAStruct = sutures[0].unstitch(&json_a).unwrap();
    assert_eq!(
        result_a,
        SetAStruct {
            model: "gpt-4".into()
        }
    );

    let json_b = json!({ "temperature": 0.9 });
    let result_b: SetBStruct = sutures[1].unstitch(&json_b).unwrap();
    assert_eq!(result_b, SetBStruct { temperature: 0.9 });
}

#[test]
fn multiple_suture_sets_roundtrip() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "set_a",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "set_b",
                "capture": "request",
                "sutures": [{ "temperature": "/temperature" }]
            }
        ]
    }"#);

    let original_a = SetAStruct {
        model: "claude".into(),
    };
    let stitched_a = sutures[0].stitch(&original_a).unwrap();
    let recovered_a: SetAStruct = sutures[0].unstitch(&stitched_a).unwrap();
    assert_eq!(original_a, recovered_a);

    let original_b = SetBStruct {
        temperature: 1.0,
    };
    let stitched_b = sutures[1].stitch(&original_b).unwrap();
    let recovered_b: SetBStruct = sutures[1].unstitch(&stitched_b).unwrap();
    assert_eq!(original_b, recovered_b);
}

// ===========================================================================
// ADDITIONAL COVERAGE tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Comptime suture direction verification
// ---------------------------------------------------------------------------

#[test]
fn comptime_suture_is_request_direction() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "dir",
            "capture": "request",
            "sutures": [{ "x": "/x" }]
        }]
    }"#);
    let suture = &sutures[0];
    assert!(suture.is_request());
    assert!(!suture.is_response());
}

// ---------------------------------------------------------------------------
// Comptime suture metadata accessors
// ---------------------------------------------------------------------------

#[test]
fn comptime_suture_name_accessor() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "my_manifest",
        "suture_sets": [{
            "name": "my_set",
            "capture": "request",
            "sutures": [{ "x": "/x" }]
        }]
    }"#);
    let suture = &sutures[0];
    assert_eq!(suture.name(), "my_set");
}

#[test]
fn comptime_suture_optional_metadata() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "meta",
            "id": "abc-123",
            "description": "A test suture",
            "version": "1.0.0",
            "capture": "request",
            "sutures": [{ "x": "/x" }]
        }]
    }"#);
    let suture = &sutures[0];
    assert_eq!(suture.id(), Some("abc-123"));
    assert_eq!(suture.description(), Some("A test suture"));
    assert_eq!(suture.version(), Some("1.0.0"));
}

// ---------------------------------------------------------------------------
// Deep struct (5 levels) round-trip via comptime
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Cd5 {
    val: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Cd4 {
    #[seam(to_struct)]
    d5: Cd5,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Cd3 {
    #[seam(to_struct)]
    d4: Cd4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Cd2 {
    #[seam(to_struct)]
    d3: Cd3,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Cd1 {
    #[seam(to_struct)]
    d2: Cd2,
}

#[test]
fn roundtrip_five_level_nesting() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep5",
            "capture": "request",
            "sutures": [{ "d2.d3.d4.d5.val": "/a/b/c/d/e" }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = Cd1 {
        d2: Cd2 {
            d3: Cd3 {
                d4: Cd4 {
                    d5: Cd5 {
                        val: "bottom".into(),
                    },
                },
            },
        },
    };
    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(
        stitched,
        json!({ "a": { "b": { "c": { "d": { "e": "bottom" } } } } })
    );
    let recovered: Cd1 = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// Partial mapping: struct field not in suture is excluded from output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct PartialMapping {
    included: String,
    excluded: String,
}

#[test]
fn stitch_partial_mapping_only_includes_mapped_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "partial",
            "capture": "request",
            "sutures": [{ "included": "/included" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = PartialMapping {
        included: "yes".into(),
        excluded: "no".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "included": "yes" }));
    assert!(result.get("excluded").is_none());
}

// ---------------------------------------------------------------------------
// Extra fields in JSON not in suture mapping (ignored during unstitch)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct PartialStruct {
    model: String,
}

#[test]
fn unstitch_extra_json_fields_ignored() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "partial",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({
        "model": "gpt-4",
        "temperature": 0.7,
        "stream": true,
        "extra": { "nested": "value" }
    });
    let result: PartialStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        PartialStruct {
            model: "gpt-4".into()
        }
    );
}

// ---------------------------------------------------------------------------
// Missing required field in unstitch produces error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RequiredFieldStruct {
    name: String,
}

#[test]
fn unstitch_missing_required_field_errors() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "missing",
            "capture": "request",
            "sutures": [{ "name": "/name" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({});
    let result = suture.unstitch::<RequiredFieldStruct>(&json_input);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Missing field with serde default
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MissingFieldStruct {
    name: String,
    #[serde(default)]
    age: i64,
}

#[test]
fn unstitch_missing_field_with_serde_default() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "missing",
            "capture": "request",
            "sutures": [{ "name": "/name", "age": "/age" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "name": "Alice" });
    let result: MissingFieldStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        MissingFieldStruct {
            name: "Alice".into(),
            age: 0,
        }
    );
}

// ---------------------------------------------------------------------------
// Merged trie from multiple suture objects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MergedInner {
    x: i64,
    y: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MergedOuter {
    #[seam(to_struct)]
    point: MergedInner,
}

#[test]
fn stitch_merged_trie_from_multiple_suture_objects() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "merged",
            "capture": "request",
            "sutures": [
                { "point.x": "/x" },
                { "point.y": "/y" }
            ]
        }]
    }"#);
    let suture = &sutures[0];

    let input = MergedOuter {
        point: MergedInner { x: 10, y: 20 },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "x": 10, "y": 20 }));
}

#[test]
fn unstitch_merged_trie_from_multiple_suture_objects() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "merged",
            "capture": "request",
            "sutures": [
                { "point.x": "/x" },
                { "point.y": "/y" }
            ]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "x": 10, "y": 20 });
    let result: MergedOuter = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        MergedOuter {
            point: MergedInner { x: 10, y: 20 },
        }
    );
}

// ---------------------------------------------------------------------------
// Nested object syntax with fan-out inside
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutInnerConfig {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutNested {
    #[seam(to_struct)]
    config: FanOutInnerConfig,
}

#[test]
fn stitch_nested_object_with_fanout() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout_nested",
            "capture": "request",
            "sutures": [{
                "config": {
                    "model": ["/model", "/params/model"]
                }
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = FanOutNested {
        config: FanOutInnerConfig {
            model: "gpt-4".into(),
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "model": "gpt-4", "params": { "model": "gpt-4" } })
    );
}

// ---------------------------------------------------------------------------
// Fan-out with targets at different depths
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutDepthStruct {
    name: String,
}

#[test]
fn stitch_fan_out_different_depths() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout_depth",
            "capture": "request",
            "sutures": [{
                "name": ["/name", "/meta/info/name", "/flat_name"]
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = FanOutDepthStruct {
        name: "Alice".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["name"], json!("Alice"));
    assert_eq!(result["meta"]["info"]["name"], json!("Alice"));
    assert_eq!(result["flat_name"], json!("Alice"));
}

// ---------------------------------------------------------------------------
// Multiple nested struct fields mapping to the same JSON parent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct LeftRight {
    left: String,
    right: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SameJsonParent {
    #[seam(to_struct)]
    coords: LeftRight,
}

#[test]
fn stitch_nested_fields_same_json_parent() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "same_parent",
            "capture": "request",
            "sutures": [{
                "coords.left": "/position/left",
                "coords.right": "/position/right"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = SameJsonParent {
        coords: LeftRight {
            left: "L".into(),
            right: "R".into(),
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "position": { "left": "L", "right": "R" } })
    );
}

#[test]
fn unstitch_nested_fields_same_json_parent() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "same_parent",
            "capture": "request",
            "sutures": [{
                "coords.left": "/position/left",
                "coords.right": "/position/right"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "position": { "left": "L", "right": "R" } });
    let result: SameJsonParent = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        SameJsonParent {
            coords: LeftRight {
                left: "L".into(),
                right: "R".into(),
            },
        }
    );
}

// ---------------------------------------------------------------------------
// Vec field (terminal, handled by serde)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct WithVec {
    tags: Vec<String>,
}

#[test]
fn stitch_vec_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "vec",
            "capture": "request",
            "sutures": [{ "tags": "/tags" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = WithVec {
        tags: vec!["a".into(), "b".into(), "c".into()],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "tags": ["a", "b", "c"] }));
}

#[test]
fn unstitch_vec_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "vec",
            "capture": "request",
            "sutures": [{ "tags": "/tags" }]
        }]
    }"#);
    let suture = &sutures[0];

    let json_input = json!({ "tags": ["a", "b", "c"] });
    let result: WithVec = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        WithVec {
            tags: vec!["a".into(), "b".into(), "c".into()],
        }
    );
}

#[test]
fn roundtrip_vec_field() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "vec",
            "capture": "request",
            "sutures": [{ "tags": "/tags" }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = WithVec {
        tags: vec!["x".into(), "y".into()],
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: WithVec = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// Complex nested + fan-out + mixed round-trip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ComplexInner {
    prompt: String,
    max_tokens: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ComplexOuter {
    model: String,
    #[seam(to_struct)]
    params: ComplexInner,
    stream: bool,
}

#[test]
fn roundtrip_complex_mixed() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "complex",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "params": {
                    "prompt": "/prompt",
                    "max_tokens": "/config/max_tokens"
                },
                "stream": "/stream"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = ComplexOuter {
        model: "gpt-4".into(),
        params: ComplexInner {
            prompt: "Hello world".into(),
            max_tokens: 1024,
        },
        stream: false,
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(
        stitched,
        json!({
            "model": "gpt-4",
            "prompt": "Hello world",
            "config": { "max_tokens": 1024 },
            "stream": false
        })
    );

    let recovered: ComplexOuter = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// Deep nested struct with nested object syntax round-trip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RtDeepLeaf {
    val: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RtDeepMid {
    #[seam(to_struct)]
    leaf: RtDeepLeaf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RtDeepRoot {
    #[seam(to_struct)]
    mid: RtDeepMid,
}

#[test]
fn roundtrip_deep_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_rt",
            "capture": "request",
            "sutures": [{
                "mid": {
                    "leaf": {
                        "val": "/data/inner/value"
                    }
                }
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = RtDeepRoot {
        mid: RtDeepMid {
            leaf: RtDeepLeaf {
                val: "bottom".into(),
            },
        },
    };
    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(
        stitched,
        json!({ "data": { "inner": { "value": "bottom" } } })
    );
    let recovered: RtDeepRoot = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// Overlapping JSON targets (last write wins)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct OverlapStruct {
    first: String,
    second: String,
}

#[test]
fn stitch_overlapping_json_targets() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "overlap",
            "capture": "request",
            "sutures": [{ "first": "/output", "second": "/output" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = OverlapStruct {
        first: "aaa".into(),
        second: "bbb".into(),
    };
    let result = suture.stitch(&input).unwrap();
    // Both fields write to /output — the last one processed wins.
    assert_eq!(result["output"], json!("bbb"));
}

// ---------------------------------------------------------------------------
// Dot-path and object syntax produce same result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SyntaxCompareInner {
    val: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SyntaxCompareOuter {
    #[seam(to_struct)]
    inner: SyntaxCompareInner,
}

#[test]
fn dot_path_and_object_syntax_produce_same_result() {
    let sutures_dot = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "dot",
            "capture": "request",
            "sutures": [{ "inner.val": "/out/value" }]
        }]
    }"#);

    let sutures_obj = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "obj",
            "capture": "request",
            "sutures": [{
                "inner": { "val": "/out/value" }
            }]
        }]
    }"#);

    let input = SyntaxCompareOuter {
        inner: SyntaxCompareInner {
            val: "same".into(),
        },
    };

    let result_dot = sutures_dot[0].stitch(&input).unwrap();
    let result_obj = sutures_obj[0].stitch(&input).unwrap();
    assert_eq!(result_dot, result_obj);
    assert_eq!(result_dot, json!({ "out": { "value": "same" } }));
}

// ---------------------------------------------------------------------------
// Sparse mapping: struct with many fields, only some mapped
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SparseStruct {
    a: String,
    b: String,
    c: String,
    d: String,
    e: String,
}

#[test]
fn stitch_sparse_mapping() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "sparse",
            "capture": "request",
            "sutures": [{ "a": "/alpha", "c": "/gamma", "e": "/epsilon" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = SparseStruct {
        a: "1".into(),
        b: "2".into(),
        c: "3".into(),
        d: "4".into(),
        e: "5".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "alpha": "1", "gamma": "3", "epsilon": "5" })
    );
    assert!(result.get("b").is_none());
    assert!(result.get("d").is_none());
}

// ---------------------------------------------------------------------------
// Deep JSON path via stitch (5+ levels) with deep JSON target
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepJsonTarget {
    name: String,
}

#[test]
fn stitch_deep_json_path() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_path",
            "capture": "request",
            "sutures": [{ "name": "/data/attributes/name" }]
        }]
    }"#);
    let suture = &sutures[0];

    let input = DeepJsonTarget {
        name: "Alice".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "data": { "attributes": { "name": "Alice" } } })
    );
}

// ---------------------------------------------------------------------------
// Round-trip: multi-type struct
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_stitch_unstitch_multi_types() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
    struct RtMulti {
        a: String,
        b: i64,
        c: bool,
        d: f64,
    }

    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rt",
            "capture": "request",
            "sutures": [{
                "a": "/alpha",
                "b": "/beta",
                "c": "/gamma",
                "d": "/delta"
            }]
        }]
    }"#);
    let suture = &sutures[0];

    let original = RtMulti {
        a: "hello".into(),
        b: 42,
        c: true,
        d: 3.14,
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: RtMulti = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// Comptime vs runtime equivalence: verify same output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct EquivStruct {
    model: String,
    temperature: f64,
}

#[test]
fn comptime_and_runtime_produce_same_stitch_result() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "equiv",
            "capture": "request",
            "sutures": [{ "model": "/model", "temperature": "/temperature" }]
        }]
    }"#);

    let runtime_sutures = sutures::v1::parse(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "equiv",
            "capture": "request",
            "sutures": [{ "model": "/model", "temperature": "/temperature" }]
        }]
    }"#)
    .unwrap();
    let runtime_suture = runtime_sutures.into_iter().next().unwrap().unwrap();

    let input = EquivStruct {
        model: "gpt-4".into(),
        temperature: 0.7,
    };

    let comptime_result = comptime_sutures[0].stitch(&input).unwrap();
    let runtime_result = runtime_suture.stitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);
}

#[test]
fn comptime_and_runtime_produce_same_unstitch_result() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "equiv",
            "capture": "request",
            "sutures": [{ "model": "/model", "temperature": "/temperature" }]
        }]
    }"#);

    let runtime_sutures = sutures::v1::parse(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "equiv",
            "capture": "request",
            "sutures": [{ "model": "/model", "temperature": "/temperature" }]
        }]
    }"#)
    .unwrap();
    let runtime_suture = runtime_sutures.into_iter().next().unwrap().unwrap();

    let json_input = json!({ "model": "gpt-4", "temperature": 0.7 });

    let comptime_result: EquivStruct = comptime_sutures[0].unstitch(&json_input).unwrap();
    let runtime_result: EquivStruct = runtime_suture.unstitch(&json_input).unwrap();
    assert_eq!(comptime_result, runtime_result);
}
