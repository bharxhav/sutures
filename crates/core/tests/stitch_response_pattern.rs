// ============================================================================
// Integration tests: Response direction + IteratePattern binding (unstitch)
//
// For response sutures:
//   - Keys (LHS) are JSON terminals (slash-separated, start with `/`)
//   - Values (RHS) are struct terminals (dot-separated, start with letter)
//   - `unstitch` is natural direction: JSON -> struct
//   - `stitch` is reverse direction: struct -> JSON
//
// IteratePattern on the key (JSON/LHS) side uses backtick-delimited regex:
//   /`field_\d+`         -- match object keys like field_0, field_1
//   /data/`item_\d+`     -- nested path then pattern match
//   /`field_\d+`[0:2]    -- pattern with slice qualifier
//
// Regex is ONLY allowed on the key/LHS side. The value/RHS (struct) side
// CANNOT have backticks.
// ============================================================================

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

/// Build a response suture from a single mapping object.
/// Wraps the mapping in the required `.sutures.json` envelope.
fn response_suture(mappings: serde_json::Value) -> sutures::v1::Suture {
    let manifest = json!({
        "name": "test",
        "suture_sets": [
            {
                "name": "test_set",
                "capture": "response",
                "sutures": [mappings]
            }
        ]
    });
    parse_first(&manifest.to_string())
}

// ============================================================================
// 1. Basic pattern matching: /`field_\d+` -> values[:]
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct BasicPatternResult {
    values: Vec<i64>,
}

#[test]
fn basic_pattern_field_digits_to_vec() {
    let suture = response_suture(json!({
        "/`field_\\d+`": "values[:]"
    }));

    let input = json!({
        "field_0": 10,
        "field_1": 20,
        "field_2": 30
    });

    let result: BasicPatternResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 3);
    // All three values should be present (order depends on JSON object iteration)
    assert!(result.values.contains(&10));
    assert!(result.values.contains(&20));
    assert!(result.values.contains(&30));
}

// ============================================================================
// 2. Alpha pattern: /`item_[a-z]+` -> items[:]
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct AlphaPatternResult {
    items: Vec<String>,
}

#[test]
fn alpha_pattern_item_lowercase_to_vec() {
    let suture = response_suture(json!({
        "/`item_[a-z]+`": "items[:]"
    }));

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
// 3. Pattern with no matches -> empty struct array
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NoMatchResult {
    #[serde(default)]
    values: Vec<i64>,
}

#[test]
fn pattern_no_matches_yields_empty_vec() {
    let suture = response_suture(json!({
        "/`field_\\d+`": "values[:]"
    }));

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct PartialMatchResult {
    values: Vec<i64>,
}

#[test]
fn pattern_partial_match_ignores_non_matching_keys() {
    let suture = response_suture(json!({
        "/`field_\\d+`": "values[:]"
    }));

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
// 5. Non-capturing group: /`(?:in|out)_\d+` -> values[:]
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NonCapturingGroupResult {
    values: Vec<i64>,
}

#[test]
fn non_capturing_group_pattern() {
    let suture = response_suture(json!({
        "/`(?:in|out)_\\d+`": "values[:]"
    }));

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
// 6. Pattern with slice qualifier [0] -- only first match
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FirstMatchResult {
    values: Vec<i64>,
}

#[test]
fn pattern_slice_first_only() {
    let suture = response_suture(json!({
        "/`field_\\d+`[0]": "values[:]"
    }));

    let input = json!({
        "field_0": 10,
        "field_1": 20,
        "field_2": 30
    });

    let result: FirstMatchResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 1);
}

// ============================================================================
// 7. Pattern with slice qualifier [0:2] -- first two matches
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FirstTwoResult {
    values: Vec<i64>,
}

#[test]
fn pattern_slice_first_two() {
    let suture = response_suture(json!({
        "/`field_\\d+`[0:2]": "values[:]"
    }));

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
// 8. Pattern with slice qualifier [1:] -- skip first match
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SkipFirstResult {
    values: Vec<i64>,
}

#[test]
fn pattern_slice_skip_first() {
    let suture = response_suture(json!({
        "/`field_\\d+`[1:]": "values[:]"
    }));

    let input = json!({
        "field_0": 10,
        "field_1": 20,
        "field_2": 30
    });

    let result: SkipFirstResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 2);
}

// ============================================================================
// 9. Pattern with slice qualifier [::-1] -- reverse order
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ReverseOrderResult {
    values: Vec<i64>,
}

#[test]
fn pattern_slice_reverse_order() {
    let suture = response_suture(json!({
        "/`field_\\d+`[::-1]": "values[:]"
    }));

    let input = json!({
        "field_0": 10,
        "field_1": 20,
        "field_2": 30
    });

    let result: ReverseOrderResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 3);
    // The reversed slice should reverse the order of the matched keys.
    // Since JSON object key order is implementation-defined, we verify
    // the first element in the reversed result equals the last in the
    // forward iteration (and vice versa).
    let suture_fwd = response_suture(json!({
        "/`field_\\d+`": "values[:]"
    }));
    let fwd: ReverseOrderResult = suture_fwd.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), fwd.values.len());
    // The reversed result should be the forward result reversed.
    let mut expected = fwd.values.clone();
    expected.reverse();
    assert_eq!(result.values, expected);
}

// ============================================================================
// 10. Nested path then pattern: /data/`entry_\d+`/name -> extract name field
//     from each matching entry
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NestedPatternResult {
    names: Vec<String>,
}

#[test]
fn nested_path_then_pattern_with_child_extraction() {
    let suture = response_suture(json!({
        "/data/`entry_\\d+`/name": "names[:]"
    }));

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
// 11. Pattern as nested object key: iterate pattern matches and extract
//     multiple sub-fields
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MultiSubFieldResult {
    names: Vec<String>,
    scores: Vec<i64>,
}

#[test]
fn pattern_extract_multiple_sub_fields() {
    let suture = response_suture(json!({
        "/`player_\\d+`": {
            "/name": "names[:]",
            "/score": "scores[:]"
        }
    }));

    let input = json!({
        "player_0": { "name": "Alice", "score": 100 },
        "player_1": { "name": "Bob", "score": 200 }
    });

    let result: MultiSubFieldResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.names.len(), 2);
    assert_eq!(result.scores.len(), 2);
    assert!(result.names.contains(&"Alice".to_string()));
    assert!(result.names.contains(&"Bob".to_string()));
    assert!(result.scores.contains(&100));
    assert!(result.scores.contains(&200));
}

// ============================================================================
// 12. Fan-out: pattern match -> multiple struct fields
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FanOutResult {
    primary: Vec<i64>,
    secondary: Vec<i64>,
}

#[test]
fn pattern_fan_out_to_multiple_struct_fields() {
    let suture = response_suture(json!({
        "/`val_\\d+`": ["primary[:]", "secondary[:]"]
    }));

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
// 13. Two different patterns matching different JSON key families
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct TwoPatternsResult {
    inputs: Vec<f64>,
    outputs: Vec<f64>,
}

#[test]
fn two_different_patterns_in_same_suture() {
    let suture = response_suture(json!({
        "/`in_\\d+`": "inputs[:]",
        "/`out_\\d+`": "outputs[:]"
    }));

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
// 14. Mix of direct and pattern fields
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedResult {
    name: String,
    values: Vec<i64>,
}

#[test]
fn mix_of_direct_and_pattern_bindings() {
    let suture = response_suture(json!({
        "/name": "name",
        "/`field_\\d+`": "values[:]"
    }));

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
// 15. Pattern matching all keys
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct AllKeysResult {
    values: Vec<String>,
}

#[test]
fn pattern_matching_all_keys() {
    let suture = response_suture(json!({
        "/`[a-z]+`": "values[:]"
    }));

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
// 16. Pattern with special regex characters (dots, brackets, etc.)
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SpecialCharsResult {
    values: Vec<String>,
}

#[test]
fn pattern_with_escaped_regex_chars() {
    // Match keys like "v1.0", "v2.0" — the dot must be escaped in regex
    let suture = response_suture(json!({
        "/`v\\d+\\.\\d+`": "values[:]"
    }));

    let input = json!({
        "v1.0": "first",
        "v2.0": "second",
        "v30": "should_not_match"
    });

    let result: SpecialCharsResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 2);
    assert!(result.values.contains(&"first".to_string()));
    assert!(result.values.contains(&"second".to_string()));
}

// ============================================================================
// 17. Numeric-only keys matching \d+
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NumericKeysResult {
    values: Vec<String>,
}

#[test]
fn pattern_numeric_only_keys() {
    let suture = response_suture(json!({
        "/`\\d+`": "values[:]"
    }));

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
// 18. Single-character keys matching [a-z]
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SingleCharResult {
    values: Vec<i64>,
}

#[test]
fn pattern_single_char_keys() {
    let suture = response_suture(json!({
        "/`[a-z]`": "values[:]"
    }));

    let input = json!({
        "a": 1,
        "b": 2,
        "z": 26,
        "ab": 99,
        "A": 100
    });

    let result: SingleCharResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 3);
    assert!(result.values.contains(&1));
    assert!(result.values.contains(&2));
    assert!(result.values.contains(&26));
}

// ============================================================================
// 19. Empty JSON object -> no matches -> empty arrays
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct EmptyObjectResult {
    #[serde(default)]
    values: Vec<i64>,
}

#[test]
fn empty_json_object_yields_empty_arrays() {
    let suture = response_suture(json!({
        "/`field_\\d+`": "values[:]"
    }));

    let input = json!({});

    let result: EmptyObjectResult = suture.unstitch(&input).unwrap();
    assert!(result.values.is_empty());
}
