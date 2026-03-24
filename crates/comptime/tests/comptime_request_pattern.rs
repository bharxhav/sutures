//! Exhaustive integration tests for **compile-time** `parse!()` + `Stitch` with
//! **Request direction + IteratePattern binding**.
//!
//! Every test uses `sutures_comptime::parse!()` to build sutures at compile time,
//! constructs a typed struct, calls `suture.stitch(&input)`, and asserts the
//! output matches the expected JSON.
//!
//! IteratePattern uses backtick-delimited regex on keys (struct/LHS). The source
//! value must be a JSON object (use `HashMap<String, T>` in the struct). Regex is
//! NOT allowed on the value/RHS side.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Seam, Stitch};

// ============================================================================
// Test structs
// ============================================================================

/// Struct with a single HashMap field for dynamic keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DynamicFields {
    data: HashMap<String, i32>,
}

/// Struct with a top-level HashMap behind a named field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FlatDynamic {
    fields: HashMap<String, String>,
}

/// Struct with both a fixed field and a dynamic field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct MixedStruct {
    name: String,
    data: HashMap<String, i32>,
}

/// Struct with two separate dynamic-key families.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TwoFamilies {
    alpha: HashMap<String, i32>,
    beta: HashMap<String, i32>,
}

/// Struct with nested objects as dynamic values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NestedDynamic {
    entries: HashMap<String, Entry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Entry {
    name: String,
    value: i32,
}

/// Struct for items mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TopLevelMap {
    items: HashMap<String, String>,
}

/// Struct for single-char keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SingleCharKeys {
    bucket: HashMap<String, i32>,
}

/// Struct for numeric-only keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NumericKeys {
    index: HashMap<String, String>,
}

/// Struct whose dynamic field is not an object (Vec instead of HashMap).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NonObjectField {
    data: Vec<i32>,
}

// ============================================================================
// 1. `field_\d+` matching field_0, field_1, field_2 -> /values/[:]
// ============================================================================

#[test]
fn basic_pattern_field_digits() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "basic_pattern",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/values/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// 2. `item_[a-z]+` matching item_foo, item_bar -> /items/[:]
// ============================================================================

#[test]
fn basic_pattern_item_alpha() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "alpha_pattern",
            "capture": "request",
            "sutures": [{
                "items.`item_[a-z]+`": "/items/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// 3. Pattern with no matches -> empty output
// ============================================================================

#[test]
fn pattern_no_matches_empty_output() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "no_match",
            "capture": "request",
            "sutures": [{
                "data.`content_\\d+`": "/output/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = DynamicFields {
        data: HashMap::from([
            ("other_0".into(), 1),
            ("something_else".into(), 2),
        ]),
    };

    let result = suture.stitch(&input).unwrap();

    // No keys match `content_\d+`, so `/output` should not be populated.
    assert!(
        result.get("output").is_none()
            || result["output"].as_array().map_or(true, |a| a.is_empty())
    );
}

// ============================================================================
// 4. Pattern matching only some keys (non-matching ignored)
// ============================================================================

#[test]
fn pattern_partial_match_ignores_non_matching() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "partial",
            "capture": "request",
            "sutures": [{
                "data.`data_\\d+`": "/extracted/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// 5. Non-capturing group: `(?:input|output)_\d+`
// ============================================================================

#[test]
fn pattern_non_capturing_group() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "noncap",
            "capture": "request",
            "sutures": [{
                "data.`(?:input|output)_\\d+`": "/signals/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// 6. Pattern with slice `[0]` -- first match only
// ============================================================================

#[test]
fn pattern_slice_first_only() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "first_only",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[0]": "/first/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// 7. Pattern with slice `[0:2]` -- first two matches
// ============================================================================

#[test]
fn pattern_slice_first_two() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "first_two",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[0:2]": "/two/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// 8. Pattern with child extraction: nested object under pattern
// ============================================================================

#[test]
fn pattern_child_extraction() {
    let sutures = sutures_comptime::parse!(r#"{
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
    }"#);

    let suture = &sutures[0];

    let input = NestedDynamic {
        entries: HashMap::from([
            ("entry_0".into(), Entry { name: "alpha".into(), value: 10 }),
            ("entry_1".into(), Entry { name: "beta".into(), value: 20 }),
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
// 9. Fan-out from pattern: single pattern to multiple targets
// ============================================================================

#[test]
fn pattern_fan_out() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fanout",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": ["/primary/[:]", "/backup/[:]"]
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 10),
            ("field_1".into(), 20),
        ]),
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
// 10. Multiple patterns in same suture (two key families)
// ============================================================================

#[test]
fn multiple_patterns_two_families() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "multi_pattern",
            "capture": "request",
            "sutures": [{
                "alpha.`a_\\d+`": "/group_a/[:]",
                "beta.`b_\\d+`": "/group_b/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = TwoFamilies {
        alpha: HashMap::from([
            ("a_0".into(), 1),
            ("a_1".into(), 2),
        ]),
        beta: HashMap::from([
            ("b_0".into(), 100),
            ("b_1".into(), 200),
        ]),
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
// 11. Mix of direct and pattern fields
// ============================================================================

#[test]
fn mixed_direct_and_pattern() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "request",
            "sutures": [{
                "name": "/name",
                "data.`data_\\d+`": "/data/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = MixedStruct {
        name: "my_name".into(),
        data: HashMap::from([
            ("data_0".into(), 10),
            ("data_1".into(), 20),
        ]),
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
// 12. Pattern matching ALL keys in the object
// ============================================================================

#[test]
fn pattern_matches_all_keys() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "match_all",
            "capture": "request",
            "sutures": [{
                "data.`.*`": "/all/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// 13. Single-char key pattern `[a-z]`
// ============================================================================

#[test]
fn pattern_single_char_keys() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_char",
            "capture": "request",
            "sutures": [{
                "bucket.`[a-z]`": "/chars/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = SingleCharKeys {
        bucket: HashMap::from([
            ("a".into(), 1),
            ("z".into(), 2),
            ("AB".into(), 3),     // not matched (uppercase + 2 chars)
            ("1".into(), 4),      // not matched (digit)
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
// 14. Numeric-only keys `\d+`
// ============================================================================

#[test]
fn pattern_numeric_only_keys() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "numeric",
            "capture": "request",
            "sutures": [{
                "index.`\\d+`": "/numbers/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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
// Edge cases
// ============================================================================

// -- Pattern is auto-anchored (^...$), so partial matches are rejected -------

#[test]
fn pattern_auto_anchored() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "anchored",
            "capture": "request",
            "sutures": [{
                "data.`field`": "/exact/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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

// -- Pattern on source that is not an object (Vec) -> should skip ------------

#[test]
fn pattern_on_non_object_source_skips() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "non_obj",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = NonObjectField {
        data: vec![1, 2, 3],
    };

    let result = suture.stitch(&input).unwrap();
    assert!(
        result.get("output").is_none()
            || result["output"].as_array().map_or(true, |a| a.is_empty())
    );
}

// -- Empty HashMap -> no matches, no output ----------------------------------

#[test]
fn pattern_empty_hashmap() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_map",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = DynamicFields {
        data: HashMap::new(),
    };

    let result = suture.stitch(&input).unwrap();
    assert!(
        result.get("output").is_none()
            || result["output"].as_array().map_or(true, |a| a.is_empty())
    );
}

// -- Exactly one matching key ------------------------------------------------

#[test]
fn pattern_single_matching_key() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_key",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/output/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = DynamicFields {
        data: HashMap::from([
            ("field_99".into(), 77),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let output = result["output"].as_array().unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].as_i64().unwrap(), 77);
}

// -- Pattern with alternation: `(?:alpha|beta)_\d+` --------------------------

#[test]
fn pattern_alternation() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "alternation",
            "capture": "request",
            "sutures": [{
                "data.`(?:alpha|beta)_\\d+`": "/matched/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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

// -- Pattern with escaped dot: `field\.name` matching literal dot ------------

#[test]
fn pattern_escaped_dot() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "escaped_dot",
            "capture": "request",
            "sutures": [{
                "fields.`field\\.name`": "/dotted/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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

// -- Slice qualifier `[1:]` -- skip first match ------------------------------

#[test]
fn pattern_slice_skip_first() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "skip_first",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[1:]": "/rest/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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

// -- Slice qualifier `[::-1]` -- reverse order of matches --------------------

#[test]
fn pattern_slice_reverse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "reverse",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[::-1]": "/reversed/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

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

    // All three values present (HashMap order is nondeterministic).
    let mut sorted: Vec<i64> = reversed.iter().map(|v| v.as_i64().unwrap()).collect();
    sorted.sort();
    assert_eq!(sorted, vec![10, 20, 30]);
}

// -- Multiple suture sets compiled from the same parse! call -----------------

#[test]
fn multiple_suture_sets_from_single_parse() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [
            {
                "name": "first",
                "capture": "request",
                "sutures": [{
                    "data.`field_\\d+`": "/first_out/[:]"
                }]
            },
            {
                "name": "second",
                "capture": "request",
                "sutures": [{
                    "data.`field_\\d+`": "/second_out/[:]"
                }]
            }
        ]
    }"#);

    assert_eq!(sutures.len(), 2);

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 1),
            ("field_1".into(), 2),
        ]),
    };

    let r1 = sutures[0].stitch(&input).unwrap();
    let r2 = sutures[1].stitch(&input).unwrap();

    let mut s1: Vec<i64> = r1["first_out"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
    let mut s2: Vec<i64> = r2["second_out"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
    s1.sort();
    s2.sort();
    assert_eq!(s1, vec![1, 2]);
    assert_eq!(s2, vec![1, 2]);
}

// -- Pattern with deeply nested target path ----------------------------------

#[test]
fn pattern_deep_target_path() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/a/b/c/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 42),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let arr = result["a"]["b"]["c"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_i64().unwrap(), 42);
}

// -- Pattern slice [0] on HashMap with only one matching key -----------------

#[test]
fn pattern_slice_zero_single_match() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "one_match_slice",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`[0]": "/only/[:]"
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 55),
            ("other".into(), 99),
        ]),
    };

    let result = suture.stitch(&input).unwrap();
    let only = result["only"].as_array().unwrap();
    assert_eq!(only.len(), 1);
    assert_eq!(only[0].as_i64().unwrap(), 55);
}

// -- Pattern with child extraction and fan-out combined ----------------------

#[test]
fn pattern_child_extraction_with_fan_out() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "child_fanout",
            "capture": "request",
            "sutures": [{
                "entries.`entry_\\d+`": {
                    "name": ["/names/[:]", "/all_names/[:]"]
                }
            }]
        }]
    }"#);

    let suture = &sutures[0];

    let input = NestedDynamic {
        entries: HashMap::from([
            ("entry_0".into(), Entry { name: "alpha".into(), value: 10 }),
            ("entry_1".into(), Entry { name: "beta".into(), value: 20 }),
        ]),
    };

    let result = suture.stitch(&input).unwrap();

    let names = result["names"].as_array().unwrap();
    let all_names = result["all_names"].as_array().unwrap();
    assert_eq!(names.len(), 2);
    assert_eq!(all_names.len(), 2);

    let mut sorted_names: Vec<&str> = names.iter().map(|v| v.as_str().unwrap()).collect();
    sorted_names.sort();
    assert_eq!(sorted_names, vec!["alpha", "beta"]);

    let mut sorted_all: Vec<&str> = all_names.iter().map(|v| v.as_str().unwrap()).collect();
    sorted_all.sort();
    assert_eq!(sorted_all, vec!["alpha", "beta"]);
}

// -- Verify the suture metadata is set correctly ----------------------------

#[test]
fn comptime_suture_has_correct_metadata() {
    let sutures = sutures_comptime::parse!(r#"{
        "name": "test_manifest",
        "suture_sets": [{
            "name": "pattern_set",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/values/[:]"
            }]
        }]
    }"#);

    assert_eq!(sutures.len(), 1);
    assert_eq!(sutures[0].name(), "pattern_set");
    assert!(sutures[0].is_request());
    assert!(!sutures[0].is_response());
}

// -- Comptime produces identical output to runtime parse ---------------------

#[test]
fn comptime_matches_runtime_output() {
    let json = r#"{
        "name": "test",
        "suture_sets": [{
            "name": "compare",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/values/[:]"
            }]
        }]
    }"#;

    let comptime_sutures = sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "compare",
            "capture": "request",
            "sutures": [{
                "data.`field_\\d+`": "/values/[:]"
            }]
        }]
    }"#);

    let runtime_sutures = sutures::v1::parse(json).unwrap();
    let runtime_suture = runtime_sutures.into_iter().next().unwrap().unwrap();
    let comptime_suture = &comptime_sutures[0];

    let input = DynamicFields {
        data: HashMap::from([
            ("field_0".into(), 10),
            ("field_1".into(), 20),
        ]),
    };

    let comptime_result = comptime_suture.stitch(&input).unwrap();
    let runtime_result = runtime_suture.stitch(&input).unwrap();

    // Both should produce the same values array (order may vary due to HashMap).
    let mut comptime_vals: Vec<i64> = comptime_result["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    let mut runtime_vals: Vec<i64> = runtime_result["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    comptime_vals.sort();
    runtime_vals.sort();
    assert_eq!(comptime_vals, runtime_vals);
}
