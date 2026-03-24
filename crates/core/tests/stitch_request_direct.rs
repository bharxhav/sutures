//! Exhaustive integration tests for **Request direction + Direct binding**
//! covering both `stitch` (struct -> JSON) and `unstitch` (JSON -> struct).
//!
//! Every test is self-contained: it defines its own structs, builds a
//! `.sutures.json` string, parses it, and exercises `Stitch::stitch` /
//! `Stitch::unstitch`.

use serde_json::json;
use sutures::Stitch;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn parse_first(json: &str) -> sutures::v1::Suture {
    sutures::v1::parse(json)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .unwrap()
}

// ===========================================================================
// STITCH (struct -> JSON) tests
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. Single field, single target
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SingleField {
    model: String,
}

#[test]
fn stitch_single_field_single_target() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "single",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            }]
        }"#,
    );

    let input = SingleField {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

// ---------------------------------------------------------------------------
// 2. Multiple fields
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MultiField {
    model: String,
    temperature: f64,
}

#[test]
fn stitch_multiple_fields() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "multi",
                "capture": "request",
                "sutures": [{ "model": "/model", "temperature": "/temperature" }]
            }]
        }"#,
    );

    let input = MultiField {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "temperature": 0.7 }));
}

// ---------------------------------------------------------------------------
// 3. Nested JSON target (deep JSON path)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ModelOnly {
    model: String,
}

#[test]
fn stitch_nested_json_target() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "nested_target",
                "capture": "request",
                "sutures": [{ "model": "/config/model" }]
            }]
        }"#,
    );

    let input = ModelOnly {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "config": { "model": "gpt-4" } }));
}

// ---------------------------------------------------------------------------
// 4. Fan-out (one field -> multiple targets)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FanOutStruct {
    model: String,
}

#[test]
fn stitch_fan_out() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "fanout",
                "capture": "request",
                "sutures": [{ "model": ["/model", "/config/model_name"] }]
            }]
        }"#,
    );

    let input = FanOutStruct {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "model": "gpt-4", "config": { "model_name": "gpt-4" } })
    );
}

// ---------------------------------------------------------------------------
// 5. Nested struct source with dot path
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct InnerConfig {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct OuterWithConfig {
    #[seam(to_struct)]
    config: InnerConfig,
}

#[test]
fn stitch_nested_struct_dot_path() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "nested_struct",
                "capture": "request",
                "sutures": [{ "config.model": "/model" }]
            }]
        }"#,
    );

    let input = OuterWithConfig {
        config: InnerConfig {
            model: "gpt-4".into(),
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4" }));
}

// ---------------------------------------------------------------------------
// 6. Deep nested struct (a.b.c -> /x/y/z)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct LevelC {
    c: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct LevelB {
    #[seam(to_struct)]
    b: LevelC,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct LevelA {
    #[seam(to_struct)]
    a: LevelB,
}

#[test]
fn stitch_deep_nested_struct() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "deep",
                "capture": "request",
                "sutures": [{ "a.b.c": "/x/y/z" }]
            }]
        }"#,
    );

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

// ---------------------------------------------------------------------------
// 7. Nested object syntax (recursive suture)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ConfigForObject {
    model: String,
    temp: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct WithConfigObject {
    #[seam(to_struct)]
    config: ConfigForObject,
}

#[test]
fn stitch_nested_object_syntax() {
    // Nested object syntax: the key "config" has an object value with sub-mappings.
    // This is equivalent to "config.model": "/config/model", "config.temp": "/config/temp"
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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

// ---------------------------------------------------------------------------
// 8. Mixed: flat, nested, and fan-out in the same suture
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedInner {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedStruct {
    name: String,
    #[seam(to_struct)]
    settings: MixedInner,
    tag: String,
}

#[test]
fn stitch_mixed() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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

// ---------------------------------------------------------------------------
// 9. Multiple suture objects in the sutures array (merged mappings)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MultiSutureStruct {
    model: String,
    temperature: f64,
    stream: bool,
}

#[test]
fn stitch_multiple_suture_objects() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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

// ---------------------------------------------------------------------------
// 10. Renamed fields with #[seam(rename)] — suture key matches serde name
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RenamedStruct {
    // The suture key references the serde-serialized field name.
    // Since stitch serializes the struct first, the serde field name
    // must match the trie key. We use #[serde(rename)] to align them.
    #[serde(rename = "model_type")]
    #[seam(rename = "model_type")]
    ty: String,
}

#[test]
fn stitch_renamed_field() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "renamed",
                "capture": "request",
                "sutures": [{ "model_type": "/type" }]
            }]
        }"#,
    );

    let input = RenamedStruct { ty: "chat".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "type": "chat" }));
}

// ---------------------------------------------------------------------------
// 11. Field with deep JSON path (/data/attributes/name)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DeepJsonTarget {
    name: String,
}

#[test]
fn stitch_deep_json_path() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "deep_path",
                "capture": "request",
                "sutures": [{ "name": "/data/attributes/name" }]
            }]
        }"#,
    );

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
// 12. Same JSON path targeted by different struct fields (last write wins)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct OverlapStruct {
    first: String,
    second: String,
}

#[test]
fn stitch_overlapping_json_targets() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "overlap",
                "capture": "request",
                "sutures": [{ "first": "/output", "second": "/output" }]
            }]
        }"#,
    );

    let input = OverlapStruct {
        first: "aaa".into(),
        second: "bbb".into(),
    };
    let result = suture.stitch(&input).unwrap();
    // Both fields write to /output — the last one processed wins.
    // The trie processes children in order: first, then second.
    assert_eq!(result["output"], json!("bbb"));
}

// ===========================================================================
// UNSTITCH (JSON -> struct) tests
// ===========================================================================

// ---------------------------------------------------------------------------
// 13. Single field reverse
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnstitchSingle {
    model: String,
}

#[test]
fn unstitch_single_field() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "single",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            }]
        }"#,
    );

    let json_input = json!({ "model": "gpt-4" });
    let result: UnstitchSingle = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnstitchSingle {
            model: "gpt-4".into()
        }
    );
}

// ---------------------------------------------------------------------------
// 14. Multiple fields reverse
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnstitchMulti {
    model: String,
    temperature: f64,
}

#[test]
fn unstitch_multiple_fields() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "multi",
                "capture": "request",
                "sutures": [{ "model": "/model", "temperature": "/temperature" }]
            }]
        }"#,
    );

    let json_input = json!({ "model": "gpt-4", "temperature": 0.7 });
    let result: UnstitchMulti = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnstitchMulti {
            model: "gpt-4".into(),
            temperature: 0.7,
        }
    );
}

// ---------------------------------------------------------------------------
// 15. Nested JSON source
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnstitchNestedJson {
    model: String,
}

#[test]
fn unstitch_nested_json_source() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "nested",
                "capture": "request",
                "sutures": [{ "model": "/config/model" }]
            }]
        }"#,
    );

    let json_input = json!({ "config": { "model": "gpt-4" } });
    let result: UnstitchNestedJson = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnstitchNestedJson {
            model: "gpt-4".into()
        }
    );
}

// ---------------------------------------------------------------------------
// 16. Fan-out reverse: value read from the first matching target
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnstitchFanOut {
    model: String,
}

#[test]
fn unstitch_fan_out() {
    // For request direction unstitch: the reverse walk collects leaf mappings.
    // Fan-out means the same trie leaf has multiple targets. The reverse walk
    // reads from each target and writes to the struct field — last write wins.
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "fanout",
                "capture": "request",
                "sutures": [{ "model": ["/model", "/config/model_name"] }]
            }]
        }"#,
    );

    // Both targets present — the second target's value overwrites.
    let json_input = json!({
        "model": "gpt-3",
        "config": { "model_name": "gpt-4" }
    });
    let result: UnstitchFanOut = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnstitchFanOut {
            model: "gpt-4".into()
        }
    );
}

#[test]
fn unstitch_fan_out_first_target_only() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "fanout",
                "capture": "request",
                "sutures": [{ "model": ["/model", "/config/model_name"] }]
            }]
        }"#,
    );

    // Only the first target present.
    let json_input = json!({ "model": "gpt-4" });
    let result: UnstitchFanOut = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnstitchFanOut {
            model: "gpt-4".into()
        }
    );
}

// ---------------------------------------------------------------------------
// 17. Nested struct target with dot path
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnstitchInner {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnstitchOuter {
    #[seam(to_struct)]
    config: UnstitchInner,
}

#[test]
fn unstitch_nested_struct_dot_path() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "nested_struct",
                "capture": "request",
                "sutures": [{ "config.model": "/model" }]
            }]
        }"#,
    );

    let json_input = json!({ "model": "gpt-4" });
    let result: UnstitchOuter = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnstitchOuter {
            config: UnstitchInner {
                model: "gpt-4".into(),
            },
        }
    );
}

// ---------------------------------------------------------------------------
// 18. Deep nested struct reverse
// ---------------------------------------------------------------------------

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

#[test]
fn unstitch_deep_nested_struct() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "deep",
                "capture": "request",
                "sutures": [{ "a.b.c": "/x/y/z" }]
            }]
        }"#,
    );

    let json_input = json!({ "x": { "y": { "z": "deep_value" } } });
    let result: DeepA = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        DeepA {
            a: DeepB {
                b: DeepC {
                    c: "deep_value".into(),
                },
            },
        }
    );
}

// ---------------------------------------------------------------------------
// 19. Nested object syntax reverse
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ObjInner {
    model: String,
    temp: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ObjOuter {
    #[seam(to_struct)]
    config: ObjInner,
}

#[test]
fn unstitch_nested_object_syntax() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

    let json_input = json!({ "config": { "model": "gpt-4", "temp": 0.5 } });
    let result: ObjOuter = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        ObjOuter {
            config: ObjInner {
                model: "gpt-4".into(),
                temp: 0.5,
            },
        }
    );
}

// ---------------------------------------------------------------------------
// 20. Mixed reverse
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedRevInner {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedRevStruct {
    name: String,
    #[seam(to_struct)]
    settings: MixedRevInner,
    tag: String,
}

#[test]
fn unstitch_mixed() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

    let json_input = json!({
        "name": "test-run",
        "model": "gpt-4",
        "metadata": { "tag": "v1" }
    });
    let result: MixedRevStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        MixedRevStruct {
            name: "test-run".into(),
            settings: MixedRevInner {
                model: "gpt-4".into(),
            },
            tag: "v1".into(),
        }
    );
}

// ===========================================================================
// ROUND-TRIP tests
// ===========================================================================

// ---------------------------------------------------------------------------
// 21. stitch then unstitch = identity for simple struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtSimple {
    model: String,
    temperature: f64,
}

#[test]
fn roundtrip_stitch_unstitch_simple() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "rt",
                "capture": "request",
                "sutures": [{ "model": "/model", "temperature": "/temperature" }]
            }]
        }"#,
    );

    let original = RtSimple {
        model: "gpt-4".into(),
        temperature: 0.7,
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: RtSimple = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// 22. stitch then unstitch = identity for nested struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtInner {
    model: String,
    temp: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtNested {
    name: String,
    #[seam(to_struct)]
    config: RtInner,
}

#[test]
fn roundtrip_stitch_unstitch_nested() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

    let original = RtNested {
        name: "run".into(),
        config: RtInner {
            model: "gpt-4".into(),
            temp: 0.5,
        },
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: RtNested = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// 23. stitch then unstitch = identity for multi-field struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtMulti {
    a: String,
    b: i64,
    c: bool,
    d: f64,
}

#[test]
fn roundtrip_stitch_unstitch_multi() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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
// 24. unstitch then stitch = identity for simple JSON
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtJsonSimple {
    model: String,
    count: i64,
}

#[test]
fn roundtrip_unstitch_stitch_simple_json() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "rt",
                "capture": "request",
                "sutures": [{ "model": "/model", "count": "/count" }]
            }]
        }"#,
    );

    let original_json = json!({ "model": "gpt-4", "count": 5 });
    let intermediate: RtJsonSimple = suture.unstitch(&original_json).unwrap();
    let recovered_json = suture.stitch(&intermediate).unwrap();
    assert_eq!(original_json, recovered_json);
}

// ---------------------------------------------------------------------------
// 25. unstitch then stitch = identity for nested JSON
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtJsonInner {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtJsonNested {
    #[seam(to_struct)]
    config: RtJsonInner,
}

#[test]
fn roundtrip_unstitch_stitch_nested_json() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "rt",
                "capture": "request",
                "sutures": [{ "config.model": "/config/model" }]
            }]
        }"#,
    );

    let original_json = json!({ "config": { "model": "gpt-4" } });
    let intermediate: RtJsonNested = suture.unstitch(&original_json).unwrap();
    let recovered_json = suture.stitch(&intermediate).unwrap();
    assert_eq!(original_json, recovered_json);
}

// ===========================================================================
// EDGE CASE tests
// ===========================================================================

// ---------------------------------------------------------------------------
// 26. Empty sutures array
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct EmptyMappingStruct {}

#[test]
fn stitch_empty_sutures_array() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "empty",
                "capture": "request",
                "sutures": [{}]
            }]
        }"#,
    );

    let input = EmptyMappingStruct {};
    let result = suture.stitch(&input).unwrap();
    // Empty sutures array with empty suture object means no mappings —
    // the output should be an empty JSON object.
    assert_eq!(result, json!({}));
}

#[test]
fn unstitch_empty_sutures_array() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "empty",
                "capture": "request",
                "sutures": [{}]
            }]
        }"#,
    );

    let json_input = json!({});
    let result: EmptyMappingStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(result, EmptyMappingStruct {});
}

// ---------------------------------------------------------------------------
// 27. Optional/nullable fields (Option<String> with null JSON value)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct OptionalFields {
    name: String,
    nickname: Option<String>,
}

#[test]
fn stitch_optional_field_some() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "optional",
                "capture": "request",
                "sutures": [{ "name": "/name", "nickname": "/nickname" }]
            }]
        }"#,
    );

    let input = OptionalFields {
        name: "Alice".into(),
        nickname: Some("Ali".into()),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "name": "Alice", "nickname": "Ali" }));
}

#[test]
fn stitch_optional_field_none() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "optional",
                "capture": "request",
                "sutures": [{ "name": "/name", "nickname": "/nickname" }]
            }]
        }"#,
    );

    let input = OptionalFields {
        name: "Alice".into(),
        nickname: None,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "name": "Alice", "nickname": null }));
}

#[test]
fn unstitch_optional_field_null() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "optional",
                "capture": "request",
                "sutures": [{ "name": "/name", "nickname": "/nickname" }]
            }]
        }"#,
    );

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
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "optional",
                "capture": "request",
                "sutures": [{ "name": "/name", "nickname": "/nickname" }]
            }]
        }"#,
    );

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

// ---------------------------------------------------------------------------
// 28. Struct field missing from JSON in unstitch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MissingFieldStruct {
    name: String,
    #[serde(default)]
    age: i64,
}

#[test]
fn unstitch_missing_field_with_serde_default() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "missing",
                "capture": "request",
                "sutures": [{ "name": "/name", "age": "/age" }]
            }]
        }"#,
    );

    // Only /name present, /age is missing. The reverse walk won't find a
    // value at /age, so the struct's "age" field stays as Null. Serde's
    // #[serde(default)] handles the deserialization from null to 0.
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

#[test]
fn unstitch_missing_required_field_errors() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "missing",
                "capture": "request",
                "sutures": [{ "name": "/name" }]
            }]
        }"#,
    );

    // The struct requires "name" as a non-optional String. If the JSON
    // value at /name is absent, the reverse walk produces null, and serde
    // deserialization fails.
    let json_input = json!({});
    let result = suture.unstitch::<UnstitchSingle>(&json_input);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 29. Extra fields in JSON not in suture mapping (ignored)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct PartialStruct {
    model: String,
}

#[test]
fn unstitch_extra_json_fields_ignored() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "partial",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            }]
        }"#,
    );

    // The JSON has extra fields not mentioned in the suture mapping.
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
// 30. Unicode in JSON paths
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct UnicodeStruct {
    name: String,
}

#[test]
fn stitch_unicode_json_value() {
    // The JSON path keys are ASCII (validation requires it), but the
    // values themselves can contain Unicode.
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "unicode",
                "capture": "request",
                "sutures": [{ "name": "/name" }]
            }]
        }"#,
    );

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
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "unicode",
                "capture": "request",
                "sutures": [{ "name": "/name" }]
            }]
        }"#,
    );

    let json_input = json!({ "name": "\u{1f600} Hello \u{4e16}\u{754c}" });
    let result: UnicodeStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(
        result,
        UnicodeStruct {
            name: "\u{1f600} Hello \u{4e16}\u{754c}".into(),
        }
    );
}

// ---------------------------------------------------------------------------
// 31. Very deep JSON nesting (5+ levels)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct VeryDeepStruct {
    value: String,
}

#[test]
fn stitch_very_deep_json_nesting() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "deep",
                "capture": "request",
                "sutures": [{ "value": "/a/b/c/d/e/f/g" }]
            }]
        }"#,
    );

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
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "deep",
                "capture": "request",
                "sutures": [{ "value": "/a/b/c/d/e/f/g" }]
            }]
        }"#,
    );

    let json_input = json!({ "a": { "b": { "c": { "d": { "e": { "f": { "g": "deep!" } } } } } } });
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
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "deep",
                "capture": "request",
                "sutures": [{ "value": "/a/b/c/d/e/f/g" }]
            }]
        }"#,
    );

    let original = VeryDeepStruct {
        value: "round-trip".into(),
    };
    let stitched = suture.stitch(&original).unwrap();
    let recovered: VeryDeepStruct = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ===========================================================================
// ADDITIONAL COVERAGE tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Multiple suture objects with overlapping struct paths (merged trie)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MergedInner {
    x: i64,
    y: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MergedOuter {
    #[seam(to_struct)]
    point: MergedInner,
}

#[test]
fn stitch_merged_trie_from_multiple_suture_objects() {
    // Two separate suture objects, each mapping one sub-field of point.
    // The compiler should merge them into a single trie node for "point".
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "merged",
                "capture": "request",
                "sutures": [
                    { "point.x": "/x" },
                    { "point.y": "/y" }
                ]
            }]
        }"#,
    );

    let input = MergedOuter {
        point: MergedInner { x: 10, y: 20 },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "x": 10, "y": 20 }));
}

#[test]
fn unstitch_merged_trie_from_multiple_suture_objects() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "merged",
                "capture": "request",
                "sutures": [
                    { "point.x": "/x" },
                    { "point.y": "/y" }
                ]
            }]
        }"#,
    );

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FanOutNested {
    #[seam(to_struct)]
    config: FanOutInnerConfig,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FanOutInnerConfig {
    model: String,
}

#[test]
fn stitch_nested_object_with_fanout() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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
// Boolean and integer field types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct TypesStruct {
    flag: bool,
    count: i64,
    ratio: f64,
    label: String,
}

#[test]
fn stitch_various_types() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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
// Suture direction verification
// ---------------------------------------------------------------------------

#[test]
fn suture_is_request_direction() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "dir",
                "capture": "request",
                "sutures": [{ "x": "/x" }]
            }]
        }"#,
    );
    assert!(suture.is_request());
    assert!(!suture.is_response());
}

// ---------------------------------------------------------------------------
// Struct field not in suture mapping is NOT included in stitch output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct PartialMapping {
    included: String,
    excluded: String,
}

#[test]
fn stitch_partial_mapping_only_includes_mapped_fields() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "partial",
                "capture": "request",
                "sutures": [{ "included": "/included" }]
            }]
        }"#,
    );

    let input = PartialMapping {
        included: "yes".into(),
        excluded: "no".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "included": "yes" }));
    // The "excluded" field should NOT appear in the output.
    assert!(result.get("excluded").is_none());
}

// ---------------------------------------------------------------------------
// Deep struct with nested object syntax round-trip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtDeepLeaf {
    val: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtDeepMid {
    #[seam(to_struct)]
    leaf: RtDeepLeaf,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RtDeepRoot {
    #[seam(to_struct)]
    mid: RtDeepMid,
}

#[test]
fn roundtrip_deep_nested_object_syntax() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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
// Fan-out with nested JSON targets at different depths
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FanOutDepthStruct {
    name: String,
}

#[test]
fn stitch_fan_out_different_depths() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "fanout_depth",
                "capture": "request",
                "sutures": [{
                    "name": ["/name", "/meta/info/name", "/flat_name"]
                }]
            }]
        }"#,
    );

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct LeftRight {
    left: String,
    right: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SameJsonParent {
    #[seam(to_struct)]
    coords: LeftRight,
}

#[test]
fn stitch_nested_fields_same_json_parent() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "same_parent",
                "capture": "request",
                "sutures": [{
                    "coords.left": "/position/left",
                    "coords.right": "/position/right"
                }]
            }]
        }"#,
    );

    let input = SameJsonParent {
        coords: LeftRight {
            left: "L".into(),
            right: "R".into(),
        },
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "position": { "left": "L", "right": "R" } }));
}

#[test]
fn unstitch_nested_fields_same_json_parent() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "same_parent",
                "capture": "request",
                "sutures": [{
                    "coords.left": "/position/left",
                    "coords.right": "/position/right"
                }]
            }]
        }"#,
    );

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
// Renamed field round-trip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct RenamedRoundTrip {
    #[serde(rename = "model_type")]
    #[seam(rename = "model_type")]
    ty: String,
    value: i64,
}

#[test]
fn roundtrip_renamed_field() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "rename_rt",
                "capture": "request",
                "sutures": [{
                    "model_type": "/type",
                    "value": "/value"
                }]
            }]
        }"#,
    );

    let original = RenamedRoundTrip {
        ty: "chat".into(),
        value: 99,
    };
    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "type": "chat", "value": 99 }));
    let recovered: RenamedRoundTrip = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// Suture metadata accessors
// ---------------------------------------------------------------------------

#[test]
fn suture_name_accessor() {
    let suture = parse_first(
        r#"{
            "name": "my_manifest",
            "suture_sets": [{
                "name": "my_set",
                "capture": "request",
                "sutures": [{ "x": "/x" }]
            }]
        }"#,
    );
    assert_eq!(suture.name(), "my_set");
}

#[test]
fn suture_optional_metadata() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "meta",
                "id": "abc-123",
                "description": "A test suture",
                "version": "1.0.0",
                "capture": "request",
                "sutures": [{ "x": "/x" }]
            }]
        }"#,
    );
    assert_eq!(suture.id(), Some("abc-123"));
    assert_eq!(suture.description(), Some("A test suture"));
    assert_eq!(suture.version(), Some("1.0.0"));
}

// ---------------------------------------------------------------------------
// Stitch with Vec field (terminal, handled by serde)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct WithVec {
    tags: Vec<String>,
}

#[test]
fn stitch_vec_field() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "vec",
                "capture": "request",
                "sutures": [{ "tags": "/tags" }]
            }]
        }"#,
    );

    let input = WithVec {
        tags: vec!["a".into(), "b".into(), "c".into()],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "tags": ["a", "b", "c"] }));
}

#[test]
fn unstitch_vec_field() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "vec",
                "capture": "request",
                "sutures": [{ "tags": "/tags" }]
            }]
        }"#,
    );

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
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "vec",
                "capture": "request",
                "sutures": [{ "tags": "/tags" }]
            }]
        }"#,
    );

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ComplexInner {
    prompt: String,
    max_tokens: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ComplexOuter {
    model: String,
    #[seam(to_struct)]
    params: ComplexInner,
    stream: bool,
}

#[test]
fn roundtrip_complex_mixed() {
    let suture = parse_first(
        r#"{
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
        }"#,
    );

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
// Deeply nested struct (5 levels) round-trip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct D5 {
    val: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct D4 {
    #[seam(to_struct)]
    d5: D5,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct D3 {
    #[seam(to_struct)]
    d4: D4,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct D2 {
    #[seam(to_struct)]
    d3: D3,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct D1 {
    #[seam(to_struct)]
    d2: D2,
}

#[test]
fn roundtrip_five_level_nesting() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "deep5",
                "capture": "request",
                "sutures": [{ "d2.d3.d4.d5.val": "/a/b/c/d/e" }]
            }]
        }"#,
    );

    let original = D1 {
        d2: D2 {
            d3: D3 {
                d4: D4 {
                    d5: D5 {
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
    let recovered: D1 = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, recovered);
}

// ---------------------------------------------------------------------------
// Same struct field mapped to the same deep JSON path via object syntax vs
// dot-path syntax — should produce identical results
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SyntaxCompareInner {
    val: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SyntaxCompareOuter {
    #[seam(to_struct)]
    inner: SyntaxCompareInner,
}

#[test]
fn dot_path_and_object_syntax_produce_same_result() {
    let suture_dot = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "dot",
                "capture": "request",
                "sutures": [{ "inner.val": "/out/value" }]
            }]
        }"#,
    );

    let suture_obj = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "obj",
                "capture": "request",
                "sutures": [{
                    "inner": { "val": "/out/value" }
                }]
            }]
        }"#,
    );

    let input = SyntaxCompareOuter {
        inner: SyntaxCompareInner { val: "same".into() },
    };

    let result_dot = suture_dot.stitch(&input).unwrap();
    let result_obj = suture_obj.stitch(&input).unwrap();
    assert_eq!(result_dot, result_obj);
    assert_eq!(result_dot, json!({ "out": { "value": "same" } }));
}

// ---------------------------------------------------------------------------
// Null value round-trip through optional field
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NullRoundTrip {
    required: String,
    optional: Option<i64>,
}

#[test]
fn roundtrip_null_optional() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "null_rt",
                "capture": "request",
                "sutures": [{
                    "required": "/required",
                    "optional": "/optional"
                }]
            }]
        }"#,
    );

    // With None
    let original_none = NullRoundTrip {
        required: "hello".into(),
        optional: None,
    };
    let stitched = suture.stitch(&original_none).unwrap();
    assert_eq!(stitched, json!({ "required": "hello", "optional": null }));
    let recovered: NullRoundTrip = suture.unstitch(&stitched).unwrap();
    assert_eq!(original_none, recovered);

    // With Some
    let original_some = NullRoundTrip {
        required: "hello".into(),
        optional: Some(42),
    };
    let stitched = suture.stitch(&original_some).unwrap();
    assert_eq!(stitched, json!({ "required": "hello", "optional": 42 }));
    let recovered: NullRoundTrip = suture.unstitch(&stitched).unwrap();
    assert_eq!(original_some, recovered);
}

// ---------------------------------------------------------------------------
// Struct with many fields, only some mapped
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SparseStruct {
    a: String,
    b: String,
    c: String,
    d: String,
    e: String,
}

#[test]
fn stitch_sparse_mapping() {
    let suture = parse_first(
        r#"{
            "name": "test",
            "suture_sets": [{
                "name": "sparse",
                "capture": "request",
                "sutures": [{ "a": "/alpha", "c": "/gamma", "e": "/epsilon" }]
            }]
        }"#,
    );

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
    // b and d should not appear.
    assert!(result.get("b").is_none());
    assert!(result.get("d").is_none());
}
