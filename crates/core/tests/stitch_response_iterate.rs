// Exhaustive integration tests for Response direction + Iterate bindings.
//
// Response sutures: keys are JSON terminals (slash-separated, start with `/`),
// values are struct terminals (dot-separated, start with letter).
//
// `unstitch` is the natural/forward direction (JSON -> struct).
// `stitch` is the reverse direction (struct -> JSON).

use serde_json::json;
use sutures::Stitch;

// ============================================================================
// Helpers
// ============================================================================

fn parse_first(json: &str) -> sutures::v1::Suture {
    sutures::v1::parse(json)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .unwrap()
}

/// Wrap a single response suture mapping object in a full `.sutures.json` envelope.
fn response_suture(mapping: serde_json::Value) -> String {
    json!({
        "name": "test",
        "suture_sets": [{
            "name": "test_set",
            "capture": "response",
            "sutures": [mapping]
        }]
    })
    .to_string()
}

// ============================================================================
// Test structs
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ItemsVec {
    #[serde(default)]
    items: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NamesVec {
    names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ItemWithName {
    name: String,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ItemsWithFields {
    items: Vec<ItemWithName>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FirstItem {
    first: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct LastItem {
    last: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct FirstName {
    first_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SliceVec {
    #[serde(default)]
    slice: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct EvensVec {
    evens: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ReversedVec {
    #[serde(default)]
    reversed: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SelectedVec {
    selected: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NamesAndLabels {
    names: Vec<String>,
    labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ValuesNested {
    values: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MatrixValues {
    cells: Vec<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SliceNames {
    names: Vec<String>,
}

// ============================================================================
// 1. /items/[:] -> items[:] — copy full array
// ============================================================================

#[test]
fn unstitch_full_array_copy() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let input = json!({
        "items": [1, 2, 3]
    });

    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items, vec![json!(1), json!(2), json!(3)]);
}

// ============================================================================
// 2. /items/[:]/name -> names[:] — extract field from each JSON array element
// ============================================================================

#[test]
fn unstitch_extract_field_from_array_elements() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": "names[:]"
    })));

    let input = json!({
        "items": [
            {"name": "alice", "age": 30},
            {"name": "bob", "age": 25},
            {"name": "charlie", "age": 35}
        ]
    });

    let result: NamesVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.names, vec!["alice", "bob", "charlie"]);
}

// ============================================================================
// 3. /data/[:]/name -> items[:].name — JSON array field -> struct array field
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NameOnly {
    name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DataItems {
    items: Vec<NameOnly>,
}

#[test]
fn unstitch_json_array_field_to_struct_array_field() {
    let suture = parse_first(&response_suture(json!({
        "/data/[:]/name": "items[:].name"
    })));

    let input = json!({
        "data": [
            {"name": "first", "extra": true},
            {"name": "second", "extra": false}
        ]
    });

    let result: DataItems = suture.unstitch(&input).unwrap();
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.items[0].name, "first");
    assert_eq!(result.items[1].name, "second");
}

// ============================================================================
// 4. /items/[0] -> first — single index extraction
// ============================================================================

#[test]
fn unstitch_single_index_extraction() {
    let suture = parse_first(&response_suture(json!({
        "/items/[0]": "first"
    })));

    let input = json!({
        "items": ["alpha", "beta", "gamma"]
    });

    let result: FirstItem = suture.unstitch(&input).unwrap();
    assert_eq!(result.first, json!("alpha"));
}

// ============================================================================
// 5. /items/[-1] -> last — last element
// ============================================================================

#[test]
fn unstitch_last_element() {
    let suture = parse_first(&response_suture(json!({
        "/items/[-1]": "last"
    })));

    let input = json!({
        "items": ["alpha", "beta", "gamma"]
    });

    let result: LastItem = suture.unstitch(&input).unwrap();
    assert_eq!(result.last, json!("gamma"));
}

// ============================================================================
// 6. /items/[0]/name -> first_name — field from single index
// ============================================================================

#[test]
fn unstitch_field_from_single_index() {
    let suture = parse_first(&response_suture(json!({
        "/items/[0]/name": "first_name"
    })));

    let input = json!({
        "items": [
            {"name": "alice", "role": "admin"},
            {"name": "bob", "role": "user"}
        ]
    });

    let result: FirstName = suture.unstitch(&input).unwrap();
    assert_eq!(result.first_name, "alice");
}

// ============================================================================
// 7. /items/[1:3] -> slice[:] — slice extraction
// ============================================================================

#[test]
fn unstitch_slice_extraction() {
    let suture = parse_first(&response_suture(json!({
        "/items/[1:3]": "slice[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c", "d", "e"]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!("b"), json!("c")]);
}

// ============================================================================
// 8. /items/[1:3]/name -> names[:] — field from sliced elements
// ============================================================================

#[test]
fn unstitch_field_from_sliced_elements() {
    let suture = parse_first(&response_suture(json!({
        "/items/[1:3]/name": "names[:]"
    })));

    let input = json!({
        "items": [
            {"name": "zero"},
            {"name": "one"},
            {"name": "two"},
            {"name": "three"},
            {"name": "four"}
        ]
    });

    let result: NamesVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.names, vec!["one", "two"]);
}

// ============================================================================
// 9. /items/[::2] -> evens[:] — step iteration
// ============================================================================

#[test]
fn unstitch_step_iteration() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::2]": "evens[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c", "d", "e"]
    });

    let result: EvensVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.evens, vec![json!("a"), json!("c"), json!("e")]);
}

// ============================================================================
// 10. /items/[::-1] -> reversed[:] — reverse
// ============================================================================

#[test]
fn unstitch_reverse_iteration() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::-1]": "reversed[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c"]
    });

    let result: ReversedVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.reversed, vec![json!("c"), json!("b"), json!("a")]);
}

// ============================================================================
// 11. /items/[1:5:2] -> selected[:] — range+step
// ============================================================================

#[test]
fn unstitch_range_with_step() {
    let suture = parse_first(&response_suture(json!({
        "/items/[1:5:2]": "selected[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c", "d", "e", "f"]
    });

    let result: SelectedVec = suture.unstitch(&input).unwrap();
    // indices 1, 3
    assert_eq!(result.selected, vec![json!("b"), json!("d")]);
}

// ============================================================================
// 12. Nested iteration: /groups/[:]/items/[:] -> values[:].[:]
// ============================================================================

#[test]
fn unstitch_doubly_nested_arrays() {
    let suture = parse_first(&response_suture(json!({
        "/groups/[:]/items/[:]": "values[:].[:]"
    })));

    let input = json!({
        "groups": [
            {"items": [1, 2]},
            {"items": [3, 4, 5]}
        ]
    });

    let result: ValuesNested = suture.unstitch(&input).unwrap();
    assert_eq!(result.values.len(), 2);
    assert_eq!(result.values[0], vec![json!(1), json!(2)]);
    assert_eq!(result.values[1], vec![json!(3), json!(4), json!(5)]);
}

// ============================================================================
// 13. /matrix/[:]/row/[:]/value -> nested struct extraction
// ============================================================================

#[test]
fn unstitch_nested_field_extraction() {
    let suture = parse_first(&response_suture(json!({
        "/matrix/[:]/row/[:]/value": "cells[:].[:]"
    })));

    let input = json!({
        "matrix": [
            {"row": [{"value": 1}, {"value": 2}]},
            {"row": [{"value": 3}, {"value": 4}, {"value": 5}]}
        ]
    });

    let result: MatrixValues = suture.unstitch(&input).unwrap();
    assert_eq!(result.cells.len(), 2);
    assert_eq!(result.cells[0], vec![1, 2]);
    assert_eq!(result.cells[1], vec![3, 4, 5]);
}

// ============================================================================
// 14. Nested object syntax with iteration (unstitch)
//     /items/[:]: { "/name": "items[:].name", "/value": "items[:].value" }
// ============================================================================

#[test]
fn unstitch_nested_object_extract_multiple_fields() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": {
            "/name": "items[:].name",
            "/value": "items[:].value"
        }
    })));

    let input = json!({
        "items": [
            {"name": "apple", "value": 10},
            {"name": "banana", "value": 20}
        ]
    });

    let result: ItemsWithFields = suture.unstitch(&input).unwrap();
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.items[0].name, "apple");
    assert_eq!(result.items[0].value, 10);
    assert_eq!(result.items[1].name, "banana");
    assert_eq!(result.items[1].value, 20);
}

// ============================================================================
// 15. Fan-out: /items/[:]/name -> ["names[:]", "labels[:]"]
// ============================================================================

#[test]
fn unstitch_fan_out_to_multiple_arrays() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": ["names[:]", "labels[:]"]
    })));

    let input = json!({
        "items": [
            {"name": "tag1"},
            {"name": "tag2"},
            {"name": "tag3"}
        ]
    });

    let result: NamesAndLabels = suture.unstitch(&input).unwrap();
    assert_eq!(result.names, vec!["tag1", "tag2", "tag3"]);
    assert_eq!(result.labels, vec!["tag1", "tag2", "tag3"]);
}

// ============================================================================
// 16. Empty JSON array with [:] -> empty struct array
// ============================================================================

#[test]
fn unstitch_empty_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let input = json!({
        "items": []
    });

    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert!(result.items.is_empty());
}

// ============================================================================
// 17. Single-element JSON array
// ============================================================================

#[test]
fn unstitch_single_element_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let input = json!({
        "items": [42]
    });

    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items, vec![json!(42)]);
}

// ============================================================================
// 18. Out-of-range slice -> empty
// ============================================================================

#[test]
fn unstitch_out_of_range_slice_yields_empty() {
    let suture = parse_first(&response_suture(json!({
        "/items/[10:20]": "slice[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c"]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert!(result.slice.is_empty());
}

// ============================================================================
// 19. Negative indices
// ============================================================================

#[test]
fn unstitch_negative_index_minus_one() {
    let suture = parse_first(&response_suture(json!({
        "/items/[-1]": "last"
    })));

    let input = json!({
        "items": [10, 20, 30, 40, 50]
    });

    let result: LastItem = suture.unstitch(&input).unwrap();
    assert_eq!(result.last, json!(50));
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SecondToLast {
    item: serde_json::Value,
}

#[test]
fn unstitch_negative_index_minus_two() {
    let suture = parse_first(&response_suture(json!({
        "/items/[-2]": "item"
    })));

    let input = json!({
        "items": [10, 20, 30, 40, 50]
    });

    let result: SecondToLast = suture.unstitch(&input).unwrap();
    assert_eq!(result.item, json!(40));
}

// ============================================================================
// 20. [::3] on 10-element array
// ============================================================================

#[test]
fn unstitch_step_three_on_ten_elements() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::3]": "evens[:]"
    })));

    let input = json!({
        "items": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    });

    let result: EvensVec = suture.unstitch(&input).unwrap();
    // indices 0, 3, 6, 9
    assert_eq!(result.evens, vec![json!(0), json!(3), json!(6), json!(9)]);
}

// ============================================================================
// 21. [::-2] on 6-element array
// ============================================================================

#[test]
fn unstitch_reverse_step_two_on_six_elements() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::-2]": "reversed[:]"
    })));

    let input = json!({
        "items": [0, 1, 2, 3, 4, 5]
    });

    let result: ReversedVec = suture.unstitch(&input).unwrap();
    // Python: [0,1,2,3,4,5][::-2] = [5, 3, 1]
    assert_eq!(result.reversed, vec![json!(5), json!(3), json!(1)]);
}

// ============================================================================
// 22. stitch — Basic array stitch reverse (struct -> JSON)
// ============================================================================

#[test]
fn stitch_basic_array_reverse() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let input = ItemsVec {
        items: vec![json!(1), json!(2), json!(3)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [1, 2, 3] }));
}

// ============================================================================
// 23. stitch — Nested field stitch reverse
// ============================================================================

#[test]
fn stitch_nested_field_reverse() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": "names[:]"
    })));

    let input = NamesVec {
        names: vec!["alice".to_string(), "bob".to_string()],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "items": [
                {"name": "alice"},
                {"name": "bob"}
            ]
        })
    );
}

// ============================================================================
// 24. stitch — Single index stitch reverse
// ============================================================================

#[test]
fn stitch_single_index_reverse() {
    let suture = parse_first(&response_suture(json!({
        "/items/[0]": "first"
    })));

    let input = FirstItem {
        first: json!("alpha"),
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": ["alpha"] }));
}

// ============================================================================
// 25. stitch — Slice stitch reverse
// ============================================================================

#[test]
fn stitch_slice_reverse() {
    let suture = parse_first(&response_suture(json!({
        "/items/[1:3]": "slice[:]"
    })));

    let input = SliceVec {
        slice: vec![json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    // The reverse walk reads struct-side slice[:] which yields all elements
    // at enumeration indices [0, 1], producing a 2-element JSON array.
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], json!("b"));
    assert_eq!(items[1], json!("c"));
}

// ============================================================================
// 26. Round-trip: unstitch then stitch = identity for array fields
// ============================================================================

#[test]
fn roundtrip_unstitch_then_stitch_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let original_json = json!({
        "items": [1, 2, 3, 4, 5]
    });

    let intermediate: ItemsVec = suture.unstitch(&original_json).unwrap();
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original_json);
}

// ============================================================================
// 27. Round-trip: stitch then unstitch = identity for array fields
// ============================================================================

#[test]
fn roundtrip_stitch_then_unstitch_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let original = ItemsVec {
        items: vec![json!("x"), json!("y"), json!("z")],
    };

    let json_repr = suture.stitch(&original).unwrap();
    let recovered: ItemsVec = suture.unstitch(&json_repr).unwrap();
    assert_eq!(recovered, original);
}

// ============================================================================
// 28. Round-trip with nested iteration
// ============================================================================

#[test]
fn roundtrip_nested_iteration() {
    let suture = parse_first(&response_suture(json!({
        "/groups/[:]/items/[:]": "values[:].[:]"
    })));

    let original_json = json!({
        "groups": [
            {"items": [10, 20]},
            {"items": [30]}
        ]
    });

    let intermediate: ValuesNested = suture.unstitch(&original_json).unwrap();
    assert_eq!(intermediate.values.len(), 2);
    assert_eq!(intermediate.values[0], vec![json!(10), json!(20)]);
    assert_eq!(intermediate.values[1], vec![json!(30)]);

    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original_json);
}

// ============================================================================
// 29. Round-trip with slicing
// ============================================================================

#[test]
fn roundtrip_with_field_extraction() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": "names[:]"
    })));

    let original = NamesVec {
        names: vec!["hello".to_string(), "world".to_string()],
    };

    let json_repr = suture.stitch(&original).unwrap();
    let recovered: NamesVec = suture.unstitch(&json_repr).unwrap();
    assert_eq!(recovered, original);
}

// ============================================================================
// Additional edge cases
// ============================================================================

// -- Multiple fields from nested object syntax round-trip --

#[test]
fn roundtrip_nested_object_multiple_fields() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": {
            "/name": "items[:].name",
            "/value": "items[:].value"
        }
    })));

    let original = ItemsWithFields {
        items: vec![
            ItemWithName {
                name: "foo".to_string(),
                value: 100,
            },
            ItemWithName {
                name: "bar".to_string(),
                value: 200,
            },
        ],
    };

    let json_repr = suture.stitch(&original).unwrap();
    let recovered: ItemsWithFields = suture.unstitch(&json_repr).unwrap();
    assert_eq!(recovered, original);
}

// -- Verify stitch with empty struct array produces empty JSON array --

#[test]
fn stitch_empty_struct_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let input = ItemsVec { items: vec![] };
    let result = suture.stitch(&input).unwrap();
    // Empty array reads produce no iterations, so `items` key may not appear
    // or may be an empty array. Either is acceptable.
    if let Some(arr) = result.get("items") {
        assert_eq!(arr.as_array().unwrap().len(), 0);
    }
}

// -- Fan-out round-trip: both target arrays get populated --

#[test]
fn unstitch_fan_out_preserves_all_elements() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": ["names[:]", "labels[:]"]
    })));

    let input = json!({
        "items": [
            {"name": "a"},
            {"name": "b"}
        ]
    });

    let result: NamesAndLabels = suture.unstitch(&input).unwrap();
    assert_eq!(result.names, result.labels);
    assert_eq!(result.names, vec!["a", "b"]);
}

// -- Step iteration with field extraction --

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct StepNames {
    names: Vec<String>,
}

#[test]
fn unstitch_step_iteration_with_field_extraction() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::2]/name": "names[:]"
    })));

    let input = json!({
        "items": [
            {"name": "zero"},
            {"name": "one"},
            {"name": "two"},
            {"name": "three"},
            {"name": "four"}
        ]
    });

    let result: StepNames = suture.unstitch(&input).unwrap();
    // indices 0, 2, 4
    assert_eq!(result.names, vec!["zero", "two", "four"]);
}

// -- Reverse with field extraction --

#[test]
fn unstitch_reverse_with_field_extraction() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::-1]/name": "names[:]"
    })));

    let input = json!({
        "items": [
            {"name": "first"},
            {"name": "second"},
            {"name": "third"}
        ]
    });

    let result: NamesVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.names, vec!["third", "second", "first"]);
}

// -- Verify the suture is indeed a Response binding --

#[test]
fn parsed_suture_is_response() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    assert!(suture.is_response());
    assert!(!suture.is_request());
}

// -- [0:] selects all elements (same as [:]) --

#[test]
fn unstitch_zero_start_open_end() {
    let suture = parse_first(&response_suture(json!({
        "/items/[0:]": "items[:]"
    })));

    let input = json!({
        "items": [10, 20, 30]
    });

    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items, vec![json!(10), json!(20), json!(30)]);
}

// -- [:2] selects first two elements --

#[test]
fn unstitch_open_start_fixed_end() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:2]": "slice[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c", "d"]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!("a"), json!("b")]);
}

// -- Negative start in slice: [-3:] selects last 3 --

#[test]
fn unstitch_negative_start_slice() {
    let suture = parse_first(&response_suture(json!({
        "/items/[-3:]": "slice[:]"
    })));

    let input = json!({
        "items": [1, 2, 3, 4, 5]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!(3), json!(4), json!(5)]);
}

// -- Stitch with nested field creates proper JSON structure --

#[test]
fn stitch_struct_array_field_to_json_array_objects() {
    let suture = parse_first(&response_suture(json!({
        "/data/[:]/name": "items[:].name"
    })));

    let input = DataItems {
        items: vec![
            NameOnly {
                name: "x".to_string(),
            },
            NameOnly {
                name: "y".to_string(),
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "data": [
                {"name": "x"},
                {"name": "y"}
            ]
        })
    );
}

// -- Roundtrip struct array with nested fields --

#[test]
fn roundtrip_struct_array_nested_field() {
    let suture = parse_first(&response_suture(json!({
        "/data/[:]/name": "items[:].name"
    })));

    let original_json = json!({
        "data": [
            {"name": "hello"},
            {"name": "world"}
        ]
    });

    let intermediate: DataItems = suture.unstitch(&original_json).unwrap();
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original_json);
}

// -- Large array with step --

#[test]
fn unstitch_large_array_step_five() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::5]": "selected[:]"
    })));

    let input = json!({
        "items": (0..20).collect::<Vec<i32>>()
    });

    let result: SelectedVec = suture.unstitch(&input).unwrap();
    // indices 0, 5, 10, 15
    assert_eq!(
        result.selected,
        vec![json!(0), json!(5), json!(10), json!(15)]
    );
}

// -- Slice past end of array clamps to length --

#[test]
fn unstitch_slice_past_end_clamps() {
    let suture = parse_first(&response_suture(json!({
        "/items/[2:100]": "slice[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c", "d"]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!("c"), json!("d")]);
}

// -- Stitch fan-out: reading from one struct array populates JSON --

#[test]
fn stitch_fan_out_reads_first_target() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": ["names[:]", "labels[:]"]
    })));

    let input = NamesAndLabels {
        names: vec!["x".to_string(), "y".to_string()],
        labels: vec!["x".to_string(), "y".to_string()],
    };

    let result = suture.stitch(&input).unwrap();
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["name"], json!("x"));
    assert_eq!(items[1]["name"], json!("y"));
}

// -- Single element array with field extraction --

#[test]
fn unstitch_single_element_field_extraction() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": "names[:]"
    })));

    let input = json!({
        "items": [{"name": "only"}]
    });

    let result: NamesVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.names, vec!["only"]);
}

// -- Deeply nested path with iteration --

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct DeepValues {
    values: Vec<String>,
}

#[test]
fn unstitch_deep_nested_path_with_iteration() {
    let suture = parse_first(&response_suture(json!({
        "/response/data/items/[:]/info/label": "values[:]"
    })));

    let input = json!({
        "response": {
            "data": {
                "items": [
                    {"info": {"label": "a"}},
                    {"info": {"label": "b"}}
                ]
            }
        }
    });

    let result: DeepValues = suture.unstitch(&input).unwrap();
    assert_eq!(result.values, vec!["a", "b"]);
}

// -- Reverse iteration on 1-element array --

#[test]
fn unstitch_reverse_single_element() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::-1]": "reversed[:]"
    })));

    let input = json!({
        "items": [42]
    });

    let result: ReversedVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.reversed, vec![json!(42)]);
}

// -- Reverse iteration on empty array --

#[test]
fn unstitch_reverse_empty_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::-1]": "reversed[:]"
    })));

    let input = json!({
        "items": []
    });

    let result: ReversedVec = suture.unstitch(&input).unwrap();
    assert!(result.reversed.is_empty());
}

// -- [1:1] yields empty (start == end) --

#[test]
fn unstitch_equal_start_end_yields_empty() {
    let suture = parse_first(&response_suture(json!({
        "/items/[1:1]": "slice[:]"
    })));

    let input = json!({
        "items": ["a", "b", "c"]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert!(result.slice.is_empty());
}

// -- Stitch with empty input produces minimal or empty output --

#[test]
fn stitch_empty_names_produces_empty_items() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]/name": "names[:]"
    })));

    let input = NamesVec { names: vec![] };
    let result = suture.stitch(&input).unwrap();
    if let Some(items) = result.get("items") {
        assert!(items.as_array().unwrap().is_empty());
    }
}

// -- Mixed object value types in array --

#[test]
fn unstitch_mixed_value_types_in_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let input = json!({
        "items": [1, "two", true, null, {"nested": "obj"}, [1, 2]]
    });

    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items.len(), 6);
    assert_eq!(result.items[0], json!(1));
    assert_eq!(result.items[1], json!("two"));
    assert_eq!(result.items[2], json!(true));
    assert_eq!(result.items[3], json!(null));
    assert_eq!(result.items[4], json!({"nested": "obj"}));
    assert_eq!(result.items[5], json!([1, 2]));
}

// -- Roundtrip with mixed types preserves all --

#[test]
fn roundtrip_mixed_types_array() {
    let suture = parse_first(&response_suture(json!({
        "/items/[:]": "items[:]"
    })));

    let original_json = json!({
        "items": [1, "two", true, null]
    });

    let intermediate: ItemsVec = suture.unstitch(&original_json).unwrap();
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original_json);
}

// -- [2:] on a 2-element array yields empty --

#[test]
fn unstitch_slice_start_at_length() {
    let suture = parse_first(&response_suture(json!({
        "/items/[2:]": "slice[:]"
    })));

    let input = json!({
        "items": ["a", "b"]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert!(result.slice.is_empty());
}

// -- [-1:] on a 5-element array yields last element --

#[test]
fn unstitch_negative_one_start_open_end() {
    let suture = parse_first(&response_suture(json!({
        "/items/[-1:]": "slice[:]"
    })));

    let input = json!({
        "items": [10, 20, 30, 40, 50]
    });

    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!(50)]);
}

// -- Roundtrip with step iteration loses non-selected elements --

#[test]
fn roundtrip_step_iteration_is_lossy() {
    let suture = parse_first(&response_suture(json!({
        "/items/[::2]": "evens[:]"
    })));

    let original_json = json!({
        "items": ["a", "b", "c", "d", "e"]
    });

    // unstitch extracts every other element
    let intermediate: EvensVec = suture.unstitch(&original_json).unwrap();
    assert_eq!(intermediate.evens, vec![json!("a"), json!("c"), json!("e")]);

    // stitch puts them back but can't recover the skipped elements
    // so the JSON won't match the original
    let reconstructed = suture.stitch(&intermediate).unwrap();
    let items = reconstructed["items"].as_array().unwrap();
    // The reconstructed JSON will have elements at indices 0, 1, 2 (the iteration
    // reads 3 struct items at slice[:] positions 0, 1, 2)
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], json!("a"));
    assert_eq!(items[1], json!("c"));
    assert_eq!(items[2], json!("e"));
}

// -- Stitch reverse with doubly nested arrays --

#[test]
fn stitch_doubly_nested_arrays() {
    let suture = parse_first(&response_suture(json!({
        "/groups/[:]/items/[:]": "values[:].[:]"
    })));

    let input = ValuesNested {
        values: vec![vec![json!(1), json!(2)], vec![json!(3)]],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "groups": [
                {"items": [1, 2]},
                {"items": [3]}
            ]
        })
    );
}

// -- Stitch with matrix extraction --

#[test]
fn stitch_matrix_values() {
    let suture = parse_first(&response_suture(json!({
        "/matrix/[:]/row/[:]/value": "cells[:].[:]"
    })));

    let input = MatrixValues {
        cells: vec![vec![10, 20], vec![30, 40, 50]],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "matrix": [
                {"row": [{"value": 10}, {"value": 20}]},
                {"row": [{"value": 30}, {"value": 40}, {"value": 50}]}
            ]
        })
    );
}

// -- Roundtrip matrix --

#[test]
fn roundtrip_matrix() {
    let suture = parse_first(&response_suture(json!({
        "/matrix/[:]/row/[:]/value": "cells[:].[:]"
    })));

    let original_json = json!({
        "matrix": [
            {"row": [{"value": 1}]},
            {"row": [{"value": 2}, {"value": 3}]}
        ]
    });

    let intermediate: MatrixValues = suture.unstitch(&original_json).unwrap();
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original_json);
}
