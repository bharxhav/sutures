//! Exhaustive integration tests verifying that compile-time sutures
//! (`sutures_comptime::parse!()`) produce **identical** results to runtime
//! sutures (`sutures::v1::parse()`), plus constants and edge cases.
//!
//! Every test is self-contained: it defines its own structs, builds a
//! `.sutures.json` string, parses it both ways, and compares results.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Seam, Stitch, Knit};

// ===========================================================================
// Helpers
// ===========================================================================

fn runtime_first(json: &str) -> sutures::v1::Suture {
    sutures::v1::parse(json)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .unwrap()
}

fn runtime_all(json: &str) -> Vec<sutures::v1::Suture> {
    sutures::v1::parse(json)
        .unwrap()
        .into_iter()
        .map(|r| r.unwrap())
        .collect()
}

// ===========================================================================
// Test structs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SimpleReq {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SimpleResp {
    #[serde(default)]
    content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MultiField {
    model: String,
    temperature: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct IterContainer {
    #[serde(default)]
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct IterItem {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NamedIterContainer {
    items: Vec<IterItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutReq {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedInner {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedOuter {
    #[seam(to_struct)]
    config: NestedInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedComplex {
    model: String,
    prompt: String,
    temperature: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ConstantRecipient {
    #[serde(default)]
    model: String,
    #[serde(default)]
    api_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct IntConstRecipient {
    #[serde(default)]
    max_tokens: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FloatConstRecipient {
    #[serde(default)]
    temperature: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct BoolConstRecipient {
    #[serde(default)]
    stream: bool,
    #[serde(default)]
    debug: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NullConstRecipient {
    #[serde(default)]
    extra: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MultiConstRecipient {
    #[serde(default)]
    model: String,
    #[serde(default)]
    api_version: String,
    #[serde(default)]
    max_tokens: i64,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Seam)]
struct NestedConstInner {
    #[serde(default)]
    version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedConstOuter {
    #[serde(default)]
    #[seam(to_struct)]
    config: NestedConstInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedBindReq {
    model: String,
    messages: Vec<MsgItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MsgItem {
    content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct AllScalars {
    s: String,
    i: i64,
    f: f64,
    b: bool,
    o: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct EmptyStr {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct LargeNums {
    big_int: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct BoolVals {
    a: bool,
    b: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DefaultVecStruct {
    #[serde(default)]
    items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RespConstRecipient {
    #[serde(default)]
    content: String,
    #[serde(default)]
    source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RespIntConst {
    #[serde(default)]
    content: String,
    #[serde(default)]
    count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RespMultiConst {
    #[serde(default)]
    content: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    count: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Seam)]
struct RespNestedInner {
    #[serde(default)]
    version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct RespNestedConst {
    #[serde(default)]
    content: String,
    #[serde(default)]
    #[seam(to_struct)]
    meta: RespNestedInner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ConstAlongsideReq {
    model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DirectIterCombo {
    model: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TripleCombo {
    model: String,
    #[serde(default)]
    items: Vec<String>,
}

// ===========================================================================
// EQUIVALENCE: comptime == runtime
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. Simple request direct: parse! and v1::parse produce same stitch output
// ---------------------------------------------------------------------------

#[test]
fn equiv_simple_request_direct() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "simple_req",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "simple_req",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    assert_eq!(comptime_sutures.len(), runtime_sutures.len());

    let input = SimpleReq { model: "gpt-4".into() };
    let comptime_result = comptime_sutures[0].stitch(&input).unwrap();
    let runtime_result = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);
    assert_eq!(comptime_result, json!({ "model": "gpt-4" }));
}

// ---------------------------------------------------------------------------
// 2. Simple response direct: same unstitch output
// ---------------------------------------------------------------------------

#[test]
fn equiv_simple_response_direct() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "simple_resp",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "simple_resp",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    assert_eq!(comptime_sutures.len(), runtime_sutures.len());

    let input = json!({ "content": "hello world" });
    let comptime_result: SimpleResp = comptime_sutures[0].unstitch(&input).unwrap();
    let runtime_result: SimpleResp = runtime_sutures[0].unstitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);
    assert_eq!(comptime_result.content, "hello world");
}

// ---------------------------------------------------------------------------
// 3. Request with iteration: same stitch output
// ---------------------------------------------------------------------------

#[test]
fn equiv_request_iterate() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_req",
            "capture": "request",
            "sutures": [{ "items[:]": "/items/[:]" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_req",
            "capture": "request",
            "sutures": [{ "items[:]": "/items/[:]" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = IterContainer {
        items: vec!["a".into(), "b".into(), "c".into()],
    };
    let comptime_result = comptime_sutures[0].stitch(&input).unwrap();
    let runtime_result = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);
    assert_eq!(comptime_result, json!({ "items": ["a", "b", "c"] }));
}

// ---------------------------------------------------------------------------
// 4. Response with iteration: same unstitch output
// ---------------------------------------------------------------------------

#[test]
fn equiv_response_iterate() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_resp",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_resp",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = json!({ "items": ["x", "y", "z"] });
    let comptime_result: IterContainer = comptime_sutures[0].unstitch(&input).unwrap();
    let runtime_result: IterContainer = runtime_sutures[0].unstitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);
    assert_eq!(comptime_result.items, vec!["x", "y", "z"]);
}

// ---------------------------------------------------------------------------
// 5. Request with fan-out: same output
// ---------------------------------------------------------------------------

#[test]
fn equiv_request_fanout() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{ "model": ["/model", "/config/model_name"] }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{ "model": ["/model", "/config/model_name"] }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = FanOutReq { model: "claude-3".into() };
    let comptime_result = comptime_sutures[0].stitch(&input).unwrap();
    let runtime_result = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);
    assert_eq!(
        comptime_result,
        json!({ "model": "claude-3", "config": { "model_name": "claude-3" } })
    );
}

// ---------------------------------------------------------------------------
// 6. Nested object syntax: same output
// ---------------------------------------------------------------------------

#[test]
fn equiv_nested_object_syntax() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested",
            "capture": "request",
            "sutures": [{
                "config": {
                    "value": "/settings/value"
                }
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested",
            "capture": "request",
            "sutures": [{
                "config": {
                    "value": "/settings/value"
                }
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = NestedOuter {
        config: NestedInner { value: "hello".into() },
    };
    let comptime_result = comptime_sutures[0].stitch(&input).unwrap();
    let runtime_result = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);
    assert_eq!(comptime_result, json!({ "settings": { "value": "hello" } }));
}

// ---------------------------------------------------------------------------
// 7. Complex mixed: same output for multi-field sutures
// ---------------------------------------------------------------------------

#[test]
fn equiv_complex_mixed() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "complex",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "prompt": "/messages/[0]/content",
                "temperature": "/temperature"
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "complex",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "prompt": "/messages/[0]/content",
                "temperature": "/temperature"
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = MixedComplex {
        model: "gpt-4".into(),
        prompt: "Hello".into(),
        temperature: 0.5,
    };
    let comptime_result = comptime_sutures[0].stitch(&input).unwrap();
    let runtime_result = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(comptime_result, runtime_result);

    // Verify structure
    assert_eq!(comptime_result["model"], "gpt-4");
    assert_eq!(comptime_result["messages"][0]["content"], "Hello");
    assert_eq!(comptime_result["temperature"], 0.5);
}

// ---------------------------------------------------------------------------
// 8. Multiple suture_sets: same number of sutures, same results
// ---------------------------------------------------------------------------

#[test]
fn equiv_multiple_suture_sets() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "req_set",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "resp_set",
                "capture": "response",
                "sutures": [{ "/content": "content" }]
            }
        ]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "req_set",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "resp_set",
                "capture": "response",
                "sutures": [{ "/content": "content" }]
            }
        ]
    }"#);
    let runtime_sutures = runtime_all(schema);

    assert_eq!(comptime_sutures.len(), 2);
    assert_eq!(comptime_sutures.len(), runtime_sutures.len());

    // Test request set
    let req_input = SimpleReq { model: "gpt-4".into() };
    let ct_req = comptime_sutures[0].stitch(&req_input).unwrap();
    let rt_req = runtime_sutures[0].stitch(&req_input).unwrap();
    assert_eq!(ct_req, rt_req);

    // Test response set
    let resp_input = json!({ "content": "done" });
    let ct_resp: SimpleResp = comptime_sutures[1].unstitch(&resp_input).unwrap();
    let rt_resp: SimpleResp = runtime_sutures[1].unstitch(&resp_input).unwrap();
    assert_eq!(ct_resp, rt_resp);
}

// ===========================================================================
// REQUEST CONSTANTS (stitch -- injected into JSON output)
// ===========================================================================

// ---------------------------------------------------------------------------
// 9. String constant
// ---------------------------------------------------------------------------

#[test]
fn req_const_string() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "str_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/api_version": "v1" }]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["model"], "gpt-4");
    assert_eq!(result["api_version"], "v1");
}

// ---------------------------------------------------------------------------
// 10. Integer constant
// ---------------------------------------------------------------------------

#[test]
fn req_const_integer() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "int_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/max_tokens": 100 }]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["max_tokens"], 100);
}

// ---------------------------------------------------------------------------
// 11. Float constant
// ---------------------------------------------------------------------------

#[test]
fn req_const_float() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "float_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/temperature": 0.7 }]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["temperature"], 0.7);
}

// ---------------------------------------------------------------------------
// 12. Boolean constants true/false
// ---------------------------------------------------------------------------

#[test]
fn req_const_bool_true() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bool_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/stream": true }]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["stream"], true);
}

#[test]
fn req_const_bool_false() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bool_const_f",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/stream": false }]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["stream"], false);
}

// ---------------------------------------------------------------------------
// 13. Null constant
// ---------------------------------------------------------------------------

#[test]
fn req_const_null() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "null_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/extra": null }]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert!(result["extra"].is_null());
}

// ---------------------------------------------------------------------------
// 14. Multiple constants
// ---------------------------------------------------------------------------

#[test]
fn req_const_multiple() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [
                    { "/api_version": "v1" },
                    { "/max_tokens": 100 },
                    { "/stream": true }
                ]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["model"], "gpt-4");
    assert_eq!(result["api_version"], "v1");
    assert_eq!(result["max_tokens"], 100);
    assert_eq!(result["stream"], true);
}

// ---------------------------------------------------------------------------
// 15. Constant at nested path
// ---------------------------------------------------------------------------

#[test]
fn req_const_nested_path() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_const",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/config/version": "2.0" }]
            }]
        }]
    }"#);

    let input = SimpleReq { model: "gpt-4".into() };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["model"], "gpt-4");
    assert_eq!(result["config"]["version"], "2.0");
}

// ---------------------------------------------------------------------------
// 16. Constant alongside regular mappings
// ---------------------------------------------------------------------------

#[test]
fn req_const_alongside_mappings() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/api_version": "v1" }]
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/api_version": "v1" }]
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = ConstAlongsideReq { model: "gpt-4".into() };
    let ct_result = comptime_sutures[0].stitch(&input).unwrap();
    let rt_result = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct_result, rt_result);
    assert_eq!(ct_result["model"], "gpt-4");
    assert_eq!(ct_result["api_version"], "v1");
}

// ===========================================================================
// RESPONSE CONSTANTS (unstitch -- injected into struct)
// ===========================================================================

// ---------------------------------------------------------------------------
// 17. String constant into struct field
// ---------------------------------------------------------------------------

#[test]
fn resp_const_string_unstitch() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_str",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [{ "source": "api" }]
            }]
        }]
    }"#);

    let input = json!({ "content": "hello" });
    let result: RespConstRecipient = comptime_sutures[0].unstitch(&input).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.source, "api");
}

// ---------------------------------------------------------------------------
// 18. Integer constant into struct
// ---------------------------------------------------------------------------

#[test]
fn resp_const_integer_unstitch() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_int",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [{ "count": 42 }]
            }]
        }]
    }"#);

    let input = json!({ "content": "hello" });
    let result: RespIntConst = comptime_sutures[0].unstitch(&input).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.count, 42);
}

// ---------------------------------------------------------------------------
// 19. Multiple response constants
// ---------------------------------------------------------------------------

#[test]
fn resp_const_multiple_unstitch() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_multi",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [
                    { "source": "api" },
                    { "count": 99 }
                ]
            }]
        }]
    }"#);

    let input = json!({ "content": "hello" });
    let result: RespMultiConst = comptime_sutures[0].unstitch(&input).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.source, "api");
    assert_eq!(result.count, 99);
}

// ---------------------------------------------------------------------------
// 20. Nested struct constant path
// ---------------------------------------------------------------------------

#[test]
fn resp_const_nested_path_unstitch() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_nested",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [{ "meta.version": "1.0" }]
            }]
        }]
    }"#);

    let input = json!({ "content": "hello" });
    let result: RespNestedConst = comptime_sutures[0].unstitch(&input).unwrap();
    assert_eq!(result.content, "hello");
    assert_eq!(result.meta.version, "1.0");
}

// ===========================================================================
// CONSTANTS NOT INJECTED IN WRONG DIRECTION
// ===========================================================================

// ---------------------------------------------------------------------------
// 21. Request constants NOT in unstitch
// ---------------------------------------------------------------------------

#[test]
fn req_constants_not_in_unstitch() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req_only",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [{ "/api_version": "v1" }]
            }]
        }]
    }"#);

    // Unstitch from JSON back to struct — request constants should NOT be
    // injected into the struct output.
    let input = json!({ "model": "gpt-4", "api_version": "v1" });
    let result: ConstantRecipient = comptime_sutures[0].unstitch(&input).unwrap();
    assert_eq!(result.model, "gpt-4");
    // api_version was a request constant (JSON-side); on unstitch (reverse for
    // request direction), it reads from JSON targets and writes struct via trie.
    // The constant is NOT injected during reverse walk.
    assert_eq!(result.api_version, "");
}

// ---------------------------------------------------------------------------
// 22. Response constants NOT in stitch
// ---------------------------------------------------------------------------

#[test]
fn resp_constants_not_in_stitch() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp_only",
            "capture": "response",
            "sutures": [{
                "/content": "content",
                "_": [{ "source": "api" }]
            }]
        }]
    }"#);

    // Stitch from struct to JSON — response constants should NOT be injected
    // into the JSON output.
    let input = RespConstRecipient {
        content: "hello".into(),
        source: "should_not_appear".into(),
    };
    let result = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(result["content"], "hello");
    // Response constants are struct-side; stitch for response direction is
    // reverse walk, which does NOT inject constants.
    assert!(result.get("source").is_none());
}

// ===========================================================================
// METADATA ACCESSORS
// ===========================================================================

// ---------------------------------------------------------------------------
// 23. suture.name()
// ---------------------------------------------------------------------------

#[test]
fn metadata_name() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "my_suture",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);

    assert_eq!(comptime_sutures[0].name(), "my_suture");
}

// ---------------------------------------------------------------------------
// 24. suture.id() when set
// ---------------------------------------------------------------------------

#[test]
fn metadata_id() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "id": "suture-001",
            "name": "identified",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);

    assert_eq!(comptime_sutures[0].id(), Some("suture-001"));

    // Without id
    let no_id = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "no_id",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    assert_eq!(no_id[0].id(), None);
}

// ---------------------------------------------------------------------------
// 25. suture.description() when set
// ---------------------------------------------------------------------------

#[test]
fn metadata_description() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "described",
            "description": "A test suture for validation",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);

    assert_eq!(
        comptime_sutures[0].description(),
        Some("A test suture for validation")
    );

    let no_desc = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "no_desc",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    assert_eq!(no_desc[0].description(), None);
}

// ---------------------------------------------------------------------------
// 26. suture.version() when set
// ---------------------------------------------------------------------------

#[test]
fn metadata_version() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "versioned",
            "version": "2.1.0",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);

    assert_eq!(comptime_sutures[0].version(), Some("2.1.0"));

    let no_ver = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "no_ver",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    assert_eq!(no_ver[0].version(), None);
}

// ---------------------------------------------------------------------------
// 27. suture.is_request() / suture.is_response()
// ---------------------------------------------------------------------------

#[test]
fn metadata_direction() {
    let req_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "req",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);

    assert!(req_sutures[0].is_request());
    assert!(!req_sutures[0].is_response());

    let resp_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "resp",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#);

    assert!(!resp_sutures[0].is_request());
    assert!(resp_sutures[0].is_response());
}

// ---------------------------------------------------------------------------
// 28. Display impl
// ---------------------------------------------------------------------------

#[test]
fn metadata_display() {
    // name only
    let s1 = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bare",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    assert_eq!(format!("{}", s1[0]), "bare");

    // name + version
    let s2 = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "versioned",
            "version": "1.0",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    assert_eq!(format!("{}", s2[0]), "versioned@1.0");

    // name + id
    let s3 = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "id": "abc",
            "name": "identified",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    assert_eq!(format!("{}", s3[0]), "identified (abc)");

    // name + id + version
    let s4 = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "id": "xyz",
            "name": "full",
            "version": "3.0",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    assert_eq!(format!("{}", s4[0]), "full@3.0 (xyz)");
}

// ===========================================================================
// MULTIPLE SUTURE_SETS
// ===========================================================================

// ---------------------------------------------------------------------------
// 29. Two sets in one schema, verify both compile and work
// ---------------------------------------------------------------------------

#[test]
fn multiple_sets_both_work() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
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
                "sutures": [{ "model": "/config/model_name" }]
            }
        ]
    }"#);

    assert_eq!(comptime_sutures.len(), 2);
    assert_eq!(comptime_sutures[0].name(), "set_a");
    assert_eq!(comptime_sutures[1].name(), "set_b");

    let input = SimpleReq { model: "gpt-4".into() };

    let a = comptime_sutures[0].stitch(&input).unwrap();
    assert_eq!(a, json!({ "model": "gpt-4" }));

    let b = comptime_sutures[1].stitch(&input).unwrap();
    assert_eq!(b, json!({ "config": { "model_name": "gpt-4" } }));
}

// ---------------------------------------------------------------------------
// 30. One request + one response set
// ---------------------------------------------------------------------------

#[test]
fn multiple_sets_request_and_response() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "req",
                "capture": "request",
                "sutures": [{ "model": "/model" }]
            },
            {
                "name": "resp",
                "capture": "response",
                "sutures": [{ "/content": "content" }]
            }
        ]
    }"#);

    assert_eq!(comptime_sutures.len(), 2);
    assert!(comptime_sutures[0].is_request());
    assert!(comptime_sutures[1].is_response());

    let req_input = SimpleReq { model: "gpt-4".into() };
    let req_result = comptime_sutures[0].stitch(&req_input).unwrap();
    assert_eq!(req_result, json!({ "model": "gpt-4" }));

    let resp_input = json!({ "content": "output text" });
    let resp_result: SimpleResp = comptime_sutures[1].unstitch(&resp_input).unwrap();
    assert_eq!(resp_result.content, "output text");
}

// ===========================================================================
// MIXED BINDING TYPES
// ===========================================================================

// ---------------------------------------------------------------------------
// 31. Direct + Iterate in same suture
// ---------------------------------------------------------------------------

#[test]
fn mixed_direct_and_iterate() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_bind",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "tags[:]": "/tags/[:]"
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_bind",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "tags[:]": "/tags/[:]"
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = DirectIterCombo {
        model: "gpt-4".into(),
        tags: vec!["a".into(), "b".into()],
    };

    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    assert_eq!(ct["model"], "gpt-4");
    assert_eq!(ct["tags"], json!(["a", "b"]));
}

// ---------------------------------------------------------------------------
// 32. Direct + Constants + Iterate together
// ---------------------------------------------------------------------------

#[test]
fn mixed_direct_constants_iterate() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "triple",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "items[:]": "/data/[:]",
                "_": [{ "/api_version": "v2" }]
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "triple",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "items[:]": "/data/[:]",
                "_": [{ "/api_version": "v2" }]
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = TripleCombo {
        model: "claude".into(),
        items: vec!["x".into(), "y".into()],
    };

    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    assert_eq!(ct["model"], "claude");
    assert_eq!(ct["data"], json!(["x", "y"]));
    assert_eq!(ct["api_version"], "v2");
}

// ---------------------------------------------------------------------------
// 33. Multiple suture objects in array
// ---------------------------------------------------------------------------

#[test]
fn multiple_suture_objects_in_array() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj",
            "capture": "request",
            "sutures": [
                { "model": "/model" },
                { "temperature": "/temperature" }
            ]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj",
            "capture": "request",
            "sutures": [
                { "model": "/model" },
                { "temperature": "/temperature" }
            ]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = MultiField {
        model: "gpt-4".into(),
        temperature: 0.9,
    };

    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    assert_eq!(ct["model"], "gpt-4");
    assert_eq!(ct["temperature"], 0.9);
}

// ===========================================================================
// EDGE CASES
// ===========================================================================

// ---------------------------------------------------------------------------
// 34. Null values through stitch/unstitch
// ---------------------------------------------------------------------------

#[test]
fn edge_null_values() {
    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "null_test",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#);

    // A null field in JSON — unstitch should produce the default.
    let input = json!({ "content": null });
    let result: NullConstRecipient = comptime_sutures[0].unstitch::<NullConstRecipient>(&input).unwrap();
    assert_eq!(result.extra, None);
}

// ---------------------------------------------------------------------------
// 35. Empty string values
// ---------------------------------------------------------------------------

#[test]
fn edge_empty_string() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str",
            "capture": "request",
            "sutures": [{ "value": "/value" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_str",
            "capture": "request",
            "sutures": [{ "value": "/value" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = EmptyStr { value: "".into() };
    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    assert_eq!(ct, json!({ "value": "" }));

    // Also test unstitch
    let json_input = json!({ "value": "" });
    let ct_un: EmptyStr = comptime_sutures[0].unstitch(&json_input).unwrap();
    let rt_un: EmptyStr = runtime_sutures[0].unstitch(&json_input).unwrap();
    assert_eq!(ct_un, rt_un);
    assert_eq!(ct_un.value, "");
}

// ---------------------------------------------------------------------------
// 36. Large i64 values
// ---------------------------------------------------------------------------

#[test]
fn edge_large_i64() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "large_int",
            "capture": "request",
            "sutures": [{ "big_int": "/big_int" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "large_int",
            "capture": "request",
            "sutures": [{ "big_int": "/big_int" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = LargeNums { big_int: i64::MAX };
    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    assert_eq!(ct["big_int"], i64::MAX);

    let input_min = LargeNums { big_int: i64::MIN };
    let ct_min = comptime_sutures[0].stitch(&input_min).unwrap();
    let rt_min = runtime_sutures[0].stitch(&input_min).unwrap();
    assert_eq!(ct_min, rt_min);
    assert_eq!(ct_min["big_int"], i64::MIN);
}

// ---------------------------------------------------------------------------
// 37. Boolean values
// ---------------------------------------------------------------------------

#[test]
fn edge_bool_values() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bools",
            "capture": "request",
            "sutures": [{ "a": "/a", "b": "/b" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bools",
            "capture": "request",
            "sutures": [{ "a": "/a", "b": "/b" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = BoolVals { a: true, b: false };
    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    assert_eq!(ct["a"], true);
    assert_eq!(ct["b"], false);

    // Round-trip via unstitch
    let ct_back: BoolVals = comptime_sutures[0].unstitch(&ct).unwrap();
    let rt_back: BoolVals = runtime_sutures[0].unstitch(&rt).unwrap();
    assert_eq!(ct_back, rt_back);
    assert_eq!(ct_back.a, true);
    assert_eq!(ct_back.b, false);
}

// ---------------------------------------------------------------------------
// 38. Empty arrays with #[serde(default)]
// ---------------------------------------------------------------------------

#[test]
fn edge_empty_arrays() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_arr",
            "capture": "request",
            "sutures": [{ "items[:]": "/items/[:]" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_arr",
            "capture": "request",
            "sutures": [{ "items[:]": "/items/[:]" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = DefaultVecStruct { items: vec![] };
    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    // Empty array iteration produces no output entries — result is empty object.
    assert_eq!(ct, json!({}));

    // Unstitch with empty source
    let json_input = json!({});
    let ct_un: DefaultVecStruct = comptime_sutures[0].unstitch(&json_input).unwrap();
    let rt_un: DefaultVecStruct = runtime_sutures[0].unstitch(&json_input).unwrap();
    assert_eq!(ct_un, rt_un);
    assert_eq!(ct_un.items, Vec::<String>::new());
}

// ===========================================================================
// ADDITIONAL EQUIVALENCE: round-trip stitch -> unstitch
// ===========================================================================

#[test]
fn roundtrip_request_stitch_unstitch() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "temperature": "/temperature"
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "temperature": "/temperature"
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let original = MultiField {
        model: "gpt-4".into(),
        temperature: 0.42,
    };

    // Stitch then unstitch — should round-trip.
    let ct_json = comptime_sutures[0].stitch(&original).unwrap();
    let rt_json = runtime_sutures[0].stitch(&original).unwrap();
    assert_eq!(ct_json, rt_json);

    let ct_back: MultiField = comptime_sutures[0].unstitch(&ct_json).unwrap();
    let rt_back: MultiField = runtime_sutures[0].unstitch(&rt_json).unwrap();
    assert_eq!(ct_back, rt_back);
    assert_eq!(ct_back, original);
}

#[test]
fn roundtrip_response_unstitch_stitch() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip_resp",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip_resp",
            "capture": "response",
            "sutures": [{ "/content": "content" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let original_json = json!({ "content": "response text" });

    // Unstitch then stitch — should round-trip.
    let ct_struct: SimpleResp = comptime_sutures[0].unstitch(&original_json).unwrap();
    let rt_struct: SimpleResp = runtime_sutures[0].unstitch(&original_json).unwrap();
    assert_eq!(ct_struct, rt_struct);

    let ct_json = comptime_sutures[0].stitch(&ct_struct).unwrap();
    let rt_json = runtime_sutures[0].stitch(&rt_struct).unwrap();
    assert_eq!(ct_json, rt_json);
    assert_eq!(ct_json, original_json);
}

#[test]
fn roundtrip_iterate_request() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_rt",
            "capture": "request",
            "sutures": [{ "items[:]": "/data/[:]" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_rt",
            "capture": "request",
            "sutures": [{ "items[:]": "/data/[:]" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let original = IterContainer {
        items: vec!["foo".into(), "bar".into(), "baz".into()],
    };

    let ct_json = comptime_sutures[0].stitch(&original).unwrap();
    let rt_json = runtime_sutures[0].stitch(&original).unwrap();
    assert_eq!(ct_json, rt_json);
    assert_eq!(ct_json, json!({ "data": ["foo", "bar", "baz"] }));
}

// ===========================================================================
// EQUIVALENCE: metadata matches between comptime and runtime
// ===========================================================================

#[test]
fn equiv_metadata_matches() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "id": "s-42",
            "name": "meta_test",
            "description": "Full metadata suture",
            "version": "1.2.3",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "id": "s-42",
            "name": "meta_test",
            "description": "Full metadata suture",
            "version": "1.2.3",
            "capture": "request",
            "sutures": [{ "model": "/model" }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    assert_eq!(comptime_sutures[0].name(), runtime_sutures[0].name());
    assert_eq!(comptime_sutures[0].id(), runtime_sutures[0].id());
    assert_eq!(
        comptime_sutures[0].description(),
        runtime_sutures[0].description()
    );
    assert_eq!(comptime_sutures[0].version(), runtime_sutures[0].version());
    assert_eq!(
        comptime_sutures[0].is_request(),
        runtime_sutures[0].is_request()
    );
    assert_eq!(
        comptime_sutures[0].is_response(),
        runtime_sutures[0].is_response()
    );
    assert_eq!(
        format!("{}", comptime_sutures[0]),
        format!("{}", runtime_sutures[0])
    );
}

// ===========================================================================
// EQUIVALENCE: constants match between comptime and runtime
// ===========================================================================

#[test]
fn equiv_constants_match() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "const_eq",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [
                    { "/str_const": "hello" },
                    { "/int_const": 42 },
                    { "/float_const": 3.14 },
                    { "/bool_const": true },
                    { "/null_const": null }
                ]
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "const_eq",
            "capture": "request",
            "sutures": [{
                "model": "/model",
                "_": [
                    { "/str_const": "hello" },
                    { "/int_const": 42 },
                    { "/float_const": 3.14 },
                    { "/bool_const": true },
                    { "/null_const": null }
                ]
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    // Compare stitch results — both should have the same constant values
    let input = SimpleReq { model: "gpt-4".into() };
    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);

    assert_eq!(ct["str_const"], "hello");
    assert_eq!(ct["int_const"], 42);
    assert_eq!(ct["float_const"], 3.14);
    assert_eq!(ct["bool_const"], true);
    assert!(ct["null_const"].is_null());
}

// ===========================================================================
// EQUIVALENCE: nested iterate with sub-field extraction
// ===========================================================================

#[test]
fn equiv_iterate_with_subfields() {
    let schema = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_sub",
            "capture": "request",
            "sutures": [{
                "messages[:]": {
                    "content": "/messages/[:]/content"
                }
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "iter_sub",
            "capture": "request",
            "sutures": [{
                "messages[:]": {
                    "content": "/messages/[:]/content"
                }
            }]
        }]
    }"#);
    let runtime_sutures = runtime_all(schema);

    let input = MixedBindReq {
        model: "unused".into(),
        messages: vec![
            MsgItem { content: "hello".into() },
            MsgItem { content: "world".into() },
        ],
    };

    let ct = comptime_sutures[0].stitch(&input).unwrap();
    let rt = runtime_sutures[0].stitch(&input).unwrap();
    assert_eq!(ct, rt);
    assert_eq!(
        ct,
        json!({ "messages": [{ "content": "hello" }, { "content": "world" }] })
    );
}
