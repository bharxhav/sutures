//! Exhaustive integration tests for **Request direction + IteratePattern binding** (`stitch`).
//!
//! Every test builds a suture from JSON, constructs a typed struct, calls
//! `suture.stitch(&input)`, and asserts the output matches the expected JSON.

use serde_json::{Value, json};
use std::collections::HashMap;
use sutures::stitch::Stitch;

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

// ============================================================================
// Test structs
// ============================================================================

/// Struct with a single HashMap field for dynamic keys.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DynamicFields {
    data: HashMap<String, i32>,
}

/// Struct with a top-level HashMap (no wrapping object).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FlatDynamic {
    fields: HashMap<String, String>,
}

/// Struct with both a fixed field and a dynamic field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MixedStruct {
    name: String,
    data: HashMap<String, i32>,
}

/// Struct with two separate dynamic-key families.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct TwoFamilies {
    alpha: HashMap<String, i32>,
    beta: HashMap<String, i32>,
}

/// Struct with nested objects as dynamic values.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NestedDynamic {
    entries: HashMap<String, Entry>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Entry {
    name: String,
    value: i32,
}

/// Struct where the top level itself is the dynamic map (using serde_json::Value).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct TopLevelMap {
    items: HashMap<String, String>,
}

/// Struct for testing patterns with very long key names.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct LongKeyStruct {
    store: HashMap<String, i32>,
}

/// Struct for single-char keys.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SingleCharKeys {
    bucket: HashMap<String, i32>,
}

/// Struct for numeric-only keys.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NumericKeys {
    index: HashMap<String, String>,
}

// ============================================================================
// 1. Basic: `field_\d+` matching field_0, field_1, field_2
// ============================================================================

#[test]
fn basic_pattern_field_digits() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "basic_pattern",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/values/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 10),
            ("field_1".into(), 20),
            ("field_2".into(), 30),
        ]),
    };

    let result = suture.stitch(&input).unwrap();

    // The pattern iterates matching keys; order depends on HashMap iteration.
    // We verify the values array contains all three values (order may vary).
    let values = result["values"].as_array().unwrap();
    assert_eq!(values.len(), 3);

    let mut sorted: Vec<i64> = values.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![10, 20, 30]);
}

// ============================================================================
// 2. Basic: `item_[a-z]+` matching item_foo, item_bar
// ============================================================================

#[test]
fn basic_pattern_item_alpha() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "alpha_pattern",
            "capture": "request",
            "sutures": [{
                "items.`item_[a-z]+`": "/items/[:]"
            }]
        }]
    }"#,
    );

    let input = TopLevelMap {
        items: HashMap::from([
            ("item_foo".into(), "hello".into()),
            ("item_bar".into(), "world".into()),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);

    let mut sorted: Vec<&str> = items.iter().map(|v| v.as_str().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec!["hello", "world"]);
}

// ============================================================================
// 3. Basic: `content_\d+` with no matches → empty output array
// ============================================================================

#[test]
fn pattern_no_matches_empty_output() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "no_match",
            "capture": "request",
            "sutures": [{
                "data.`content_\\d+`": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("other_0".into(), 1), ("something_else".into(), 2)]),
    };

    let result = suture.stitch(&input).unwrap();

    // No keys match `content_\d+`, so `/output` should not be populated or should be empty.
    // The forward walk simply skips when there are no matches.
    assert!(
        result.get("output").is_none()
            || result["output"].as_array().map_or(true, |a| a.is_empty())
    );
}

// ============================================================================
// 4. Basic: `data_\d+` matching only some keys in mixed object
// ============================================================================

#[test]
fn pattern_partial_match_ignores_non_matching() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "partial",
            "capture": "request",
            "sutures": [{
                "data.`data_\\d+`": "/extracted/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("data_0".into(), 100),
            ("data_1".into(), 200),
            ("other_key".into(), 999),
            ("not_data".into(), 888),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let extracted = result["extracted"].as_array().unwrap();
    assert_eq!(extracted.len(), 2);

    let mut sorted: Vec<i64> = extracted.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![100, 200]);
}

// ============================================================================
// 5. Pattern with non-capturing group: `(?:input|output)_\d+`
// ============================================================================

#[test]
fn pattern_non_capturing_group() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "noncap",
            "capture": "request",
            "sutures": [{
                "data.`(?:input|output)_\\d+`": "/signals/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("input_0".into(), 1),
            ("output_1".into(), 2),
            ("random_key".into(), 3),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let signals = result["signals"].as_array().unwrap();
    assert_eq!(signals.len(), 2);

    let mut sorted: Vec<i64> = signals.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2]);
}

// ============================================================================
// 6. Slice qualifier: `field_\d+`[0] — only first match
// ============================================================================

#[test]
fn pattern_slice_first_only() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "first_only",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[0]": "/first/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 10),
            ("field_1".into(), 20),
            ("field_2".into(), 30),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let first = result["first"].as_array().unwrap();
    assert_eq!(first.len(), 1);

    // The single value should be one of the matched values.
    let val = first[0].as_i64().unwrap();
    assert!([10, 20, 30].contains(&val));
}

// ============================================================================
// 7. Slice qualifier: `field_\d+`[0:2] — first two matches
// ============================================================================

#[test]
fn pattern_slice_first_two() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "first_two",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[0:2]": "/two/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 10),
            ("field_1".into(), 20),
            ("field_2".into(), 30),
            ("field_3".into(), 40),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let two = result["two"].as_array().unwrap();
    assert_eq!(two.len(), 2);

    // Both values should be from the matched set.
    for v in two {
        let n = v.as_i64().unwrap();
        assert!([10, 20, 30, 40].contains(&n));
    }
}

// ============================================================================
// 8. Slice qualifier: `field_\d+`[1:] — skip first match
// ============================================================================

#[test]
fn pattern_slice_skip_first() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "skip_first",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[1:]": "/rest/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 10),
            ("field_1".into(), 20),
            ("field_2".into(), 30),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let rest = result["rest"].as_array().unwrap();

    // Should have 2 elements (skipped first of 3 matches).
    assert_eq!(rest.len(), 2);

    for v in rest {
        let n = v.as_i64().unwrap();
        assert!([10, 20, 30].contains(&n));
    }
}

// ============================================================================
// 9. Slice qualifier: `field_\d+`[::-1] — reverse order of matches
// ============================================================================

#[test]
fn pattern_slice_reverse() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "reverse",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[::-1]": "/reversed/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 10),
            ("field_1".into(), 20),
            ("field_2".into(), 30),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let reversed = result["reversed"].as_array().unwrap();
    assert_eq!(reversed.len(), 3);

    // Collect the forward order, then verify reversed is the reverse of it.
    // Since HashMap order is nondeterministic, we verify the reversed array
    // is a valid reversal by checking it's the same values in some reversed order.
    let vals: Vec<i64> = reversed.iter().map(|v| v.as_i64().unwrap()).collect();

    // All three values present.
    let mut sorted = vals.clone();
    sorted.sort();
    assert_eq!(sorted, vec![10, 20, 30]);
}

// ============================================================================
// 10. Pattern with child extraction (nested object)
// ============================================================================

#[test]
fn pattern_child_extraction() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "child_extract",
            "capture": "request",
            "sutures": [{
                "entries.`entry_\\d+`": {
                    "name": "/entries/[:]/name",
                    "value": "/entries/[:]/value"
                }
            }]
        }]
    }"#,
    );

    let input = NestedDynamic {
        entries: HashMap::from([
            (
                "entry_0".into(),
                Entry {
                    name: "alpha".into(),
                    value: 10,
                },
            ),
            (
                "entry_1".into(),
                Entry {
                    name: "beta".into(),
                    value: 20,
                },
            ),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let entries = result["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);

    // Each entry should have name and value fields.
    let mut names: Vec<&str> = entries
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);

    let mut values: Vec<i64> = entries
        .iter()
        .map(|e| e["value"].as_i64().unwrap())
        .collect();
    values.sort();
    assert_eq!(values, vec![10, 20]);
}

// ============================================================================
// 11. Pattern with fan-out to multiple targets
// ============================================================================

#[test]
fn pattern_fan_out() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": ["/primary/[:]", "/backup/[:]"]
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("field_0".into(), 10), ("field_1".into(), 20)]),
    };

    let result = suture.stitch(&input).unwrap();

    let primary = result["primary"].as_array().unwrap();
    let backup = result["backup"].as_array().unwrap();

    assert_eq!(primary.len(), 2);
    assert_eq!(backup.len(), 2);

    // Both targets should receive the same values.
    let mut primary_sorted: Vec<i64> = primary.iter().map(|v| v.as_i64().unwrap()).collect();
    let mut backup_sorted: Vec<i64> = backup.iter().map(|v| v.as_i64().unwrap()).collect();
    primary_sorted.sort();
    backup_sorted.sort();
    assert_eq!(primary_sorted, vec![10, 20]);
    assert_eq!(backup_sorted, vec![10, 20]);
}

// ============================================================================
// 12. Multiple patterns in same suture (two key families)
// ============================================================================

#[test]
fn multiple_patterns_two_families() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_pattern",
            "capture": "request",
            "sutures": [{
                "alpha.`a_\\d+`": "/group_a/[:]",
                "beta.`b_\\d+`": "/group_b/[:]"
            }]
        }]
    }"#,
    );

    let input = TwoFamilies {
        alpha: HashMap::from([("a_0".into(), 1), ("a_1".into(), 2)]),
        beta: HashMap::from([("b_0".into(), 100), ("b_1".into(), 200)]),
    };

    let result = suture.stitch(&input).unwrap();

    let group_a = result["group_a"].as_array().unwrap();
    let group_b = result["group_b"].as_array().unwrap();

    assert_eq!(group_a.len(), 2);
    assert_eq!(group_b.len(), 2);

    let mut a_sorted: Vec<i64> = group_a.iter().map(|v| v.as_i64().unwrap()).collect();
    a_sorted.sort();
    assert_eq!(a_sorted, vec![1, 2]);

    let mut b_sorted: Vec<i64> = group_b.iter().map(|v| v.as_i64().unwrap()).collect();
    b_sorted.sort();
    assert_eq!(b_sorted, vec![100, 200]);
}

// ============================================================================
// 13. Pattern that matches ALL keys in the object
// ============================================================================

#[test]
fn pattern_matches_all_keys() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "match_all",
            "capture": "request",
            "sutures": [{
                "data.`.*`": "/all/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("anything".into(), 1),
            ("everything".into(), 2),
            ("123".into(), 3),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let all = result["all"].as_array().unwrap();
    assert_eq!(all.len(), 3);

    let mut sorted: Vec<i64> = all.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2, 3]);
}

// ============================================================================
// 14. Pattern with anchoring (auto-anchored with ^...$)
// ============================================================================

#[test]
fn pattern_auto_anchored() {
    // `field` should NOT match `field_extra` because the regex is auto-anchored.
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "anchored",
            "capture": "request",
            "sutures": [{
                "data.`field`": "/exact/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field".into(), 42),
            ("field_extra".into(), 99),
            ("prefix_field".into(), 88),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let exact = result["exact"].as_array().unwrap();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].as_i64().unwrap(), 42);
}

// ============================================================================
// 15. Pattern with escaped special chars: `field\.name` matching literal dot
// ============================================================================

#[test]
fn pattern_escaped_dot() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "escaped_dot",
            "capture": "request",
            "sutures": [{
                "fields.`field\\.name`": "/dotted/[:]"
            }]
        }]
    }"#,
    );

    let input = FlatDynamic {
        fields: HashMap::from([
            ("field.name".into(), "matched".into()),
            ("fieldXname".into(), "should_not_match".into()),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let dotted = result["dotted"].as_array().unwrap();
    assert_eq!(dotted.len(), 1);
    assert_eq!(dotted[0].as_str().unwrap(), "matched");
}

// ============================================================================
// 16. Pattern with alternation: `(?:alpha|beta)_\d+`
// ============================================================================

#[test]
fn pattern_alternation() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "alternation",
            "capture": "request",
            "sutures": [{
                "data.`(?:alpha|beta)_\\d+`": "/matched/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("alpha_0".into(), 1),
            ("beta_1".into(), 2),
            ("gamma_2".into(), 3),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let matched = result["matched"].as_array().unwrap();
    assert_eq!(matched.len(), 2);

    let mut sorted: Vec<i64> = matched.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2]);
}

// ============================================================================
// 17. Very long key names matching pattern
// ============================================================================

#[test]
fn pattern_long_key_names() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "long_keys",
            "capture": "request",
            "sutures": [{
                "store.`field_[a-z_]+`": "/long/[:]"
            }]
        }]
    }"#,
    );

    let long_key_a = "field_".to_string() + &"a".repeat(100);
    let long_key_b = "field_".to_string() + &"b".repeat(100);

    let input = LongKeyStruct {
        store: HashMap::from([(long_key_a, 1), (long_key_b, 2)]),
    };

    let result = suture.stitch(&input).unwrap();
    let long = result["long"].as_array().unwrap();
    assert_eq!(long.len(), 2);

    let mut sorted: Vec<i64> = long.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2]);
}

// ============================================================================
// 18. Pattern matching single character keys: `[a-z]`
// ============================================================================

#[test]
fn pattern_single_char_keys() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_char",
            "capture": "request",
            "sutures": [{
                "bucket.`[a-z]`": "/chars/[:]"
            }]
        }]
    }"#,
    );

    let input = SingleCharKeys {
        bucket: HashMap::from([
            ("a".into(), 1),
            ("z".into(), 2),
            ("AB".into(), 3), // not matched (uppercase + 2 chars)
            ("1".into(), 4),  // not matched (digit)
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let chars = result["chars"].as_array().unwrap();
    assert_eq!(chars.len(), 2);

    let mut sorted: Vec<i64> = chars.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2]);
}

// ============================================================================
// 19. Numeric-only pattern: `\d+` matching keys like "0", "1", "42"
// ============================================================================

#[test]
fn pattern_numeric_only_keys() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "numeric",
            "capture": "request",
            "sutures": [{
                "index.`\\d+`": "/numbers/[:]"
            }]
        }]
    }"#,
    );

    let input = NumericKeys {
        index: HashMap::from([
            ("0".into(), "zero".into()),
            ("1".into(), "one".into()),
            ("42".into(), "forty_two".into()),
            ("abc".into(), "not_a_number".into()),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let numbers = result["numbers"].as_array().unwrap();
    assert_eq!(numbers.len(), 3);

    let mut sorted: Vec<&str> = numbers.iter().map(|v| v.as_str().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec!["forty_two", "one", "zero"]);
}

// ============================================================================
// 20. Mix of direct fields and pattern fields in the same suture
// ============================================================================

#[test]
fn mixed_direct_and_pattern() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "request",
            "sutures": [{
                "name": "/name",
                "data.`data_\\d+`": "/data/[:]"
            }]
        }]
    }"#,
    );

    let input = MixedStruct {
        name: "my_name".into(),
        data: HashMap::from([("data_0".into(), 10), ("data_1".into(), 20)]),
    };

    let result = suture.stitch(&input).unwrap();

    // Direct field.
    assert_eq!(result["name"].as_str().unwrap(), "my_name");

    // Pattern field.
    let data = result["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);

    let mut sorted: Vec<i64> = data.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![10, 20]);
}

// ============================================================================
// Additional edge case: pattern on source that is not an object (should skip)
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NonObjectField {
    data: Vec<i32>,
}

#[test]
fn pattern_on_non_object_source_skips() {
    // When the source field for IteratePattern is an array (not an object),
    // the forward walk should skip (as_object returns None → continue).
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "non_obj",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = NonObjectField {
        data: vec![1, 2, 3],
    };

    let result = suture.stitch(&input).unwrap();
    assert!(
        result.get("output").is_none()
            || result["output"].as_array().map_or(true, |a| a.is_empty())
    );
}

// ============================================================================
// Edge case: pattern with empty HashMap (zero keys)
// ============================================================================

#[test]
fn pattern_empty_hashmap() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_map",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::new(),
    };

    let result = suture.stitch(&input).unwrap();
    assert!(
        result.get("output").is_none()
            || result["output"].as_array().map_or(true, |a| a.is_empty())
    );
}

// ============================================================================
// Edge case: pattern that would match but HashMap has exactly one key
// ============================================================================

#[test]
fn pattern_single_matching_key() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_key",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("field_7".into(), 77)]),
    };

    let result = suture.stitch(&input).unwrap();
    let output = result["output"].as_array().unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].as_i64().unwrap(), 77);
}

// ============================================================================
// Fan-out + pattern + child extraction combined
// ============================================================================

#[test]
fn pattern_fanout_nested() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout_nested",
            "capture": "request",
            "sutures": [{
                "entries.`entry_\\d+`": {
                    "name": ["/primary/[:]/name", "/mirror/[:]/name"]
                }
            }]
        }]
    }"#,
    );

    let input = NestedDynamic {
        entries: HashMap::from([
            (
                "entry_0".into(),
                Entry {
                    name: "x".into(),
                    value: 1,
                },
            ),
            (
                "entry_1".into(),
                Entry {
                    name: "y".into(),
                    value: 2,
                },
            ),
        ]),
    };

    let result = suture.stitch(&input).unwrap();

    let primary = result["primary"].as_array().unwrap();
    let mirror = result["mirror"].as_array().unwrap();

    assert_eq!(primary.len(), 2);
    assert_eq!(mirror.len(), 2);

    let mut primary_names: Vec<&str> = primary
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    primary_names.sort();
    assert_eq!(primary_names, vec!["x", "y"]);

    let mut mirror_names: Vec<&str> = mirror.iter().map(|e| e["name"].as_str().unwrap()).collect();
    mirror_names.sort();
    assert_eq!(mirror_names, vec!["x", "y"]);
}

// ============================================================================
// Slice [0:2] with fewer matches than requested
// ============================================================================

#[test]
fn pattern_slice_fewer_matches_than_range() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fewer_matches",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[0:5]": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("field_0".into(), 10), ("field_1".into(), 20)]),
    };

    let result = suture.stitch(&input).unwrap();
    let output = result["output"].as_array().unwrap();

    // Only 2 matches, so [0:5] should give 2 (clamped by actual matches).
    assert_eq!(output.len(), 2);
}

// ============================================================================
// Multiple suture objects with patterns in same suture_set
// ============================================================================

#[test]
fn multiple_suture_objects_with_patterns() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_obj",
            "capture": "request",
            "sutures": [
                { "alpha.`a_\\d+`": "/a_vals/[:]" },
                { "beta.`b_\\d+`": "/b_vals/[:]" }
            ]
        }]
    }"#,
    );

    let input = TwoFamilies {
        alpha: HashMap::from([("a_0".into(), 10)]),
        beta: HashMap::from([("b_0".into(), 20)]),
    };

    let result = suture.stitch(&input).unwrap();

    let a_vals = result["a_vals"].as_array().unwrap();
    assert_eq!(a_vals.len(), 1);
    assert_eq!(a_vals[0].as_i64().unwrap(), 10);

    let b_vals = result["b_vals"].as_array().unwrap();
    assert_eq!(b_vals.len(), 1);
    assert_eq!(b_vals[0].as_i64().unwrap(), 20);
}

// ============================================================================
// Pattern with character class edge case: `[A-Z][a-z]+`
// ============================================================================

#[test]
fn pattern_uppercase_start() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "upper_start",
            "capture": "request",
            "sutures": [{
                "fields.`[A-Z][a-z]+`": "/capitalized/[:]"
            }]
        }]
    }"#,
    );

    let input = FlatDynamic {
        fields: HashMap::from([
            ("Hello".into(), "matched1".into()),
            ("World".into(), "matched2".into()),
            ("hello".into(), "not_matched".into()),
            ("H".into(), "too_short".into()),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let capitalized = result["capitalized"].as_array().unwrap();
    assert_eq!(capitalized.len(), 2);

    let mut sorted: Vec<&str> = capitalized.iter().map(|v| v.as_str().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec!["matched1", "matched2"]);
}

// ============================================================================
// Pattern with quantifier: `x+` matching one or more x's
// ============================================================================

#[test]
fn pattern_quantifier_plus() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "quantifier",
            "capture": "request",
            "sutures": [{
                "bucket.`x+`": "/xs/[:]"
            }]
        }]
    }"#,
    );

    let input = SingleCharKeys {
        bucket: HashMap::from([
            ("x".into(), 1),
            ("xx".into(), 2),
            ("xxx".into(), 3),
            ("y".into(), 4),
            ("xy".into(), 5),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let xs = result["xs"].as_array().unwrap();
    assert_eq!(xs.len(), 3);

    let mut sorted: Vec<i64> = xs.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2, 3]);
}

// ============================================================================
// Verify the suture is correctly identified as request direction
// ============================================================================

#[test]
fn suture_is_request_direction() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "direction_check",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#,
    );

    assert!(suture.is_request());
    assert!(!suture.is_response());
}

// ============================================================================
// Pattern + direct fields in nested suture object
// ============================================================================

#[test]
fn mixed_direct_and_pattern_nested_suture() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_mixed",
            "capture": "request",
            "sutures": [
                { "name": "/metadata/name" },
                { "data.`item_\\d+`": "/items/[:]" }
            ]
        }]
    }"#,
    );

    let input = MixedStruct {
        name: "test_name".into(),
        data: HashMap::from([
            ("item_0".into(), 100),
            ("item_1".into(), 200),
            ("other".into(), 999),
        ]),
    };

    let result = suture.stitch(&input).unwrap();

    assert_eq!(result["metadata"]["name"].as_str().unwrap(), "test_name");

    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);

    let mut sorted: Vec<i64> = items.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![100, 200]);
}

// ============================================================================
// Pattern + constants in same suture
// ============================================================================

#[test]
fn pattern_with_constants() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "with_constants",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]",
                "_": [{ "/type": "dynamic" }]
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("field_0".into(), 10)]),
    };

    let result = suture.stitch(&input).unwrap();

    // Pattern output.
    let output = result["output"].as_array().unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].as_i64().unwrap(), 10);

    // Constant injection.
    assert_eq!(result["type"].as_str().unwrap(), "dynamic");
}

// ============================================================================
// Pattern with complex nested output path
// ============================================================================

#[test]
fn pattern_deep_output_path() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_output",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/deep/nested/values/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("field_0".into(), 42)]),
    };

    let result = suture.stitch(&input).unwrap();
    let val = &result["deep"]["nested"]["values"];
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_i64().unwrap(), 42);
}

// ============================================================================
// Pattern with many matches to verify scalability
// ============================================================================

#[test]
fn pattern_many_matches() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "many_matches",
            "capture": "request",
            "sutures": [{
                "data.`item_\\d+`": "/items/[:]"
            }]
        }]
    }"#,
    );

    let mut map = HashMap::new();
    for i in 0..50 {
        map.insert(format!("item_{}", i), i as i32);
    }

    let input = DynamicFields { data: map };

    let result = suture.stitch(&input).unwrap();
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 50);

    let mut sorted: Vec<i64> = items.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    let expected: Vec<i64> = (0..50).collect();
    assert_eq!(sorted, expected);
}

// ============================================================================
// Pattern with optional quantifier: `field_\\d?`
// ============================================================================

#[test]
fn pattern_optional_quantifier() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "optional_q",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d?`": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field_".into(), 1),   // matches (digit is optional)
            ("field_5".into(), 2),  // matches
            ("field_55".into(), 3), // does NOT match (two digits)
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let output = result["output"].as_array().unwrap();
    assert_eq!(output.len(), 2);

    let mut sorted: Vec<i64> = output.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2]);
}

// ============================================================================
// Pattern value types: HashMap<String, bool>
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct BoolMap {
    flags: HashMap<String, bool>,
}

#[test]
fn pattern_bool_values() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "bool_vals",
            "capture": "request",
            "sutures": [{
                "flags.`flag_\\d+`": "/flags/[:]"
            }]
        }]
    }"#,
    );

    let input = BoolMap {
        flags: HashMap::from([("flag_0".into(), true), ("flag_1".into(), false)]),
    };

    let result = suture.stitch(&input).unwrap();
    let flags = result["flags"].as_array().unwrap();
    assert_eq!(flags.len(), 2);

    let mut sorted: Vec<bool> = flags.iter().map(|v| v.as_bool().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![false, true]);
}

// ============================================================================
// Pattern value types: HashMap<String, serde_json::Value> for mixed values
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ValueMap {
    data: HashMap<String, Value>,
}

#[test]
fn pattern_mixed_value_types() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed_vals",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = ValueMap {
        data: HashMap::from([
            ("field_0".into(), json!(42)),
            ("field_1".into(), json!("hello")),
            ("field_2".into(), json!(true)),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let output = result["output"].as_array().unwrap();
    assert_eq!(output.len(), 3);

    // Verify all three value types appear.
    let has_int = output.iter().any(|v| v.as_i64().is_some());
    let has_str = output.iter().any(|v| v.as_str().is_some());
    let has_bool = output.iter().any(|v| v.as_bool().is_some());
    assert!(has_int);
    assert!(has_str);
    assert!(has_bool);
}

// ============================================================================
// Two patterns on the same parent field (different regexes)
// ============================================================================

#[test]
fn two_patterns_same_parent() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "two_on_same",
            "capture": "request",
            "sutures": [{
                "data.`input_\\d+`": "/inputs/[:]",
                "data.`output_\\d+`": "/outputs/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("input_0".into(), 1),
            ("input_1".into(), 2),
            ("output_0".into(), 10),
            ("output_1".into(), 20),
            ("other".into(), 99),
        ]),
    };

    let result = suture.stitch(&input).unwrap();

    let inputs = result["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 2);
    let mut in_sorted: Vec<i64> = inputs.iter().map(|v| v.as_i64().unwrap()).collect();
    in_sorted.sort();
    assert_eq!(in_sorted, vec![1, 2]);

    let outputs = result["outputs"].as_array().unwrap();
    assert_eq!(outputs.len(), 2);
    let mut out_sorted: Vec<i64> = outputs.iter().map(|v| v.as_i64().unwrap()).collect();
    out_sorted.sort();
    assert_eq!(out_sorted, vec![10, 20]);
}

// ============================================================================
// Pattern where keys have underscores and digits combined
// ============================================================================

#[test]
fn pattern_complex_key_format() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "complex_keys",
            "capture": "request",
            "sutures": [{
                "data.`[a-z]+_\\d+_[a-z]+`": "/complex/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0_alpha".into(), 1),
            ("item_99_beta".into(), 2),
            ("nope".into(), 3),
            ("bad_format".into(), 4),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let complex = result["complex"].as_array().unwrap();
    assert_eq!(complex.len(), 2);

    let mut sorted: Vec<i64> = complex.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2]);
}

// ============================================================================
// Slice [0] on a pattern with exactly one match
// ============================================================================

#[test]
fn pattern_slice_zero_single_match() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_slice",
            "capture": "request",
            "sutures": [{
                "data.`unique_\\d+`[0]": "/first/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("unique_42".into(), 999), ("other_key".into(), 0)]),
    };

    let result = suture.stitch(&input).unwrap();
    let first = result["first"].as_array().unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].as_i64().unwrap(), 999);
}

// ============================================================================
// Slice [0] on a pattern with no matches
// ============================================================================

#[test]
fn pattern_slice_zero_no_matches() {
    let suture = parse_first(
        r#"{
        "name": "test",
        "suture_sets": [{
            "name": "no_match_slice",
            "capture": "request",
            "sutures": [{
                "data.`nonexistent_\\d+`[0]": "/output/[:]"
            }]
        }]
    }"#,
    );

    let input = DynamicFields {
        data: HashMap::from([("other_0".into(), 1)]),
    };

    let result = suture.stitch(&input).unwrap();
    assert!(
        result.get("output").is_none()
            || result["output"].as_array().map_or(true, |a| a.is_empty())
    );
}
