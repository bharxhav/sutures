// Exhaustive integration tests for compile-time Response direction + Iterate bindings.
//
// Response sutures: keys are JSON terminals (slash-separated, start with `/`),
// values are struct terminals (dot-separated, start with letter).
//
// `unstitch` is the natural/forward direction (JSON -> struct).
// `stitch` is the reverse direction (struct -> JSON).
//
// All sutures are parsed at compile time via `sutures_comptime::parse!()`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Seam, Stitch};

// ============================================================================
// Helpers
// ============================================================================

/// Extract the first compiled suture from a `Vec<Suture>` produced by `parse!`.
fn first(sutures: Vec<sutures::v1::Suture>) -> sutures::v1::Suture {
    sutures.into_iter().next().unwrap()
}

// ============================================================================
// Test structs
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ItemsVec {
    #[serde(default)]
    items: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NamesVec {
    #[serde(default)]
    names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ItemWithName {
    name: String,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ItemsWithFields {
    #[serde(default)]
    items: Vec<ItemWithName>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct FirstItem {
    first: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct LastItem {
    last: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SliceVec {
    #[serde(default)]
    slice: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct EvensVec {
    #[serde(default)]
    evens: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ReversedVec {
    #[serde(default)]
    reversed: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SelectedVec {
    #[serde(default)]
    selected: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NamesAndLabels {
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NameOnly {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DataItems {
    #[serde(default)]
    items: Vec<NameOnly>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SecondToLast {
    item: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct TailSlice {
    #[serde(default)]
    slice: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct HeadSlice {
    #[serde(default)]
    slice: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct StepThreeVec {
    #[serde(default)]
    picked: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ReverseStepTwoVec {
    #[serde(default)]
    picked: Vec<Value>,
}

// ============================================================================
// 1. /items/[:] -> items[:] — full array copy (unstitch + stitch)
// ============================================================================

#[test]
fn unstitch_full_array_copy() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "full_array",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = json!({ "items": [1, 2, 3] });
    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items, vec![json!(1), json!(2), json!(3)]);
}

#[test]
fn stitch_full_array_copy() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "full_array",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = ItemsVec {
        items: vec![json!(1), json!(2), json!(3)],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [1, 2, 3] }));
}

// ============================================================================
// 2. /items/[:]/name -> names[:] — field from each element
// ============================================================================

#[test]
fn unstitch_field_from_each_element() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "field_extract",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": "names[:]" }]
        }]
    }"#));

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

#[test]
fn stitch_field_from_each_element() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "field_extract",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": "names[:]" }]
        }]
    }"#));

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
// 3. /data/[:]/name -> items[:].name — nested field into struct array
// ============================================================================

#[test]
fn unstitch_nested_field_into_struct_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_field",
            "capture": "response",
            "sutures": [{ "/data/[:]/name": "items[:].name" }]
        }]
    }"#));

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

#[test]
fn stitch_nested_field_from_struct_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_field",
            "capture": "response",
            "sutures": [{ "/data/[:]/name": "items[:].name" }]
        }]
    }"#));

    let input = DataItems {
        items: vec![
            NameOnly { name: "x".to_string() },
            NameOnly { name: "y".to_string() },
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

// ============================================================================
// 4. /items/[0] -> first — single index
// ============================================================================

#[test]
fn unstitch_single_index() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_idx",
            "capture": "response",
            "sutures": [{ "/items/[0]": "first" }]
        }]
    }"#));

    let input = json!({ "items": ["alpha", "beta", "gamma"] });
    let result: FirstItem = suture.unstitch(&input).unwrap();
    assert_eq!(result.first, json!("alpha"));
}

#[test]
fn stitch_single_index() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_idx",
            "capture": "response",
            "sutures": [{ "/items/[0]": "first" }]
        }]
    }"#));

    let input = FirstItem { first: json!("alpha") };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": ["alpha"] }));
}

// ============================================================================
// 5. /items/[-1] -> last — last element
// ============================================================================

#[test]
fn unstitch_last_element() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "last_elem",
            "capture": "response",
            "sutures": [{ "/items/[-1]": "last" }]
        }]
    }"#));

    let input = json!({ "items": ["alpha", "beta", "gamma"] });
    let result: LastItem = suture.unstitch(&input).unwrap();
    assert_eq!(result.last, json!("gamma"));
}

// ============================================================================
// 6. /items/[1:3] -> slice[:] — slice
// ============================================================================

#[test]
fn unstitch_slice() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "slice",
            "capture": "response",
            "sutures": [{ "/items/[1:3]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c", "d", "e"] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!("b"), json!("c")]);
}

#[test]
fn stitch_slice() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "slice",
            "capture": "response",
            "sutures": [{ "/items/[1:3]": "slice[:]" }]
        }]
    }"#));

    let input = SliceVec {
        slice: vec![json!("b"), json!("c")],
    };
    let result = suture.stitch(&input).unwrap();
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], json!("b"));
    assert_eq!(items[1], json!("c"));
}

// ============================================================================
// 7. /items/[::2] -> evens[:] — step iteration
// ============================================================================

#[test]
fn unstitch_step_two() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "step",
            "capture": "response",
            "sutures": [{ "/items/[::2]": "evens[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c", "d", "e"] });
    let result: EvensVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.evens, vec![json!("a"), json!("c"), json!("e")]);
}

// ============================================================================
// 8. /items/[::-1] -> reversed[:] — reverse
// ============================================================================

#[test]
fn unstitch_reverse() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "reverse",
            "capture": "response",
            "sutures": [{ "/items/[::-1]": "reversed[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c"] });
    let result: ReversedVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.reversed, vec![json!("c"), json!("b"), json!("a")]);
}

// ============================================================================
// 9. /items/[1:5:2] -> selected[:] — range+step
// ============================================================================

#[test]
fn unstitch_range_with_step() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "range_step",
            "capture": "response",
            "sutures": [{ "/items/[1:5:2]": "selected[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c", "d", "e", "f"] });
    let result: SelectedVec = suture.unstitch(&input).unwrap();
    // indices 1, 3
    assert_eq!(result.selected, vec![json!("b"), json!("d")]);
}

// ============================================================================
// 10. Nested object: /items/[:] -> { "/name": "items[:].name", "/value": "items[:].value" }
// ============================================================================

#[test]
fn unstitch_nested_object_multiple_fields() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_obj",
            "capture": "response",
            "sutures": [{
                "/items/[:]": {
                    "/name": "items[:].name",
                    "/value": "items[:].value"
                }
            }]
        }]
    }"#));

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

#[test]
fn stitch_nested_object_multiple_fields() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_obj",
            "capture": "response",
            "sutures": [{
                "/items/[:]": {
                    "/name": "items[:].name",
                    "/value": "items[:].value"
                }
            }]
        }]
    }"#));

    let input = ItemsWithFields {
        items: vec![
            ItemWithName { name: "foo".to_string(), value: 100 },
            ItemWithName { name: "bar".to_string(), value: 200 },
        ],
    };

    let json_repr = suture.stitch(&input).unwrap();
    let recovered: ItemsWithFields = suture.unstitch(&json_repr).unwrap();
    assert_eq!(recovered, input);
}

// ============================================================================
// 11. Fan-out: /items/[:]/name -> ["names[:]", "labels[:]"]
// ============================================================================

#[test]
fn unstitch_fan_out() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fan_out",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": ["names[:]", "labels[:]"] }]
        }]
    }"#));

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

#[test]
fn stitch_fan_out_reads_first_target() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fan_out",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": ["names[:]", "labels[:]"] }]
        }]
    }"#));

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

// ============================================================================
// 12. Empty JSON array — Vec fields default to empty
// ============================================================================

#[test]
fn unstitch_empty_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = json!({ "items": [] });
    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert!(result.items.is_empty());
}

#[test]
fn stitch_empty_struct_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = ItemsVec { items: vec![] };
    let result = suture.stitch(&input).unwrap();
    if let Some(arr) = result.get("items") {
        assert_eq!(arr.as_array().unwrap().len(), 0);
    }
}

// ============================================================================
// 13. Single-element array
// ============================================================================

#[test]
fn unstitch_single_element_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = json!({ "items": [42] });
    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items, vec![json!(42)]);
}

#[test]
fn stitch_single_element_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = ItemsVec {
        items: vec![json!(42)],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [42] }));
}

// ============================================================================
// 14. Round-trip: unstitch -> stitch = identity for arrays
// ============================================================================

#[test]
fn roundtrip_unstitch_then_stitch_full_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let original = json!({ "items": [1, 2, 3, 4, 5] });
    let intermediate: ItemsVec = suture.unstitch(&original).unwrap();
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original);
}

#[test]
fn roundtrip_stitch_then_unstitch_full_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let original = ItemsVec {
        items: vec![json!("x"), json!("y"), json!("z")],
    };
    let json_repr = suture.stitch(&original).unwrap();
    let recovered: ItemsVec = suture.unstitch(&json_repr).unwrap();
    assert_eq!(recovered, original);
}

#[test]
fn roundtrip_field_extraction() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip_field",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": "names[:]" }]
        }]
    }"#));

    let original = NamesVec {
        names: vec!["hello".to_string(), "world".to_string()],
    };
    let json_repr = suture.stitch(&original).unwrap();
    let recovered: NamesVec = suture.unstitch(&json_repr).unwrap();
    assert_eq!(recovered, original);
}

#[test]
fn roundtrip_nested_object_fields() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip_nested",
            "capture": "response",
            "sutures": [{
                "/items/[:]": {
                    "/name": "items[:].name",
                    "/value": "items[:].value"
                }
            }]
        }]
    }"#));

    let original = ItemsWithFields {
        items: vec![
            ItemWithName { name: "foo".to_string(), value: 100 },
            ItemWithName { name: "bar".to_string(), value: 200 },
        ],
    };
    let json_repr = suture.stitch(&original).unwrap();
    let recovered: ItemsWithFields = suture.unstitch(&json_repr).unwrap();
    assert_eq!(recovered, original);
}

#[test]
fn roundtrip_nested_field_into_struct_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip_nested_struct",
            "capture": "response",
            "sutures": [{ "/data/[:]/name": "items[:].name" }]
        }]
    }"#));

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

// ============================================================================
// 15. [-2:] — negative start (last 2 elements)
// ============================================================================

#[test]
fn unstitch_negative_start_slice() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_start",
            "capture": "response",
            "sutures": [{ "/items/[-2:]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": [1, 2, 3, 4, 5] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!(4), json!(5)]);
}

#[test]
fn unstitch_negative_start_three() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_start_3",
            "capture": "response",
            "sutures": [{ "/items/[-3:]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": [10, 20, 30, 40, 50] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!(30), json!(40), json!(50)]);
}

// ============================================================================
// 16. [:-2] — negative end (all except last 2)
// ============================================================================

#[test]
fn unstitch_negative_end_slice() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_end",
            "capture": "response",
            "sutures": [{ "/items/[:-2]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": [1, 2, 3, 4, 5] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!(1), json!(2), json!(3)]);
}

#[test]
fn unstitch_negative_end_one() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_end_1",
            "capture": "response",
            "sutures": [{ "/items/[:-1]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c", "d"] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!("a"), json!("b"), json!("c")]);
}

// ============================================================================
// 17. [::3] — step 3 on large array
// ============================================================================

#[test]
fn unstitch_step_three_on_ten_elements() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "step3",
            "capture": "response",
            "sutures": [{ "/items/[::3]": "evens[:]" }]
        }]
    }"#));

    let input = json!({ "items": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] });
    let result: EvensVec = suture.unstitch(&input).unwrap();
    // indices 0, 3, 6, 9
    assert_eq!(
        result.evens,
        vec![json!(0), json!(3), json!(6), json!(9)]
    );
}

#[test]
fn unstitch_step_three_on_twelve_elements() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "step3_12",
            "capture": "response",
            "sutures": [{ "/items/[::3]": "selected[:]" }]
        }]
    }"#));

    let input = json!({ "items": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] });
    let result: SelectedVec = suture.unstitch(&input).unwrap();
    // indices 0, 3, 6, 9
    assert_eq!(
        result.selected,
        vec![json!(0), json!(3), json!(6), json!(9)]
    );
}

// ============================================================================
// 18. [::-2] — reverse step 2
// ============================================================================

#[test]
fn unstitch_reverse_step_two_on_six_elements() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rev_step2",
            "capture": "response",
            "sutures": [{ "/items/[::-2]": "reversed[:]" }]
        }]
    }"#));

    let input = json!({ "items": [0, 1, 2, 3, 4, 5] });
    let result: ReversedVec = suture.unstitch(&input).unwrap();
    // Python: [0,1,2,3,4,5][::-2] = [5, 3, 1]
    assert_eq!(
        result.reversed,
        vec![json!(5), json!(3), json!(1)]
    );
}

#[test]
fn unstitch_reverse_step_two_on_five_elements() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rev_step2_5",
            "capture": "response",
            "sutures": [{ "/items/[::-2]": "reversed[:]" }]
        }]
    }"#));

    let input = json!({ "items": [0, 1, 2, 3, 4] });
    let result: ReversedVec = suture.unstitch(&input).unwrap();
    // Python: [0,1,2,3,4][::-2] = [4, 2, 0]
    assert_eq!(
        result.reversed,
        vec![json!(4), json!(2), json!(0)]
    );
}

// ============================================================================
// Additional edge cases and combinations
// ============================================================================

// -- Negative index -2 --

#[test]
fn unstitch_negative_index_minus_two() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_idx",
            "capture": "response",
            "sutures": [{ "/items/[-2]": "item" }]
        }]
    }"#));

    let input = json!({ "items": [10, 20, 30, 40, 50] });
    let result: SecondToLast = suture.unstitch(&input).unwrap();
    assert_eq!(result.item, json!(40));
}

// -- Negative index -1 on larger array --

#[test]
fn unstitch_negative_index_minus_one_large() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_idx_large",
            "capture": "response",
            "sutures": [{ "/items/[-1]": "last" }]
        }]
    }"#));

    let input = json!({ "items": [10, 20, 30, 40, 50] });
    let result: LastItem = suture.unstitch(&input).unwrap();
    assert_eq!(result.last, json!(50));
}

// -- Out-of-range slice -> empty --

#[test]
fn unstitch_out_of_range_slice_yields_empty() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "oor",
            "capture": "response",
            "sutures": [{ "/items/[10:20]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c"] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert!(result.slice.is_empty());
}

// -- Slice past end of array clamps to length --

#[test]
fn unstitch_slice_past_end_clamps() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "clamp",
            "capture": "response",
            "sutures": [{ "/items/[2:100]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c", "d"] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!("c"), json!("d")]);
}

// -- [0:] selects all (same as [:]) --

#[test]
fn unstitch_zero_start_open_end() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "zero_start",
            "capture": "response",
            "sutures": [{ "/items/[0:]": "items[:]" }]
        }]
    }"#));

    let input = json!({ "items": [10, 20, 30] });
    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items, vec![json!(10), json!(20), json!(30)]);
}

// -- [:2] selects first two --

#[test]
fn unstitch_open_start_fixed_end() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "first_two",
            "capture": "response",
            "sutures": [{ "/items/[:2]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c", "d"] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.slice, vec![json!("a"), json!("b")]);
}

// -- Reverse full array --

#[test]
fn unstitch_reverse_full_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rev_full",
            "capture": "response",
            "sutures": [{ "/items/[::-1]": "reversed[:]" }]
        }]
    }"#));

    let input = json!({ "items": [1, 2, 3, 4, 5] });
    let result: ReversedVec = suture.unstitch(&input).unwrap();
    assert_eq!(
        result.reversed,
        vec![json!(5), json!(4), json!(3), json!(2), json!(1)]
    );
}

// -- Step iteration with field extraction --

#[test]
fn unstitch_step_with_field_extraction() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "step_field",
            "capture": "response",
            "sutures": [{ "/items/[::2]/name": "names[:]" }]
        }]
    }"#));

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
    // indices 0, 2, 4
    assert_eq!(result.names, vec!["zero", "two", "four"]);
}

// -- Reverse with field extraction --

#[test]
fn unstitch_reverse_with_field_extraction() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "rev_field",
            "capture": "response",
            "sutures": [{ "/items/[::-1]/name": "names[:]" }]
        }]
    }"#));

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

// -- Large array with step 5 --

#[test]
fn unstitch_large_array_step_five() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "step5",
            "capture": "response",
            "sutures": [{ "/items/[::5]": "selected[:]" }]
        }]
    }"#));

    let input = json!({ "items": (0..20).collect::<Vec<i32>>() });
    let result: SelectedVec = suture.unstitch(&input).unwrap();
    // indices 0, 5, 10, 15
    assert_eq!(
        result.selected,
        vec![json!(0), json!(5), json!(10), json!(15)]
    );
}

// -- Fan-out preserves all elements --

#[test]
fn unstitch_fan_out_preserves_all_elements() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "fan_all",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": ["names[:]", "labels[:]"] }]
        }]
    }"#));

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

// -- Verify the suture is indeed a Response binding --

#[test]
fn parsed_suture_is_response() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "verify_response",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    assert!(suture.is_response());
    assert!(!suture.is_request());
}

// -- Range+step on larger array: /items/[1:5:2] --

#[test]
fn unstitch_range_step_larger_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "range_step_lg",
            "capture": "response",
            "sutures": [{ "/items/[1:5:2]": "selected[:]" }]
        }]
    }"#));

    let input = json!({ "items": ["a", "b", "c", "d", "e", "f", "g", "h"] });
    let result: SelectedVec = suture.unstitch(&input).unwrap();
    // indices 1, 3
    assert_eq!(result.selected, vec![json!("b"), json!("d")]);
}

// -- Single-element array with field extraction --

#[test]
fn unstitch_single_element_field_extraction() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "single_field",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": "names[:]" }]
        }]
    }"#));

    let input = json!({
        "items": [{"name": "only"}]
    });

    let result: NamesVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.names, vec!["only"]);
}

// -- Empty array with field extraction --

#[test]
fn unstitch_empty_array_field_extraction() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "empty_field",
            "capture": "response",
            "sutures": [{ "/items/[:]/name": "names[:]" }]
        }]
    }"#));

    let input = json!({ "items": [] });
    let result: NamesVec = suture.unstitch(&input).unwrap();
    assert!(result.names.is_empty());
}

// -- Slice extraction with field extraction --

#[test]
fn unstitch_slice_with_field_extraction() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "slice_field",
            "capture": "response",
            "sutures": [{ "/items/[1:3]/name": "names[:]" }]
        }]
    }"#));

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

// -- Roundtrip with single-element array --

#[test]
fn roundtrip_single_element() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip_single",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let original = json!({ "items": [99] });
    let intermediate: ItemsVec = suture.unstitch(&original).unwrap();
    assert_eq!(intermediate.items, vec![json!(99)]);
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original);
}

// -- Negative start + positive end: [-3:4] --

#[test]
fn unstitch_negative_start_positive_end() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "neg_start_pos_end",
            "capture": "response",
            "sutures": [{ "/items/[-3:4]": "slice[:]" }]
        }]
    }"#));

    let input = json!({ "items": [0, 1, 2, 3, 4] });
    let result: SliceVec = suture.unstitch(&input).unwrap();
    // -3 resolves to index 2; [2:4] = [2, 3]
    assert_eq!(result.slice, vec![json!(2), json!(3)]);
}

// -- Mixed types in array --

#[test]
fn unstitch_mixed_types_in_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "mixed",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = json!({ "items": [1, "two", true, null, 3.14] });
    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(
        result.items,
        vec![json!(1), json!("two"), json!(true), json!(null), json!(3.14)]
    );
}

// -- Roundtrip mixed types --

#[test]
fn roundtrip_mixed_types() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "roundtrip_mixed",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let original = json!({ "items": [1, "two", true, null] });
    let intermediate: ItemsVec = suture.unstitch(&original).unwrap();
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original);
}

// -- Nested objects in array --

#[test]
fn unstitch_nested_objects_in_array() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "nested_objs",
            "capture": "response",
            "sutures": [{ "/items/[:]": "items[:]" }]
        }]
    }"#));

    let input = json!({
        "items": [
            {"a": 1, "b": 2},
            {"a": 3, "b": 4}
        ]
    });

    let result: ItemsVec = suture.unstitch(&input).unwrap();
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.items[0], json!({"a": 1, "b": 2}));
    assert_eq!(result.items[1], json!({"a": 3, "b": 4}));
}

// -- Deeply nested path: /data/results/[:]  --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct DeepResult {
    #[serde(default)]
    vals: Vec<Value>,
}

#[test]
fn unstitch_deeply_nested_json_path() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep",
            "capture": "response",
            "sutures": [{ "/data/results/[:]": "vals[:]" }]
        }]
    }"#));

    let input = json!({
        "data": {
            "results": [100, 200, 300]
        }
    });

    let result: DeepResult = suture.unstitch(&input).unwrap();
    assert_eq!(result.vals, vec![json!(100), json!(200), json!(300)]);
}

// -- Stitch reverse for deeply nested path --

#[test]
fn stitch_deeply_nested_json_path() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_stitch",
            "capture": "response",
            "sutures": [{ "/data/results/[:]": "vals[:]" }]
        }]
    }"#));

    let input = DeepResult {
        vals: vec![json!(100), json!(200), json!(300)],
    };
    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "data": {
                "results": [100, 200, 300]
            }
        })
    );
}

// -- Roundtrip deeply nested --

#[test]
fn roundtrip_deeply_nested() {
    let suture = first(sutures_comptime::parse!(r#"{
        "name": "test",
        "suture_sets": [{
            "name": "deep_rt",
            "capture": "response",
            "sutures": [{ "/data/results/[:]": "vals[:]" }]
        }]
    }"#));

    let original = json!({
        "data": {
            "results": [1, 2, 3]
        }
    });
    let intermediate: DeepResult = suture.unstitch(&original).unwrap();
    let reconstructed = suture.stitch(&intermediate).unwrap();
    assert_eq!(reconstructed, original);
}
