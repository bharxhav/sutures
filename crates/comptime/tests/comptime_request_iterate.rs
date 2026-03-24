//! Exhaustive integration tests for **compile-time Request direction + Iterate binding**
//! using `sutures_comptime::parse!()` and the `Stitch` trait.
//!
//! Tests every permutation of:
//!   - Pythonic slice syntax on the key (struct/LHS) side
//!   - Fixed-index and slice syntax on the value (JSON/RHS) side
//!   - Fan-out, nested object syntax
//!   - Boundary and empty-array edge cases
//!   - Round-trip identity (stitch then unstitch)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sutures::{Seam, Stitch};

// ============================================================================
// Test structs
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Item {
    name: String,
    value: i64,
    id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Container {
    items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct SimpleContainer {
    #[serde(default)]
    items: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NamedContainer {
    items: Vec<NamedItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NamedItem {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ValueItem {
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct IdItem {
    id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct IdContainer {
    items: Vec<IdItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct NameValue {
    name: String,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ItemsWithNameValue {
    items: Vec<NameValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct Numbers {
    items: Vec<i64>,
}

// ============================================================================
// 1. items[:] -> /items/[:] — full array copy (stitch + unstitch)
// ============================================================================

#[test]
fn stitch_full_array_copy() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!(1), json!(2), json!(3)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [1, 2, 3] }));
}

#[test]
fn unstitch_full_array_copy() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "items": [10, 20, 30] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!(10), json!(20), json!(30)],
        }
    );
}

// ============================================================================
// 2. items[:].name -> /names/[:] — field extraction from array
// ============================================================================

#[test]
fn stitch_field_extraction_from_array() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "names": ["alice", "bob", "charlie"] }));
}

#[test]
fn unstitch_field_extraction_from_array() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "names": ["alice", "bob", "charlie"] });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![
                NamedItem { name: "alice".into() },
                NamedItem { name: "bob".into() },
                NamedItem { name: "charlie".into() },
            ],
        }
    );
}

// ============================================================================
// 3. items[:].name -> /data/[:]/name — nested array target
// ============================================================================

#[test]
fn stitch_field_into_nested_array_target() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/data/[:]/name" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "data": [
                { "name": "alice" },
                { "name": "bob" }
            ]
        })
    );
}

#[test]
fn unstitch_field_from_nested_array_target() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/data/[:]/name" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({
        "data": [
            { "name": "alice" },
            { "name": "bob" }
        ]
    });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![
                NamedItem { name: "alice".into() },
                NamedItem { name: "bob".into() },
            ],
        }
    );
}

// ============================================================================
// 4. items[0] -> /first — single index
// ============================================================================

#[test]
fn stitch_single_index_zero() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0]": "/first" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("alpha"), json!("beta"), json!("gamma")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first": "alpha" }));
}

#[test]
fn unstitch_single_index_zero() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0]": "/first" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "first": "alpha" });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("alpha")],
        }
    );
}

// ============================================================================
// 5. items[-1] -> /last — last element
// ============================================================================

#[test]
fn stitch_last_element() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[-1]": "/last" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("alpha"), json!("beta"), json!("gamma")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "last": "gamma" }));
}

// ============================================================================
// 6. items[1:3] -> /slice/[:] — slice extraction
// ============================================================================

#[test]
fn stitch_slice_extraction() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:3]": "/slice/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "slice": ["b", "c"] }));
}

#[test]
fn unstitch_slice_extraction() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:3]": "/slice/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "slice": ["b", "c"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    // Reverse walk reads /slice/[:] -> values ["b", "c"] with indices [0, 1]
    // Trie path items[1:3] is a range iteration, consumes from read-side indices.
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("b"), json!("c")],
        }
    );
}

// ============================================================================
// 7. items[::2] -> /evens/[:] — step iteration
// ============================================================================

#[test]
fn stitch_step_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::2]": "/evens/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 0, 2, 4 -> "a", "c", "e"
    assert_eq!(result, json!({ "evens": ["a", "c", "e"] }));
}

#[test]
fn unstitch_step_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::2]": "/evens/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "evens": ["a", "c", "e"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("a"), json!("c"), json!("e")],
        }
    );
}

// ============================================================================
// 8. items[::-1] -> /reversed/[:] — reverse iteration
// ============================================================================

#[test]
fn stitch_reverse_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::-1]": "/reversed/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    // Reversed: indices 2, 1, 0 -> "c", "b", "a"
    assert_eq!(result, json!({ "reversed": ["c", "b", "a"] }));
}

#[test]
fn unstitch_reverse_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::-1]": "/reversed/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "reversed": ["c", "b", "a"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("c"), json!("b"), json!("a")],
        }
    );
}

// ============================================================================
// 9. items[1:5:2] -> /selected/[:] — start:end:step
// ============================================================================

#[test]
fn stitch_start_end_step() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:5:2]": "/selected/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![
            json!("a"),
            json!("b"),
            json!("c"),
            json!("d"),
            json!("e"),
            json!("f"),
        ],
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 1, 3 -> "b", "d"
    assert_eq!(result, json!({ "selected": ["b", "d"] }));
}

#[test]
fn unstitch_start_end_step() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:5:2]": "/selected/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "selected": ["b", "d"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("b"), json!("d")],
        }
    );
}

// ============================================================================
// 10. Fan-out: items[:].name -> ["/names/[:]", "/labels/[:]"]
// ============================================================================

#[test]
fn stitch_fan_out() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": ["/names/[:]", "/labels/[:]"] }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "names": ["alice", "bob"],
            "labels": ["alice", "bob"]
        })
    );
}

#[test]
fn unstitch_fan_out() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": ["/names/[:]", "/labels/[:]"] }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({
        "names": ["alice", "bob"],
        "labels": ["alice", "bob"]
    });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![
                NamedItem { name: "alice".into() },
                NamedItem { name: "bob".into() },
            ],
        }
    );
}

// ============================================================================
// 11. Nested object: items[:] -> { "name": "/data/[:]/name", "value": "/data/[:]/value" }
// ============================================================================

#[test]
fn stitch_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": { "name": "/data/[:]/name", "value": "/data/[:]/value" } }] }] }"#
    );
    let suture = &sutures[0];

    let input = ItemsWithNameValue {
        items: vec![
            NameValue { name: "alice".into(), value: 10 },
            NameValue { name: "bob".into(), value: 20 },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "data": [
                { "name": "alice", "value": 10 },
                { "name": "bob", "value": 20 }
            ]
        })
    );
}

#[test]
fn unstitch_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": { "name": "/data/[:]/name", "value": "/data/[:]/value" } }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({
        "data": [
            { "name": "x", "value": 1 },
            { "name": "y", "value": 2 },
            { "name": "z", "value": 3 }
        ]
    });

    let result: ItemsWithNameValue = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        ItemsWithNameValue {
            items: vec![
                NameValue { name: "x".into(), value: 1 },
                NameValue { name: "y".into(), value: 2 },
                NameValue { name: "z".into(), value: 3 },
            ],
        }
    );
}

// ============================================================================
// 12. Empty array with [:] — empty output
// ============================================================================

#[test]
fn stitch_empty_array_produces_empty_output() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer { items: vec![] };

    let result = suture.stitch(&input).unwrap();
    // Empty array produces no writes, so the key won't exist at all
    assert_eq!(result, json!({}));
}

#[test]
fn unstitch_empty_json_array() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "items": [] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(result, SimpleContainer { items: vec![] });
}

// ============================================================================
// 13. Single-element array
// ============================================================================

#[test]
fn stitch_single_element_array() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!(42)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [42] }));
}

#[test]
fn unstitch_single_element_json_array() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "items": [42] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!(42)],
        }
    );
}

#[test]
fn stitch_single_element_with_field_extraction() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![NamedItem { name: "only".into() }],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "names": ["only"] }));
}

// ============================================================================
// 14. Round-trip stitch then unstitch = identity for array fields
// ============================================================================

#[test]
fn round_trip_full_array_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_full_array_value_copy() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = SimpleContainer {
        items: vec![json!(1), json!("hello"), json!(true), json!(null)],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_nested_object_syntax() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": { "name": "/data/[:]/name", "value": "/data/[:]/value" } }] }] }"#
    );
    let suture = &sutures[0];

    let original = ItemsWithNameValue {
        items: vec![
            NameValue { name: "alice".into(), value: 10 },
            NameValue { name: "bob".into(), value: 20 },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: ItemsWithNameValue = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_extract_field_nested_array() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/data/[:]/name" }] }] }"#
    );
    let suture = &sutures[0];

    let original = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_multiple_fields_from_iterated_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/names/[:]", "items[:].value": "/values/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = ItemsWithNameValue {
        items: vec![
            NameValue { name: "alice".into(), value: 100 },
            NameValue { name: "bob".into(), value: 200 },
            NameValue { name: "charlie".into(), value: 300 },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: ItemsWithNameValue = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// 15. Round-trip for single index
// ============================================================================

#[test]
fn round_trip_single_index_field() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0].name": "/first_name" }] }] }"#
    );
    let suture = &sutures[0];

    let original = NamedContainer {
        items: vec![NamedItem { name: "alice".into() }],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "first_name": "alice" }));
    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_single_index_whole_element() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0]": "/first" }] }] }"#
    );
    let suture = &sutures[0];

    let original = SimpleContainer {
        items: vec![json!({"key": "value"})],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "first": {"key": "value"} }));
    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// 16. [-2:] — negative start slice
// ============================================================================

#[test]
fn stitch_negative_start_slice() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[-2:]": "/tail/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    // [-2:] on 5 elements: start = 5 + (-2) = 3, end = 5 -> indices 3, 4
    assert_eq!(result, json!({ "tail": ["d", "e"] }));
}

#[test]
fn unstitch_negative_start_slice() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[-2:]": "/tail/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "tail": ["d", "e"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("d"), json!("e")],
        }
    );
}

#[test]
fn round_trip_negative_start() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[-2:]": "/tail/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "tail": ["d", "e"] }));

    // Unstitch only reconstructs the mapped elements.
    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(
        unstitched,
        SimpleContainer {
            items: vec![json!("d"), json!("e")],
        }
    );
}

// ============================================================================
// 17. [:-2] — negative end slice
// ============================================================================

#[test]
fn stitch_negative_end_slice() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:-2]": "/head/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    // [:-2] on 5 elements: start = 0, end = 5 + (-2) = 3 -> indices 0, 1, 2
    assert_eq!(result, json!({ "head": ["a", "b", "c"] }));
}

#[test]
fn unstitch_negative_end_slice() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:-2]": "/head/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "head": ["a", "b", "c"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("a"), json!("b"), json!("c")],
        }
    );
}

#[test]
fn round_trip_negative_end() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:-2]": "/head/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "head": ["a", "b", "c"] }));

    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(
        unstitched,
        SimpleContainer {
            items: vec![json!("a"), json!("b"), json!("c")],
        }
    );
}

// ============================================================================
// 18. [::3] on large array
// ============================================================================

#[test]
fn stitch_step_three_on_ten_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::3]": "/stepped/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..10).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::3] on 10 elements: indices 0, 3, 6, 9
    assert_eq!(result, json!({ "stepped": [0, 3, 6, 9] }));
}

#[test]
fn stitch_step_three_on_twelve_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::3]": "/stepped/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..12).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::3] on 12 elements: indices 0, 3, 6, 9
    assert_eq!(result, json!({ "stepped": [0, 3, 6, 9] }));
}

#[test]
fn stitch_large_step_five() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::5]": "/sparse/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..12).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::5] on 12 elements: indices 0, 5, 10
    assert_eq!(result, json!({ "sparse": [0, 5, 10] }));
}

// ============================================================================
// 19. [::-2] on array
// ============================================================================

#[test]
fn stitch_reverse_step_two_on_six_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::-2]": "/rev_stepped/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..6).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::-2] on 6 elements: start=5, end=-1 (clamp to -1), step=-2
    // indices: 5, 3, 1
    assert_eq!(result, json!({ "rev_stepped": [5, 3, 1] }));
}

#[test]
fn stitch_reverse_step_two_on_five_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::-2]": "/rev_stepped/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..5).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::-2] on 5 elements: start=4, step=-2 -> 4, 2, 0
    assert_eq!(result, json!({ "rev_stepped": [4, 2, 0] }));
}

#[test]
fn stitch_reverse_step_three() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::-3]": "/rev3/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..10).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::-3] on 10: start=9, step=-3 -> 9, 6, 3, 0
    assert_eq!(result, json!({ "rev3": [9, 6, 3, 0] }));
}

// ============================================================================
// 20. Mixed [0] and [:] in same suture
// ============================================================================

#[test]
fn stitch_mixed_fixed_index_and_full_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0].name": "/first_name", "items[:].name": "/all_names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "first_name": "alice",
            "all_names": ["alice", "bob"]
        })
    );
}

#[test]
fn stitch_mixed_fixed_indices() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0]": "/first", "items[1]": "/second" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first": "a", "second": "b" }));
}

// ============================================================================
// Additional stitch edge cases
// ============================================================================

#[test]
fn stitch_last_element_field_extraction() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[-1].name": "/last_name" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "last_name": "charlie" }));
}

#[test]
fn stitch_field_from_single_index() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0].name": "/first_name" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first_name": "alice" }));
}

#[test]
fn stitch_field_from_slice_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:3].name": "/names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
            NamedItem { name: "dave".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "names": ["bob", "charlie"] }));
}

#[test]
fn stitch_field_from_step_iterated_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::2].value": "/even_values/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = Container {
        items: vec![
            Item { name: "a".into(), value: 10, id: 1 },
            Item { name: "b".into(), value: 20, id: 2 },
            Item { name: "c".into(), value: 30, id: 3 },
            Item { name: "d".into(), value: 40, id: 4 },
            Item { name: "e".into(), value: 50, id: 5 },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 0, 2, 4 -> values 10, 30, 50
    assert_eq!(result, json!({ "even_values": [10, 30, 50] }));
}

#[test]
fn stitch_field_from_reversed_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::-1].name": "/reversed_names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "reversed_names": ["charlie", "bob", "alice"] })
    );
}

#[test]
fn stitch_field_from_range_stepped_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:5:2].id": "/selected_ids/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = Container {
        items: vec![
            Item { name: "a".into(), value: 10, id: 100 },
            Item { name: "b".into(), value: 20, id: 200 },
            Item { name: "c".into(), value: 30, id: 300 },
            Item { name: "d".into(), value: 40, id: 400 },
            Item { name: "e".into(), value: 50, id: 500 },
            Item { name: "f".into(), value: 60, id: 600 },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 1, 3 -> ids 200, 400
    assert_eq!(result, json!({ "selected_ids": [200, 400] }));
}

// ============================================================================
// Additional unstitch edge cases
// ============================================================================

#[test]
fn unstitch_field_from_single_index() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0].name": "/first_name" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "first_name": "alice" });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![NamedItem { name: "alice".into() }],
        }
    );
}

#[test]
fn unstitch_field_from_slice_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:3].name": "/names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "names": ["bob", "charlie"] });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![
                NamedItem { name: "bob".into() },
                NamedItem { name: "charlie".into() },
            ],
        }
    );
}

#[test]
fn unstitch_field_from_range_stepped_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[1:5:2].id": "/selected_ids/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({ "selected_ids": [200, 400] });

    let result: IdContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        IdContainer {
            items: vec![
                IdItem { id: 200 },
                IdItem { id: 400 },
            ],
        }
    );
}

#[test]
fn unstitch_multiple_fields_to_iterated_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/names/[:]", "items[:].value": "/values/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = json!({
        "names": ["a", "b"],
        "values": [1, 2]
    });

    let result: ItemsWithNameValue = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        ItemsWithNameValue {
            items: vec![
                NameValue { name: "a".into(), value: 1 },
                NameValue { name: "b".into(), value: 2 },
            ],
        }
    );
}

// ============================================================================
// Additional round-trip tests
// ============================================================================

#[test]
fn round_trip_step_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::2].name": "/even_names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = NamedContainer {
        items: vec![
            NamedItem { name: "a".into() },
            NamedItem { name: "b".into() },
            NamedItem { name: "c".into() },
            NamedItem { name: "d".into() },
            NamedItem { name: "e".into() },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "even_names": ["a", "c", "e"] }));

    // Unstitch only reconstructs the mapped elements (3 items, not 5).
    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(
        unstitched,
        NamedContainer {
            items: vec![
                NamedItem { name: "a".into() },
                NamedItem { name: "c".into() },
                NamedItem { name: "e".into() },
            ],
        }
    );
}

#[test]
fn round_trip_whole_object_iteration() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/data/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = SimpleContainer {
        items: vec![
            json!({"a": 1}),
            json!({"b": 2}),
            json!({"c": 3}),
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(
        stitched,
        json!({
            "data": [{"a": 1}, {"b": 2}, {"c": 3}]
        })
    );

    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// Stitch: verify exact output structure for complex targets
// ============================================================================

#[test]
fn stitch_field_to_deeply_nested_json_path() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/response/data/names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "x".into() },
            NamedItem { name: "y".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "response": {
                "data": {
                    "names": ["x", "y"]
                }
            }
        })
    );
}

#[test]
fn stitch_multiple_fields_from_iterated_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/names/[:]", "items[:].value": "/values/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = Container {
        items: vec![
            Item { name: "a".into(), value: 1, id: 10 },
            Item { name: "b".into(), value: 2, id: 20 },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "names": ["a", "b"],
            "values": [1, 2]
        })
    );
}

// ============================================================================
// Stitch: write to fixed-index target
// ============================================================================

#[test]
fn stitch_write_to_fixed_index_target() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0]": "/items/[0]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!(42), json!(99)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [42] }));
}

// ============================================================================
// Stitch: slice out of range produces empty
// ============================================================================

#[test]
fn stitch_slice_out_of_range_produces_empty() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[5:10]": "/slice/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    // [5:10] on a 3-element array: start=5 clamped to 3, end=10 clamped to 3 -> no indices
    assert_eq!(result, json!({}));
}

// ============================================================================
// Stitch: empty range [2:2] produces nothing
// ============================================================================

#[test]
fn stitch_empty_range_produces_nothing() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[2:2]": "/empty/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!(1), json!(2), json!(3)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({}));
}

// ============================================================================
// Stitch: [0:] covers all elements (open-ended)
// ============================================================================

#[test]
fn stitch_open_ended_from_zero() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0:]": "/all/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "all": ["a", "b", "c"] }));
}

// ============================================================================
// Stitch: [:2] covers first two elements
// ============================================================================

#[test]
fn stitch_first_two_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:2]": "/first_two/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first_two": ["a", "b"] }));
}

// ============================================================================
// Stitch: middle of large array with [3:7]
// ============================================================================

#[test]
fn stitch_slice_middle_of_ten_elements() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[3:7]": "/slice/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..10).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 3, 4, 5, 6
    assert_eq!(result, json!({ "slice": [3, 4, 5, 6] }));
}

// ============================================================================
// Stitch: negative start with step [-4::2]
// ============================================================================

#[test]
fn stitch_range_negative_start_with_step() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[-4::2]": "/tail_stepped/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: (0..8).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [-4::2] on 8: start = 8 + (-4) = 4, end = 8, step = 2
    // Indices: 4, 6
    assert_eq!(result, json!({ "tail_stepped": [4, 6] }));
}

// ============================================================================
// Stitch: [0:1] behaves like single-element slice
// ============================================================================

#[test]
fn stitch_slice_zero_to_one() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[0:1]": "/first/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first": ["a"] }));
}

// ============================================================================
// Stitch: iterate source, write to fixed index (last one wins)
// ============================================================================

#[test]
fn stitch_iterate_source_write_to_fixed_index() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": "/data/[0]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    // Each iteration writes to /data/[0], last write wins
    assert_eq!(result, json!({ "data": ["charlie"] }));
}

// ============================================================================
// Compilation correctness: verify the suture is marked as request
// ============================================================================

#[test]
fn comptime_request_suture_is_request_direction() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];
    assert!(suture.is_request());
    assert!(!suture.is_response());
}

// ============================================================================
// Stitch: complex objects with iteration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ComplexItem {
    label: String,
    metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Seam)]
struct ComplexContainer {
    items: Vec<ComplexItem>,
}

#[test]
fn stitch_iterate_complex_objects_field() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].label": "/labels/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = ComplexContainer {
        items: vec![
            ComplexItem {
                label: "first".into(),
                metadata: json!({"key": "value"}),
            },
            ComplexItem {
                label: "second".into(),
                metadata: json!({"key": "other"}),
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "labels": ["first", "second"] }));
}

#[test]
fn stitch_iterate_extract_whole_element() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/data/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = ComplexContainer {
        items: vec![
            ComplexItem {
                label: "first".into(),
                metadata: json!({"key": "value"}),
            },
            ComplexItem {
                label: "second".into(),
                metadata: json!(null),
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({
            "data": [
                { "label": "first", "metadata": {"key": "value"} },
                { "label": "second", "metadata": null }
            ]
        })
    );
}

// ============================================================================
// Verify comptime parse produces multiple sutures correctly
// ============================================================================

#[test]
fn comptime_parse_produces_correct_count() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    assert_eq!(sutures.len(), 1);
}

// ============================================================================
// Stitch + unstitch: heterogeneous value types in array
// ============================================================================

#[test]
fn round_trip_heterogeneous_value_types() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:]": "/items/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let original = SimpleContainer {
        items: vec![
            json!(42),
            json!("string"),
            json!(true),
            json!(null),
            json!(3.14),
            json!({"nested": "object"}),
            json!([1, 2, 3]),
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// Stitch: large array with various slicing
// ============================================================================

#[test]
fn stitch_large_array_step_three_field_extraction() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[::3].name": "/selected_names/[:]" }] }] }"#
    );
    let suture = &sutures[0];

    let input = NamedContainer {
        items: (0..9)
            .map(|i| NamedItem { name: format!("item_{}", i) })
            .collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::3] on 9 elements: indices 0, 3, 6
    assert_eq!(
        result,
        json!({ "selected_names": ["item_0", "item_3", "item_6"] })
    );
}

// ============================================================================
// Stitch + unstitch: verify identity for fan-out
// ============================================================================

#[test]
fn round_trip_fan_out() {
    let sutures = sutures_comptime::parse!(
        r#"{ "name": "test", "suture_sets": [{ "name": "s", "capture": "request", "sutures": [{ "items[:].name": ["/names/[:]", "/labels/[:]"] }] }] }"#
    );
    let suture = &sutures[0];

    let original = NamedContainer {
        items: vec![
            NamedItem { name: "alice".into() },
            NamedItem { name: "bob".into() },
            NamedItem { name: "charlie".into() },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(
        stitched,
        json!({
            "names": ["alice", "bob", "charlie"],
            "labels": ["alice", "bob", "charlie"]
        })
    );

    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}
