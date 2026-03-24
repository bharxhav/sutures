// ============================================================================
// Integration tests: compile-time parse!() + Stitch for
// Response direction + IteratePattern binding (unstitch)
//
// These mirror crates/core/tests/stitch_response_pattern.rs but exercise the
// `sutures_comptime::parse!()` proc-macro path instead of the runtime
// `sutures::v1::parse()` path. The macro parses and compiles the manifest at
// compile time, producing `Vec<Suture>` with `Cow::Borrowed` strings.
//
// For response sutures:
//   - Keys (LHS) are JSON terminals (slash-separated, start with `/`)
//   - Values (RHS) are struct terminals (dot-separated, start with letter)
//   - `unstitch` is the natural direction: JSON -> struct
//
// IteratePattern on the JSON key side uses backtick-delimited regex WITHIN
// the path:
//   /`field_\d+`             -- match JSON object keys like field_0, field_1
//   /data/`entry_\d+`/name   -- nested path then pattern
//   /`field_\d+`[0:2]        -- pattern with slice qualifier
//
// In the `parse!` raw string `r#"..."#`:
//   - Backticks are literal (no escaping needed)
//   - Backslashes need JSON escaping: `\\d+` in JSON = `\d+` regex
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Seam, Stitch};

// ============================================================================
// 1. Basic pattern: /`field_\\d+` -> values[:]
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct BasicPatternResult {
    values: Vec<i64>,
}

#[test]
fn basic_pattern_field_digits_to_vec() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "basic_pattern",
            "capture": "response",
            "sutures": [{
                "/`field_\\d+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "field_0": 10,
        "field_1": 20,
        "field_2": 30
    });

    let result: BasicPatternResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 3);
    assert!(result.values.contains(&10));
    assert!(result.values.contains(&20));
    assert!(result.values.contains(&30));
}

// ============================================================================
// 2. Alpha pattern: /`item_[a-z]+` -> items[:]
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct AlphaPatternResult {
    items: Vec<String>,
}

#[test]
fn alpha_pattern_item_lowercase_to_vec() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "alpha_pattern",
            "capture": "response",
            "sutures": [{
                "/`item_[a-z]+`": "items[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "item_foo": "hello",
        "item_bar": "world"
    });

    let result: AlphaPatternResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.items.len(), 2);
    assert!(result.items.contains(&"hello".to_string()));
    assert!(result.items.contains(&"world".to_string()));
}

// ============================================================================
// 3. Pattern with no matches -> empty Vec (serde(default))
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NoMatchResult {
    #[serde(default)]
    values: Vec<i64>,
}

#[test]
fn pattern_no_matches_yields_empty_vec() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "no_match",
            "capture": "response",
            "sutures": [{
                "/`field_\\d+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "name": "Alice",
        "age": 30,
        "active": true
    });

    let result: NoMatchResult = suture.unstitch(&input).unwrap();
    assert!(result.values.is_empty());
}

// ============================================================================
// 4. Pattern matching only some keys (non-matching ignored)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct PartialMatchResult {
    values: Vec<i64>,
}

#[test]
fn pattern_partial_match_ignores_non_matching_keys() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "partial_match",
            "capture": "response",
            "sutures": [{
                "/`field_\\d+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "field_0": 100,
        "field_1": 200,
        "name": "should_be_ignored",
        "other_key": 999
    });

    let result: PartialMatchResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 2);
    assert!(result.values.contains(&100));
    assert!(result.values.contains(&200));
}

// ============================================================================
// 5. Non-capturing group: /`(?:in|out)_\\d+` -> values[:]
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NonCapturingGroupResult {
    values: Vec<i64>,
}

#[test]
fn non_capturing_group_pattern() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "non_capturing",
            "capture": "response",
            "sutures": [{
                "/`(?:in|out)_\\d+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "in_0": 1,
        "out_1": 2,
        "in_2": 3,
        "other_0": 999
    });

    let result: NonCapturingGroupResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 3);
    assert!(result.values.contains(&1));
    assert!(result.values.contains(&2));
    assert!(result.values.contains(&3));
}

// ============================================================================
// 6. Pattern with slice [0] -- first match only
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FirstMatchResult {
    values: Vec<i64>,
}

#[test]
fn pattern_slice_first_only() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "slice_first",
            "capture": "response",
            "sutures": [{
                "/`field_\\d+`[0]": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "field_0": 10,
        "field_1": 20,
        "field_2": 30
    });

    let result: FirstMatchResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 1);
}

// ============================================================================
// 7. Pattern with slice [0:2] -- first two matches
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FirstTwoResult {
    values: Vec<i64>,
}

#[test]
fn pattern_slice_first_two() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "slice_first_two",
            "capture": "response",
            "sutures": [{
                "/`field_\\d+`[0:2]": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "field_0": 10,
        "field_1": 20,
        "field_2": 30,
        "field_3": 40
    });

    let result: FirstTwoResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 2);
}

// ============================================================================
// 8. Nested path + pattern: /data/`entry_\\d+`/name -> names[:]
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedPatternResult {
    names: Vec<String>,
}

#[test]
fn nested_path_then_pattern_with_child_extraction() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_pattern",
            "capture": "response",
            "sutures": [{
                "/data/`entry_\\d+`/name": "names[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "data": {
            "entry_0": { "name": "Alice", "age": 30 },
            "entry_1": { "name": "Bob", "age": 25 },
            "entry_2": { "name": "Carol", "age": 35 }
        }
    });

    let result: NestedPatternResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.names.len(), 3);
    assert!(result.names.contains(&"Alice".to_string()));
    assert!(result.names.contains(&"Bob".to_string()));
    assert!(result.names.contains(&"Carol".to_string()));
}

// ============================================================================
// 9. Pattern with sub-field extraction (nested object)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SubFieldResult {
    names: Vec<String>,
    scores: Vec<i64>,
}

#[test]
fn pattern_extract_multiple_sub_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "sub_fields",
            "capture": "response",
            "sutures": [{
                "/`player_\\d+`": {
                    "/name": "names[:]",
                    "/score": "scores[:]"
                }
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "player_0": { "name": "Alice", "score": 100 },
        "player_1": { "name": "Bob", "score": 200 }
    });

    let result: SubFieldResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.names.len(), 2);
    assert_eq!(result.scores.len(), 2);
    assert!(result.names.contains(&"Alice".to_string()));
    assert!(result.names.contains(&"Bob".to_string()));
    assert!(result.scores.contains(&100));
    assert!(result.scores.contains(&200));
}

// ============================================================================
// 10. Fan-out from pattern -> multiple struct fields
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FanOutResult {
    primary: Vec<i64>,
    secondary: Vec<i64>,
}

#[test]
fn pattern_fan_out_to_multiple_struct_fields() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fan_out",
            "capture": "response",
            "sutures": [{
                "/`val_\\d+`": ["primary[:]", "secondary[:]"]
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "val_0": 10,
        "val_1": 20
    });

    let result: FanOutResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.primary.len(), 2);
    assert_eq!(result.secondary.len(), 2);
    assert!(result.primary.contains(&10));
    assert!(result.primary.contains(&20));
    assert!(result.secondary.contains(&10));
    assert!(result.secondary.contains(&20));
}

// ============================================================================
// 11. Two different patterns in same suture
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TwoPatternsResult {
    inputs: Vec<f64>,
    outputs: Vec<f64>,
}

#[test]
fn two_different_patterns_in_same_suture() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "two_patterns",
            "capture": "response",
            "sutures": [{
                "/`in_\\d+`": "inputs[:]",
                "/`out_\\d+`": "outputs[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "in_0": 1.0,
        "in_1": 2.0,
        "out_0": 10.0,
        "out_1": 20.0,
        "out_2": 30.0
    });

    let result: TwoPatternsResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.inputs.len(), 2);
    assert_eq!(result.outputs.len(), 3);
    assert!(result.inputs.contains(&1.0));
    assert!(result.inputs.contains(&2.0));
    assert!(result.outputs.contains(&10.0));
    assert!(result.outputs.contains(&20.0));
    assert!(result.outputs.contains(&30.0));
}

// ============================================================================
// 12. Mix of direct and pattern bindings
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedResult {
    name: String,
    values: Vec<i64>,
}

#[test]
fn mix_of_direct_and_pattern_bindings() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "response",
            "sutures": [{
                "/name": "name",
                "/`field_\\d+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "name": "test_object",
        "field_0": 100,
        "field_1": 200,
        "field_2": 300
    });

    let result: MixedResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.name, "test_object");
    assert_eq!(result.values.len(), 3);
    assert!(result.values.contains(&100));
    assert!(result.values.contains(&200));
    assert!(result.values.contains(&300));
}

// ============================================================================
// 13. Pattern matching all keys
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct AllKeysResult {
    values: Vec<String>,
}

#[test]
fn pattern_matching_all_keys() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "all_keys",
            "capture": "response",
            "sutures": [{
                "/`[a-z]+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "alpha": "one",
        "beta": "two",
        "gamma": "three"
    });

    let result: AllKeysResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 3);
    assert!(result.values.contains(&"one".to_string()));
    assert!(result.values.contains(&"two".to_string()));
    assert!(result.values.contains(&"three".to_string()));
}

// ============================================================================
// 14. Numeric-only keys: \\d+
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NumericKeysResult {
    values: Vec<String>,
}

#[test]
fn pattern_numeric_only_keys() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "numeric_keys",
            "capture": "response",
            "sutures": [{
                "/`\\d+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({
        "0": "zero",
        "1": "one",
        "42": "answer",
        "abc": "should_not_match"
    });

    let result: NumericKeysResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 3);
    assert!(result.values.contains(&"zero".to_string()));
    assert!(result.values.contains(&"one".to_string()));
    assert!(result.values.contains(&"answer".to_string()));
}

// ============================================================================
// 15. Empty JSON object -> empty Vec (serde(default))
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct EmptyObjectResult {
    #[serde(default)]
    values: Vec<i64>,
}

#[test]
fn empty_json_object_yields_empty_vec() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_object",
            "capture": "response",
            "sutures": [{
                "/`field_\\d+`": "values[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];
    let input = json!({});

    let result: EmptyObjectResult = suture.unstitch(&input).unwrap();
    assert!(result.values.is_empty());
}
