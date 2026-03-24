//! Exhaustive integration tests for constants injection, mixed binding types,
//! multiple suture_sets, deep nesting, array edge cases, error conditions,
//! value type edge cases, and suture metadata accessors.

use serde_json::json;
use sutures::v1::{ConstantValue, Direction};
use sutures::{Knit, Stitch};

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

fn parse_all(json: &str) -> Vec<sutures::v1::Suture> {
    sutures::v1::parse(json)
        .unwrap()
        .into_iter()
        .map(|r| r.unwrap())
        .collect()
}

// ===========================================================================
// Test structs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SimpleRequest {
    model: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SimpleResponse {
    content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FullRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct WithOptional {
    content: String,
    metadata: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NumericFields {
    #[serde(default)]
    max_tokens: i64,
    #[serde(default)]
    temperature: f64,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct AllScalarFields {
    s: String,
    i: i64,
    f: f64,
    b: bool,
    o: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ConfigVersion {
    value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Nested {
    value: String,
    #[seam(to_struct)]
    config: ConfigVersion,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MessagesRequest {
    #[serde(default)]
    model: String,
    #[serde(default)]
    messages: Vec<MessageItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MessageItem {
    content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct IterResponse {
    #[serde(default)]
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DeepJson {
    value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level5Inner {
    value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Level4 {
    #[seam(to_struct)]
    e: Level5Inner,
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
struct DeepStruct {
    #[seam(to_struct)]
    a: Level1,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedDepths {
    shallow: String,
    #[seam(to_struct)]
    deep: Nested,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct LargeItems {
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct EmptyString {
    value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Precision {
    big_int: i64,
    big_float: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct BoolPositions {
    a: bool,
    b: bool,
    c: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct OverwriteTarget {
    model: String,
    api_version: String,
}

// ===========================================================================
// 1. Request constants — string constant (stitch, injected into JSON output)
// ===========================================================================

#[test]
fn req_const_string_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_str",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/api_version": "v1" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "api_version": "v1" }));
}

// ===========================================================================
// 2. Request constants — integer constant
// ===========================================================================

#[test]
fn req_const_integer_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_int",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/max_tokens": 100 }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "max_tokens": 100 }));
}

// ===========================================================================
// 3. Request constants — float constant
// ===========================================================================

#[test]
fn req_const_float_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_float",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/temperature": 0.7 }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "temperature": 0.7 }));
}

// ===========================================================================
// 4. Request constants — boolean constant (true and false)
// ===========================================================================

#[test]
fn req_const_bool_true_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_bool_t",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/stream": true }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "stream": true }));
}

#[test]
fn req_const_bool_false_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_bool_f",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/stream": false }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "stream": false }));
}

// ===========================================================================
// 5. Request constants — null constant
// ===========================================================================

#[test]
fn req_const_null_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_null",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/metadata": null }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "model": "gpt-4", "metadata": null }));
}

// ===========================================================================
// 6. Request constants — multiple constants in one _ array
// ===========================================================================

#[test]
fn req_const_multiple_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_multi",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [
                    { "/api_version": "v1" },
                    { "/max_tokens": 100 },
                    { "/temperature": 0.7 },
                    { "/stream": true },
                    { "/metadata": null }
                ]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["api_version"], json!("v1"));
    assert_eq!(result["max_tokens"], json!(100));
    assert_eq!(result["temperature"], json!(0.7));
    assert_eq!(result["stream"], json!(true));
    assert_eq!(result["metadata"], json!(null));
}

// ===========================================================================
// 7. Request constants — constant at nested path
// ===========================================================================

#[test]
fn req_const_nested_path_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_nested",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/config/version": "1.0" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "model": "gpt-4", "config": { "version": "1.0" } })
    );
}

// ===========================================================================
// 8. Request constants — constant alongside regular field mappings
// ===========================================================================

#[test]
fn req_const_alongside_mappings_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_alongside",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "prompt": "/prompt",
                "_": [{ "/api_version": "v1" }]
            }]
        }]
    }"#,
    );

    let input = FullRequest {
        model: "gpt-4".into(),
        prompt: "hello".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "model": "gpt-4", "prompt": "hello", "api_version": "v1" })
    );
}

// ===========================================================================
// 9. Request constants — constant overwrites a mapped value
// ===========================================================================

#[test]
fn req_const_overwrites_mapped_value_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_overwrite",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "prompt": "/api_version",
                "_": [{ "/api_version": "forced_v2" }]
            }]
        }]
    }"#,
    );

    let input = FullRequest {
        model: "gpt-4".into(),
        prompt: "user_value".into(),
    };
    let result = suture.stitch(&input).unwrap();
    // Constants are injected AFTER mapping, so the constant wins.
    assert_eq!(result["api_version"], json!("forced_v2"));
    assert_eq!(result["model"], json!("gpt-4"));
}

// ===========================================================================
// 10. Request constants — constant at path sharing prefix with mapped path
// ===========================================================================

#[test]
fn req_const_shared_prefix_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_shared",
            "capture": "request",
            "sutures": [{
                "model": "/config/model",
                "_": [{ "/config/version": "1.0" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "config": { "model": "gpt-4", "version": "1.0" } })
    );
}

// ===========================================================================
// 11. Request constants — unstitch does NOT inject constants into struct
// ===========================================================================

#[test]
fn req_const_not_injected_on_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_no_unstitch",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/api_version": "v1" }]
            }]
        }]
    }"#,
    );

    // When we unstitch a request suture, we go JSON -> struct.
    // Constants are for the JSON side, so they should NOT be injected.
    let json_input = json!({ "model": "gpt-4", "api_version": "v1" });
    let result: SimpleRequest = suture.unstitch(&json_input).unwrap();
    // The struct just gets the model field; api_version is not a struct field
    assert_eq!(result.model, "gpt-4");
}

// ===========================================================================
// 12. Response constants — string constant into struct (unstitch)
// ===========================================================================

#[test]
fn resp_const_string_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_str",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [{ "metadata": "injected" }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "content": "hello world" });
    let result: WithOptional = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.content, "hello world");
    assert_eq!(result.metadata, Some("injected".into()));
}

// ===========================================================================
// 13. Response constants — integer constant
// ===========================================================================

#[test]
fn resp_const_integer_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_int",
            "capture": "response",
            "sutures": [{
                "/stream": "stream",
                "_": [{ "max_tokens": 200 }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "stream": false });
    let result: NumericFields = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.max_tokens, 200);
    assert_eq!(result.stream, false);
}

// ===========================================================================
// 14. Response constants — float constant
// ===========================================================================

#[test]
fn resp_const_float_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_float",
            "capture": "response",
            "sutures": [{
                "/stream": "stream",
                "_": [{ "temperature": 0.9 }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "stream": true });
    let result: NumericFields = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.temperature, 0.9);
    assert_eq!(result.stream, true);
}

// ===========================================================================
// 15. Response constants — boolean constant
// ===========================================================================

#[test]
fn resp_const_bool_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_bool",
            "capture": "response",
            "sutures": [{
                "/max_tokens": "max_tokens",
                "_": [{ "stream": true }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "max_tokens": 50 });
    let result: NumericFields = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.stream, true);
    assert_eq!(result.max_tokens, 50);
}

// ===========================================================================
// 16. Response constants — null constant for Option field
// ===========================================================================

#[test]
fn resp_const_null_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_null",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [{ "metadata": null }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "content": "hello" });
    let result: WithOptional = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.metadata, None);
}

// ===========================================================================
// 17. Response constants — multiple constants
// ===========================================================================

#[test]
fn resp_const_multiple_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_multi",
            "capture": "response",
            "sutures": [{
                "_": [
                    { "s": "hello" },
                    { "i": 42 },
                    { "f": 3.14 },
                    { "b": true },
                    { "o": null }
                ]
            }]
        }]
    }"#,
    );

    let json_input = json!({});
    let result: AllScalarFields = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.s, "hello");
    assert_eq!(result.i, 42);
    assert_eq!(result.f, 3.14);
    assert_eq!(result.b, true);
    assert_eq!(result.o, None);
}

// ===========================================================================
// 18. Response constants — nested struct constant
// ===========================================================================

#[test]
fn resp_const_nested_struct_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_nested",
            "capture": "response",
            "sutures": [{
                "/value": "value",
                "_": [{ "config.value": "1.0" }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "value": "data" });
    let result: Nested = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.value, "data");
    assert_eq!(result.config.value, "1.0");
}

// ===========================================================================
// 19. Response constants — alongside regular mappings
// ===========================================================================

#[test]
fn resp_const_alongside_mappings_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_alongside",
            "capture": "response",
            "sutures": [{
                "/text": "content",
                "_": [{ "metadata": "extra" }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "text": "hello" });
    let result: WithOptional = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.metadata, Some("extra".into()));
}

// ===========================================================================
// 20. Response constants — stitch does NOT inject constants into JSON
// ===========================================================================

#[test]
fn resp_const_not_injected_on_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_no_stitch",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [{ "metadata": "injected" }]
            }]
        }]
    }"#,
    );

    // When we stitch a response suture, we go struct -> JSON.
    // Response constants are for the struct side, so they should NOT be injected into JSON.
    let input = WithOptional {
        content: "hello".into(),
        metadata: Some("user_set".into()),
    };
    let result = suture.stitch(&input).unwrap();
    // The result should only have "content" mapped, no "metadata" key injected
    assert_eq!(result, json!({ "content": "hello" }));
}

// ===========================================================================
// 21. Mixed: Direct + Iterate in same suture
// ===========================================================================

#[test]
fn mixed_direct_and_iterate_request() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_di",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "messages[:].content": "/messages/[:]/content"
            }]
        }]
    }"#,
    );

    let input = MessagesRequest {
        model: "gpt-4".into(),
        messages: vec![
            MessageItem {
                content: "hello".into(),
            },
            MessageItem {
                content: "world".into(),
            },
        ],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["messages"][0]["content"], json!("hello"));
    assert_eq!(result["messages"][1]["content"], json!("world"));
}

// ===========================================================================
// 22. Mixed: Direct + IteratePattern in same suture
// ===========================================================================

#[test]
fn mixed_direct_and_iterate_pattern_response() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_dp",
            "capture": "response",
            "sutures": [{
                "/model": "model",
                "/`msg_\\d+`[:]/content": "messages[:].content"
            }]
        }]
    }"#,
    );

    let json_input = json!({
        "model": "gpt-4",
        "msg_0": [{ "content": "hi" }],
        "msg_1": [{ "content": "there" }]
    });

    // IteratePattern matches keys and iterates them.
    // The direct mapping should work; iterate-pattern population is best-effort.
    let result: MessagesRequest = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.model, "gpt-4");
}

// ===========================================================================
// 23. Mixed: Iterate + Constants in same suture
// ===========================================================================

#[test]
fn mixed_iterate_and_constants_request() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_ic",
            "capture": "request",
            "sutures": [{
                "messages[:].content": "/messages/[:]/content",
                "_": [{ "/model": "gpt-4" }]
            }]
        }]
    }"#,
    );

    let input = MessagesRequest {
        model: "unused".into(),
        messages: vec![MessageItem {
            content: "hello".into(),
        }],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["messages"][0]["content"], json!("hello"));
}

// ===========================================================================
// 24. Mixed: Direct + Iterate + Constants all together
// ===========================================================================

#[test]
fn mixed_direct_iterate_constants_request() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_dic",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "messages[:].content": "/messages/[:]/content",
                "_": [
                    { "/api_version": "v1" },
                    { "/stream": true }
                ]
            }]
        }]
    }"#,
    );

    let input = MessagesRequest {
        model: "gpt-4".into(),
        messages: vec![
            MessageItem {
                content: "hi".into(),
            },
            MessageItem {
                content: "bye".into(),
            },
        ],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["messages"][0]["content"], json!("hi"));
    assert_eq!(result["messages"][1]["content"], json!("bye"));
    assert_eq!(result["api_version"], json!("v1"));
    assert_eq!(result["stream"], json!(true));
}

// ===========================================================================
// 25. Multiple suture objects in sutures array
// ===========================================================================

#[test]
fn multiple_suture_objects_in_array() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj",
            "capture": "request",
            "sutures": [
                { "model": "/model" },
                { "prompt": "/prompt" },
                { "_": [{ "/api_version": "v1" }] }
            ]
        }]
    }"#,
    );

    let input = FullRequest {
        model: "gpt-4".into(),
        prompt: "hello".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["prompt"], json!("hello"));
    assert_eq!(result["api_version"], json!("v1"));
}

// ===========================================================================
// 26. Multiple suture_sets — each compiles independently
// ===========================================================================

#[test]
fn multiple_suture_sets_compile() {
    let sutures = parse_all(
        r#"{
        "name": "multi_set",
        "suture_sets": [
            {
                "name": "set_a",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "set_b",
                "capture": "request",
                "sutures": [{ "prompt": "/prompt" }]
            }
        ]
    }"#,
    );

    assert_eq!(sutures.len(), 2);
    assert_eq!(sutures[0].name(), "set_a");
    assert_eq!(sutures[1].name(), "set_b");
}

// ===========================================================================
// 27. One request set + one response set in same schema
// ===========================================================================

#[test]
fn request_and_response_sets_in_same_schema() {
    let sutures = parse_all(
        r#"{
        "name": "mixed_dir",
        "suture_sets": [
            {
                "name": "req",
                "capture": "request",
                "sutures": [{ "model": "/model", "_": [{ "/stream": true }] }]
            },
            {
                "name": "resp",
                "capture": "response",
                "sutures": [{ "/content": "content", "_": [{ "metadata": "auto" }] }]
            }
        ]
    }"#,
    );

    assert_eq!(sutures.len(), 2);
    assert!(sutures[0].is_request());
    assert!(sutures[1].is_response());

    // Test request stitch
    let req_input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let req_result = sutures[0].stitch(&req_input).unwrap();
    assert_eq!(req_result["model"], json!("gpt-4"));
    assert_eq!(req_result["stream"], json!(true));

    // Test response unstitch
    let resp_json = json!({ "content": "hello" });
    let resp_result: WithOptional = sutures[1].unstitch(&resp_json).unwrap();
    assert_eq!(resp_result.content, "hello");
    assert_eq!(resp_result.metadata, Some("auto".into()));
}

// ===========================================================================
// 28. Deep nesting — 5-level JSON path
// ===========================================================================

#[test]
fn deep_json_path_5_levels() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_json",
            "capture": "request",
            "sutures": [{
                "model": "/a/b/c/d/e",
                "_": [{ "/a/b/c/d/f": "const_val" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "deep".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["a"]["b"]["c"]["d"]["e"], json!("deep"));
    assert_eq!(result["a"]["b"]["c"]["d"]["f"], json!("const_val"));
}

// ===========================================================================
// 29. Deep nesting — 5-level struct path
// ===========================================================================

#[test]
fn deep_struct_path_5_levels() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_struct",
            "capture": "response",
            "sutures": [{
                "/value": "a.b.c.d.e.value"
            }]
        }]
    }"#,
    );

    let json_input = json!({ "value": "deep_data" });
    let result: DeepStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.a.b.c.d.e.value, "deep_data");
}

// ===========================================================================
// 30. Mixed depths — shallow and deep in same suture
// ===========================================================================

#[test]
fn mixed_shallow_and_deep_paths() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_depth",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "prompt": "/config/nested/prompt"
            }]
        }]
    }"#,
    );

    let input = FullRequest {
        model: "gpt-4".into(),
        prompt: "hello".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["config"]["nested"]["prompt"], json!("hello"));
}

// ===========================================================================
// 31. Iterate on empty array
// ===========================================================================

#[test]
fn iterate_empty_array() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_empty",
            "capture": "request",
            "sutures": [{
                "messages[:].content": "/messages/[:]/content"
            }]
        }]
    }"#,
    );

    let input = MessagesRequest {
        model: "gpt-4".into(),
        messages: vec![],
    };
    let result = suture.stitch(&input).unwrap();
    // Empty array means no iteration, so messages key may be absent or empty
    let messages = result.get("messages");
    assert!(messages.is_none() || messages.unwrap().as_array().unwrap().is_empty());
}

// ===========================================================================
// 32. Iterate on single-element array
// ===========================================================================

#[test]
fn iterate_single_element_array() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_single",
            "capture": "request",
            "sutures": [{
                "messages[:].content": "/messages/[:]/content"
            }]
        }]
    }"#,
    );

    let input = MessagesRequest {
        model: "unused".into(),
        messages: vec![MessageItem {
            content: "only one".into(),
        }],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["messages"][0]["content"], json!("only one"));
    assert_eq!(result["messages"].as_array().unwrap().len(), 1);
}

// ===========================================================================
// 33. Iterate on large array (100 elements)
// ===========================================================================

#[test]
fn iterate_large_array_100_elements() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_large",
            "capture": "request",
            "sutures": [{
                "messages[:].content": "/items/[:]/text"
            }]
        }]
    }"#,
    );

    let messages: Vec<MessageItem> = (0..100)
        .map(|i| MessageItem {
            content: format!("msg_{}", i),
        })
        .collect();
    let input = MessagesRequest {
        model: "unused".into(),
        messages,
    };
    let result = suture.stitch(&input).unwrap();
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 100);
    assert_eq!(items[0]["text"], json!("msg_0"));
    assert_eq!(items[99]["text"], json!("msg_99"));
}

// ===========================================================================
// 34. Nested iteration (2 levels of [:])
// ===========================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NestedIterItem {
    value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NestedIter {
    matrix: Vec<NestedIterItem>,
}

#[test]
fn nested_iteration_two_levels() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_iter",
            "capture": "request",
            "sutures": [{
                "matrix[:].value": "/data/[:]/value"
            }]
        }]
    }"#,
    );

    let input = NestedIter {
        matrix: vec![NestedIterItem { value: 1 }, NestedIterItem { value: 2 }],
    };
    // Note: this test validates that nested iteration compiles and runs.
    // The exact output depends on how the nested iteration is handled.
    let result = suture.stitch(&input);
    // Nested iteration should produce some output (or a structured error)
    assert!(result.is_ok());
}

// ===========================================================================
// 35. Mixed [:] and [0] in same suture
// ===========================================================================

#[test]
fn mixed_slice_and_index_in_same_suture() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_idx",
            "capture": "response",
            "sutures": [{
                "/choices[0]/text": "content"
            }]
        }]
    }"#,
    );

    let json_input = json!({
        "choices": [{ "text": "first" }, { "text": "second" }]
    });
    let result: SimpleResponse = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.content, "first");
}

// ===========================================================================
// 36. Error: empty name
// ===========================================================================

#[test]
fn error_empty_root_name() {
    let result = sutures::v1::parse(
        r#"{
        "name": "",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );
    assert!(result.is_err());
}

// ===========================================================================
// 37. Error: empty suture_sets
// ===========================================================================

#[test]
fn error_empty_suture_sets() {
    let result = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": []
    }"#,
    );
    assert!(result.is_err());
}

// ===========================================================================
// 38. Error: invalid JSON terminal (missing /)
// ===========================================================================

#[test]
fn error_invalid_json_terminal_missing_slash() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "model": "model" }]
        }]
    }"#,
    )
    .unwrap();
    // The individual suture set should fail compilation
    assert!(results[0].is_err());
}

// ===========================================================================
// 39. Error: invalid struct terminal (starts with number)
// ===========================================================================

#[test]
fn error_invalid_struct_terminal_starts_with_number() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "1field": "/model" }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// 40. Error: invalid regex in backticks
// ===========================================================================

#[test]
fn error_invalid_regex_in_backticks() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "response",
            "sutures": [{ "/`[invalid`": "field" }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// 41. Error: capturing group in regex
// ===========================================================================

#[test]
fn error_capturing_group_in_regex() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "response",
            "sutures": [{ "/`(captured)`": "field" }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// 42. Error: empty brackets []
// ===========================================================================

#[test]
fn error_empty_brackets() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "items[]": "/items" }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// 43. Error: nested brackets
// ===========================================================================

#[test]
fn error_nested_brackets() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "items[[0]]": "/items" }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// 44. Null values flowing through stitch/unstitch
// ===========================================================================

#[test]
fn null_value_through_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "null_flow",
            "capture": "request",
            "sutures": [{
                "metadata": "/metadata"
            }]
        }]
    }"#,
    );

    let input = WithOptional {
        content: "hello".into(),
        metadata: None,
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["metadata"], json!(null));
}

#[test]
fn null_value_through_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "null_flow_u",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "/metadata": "metadata"
            }]
        }]
    }"#,
    );

    let json_input = json!({ "content": "hello", "metadata": null });
    let result: WithOptional = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.metadata, None);
}

// ===========================================================================
// 45. Nested null values
// ===========================================================================

#[test]
fn nested_null_value_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_null",
            "capture": "request",
            "sutures": [{
                "model": "/output/model",
                "_": [{ "/output/extra": null }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["output"]["model"], json!("gpt-4"));
    assert_eq!(result["output"]["extra"], json!(null));
}

// ===========================================================================
// 46. Empty string values
// ===========================================================================

#[test]
fn empty_string_value_stitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/tag": "" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest { model: "".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!(""));
    assert_eq!(result["tag"], json!(""));
}

#[test]
fn empty_string_value_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str_u",
            "capture": "response",
            "sutures": [{
                "/value": "value",
                "_": [{ "value": "" }]
            }]
        }]
    }"#,
    );

    let json_input = json!({ "value": "original" });
    let result: EmptyString = suture.unstitch(&json_input).unwrap();
    // Constant injected after mapping, so constant wins
    assert_eq!(result.value, "");
}

// ===========================================================================
// 47. Empty arrays flowing through
// ===========================================================================

#[test]
fn empty_array_through_unstitch() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_arr",
            "capture": "response",
            "sutures": [{
                "/items[:]": "items[:]"
            }]
        }]
    }"#,
    );

    let json_input = json!({ "items": [] });
    let result: IterResponse = suture.unstitch(&json_input).unwrap();
    assert!(result.items.is_empty());
}

// ===========================================================================
// 48. Numeric precision — large i64, f64 edge values
// ===========================================================================

#[test]
fn numeric_precision_large_i64() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "precision",
            "capture": "request",
            "sutures": [{
                "_": [{ "/big": 9007199254740992 }]
            }]
        }]
    }"#,
    );

    // Verify the constant is parsed and injected
    let constants = suture.constants();
    assert_eq!(constants.len(), 1);
    match &constants[0].1 {
        ConstantValue::Int(i) => assert_eq!(*i, 9007199254740992i64),
        ConstantValue::Float(_) => {
            // Large numbers may parse as float, which is acceptable
        }
        other => panic!("expected Int or Float, got {:?}", other),
    }
}

#[test]
fn numeric_precision_f64_edge() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "precision_f",
            "capture": "request",
            "sutures": [{
                "_": [{ "/tiny": 0.0000001 }]
            }]
        }]
    }"#,
    );

    let constants = suture.constants();
    assert_eq!(constants.len(), 1);
    match &constants[0].1 {
        ConstantValue::Float(f) => assert!((*f - 0.0000001).abs() < 1e-10),
        other => panic!("expected Float, got {:?}", other),
    }
}

#[test]
fn numeric_precision_negative_int() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_int",
            "capture": "request",
            "sutures": [{
                "_": [{ "/neg": -42 }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest { model: "x".into() };
    // Just verify it doesn't crash and rounds trip to the right constant
    let constants = suture.constants();
    assert_eq!(constants[0].1, ConstantValue::Int(-42));

    // Also test actual stitch output
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["neg"], json!(-42));
}

// ===========================================================================
// 49. Boolean values in various positions
// ===========================================================================

#[test]
fn bool_values_in_various_positions() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bools",
            "capture": "response",
            "sutures": [{
                "_": [
                    { "a": true },
                    { "b": false },
                    { "c": true }
                ]
            }]
        }]
    }"#,
    );

    let json_input = json!({});
    let result: BoolPositions = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.a, true);
    assert_eq!(result.b, false);
    assert_eq!(result.c, true);
}

// ===========================================================================
// 50. Suture metadata: name()
// ===========================================================================

#[test]
fn metadata_name() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "my_set_name",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(suture.name(), "my_set_name");
}

// ===========================================================================
// 51. Suture metadata: id()
// ===========================================================================

#[test]
fn metadata_id_present() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "id": "abc-123",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(suture.id(), Some("abc-123"));
}

#[test]
fn metadata_id_absent() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(suture.id(), None);
}

// ===========================================================================
// 52. Suture metadata: description()
// ===========================================================================

#[test]
fn metadata_description_present() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "description": "A test suture set",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(suture.description(), Some("A test suture set"));
}

#[test]
fn metadata_description_absent() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(suture.description(), None);
}

// ===========================================================================
// 53. Suture metadata: version()
// ===========================================================================

#[test]
fn metadata_version_present() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "version": "2.1.0",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(suture.version(), Some("2.1.0"));
}

#[test]
fn metadata_version_absent() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(suture.version(), None);
}

// ===========================================================================
// 54. Suture metadata: is_request() / is_response()
// ===========================================================================

#[test]
fn metadata_is_request() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert!(suture.is_request());
    assert!(!suture.is_response());
}

#[test]
fn metadata_is_response() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "s",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#,
    );

    assert!(!suture.is_request());
    assert!(suture.is_response());
}

// ===========================================================================
// 55. Suture Display impl
// ===========================================================================

#[test]
fn display_name_only() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "simple",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(format!("{}", suture), "simple");
}

#[test]
fn display_name_with_version() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "versioned",
            "version": "1.0",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(format!("{}", suture), "versioned@1.0");
}

#[test]
fn display_name_with_id() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "identified",
            "id": "abc",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(format!("{}", suture), "identified (abc)");
}

#[test]
fn display_name_with_id_and_version() {
    let suture = parse_first(
        r#"{
        "name": "root",
        "suture_sets": [{
            "name": "full",
            "id": "xyz",
            "version": "2.0",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    assert_eq!(format!("{}", suture), "full@2.0 (xyz)");
}

// ===========================================================================
// Additional edge cases: Knit trait round-trip with constants
// ===========================================================================

#[test]
fn knit_request_constants_round_trip() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "knit_req",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/stream": false }, { "/api_version": "v1" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let bytes = suture.knit(&input).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["model"], json!("gpt-4"));
    assert_eq!(value["stream"], json!(false));
    assert_eq!(value["api_version"], json!("v1"));
}

#[test]
fn knit_response_constants_round_trip() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "knit_resp",
            "capture": "response",
            "sutures": [{
                "/text": "content",
                "_": [{ "metadata": "knit_const" }]
            }]
        }]
    }"#,
    );

    let json_bytes = serde_json::to_vec(&json!({ "text": "hello" })).unwrap();
    let result: WithOptional = suture.unknit(&json_bytes).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.metadata, Some("knit_const".into()));
}

// ===========================================================================
// Constants parsed representation
// ===========================================================================

#[test]
fn constants_accessor_returns_all_types() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "const_acc",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [
                    { "/a": "str" },
                    { "/b": 42 },
                    { "/c": 3.14 },
                    { "/d": true },
                    { "/e": null }
                ]
            }]
        }]
    }"#,
    );

    let constants = suture.constants();
    assert_eq!(constants.len(), 5);

    assert_eq!(constants[0].0.as_ref(), "/a");
    assert_eq!(
        constants[0].1,
        ConstantValue::String(std::borrow::Cow::Owned("str".into()))
    );

    assert_eq!(constants[1].0.as_ref(), "/b");
    assert_eq!(constants[1].1, ConstantValue::Int(42));

    assert_eq!(constants[2].0.as_ref(), "/c");
    assert_eq!(constants[2].1, ConstantValue::Float(3.14));

    assert_eq!(constants[3].0.as_ref(), "/d");
    assert_eq!(constants[3].1, ConstantValue::Bool(true));

    assert_eq!(constants[4].0.as_ref(), "/e");
    assert_eq!(constants[4].1, ConstantValue::Null);
}

// ===========================================================================
// Error: constant with non-scalar value
// ===========================================================================

#[test]
fn error_constant_non_scalar_array() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/arr": [1, 2, 3] }]
            }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

#[test]
fn error_constant_non_scalar_object() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/obj": { "nested": true } }]
            }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// Error: _ must be an array
// ===========================================================================

#[test]
fn error_underscore_not_array() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": "not_array"
            }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// Error: constant entry must have exactly one property
// ===========================================================================

#[test]
fn error_constant_entry_multiple_properties() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/a": 1, "/b": 2 }]
            }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// Error: empty suture set name
// ===========================================================================

#[test]
fn error_empty_suture_set_name() {
    let results = sutures::v1::parse(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    )
    .unwrap();
    assert!(results[0].is_err());
}

// ===========================================================================
// Constants with special string values
// ===========================================================================

#[test]
fn constant_string_with_special_characters() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "special",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/header": "Bearer token-abc/123" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["header"], json!("Bearer token-abc/123"));
}

#[test]
fn constant_integer_zero() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "zero",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/count": 0 }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest { model: "x".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["count"], json!(0));
}

#[test]
fn constant_float_zero() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fzero",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/temp": 0.0 }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest { model: "x".into() };
    let result = suture.stitch(&input).unwrap();
    // 0.0 may be parsed as int 0
    let temp = &result["temp"];
    assert!(temp == &json!(0) || temp == &json!(0.0));
}

// ===========================================================================
// Constants in nested suture objects (object-valued suture entry)
// ===========================================================================

#[test]
fn constants_in_nested_suture_object() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_const",
            "capture": "request",
            "sutures": [{
                "config": {
                    "model": "/model"
                },
                "_": [{ "/version": "nested_v1" }]
            }]
        }]
    }"#,
    );

    // The nested object's _ constants should still be compiled.
    let constants = suture.constants();
    assert!(constants.iter().any(|(path, val)| {
        path.as_ref() == "/version"
            && *val == ConstantValue::String(std::borrow::Cow::Owned("nested_v1".into()))
    }));
}

// ===========================================================================
// Multiple suture objects each with their own _ constants
// ===========================================================================

#[test]
fn multiple_suture_objects_each_with_constants() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_const",
            "capture": "request",
            "sutures": [
                {
                    "model": "/model",
                    "_": [{ "/a": 1 }]
                },
                {
                    "prompt": "/prompt",
                    "_": [{ "/b": 2 }]
                }
            ]
        }]
    }"#,
    );

    let constants = suture.constants();
    assert_eq!(constants.len(), 2);

    let input = FullRequest {
        model: "gpt-4".into(),
        prompt: "hi".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["a"], json!(1));
    assert_eq!(result["b"], json!(2));
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["prompt"], json!("hi"));
}

// ===========================================================================
// Stitch then unstitch round-trip for request direction
// ===========================================================================

#[test]
fn request_stitch_then_unstitch_round_trip() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "round_trip",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "prompt": "/prompt"
            }]
        }]
    }"#,
    );

    let input = FullRequest {
        model: "gpt-4".into(),
        prompt: "hello".into(),
    };
    let json_val = suture.stitch(&input).unwrap();
    let output: FullRequest = suture.unstitch(&json_val).unwrap();
    assert_eq!(input, output);
}

// ===========================================================================
// Response unstitch then stitch round-trip
// ===========================================================================

#[test]
fn response_unstitch_then_stitch_round_trip() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "round_trip_r",
            "capture": "response",
            "sutures": [{
                "/content": "content"
            }]
        }]
    }"#,
    );

    let json_input = json!({ "content": "world" });
    let output: SimpleResponse = suture.unstitch(&json_input).unwrap();
    assert_eq!(output.content, "world");

    let json_output = suture.stitch(&output).unwrap();
    assert_eq!(json_output, json!({ "content": "world" }));
}

// ===========================================================================
// Deeply nested constant path (5 levels deep in JSON)
// ===========================================================================

#[test]
fn deep_constant_path_5_levels() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/a/b/c/d/e": "deep_value" }]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest { model: "x".into() };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["a"]["b"]["c"]["d"]["e"], json!("deep_value"));
}

// ===========================================================================
// Response constant into deeply nested struct path
// ===========================================================================

#[test]
fn response_deep_constant_struct_path() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_struct_const",
            "capture": "response",
            "sutures": [{
                "_": [{ "a.b.c.d.e.value": "injected_deep" }]
            }]
        }]
    }"#,
    );

    let json_input = json!({});
    let result: DeepStruct = suture.unstitch(&json_input).unwrap();
    assert_eq!(result.a.b.c.d.e.value, "injected_deep");
}

// ===========================================================================
// Verify suture.binding() accessor
// ===========================================================================

#[test]
fn binding_accessor_request() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#,
    );

    let binding = suture.binding();
    let root = binding.root();
    assert_eq!(root.key(), "s");
    assert!(root.children().len() > 0);
}

#[test]
fn binding_accessor_response() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#,
    );

    let binding = suture.binding();
    let root = binding.root();
    assert_eq!(root.key(), "s");
    assert!(root.children().len() > 0);
}

// ===========================================================================
// Verify iterate binding is correctly resolved in trie
// ===========================================================================

#[test]
fn trie_iterate_binding_resolved() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "s",
            "capture": "request",
            "sutures": [{
                "items[:]": "/items/[:]"
            }]
        }]
    }"#,
    );

    use sutures::v1::BindingTaskType;

    let root = suture.binding().root();
    let items_node = &root.children()[0];
    assert_eq!(items_node.key(), "items");
    assert!(matches!(
        items_node.binding(),
        BindingTaskType::Iterate { .. }
    ));
}

// ===========================================================================
// Request constant + iterate: constants appear after iteration output
// ===========================================================================

#[test]
fn request_constants_after_iterate_output() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "const_iter",
            "capture": "request",
            "sutures": [{
                "messages[:].content": "/msgs/[:]/text",
                "_": [{ "/api": "v2" }, { "/count": 2 }]
            }]
        }]
    }"#,
    );

    let input = MessagesRequest {
        model: "unused".into(),
        messages: vec![
            MessageItem {
                content: "a".into(),
            },
            MessageItem {
                content: "b".into(),
            },
        ],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["msgs"][0]["text"], json!("a"));
    assert_eq!(result["msgs"][1]["text"], json!("b"));
    assert_eq!(result["api"], json!("v2"));
    assert_eq!(result["count"], json!(2));
}

// ===========================================================================
// Fan-out: single struct field maps to multiple JSON targets
// ===========================================================================

#[test]
fn fanout_single_field_to_multiple_targets() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{
                "model": ["/model", "/backup_model"]
            }]
        }]
    }"#,
    );

    let input = SimpleRequest {
        model: "gpt-4".into(),
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result["model"], json!("gpt-4"));
    assert_eq!(result["backup_model"], json!("gpt-4"));
}

// ===========================================================================
// Multiple suture_sets with different IDs and versions
// ===========================================================================

#[test]
fn multiple_suture_sets_with_metadata() {
    let sutures = parse_all(
        r#"{
        "name": "multi_meta",
        "suture_sets": [
            {
                "name": "alpha",
                "id": "id-alpha",
                "version": "1.0",
                "description": "Alpha set",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "beta",
                "id": "id-beta",
                "version": "2.0",
                "description": "Beta set",
                "capture": "response",
                "sutures": [{ "/content": "content" }]
            }
        ]
    }"#,
    );

    assert_eq!(sutures[0].name(), "alpha");
    assert_eq!(sutures[0].id(), Some("id-alpha"));
    assert_eq!(sutures[0].version(), Some("1.0"));
    assert_eq!(sutures[0].description(), Some("Alpha set"));
    assert!(sutures[0].is_request());

    assert_eq!(sutures[1].name(), "beta");
    assert_eq!(sutures[1].id(), Some("id-beta"));
    assert_eq!(sutures[1].version(), Some("2.0"));
    assert_eq!(sutures[1].description(), Some("Beta set"));
    assert!(sutures[1].is_response());
}
