//! Exhaustive integration tests for **Request direction + Iterate binding**
//! covering both `stitch` (struct -> JSON) and `unstitch` (JSON -> struct).
//!
//! Tests every permutation of:
//!   - Pythonic slice syntax on the key (struct/LHS) side
//!   - Fixed-index and slice syntax on the value (JSON/RHS) side
//!   - Fan-out, nested iteration, nested object syntax
//!   - Boundary and empty-array edge cases
//!   - Round-trip identity (stitch then unstitch)

use serde_json::json;
use sutures::Stitch;

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

/// Build a request suture from a single suture mapping object.
fn request_suture(suture_obj: serde_json::Value) -> sutures::v1::Suture {
    let doc = json!({
        "name": "test",
        "suture_sets": [{
            "name": "test_set",
            "capture": "request",
            "sutures": [suture_obj]
        }]
    });
    parse_first(&doc.to_string())
}

// ============================================================================
// Test structs
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Item {
    name: String,
    value: i64,
    id: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Container {
    items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct SimpleContainer {
    #[serde(default)]
    items: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NamedContainer {
    items: Vec<NamedItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NamedItem {
    name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ValueItem {
    value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct IdItem {
    id: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct GroupContainer {
    groups: Vec<Group>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Group {
    items: Vec<ValueItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct MatrixContainer {
    matrix: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Row {
    row: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ItemsWithNameValue {
    items: Vec<NameValue>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct NameValue {
    name: String,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct Numbers {
    items: Vec<i64>,
}

// ============================================================================
// 1. items[:] -> /items/[:] — copy full array element by element (stitch)
// ============================================================================

#[test]
fn stitch_full_array_copy_element_by_element() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!(1), json!(2), json!(3)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [1, 2, 3] }));
}

// ============================================================================
// 2. items[:].name -> /names/[:] — extract field from each element (stitch)
// ============================================================================

#[test]
fn stitch_extract_field_from_each_element() {
    let suture = request_suture(json!({
        "items[:].name": "/names/[:]"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
            NamedItem {
                name: "charlie".into(),
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "names": ["alice", "bob", "charlie"] }));
}

// ============================================================================
// 3. items[:].name -> /data/[:]/name — extract field, place in nested array
// ============================================================================

#[test]
fn stitch_extract_field_into_nested_array_structure() {
    let suture = request_suture(json!({
        "items[:].name": "/data/[:]/name"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
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

// ============================================================================
// 4. items[0] -> /first — extract single index to flat target (stitch)
// ============================================================================

#[test]
fn stitch_single_index_to_flat_target() {
    let suture = request_suture(json!({
        "items[0]": "/first"
    }));

    let input = SimpleContainer {
        items: vec![json!("alpha"), json!("beta"), json!("gamma")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first": "alpha" }));
}

// ============================================================================
// 5. items[-1] -> /last — extract last element (stitch)
// ============================================================================

#[test]
fn stitch_last_element() {
    let suture = request_suture(json!({
        "items[-1]": "/last"
    }));

    let input = SimpleContainer {
        items: vec![json!("alpha"), json!("beta"), json!("gamma")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "last": "gamma" }));
}

// ============================================================================
// 6. items[0].name -> /first_name — extract field from single index (stitch)
// ============================================================================

#[test]
fn stitch_field_from_single_index() {
    let suture = request_suture(json!({
        "items[0].name": "/first_name"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first_name": "alice" }));
}

// ============================================================================
// 7. items[1:3] -> /slice/[:] — slice extraction (stitch)
// ============================================================================

#[test]
fn stitch_slice_extraction() {
    let suture = request_suture(json!({
        "items[1:3]": "/slice/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "slice": ["b", "c"] }));
}

// ============================================================================
// 8. items[1:3].name -> /names/[:] — field from slice elements (stitch)
// ============================================================================

#[test]
fn stitch_field_from_slice_elements() {
    let suture = request_suture(json!({
        "items[1:3].name": "/names/[:]"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
            NamedItem {
                name: "charlie".into(),
            },
            NamedItem {
                name: "dave".into(),
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "names": ["bob", "charlie"] }));
}

// ============================================================================
// 9. items[::2] -> /evens/[:] — step iteration (stitch)
// ============================================================================

#[test]
fn stitch_step_iteration() {
    let suture = request_suture(json!({
        "items[::2]": "/evens/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 0, 2, 4 -> "a", "c", "e"
    assert_eq!(result, json!({ "evens": ["a", "c", "e"] }));
}

// ============================================================================
// 10. items[::2].value -> /even_values/[:] — field from step-iterated (stitch)
// ============================================================================

#[test]
fn stitch_field_from_step_iterated_elements() {
    let suture = request_suture(json!({
        "items[::2].value": "/even_values/[:]"
    }));

    let input = Container {
        items: vec![
            Item {
                name: "a".into(),
                value: 10,
                id: 1,
            },
            Item {
                name: "b".into(),
                value: 20,
                id: 2,
            },
            Item {
                name: "c".into(),
                value: 30,
                id: 3,
            },
            Item {
                name: "d".into(),
                value: 40,
                id: 4,
            },
            Item {
                name: "e".into(),
                value: 50,
                id: 5,
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 0, 2, 4 -> values 10, 30, 50
    assert_eq!(result, json!({ "even_values": [10, 30, 50] }));
}

// ============================================================================
// 11. items[::-1] -> /reversed/[:] — reverse iteration (stitch)
// ============================================================================

#[test]
fn stitch_reverse_iteration() {
    let suture = request_suture(json!({
        "items[::-1]": "/reversed/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    // Reversed: indices 2, 1, 0 -> "c", "b", "a"
    assert_eq!(result, json!({ "reversed": ["c", "b", "a"] }));
}

// ============================================================================
// 12. items[::-1].name -> /reversed_names/[:] — field from reversed (stitch)
// ============================================================================

#[test]
fn stitch_field_from_reversed_elements() {
    let suture = request_suture(json!({
        "items[::-1].name": "/reversed_names/[:]"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
            NamedItem {
                name: "charlie".into(),
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(
        result,
        json!({ "reversed_names": ["charlie", "bob", "alice"] })
    );
}

// ============================================================================
// 13. items[1:5:2] -> /selected/[:] — start:end:step (stitch)
// ============================================================================

#[test]
fn stitch_start_end_step() {
    let suture = request_suture(json!({
        "items[1:5:2]": "/selected/[:]"
    }));

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

// ============================================================================
// 14. items[1:5:2].id -> /selected_ids/[:] — field from range-stepped (stitch)
// ============================================================================

#[test]
fn stitch_field_from_range_stepped_elements() {
    let suture = request_suture(json!({
        "items[1:5:2].id": "/selected_ids/[:]"
    }));

    let input = Container {
        items: vec![
            Item {
                name: "a".into(),
                value: 10,
                id: 100,
            },
            Item {
                name: "b".into(),
                value: 20,
                id: 200,
            },
            Item {
                name: "c".into(),
                value: 30,
                id: 300,
            },
            Item {
                name: "d".into(),
                value: 40,
                id: 400,
            },
            Item {
                name: "e".into(),
                value: 50,
                id: 500,
            },
            Item {
                name: "f".into(),
                value: 60,
                id: 600,
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 1, 3 -> ids 200, 400
    assert_eq!(result, json!({ "selected_ids": [200, 400] }));
}

// ============================================================================
// 15. Fan-out: items[:].name -> ["/names/[:]", "/labels/[:]"] (stitch)
// ============================================================================

#[test]
fn stitch_fan_out_from_iterated_field() {
    let suture = request_suture(json!({
        "items[:].name": ["/names/[:]", "/labels/[:]"]
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
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

// ============================================================================
// 16. Nested iteration: groups[:].items[:].value -> /values/[:]/[:] (stitch)
// ============================================================================

#[test]
fn stitch_doubly_nested_iteration() {
    let suture = request_suture(json!({
        "groups[:].items[:].value": "/values/[:]"
    }));

    let input = GroupContainer {
        groups: vec![
            Group {
                items: vec![ValueItem { value: 1 }, ValueItem { value: 2 }],
            },
            Group {
                items: vec![
                    ValueItem { value: 3 },
                    ValueItem { value: 4 },
                    ValueItem { value: 5 },
                ],
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    // The outer [:] produces indices 0, 1 and the inner [:] iterates within.
    // With a single flat /values/[:], the inner iteration indices are used.
    // The outer indices are [0], [1] and the inner are [0,1] and [0,1,2].
    // Each write uses the full indices stack, but /values/[:] only consumes one.
    // So the outer index determines the position: values[0]=1, values[1]=2, values[0]=3...
    // Actually, the runtime pushes/pops indices. Let's check what actually happens.
    // For group[0]: indices stack is [0], then inner loop pushes [0,0], [0,1]
    //   /values/[:] consumes first index -> values[0]=1, then values[0]=2 (overwrites)
    // For group[1]: indices stack is [1], then inner loop pushes [1,0], [1,1], [1,2]
    //   /values/[:] consumes first index -> values[1]=3, values[1]=4, values[1]=5
    // Wait - the target /values/[:] has one [:], but we have two levels of iteration.
    // The [:] consumes the FIRST index from the stack. So for [0,0] it writes to values[0],
    // for [0,1] it writes to values[0] again (overwrite), for [1,0] -> values[1], etc.
    // This means the last value in each group wins.
    assert_eq!(result, json!({ "values": [2, 5] }));
}

// ============================================================================
// 17. Nested iteration: matrix[:].row[:] -> /flat/[:] (stitch)
// ============================================================================

#[test]
fn stitch_nested_iteration_into_flat_output() {
    let suture = request_suture(json!({
        "matrix[:].row[:]": "/flat/[:]"
    }));

    let input = MatrixContainer {
        matrix: vec![Row { row: vec![1, 2] }, Row { row: vec![3, 4] }],
    };

    let result = suture.stitch(&input).unwrap();
    // With /flat/[:], the first index from the stack is consumed.
    // For [0,0]->flat[0]=1, [0,1]->flat[0]=2 (overwrite),
    // [1,0]->flat[1]=3, [1,1]->flat[1]=4 (overwrite)
    // Last inner value wins per outer index.
    assert_eq!(result, json!({ "flat": [2, 4] }));
}

// ============================================================================
// 18. Nested object syntax with iteration (stitch)
// ============================================================================

#[test]
fn stitch_iterate_with_nested_object_syntax() {
    // items[:] -> { name: /data/[:]/name, value: /data/[:]/value }
    // The nested object expands to:
    //   items[:].name -> /data/[:]/name
    //   items[:].value -> /data/[:]/value
    let suture = request_suture(json!({
        "items[:]": {
            "name": "/data/[:]/name",
            "value": "/data/[:]/value"
        }
    }));

    let input = ItemsWithNameValue {
        items: vec![
            NameValue {
                name: "alice".into(),
                value: 10,
            },
            NameValue {
                name: "bob".into(),
                value: 20,
            },
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

// ============================================================================
// 19. Empty source array with [:] -> empty output (stitch)
// ============================================================================

#[test]
fn stitch_empty_array_produces_empty_output() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));

    let input = SimpleContainer { items: vec![] };

    let result = suture.stitch(&input).unwrap();
    // Empty array produces no writes, so the key won't exist at all
    assert_eq!(result, json!({}));
}

// ============================================================================
// 20. Single-element array with [:] (stitch)
// ============================================================================

#[test]
fn stitch_single_element_array() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!(42)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [42] }));
}

// ============================================================================
// 21. 10-element array with [3:7] -> 4 elements (stitch)
// ============================================================================

#[test]
fn stitch_slice_middle_of_ten_elements() {
    let suture = request_suture(json!({
        "items[3:7]": "/slice/[:]"
    }));

    let input = SimpleContainer {
        items: (0..10).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // Indices 3, 4, 5, 6
    assert_eq!(result, json!({ "slice": [3, 4, 5, 6] }));
}

// ============================================================================
// 22. 3-element array with [5:10] -> empty (out of range) (stitch)
// ============================================================================

#[test]
fn stitch_slice_out_of_range_produces_empty() {
    let suture = request_suture(json!({
        "items[5:10]": "/slice/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    // [5:10] on a 3-element array: start=5 clamped to 3, end=10 clamped to 3 -> no indices
    assert_eq!(result, json!({}));
}

// ============================================================================
// 23. 5-element array with [-2:] -> last 2 elements (stitch)
// ============================================================================

#[test]
fn stitch_negative_start_slice() {
    let suture = request_suture(json!({
        "items[-2:]": "/tail/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    // [-2:] on 5 elements: start = 5 + (-2) = 3, end = 5 -> indices 3, 4
    assert_eq!(result, json!({ "tail": ["d", "e"] }));
}

// ============================================================================
// 24. 5-element array with [:-2] -> first 3 elements (stitch)
// ============================================================================

#[test]
fn stitch_negative_end_slice() {
    let suture = request_suture(json!({
        "items[:-2]": "/head/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let result = suture.stitch(&input).unwrap();
    // [:-2] on 5 elements: start = 0, end = 5 + (-2) = 3 -> indices 0, 1, 2
    assert_eq!(result, json!({ "head": ["a", "b", "c"] }));
}

// ============================================================================
// 25. [::3] on 10-element array -> indices 0,3,6,9 (stitch)
// ============================================================================

#[test]
fn stitch_step_three_on_ten_elements() {
    let suture = request_suture(json!({
        "items[::3]": "/stepped/[:]"
    }));

    let input = SimpleContainer {
        items: (0..10).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::3] on 10 elements: indices 0, 3, 6, 9
    assert_eq!(result, json!({ "stepped": [0, 3, 6, 9] }));
}

// ============================================================================
// 26. [::-2] on 6-element array -> indices 5,3,1 (stitch)
// ============================================================================

#[test]
fn stitch_reverse_step_two_on_six_elements() {
    let suture = request_suture(json!({
        "items[::-2]": "/rev_stepped/[:]"
    }));

    let input = SimpleContainer {
        items: (0..6).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::-2] on 6 elements: start=5, end=-1 (clamp to -1), step=-2
    // indices: 5, 3, 1
    assert_eq!(result, json!({ "rev_stepped": [5, 3, 1] }));
}

// ============================================================================
// 27. Unstitch: items[:] -> /items/[:] — rebuild array from JSON (unstitch)
// ============================================================================

#[test]
fn unstitch_full_array_copy() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));

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
// 28. Unstitch: items[:].name -> /names/[:] — extract from JSON array (unstitch)
// ============================================================================

#[test]
fn unstitch_field_from_json_array() {
    let suture = request_suture(json!({
        "items[:].name": "/names/[:]"
    }));

    let input = json!({ "names": ["alice", "bob", "charlie"] });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![
                NamedItem {
                    name: "alice".into()
                },
                NamedItem { name: "bob".into() },
                NamedItem {
                    name: "charlie".into()
                },
            ],
        }
    );
}

// ============================================================================
// 29. Unstitch: items[0] -> /first — place single value at index 0 (unstitch)
// ============================================================================

#[test]
fn unstitch_single_index() {
    let suture = request_suture(json!({
        "items[0]": "/first"
    }));

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
// 30a. Unstitch: items[1:3] slice variants (unstitch)
// ============================================================================

#[test]
fn unstitch_slice_variant() {
    let suture = request_suture(json!({
        "items[1:3]": "/slice/[:]"
    }));

    let input = json!({ "slice": ["b", "c"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    // Reverse walk reads /slice/[:] -> values ["b", "c"] with indices [0, 1]
    // Trie path is items[1:3] — this is a range iteration, so it consumes indices from read side.
    // The read-side indices [0, 1] map to array positions in items.
    // Items will be [null, "b", "c"] (indices 0, 1 map into trie write positions).
    // Wait — the trie_write_index for a range iteration consumes from read-side indices.
    // Read side gives us indices [0, 1] for the two elements.
    // These become the write indices: items[0]="b", items[1]="c".
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("b"), json!("c")],
        }
    );
}

// ============================================================================
// 30b. Unstitch: items[0].name -> /first_name (unstitch)
// ============================================================================

#[test]
fn unstitch_field_from_single_index() {
    let suture = request_suture(json!({
        "items[0].name": "/first_name"
    }));

    let input = json!({ "first_name": "alice" });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![NamedItem {
                name: "alice".into()
            }],
        }
    );
}

// ============================================================================
// 30c. Unstitch: items[:].name -> /data/[:]/name (unstitch)
// ============================================================================

#[test]
fn unstitch_field_from_nested_array_structure() {
    let suture = request_suture(json!({
        "items[:].name": "/data/[:]/name"
    }));

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
                NamedItem {
                    name: "alice".into()
                },
                NamedItem { name: "bob".into() },
            ],
        }
    );
}

// ============================================================================
// 30d. Unstitch: items[::2] step iteration (unstitch)
// ============================================================================

#[test]
fn unstitch_step_iteration() {
    let suture = request_suture(json!({
        "items[::2]": "/evens/[:]"
    }));

    let input = json!({ "evens": ["a", "c", "e"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    // Reverse: reads /evens/[:] -> 3 values with indices [0, 1, 2]
    // Trie write: items[::2] is a range, consumes from read indices.
    // Write positions: items[0], items[1], items[2]
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("a"), json!("c"), json!("e")],
        }
    );
}

// ============================================================================
// 30e. Unstitch: items[::-1] reverse iteration (unstitch)
// ============================================================================

#[test]
fn unstitch_reverse_iteration() {
    let suture = request_suture(json!({
        "items[::-1]": "/reversed/[:]"
    }));

    let input = json!({ "reversed": ["c", "b", "a"] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    // Reverse: reads /reversed/[:] -> 3 values with indices [0, 1, 2]
    // Trie write: items[::-1] is a range, consumes from read indices.
    assert_eq!(
        result,
        SimpleContainer {
            items: vec![json!("c"), json!("b"), json!("a")],
        }
    );
}

// ============================================================================
// 30f. Unstitch: items[1:3].name -> /names/[:] (unstitch)
// ============================================================================

#[test]
fn unstitch_field_from_slice_elements() {
    let suture = request_suture(json!({
        "items[1:3].name": "/names/[:]"
    }));

    let input = json!({ "names": ["bob", "charlie"] });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    // Read: /names/[:] gives ["bob", "charlie"] with indices [0, 1]
    // Trie: items[1:3] (range, consumes indices) -> items[0], items[1]
    //   .name -> sets name field on each
    assert_eq!(
        result,
        NamedContainer {
            items: vec![
                NamedItem { name: "bob".into() },
                NamedItem {
                    name: "charlie".into()
                },
            ],
        }
    );
}

// ============================================================================
// 30g. Unstitch: items[1:5:2].id -> /selected_ids/[:] (unstitch)
// ============================================================================

#[test]
fn unstitch_field_from_range_stepped_elements() {
    let suture = request_suture(json!({
        "items[1:5:2].id": "/selected_ids/[:]"
    }));

    let input = json!({ "selected_ids": [200, 400] });

    // Container has items: Vec<Item>, and Item requires name(String) + value(i64).
    // The reverse walk only sets .id, so the resulting JSON object is missing
    // required fields and serde deserialization correctly fails.
    assert!(suture.unstitch::<Container>(&input).is_err());
}

// ============================================================================
// 30g-alt. Unstitch: items[1:5:2].id -> /selected_ids/[:] with IdItem (unstitch)
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct IdContainer {
    items: Vec<IdItem>,
}

#[test]
fn unstitch_field_from_range_stepped_elements_id_only() {
    let suture = request_suture(json!({
        "items[1:5:2].id": "/selected_ids/[:]"
    }));

    let input = json!({ "selected_ids": [200, 400] });

    let result: IdContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        IdContainer {
            items: vec![IdItem { id: 200 }, IdItem { id: 400 },],
        }
    );
}

// ============================================================================
// 31. Round-trip: stitch then unstitch with [:] = identity
// ============================================================================

#[test]
fn round_trip_full_array_iteration() {
    let suture = request_suture(json!({
        "items[:].name": "/names/[:]"
    }));

    let original = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
            NamedItem {
                name: "charlie".into(),
            },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_full_array_value_copy() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));

    let original = SimpleContainer {
        items: vec![json!(1), json!("hello"), json!(true), json!(null)],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// 32. Round-trip: stitch then unstitch with [0] = identity
// ============================================================================

#[test]
fn round_trip_single_index() {
    let suture = request_suture(json!({
        "items[0].name": "/first_name"
    }));

    let original = NamedContainer {
        items: vec![NamedItem {
            name: "alice".into(),
        }],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "first_name": "alice" }));
    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_single_index_whole_element() {
    let suture = request_suture(json!({
        "items[0]": "/first"
    }));

    let original = SimpleContainer {
        items: vec![json!({"key": "value"})],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "first": {"key": "value"} }));
    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// 33. Round-trip: stitch then unstitch with nested iteration = identity
// ============================================================================

#[test]
fn round_trip_nested_object_syntax() {
    let suture = request_suture(json!({
        "items[:]": {
            "name": "/data/[:]/name",
            "value": "/data/[:]/value"
        }
    }));

    let original = ItemsWithNameValue {
        items: vec![
            NameValue {
                name: "alice".into(),
                value: 10,
            },
            NameValue {
                name: "bob".into(),
                value: 20,
            },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: ItemsWithNameValue = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

#[test]
fn round_trip_extract_field_nested_array() {
    let suture = request_suture(json!({
        "items[:].name": "/data/[:]/name"
    }));

    let original = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
            NamedItem {
                name: "charlie".into(),
            },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: NamedContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// Additional stitch edge cases
// ============================================================================

#[test]
fn stitch_single_element_with_field_extraction() {
    let suture = request_suture(json!({
        "items[:].name": "/names/[:]"
    }));

    let input = NamedContainer {
        items: vec![NamedItem {
            name: "only".into(),
        }],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "names": ["only"] }));
}

#[test]
fn stitch_last_element_field() {
    let suture = request_suture(json!({
        "items[-1].name": "/last_name"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
            NamedItem {
                name: "charlie".into(),
            },
        ],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "last_name": "charlie" }));
}

#[test]
fn stitch_write_to_fixed_index_target() {
    let suture = request_suture(json!({
        "items[0]": "/items/[0]"
    }));

    let input = SimpleContainer {
        items: vec![json!(42), json!(99)],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "items": [42] }));
}

#[test]
fn stitch_multiple_fixed_indices() {
    let suture = request_suture(json!({
        "items[0]": "/first",
        "items[1]": "/second"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first": "a", "second": "b" }));
}

// ============================================================================
// Additional unstitch edge cases
// ============================================================================

#[test]
fn unstitch_empty_json_array() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));

    let input = json!({ "items": [] });

    let result: SimpleContainer = suture.unstitch(&input).unwrap();
    assert_eq!(result, SimpleContainer { items: vec![] });
}

#[test]
fn unstitch_single_element_json_array() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));

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
fn unstitch_fan_out_targets() {
    // Fan-out: same source written to multiple targets.
    // For unstitch, the reverse walk reads from each target independently.
    let suture = request_suture(json!({
        "items[:].name": ["/names/[:]", "/labels/[:]"]
    }));

    // Both targets hold the same data — unstitch reads from both and writes to same location.
    let input = json!({
        "names": ["alice", "bob"],
        "labels": ["alice", "bob"]
    });

    let result: NamedContainer = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        NamedContainer {
            items: vec![
                NamedItem {
                    name: "alice".into()
                },
                NamedItem { name: "bob".into() },
            ],
        }
    );
}

// ============================================================================
// Round-trip with various slice patterns
// ============================================================================

#[test]
fn round_trip_step_iteration() {
    let suture = request_suture(json!({
        "items[::2].name": "/even_names/[:]"
    }));

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

    // Unstitch only reconstructs the mapped elements.
    // Original had 5 items, but only 3 are mapped (indices 0, 2, 4).
    // Unstitch produces items with 3 elements (indices from read side).
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
fn round_trip_negative_start() {
    let suture = request_suture(json!({
        "items[-2:]": "/tail/[:]"
    }));

    let original = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")],
    };

    let stitched = suture.stitch(&original).unwrap();
    assert_eq!(stitched, json!({ "tail": ["d", "e"] }));

    let unstitched: SimpleContainer = suture.unstitch(&stitched).unwrap();
    assert_eq!(
        unstitched,
        SimpleContainer {
            items: vec![json!("d"), json!("e")],
        }
    );
}

#[test]
fn round_trip_negative_end() {
    let suture = request_suture(json!({
        "items[:-2]": "/head/[:]"
    }));

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
// Stitch: verify exact output structure for complex targets
// ============================================================================

#[test]
fn stitch_field_to_deeply_nested_json_path() {
    let suture = request_suture(json!({
        "items[:].name": "/response/data/names/[:]"
    }));

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
    let suture = request_suture(json!({
        "items[:].name": "/names/[:]",
        "items[:].value": "/values/[:]"
    }));

    let input = Container {
        items: vec![
            Item {
                name: "a".into(),
                value: 1,
                id: 10,
            },
            Item {
                name: "b".into(),
                value: 2,
                id: 20,
            },
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
// Unstitch: verify struct reconstruction for complex cases
// ============================================================================

#[test]
fn unstitch_multiple_fields_to_iterated_elements() {
    let suture = request_suture(json!({
        "items[:].name": "/names/[:]",
        "items[:].value": "/values/[:]"
    }));

    let input = json!({
        "names": ["a", "b"],
        "values": [1, 2]
    });

    let result: ItemsWithNameValue = suture.unstitch(&input).unwrap();
    assert_eq!(
        result,
        ItemsWithNameValue {
            items: vec![
                NameValue {
                    name: "a".into(),
                    value: 1
                },
                NameValue {
                    name: "b".into(),
                    value: 2
                },
            ],
        }
    );
}

#[test]
fn round_trip_multiple_fields_from_iterated_elements() {
    let suture = request_suture(json!({
        "items[:].name": "/names/[:]",
        "items[:].value": "/values/[:]"
    }));

    let original = ItemsWithNameValue {
        items: vec![
            NameValue {
                name: "alice".into(),
                value: 100,
            },
            NameValue {
                name: "bob".into(),
                value: 200,
            },
            NameValue {
                name: "charlie".into(),
                value: 300,
            },
        ],
    };

    let stitched = suture.stitch(&original).unwrap();
    let unstitched: ItemsWithNameValue = suture.unstitch(&stitched).unwrap();
    assert_eq!(original, unstitched);
}

// ============================================================================
// Stitch with fixed write-side index
// ============================================================================

#[test]
fn stitch_iterate_source_write_to_fixed_index() {
    // All source items' names go to /data/[0] — last one wins (overwrite)
    let suture = request_suture(json!({
        "items[:].name": "/data/[0]"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
            NamedItem { name: "bob".into() },
            NamedItem {
                name: "charlie".into(),
            },
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
fn request_suture_is_request_direction() {
    let suture = request_suture(json!({
        "items[:]": "/items/[:]"
    }));
    assert!(suture.is_request());
    assert!(!suture.is_response());
}

// ============================================================================
// Stitch: items with nested objects as values
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ComplexItem {
    label: String,
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sutures::Seam)]
struct ComplexContainer {
    items: Vec<ComplexItem>,
}

#[test]
fn stitch_iterate_complex_objects() {
    let suture = request_suture(json!({
        "items[:].label": "/labels/[:]"
    }));

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
    let suture = request_suture(json!({
        "items[:]": "/data/[:]"
    }));

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
// Stitch: mixed fixed and iterated in same suture
// ============================================================================

#[test]
fn stitch_mixed_fixed_index_and_iteration() {
    let suture = request_suture(json!({
        "items[0].name": "/first_name",
        "items[:].name": "/all_names/[:]"
    }));

    let input = NamedContainer {
        items: vec![
            NamedItem {
                name: "alice".into(),
            },
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

// ============================================================================
// Stitch: verify [0:1] behaves like [0] for single-element extraction
// ============================================================================

#[test]
fn stitch_slice_zero_to_one_like_single_index() {
    let suture = request_suture(json!({
        "items[0:1]": "/first/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first": ["a"] }));
}

// ============================================================================
// Stitch: large step that skips most elements
// ============================================================================

#[test]
fn stitch_large_step() {
    let suture = request_suture(json!({
        "items[::5]": "/sparse/[:]"
    }));

    let input = SimpleContainer {
        items: (0..12).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::5] on 12 elements: indices 0, 5, 10
    assert_eq!(result, json!({ "sparse": [0, 5, 10] }));
}

// ============================================================================
// Stitch: reverse with step > 1
// ============================================================================

#[test]
fn stitch_reverse_with_step_three() {
    let suture = request_suture(json!({
        "items[::-3]": "/rev3/[:]"
    }));

    let input = SimpleContainer {
        items: (0..10).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [::-3] on 10: start=9, step=-3 -> 9, 6, 3, 0
    assert_eq!(result, json!({ "rev3": [9, 6, 3, 0] }));
}

// ============================================================================
// Stitch: range with step and negative bounds
// ============================================================================

#[test]
fn stitch_range_negative_start_with_step() {
    let suture = request_suture(json!({
        "items[-4::2]": "/tail_stepped/[:]"
    }));

    let input = SimpleContainer {
        items: (0..8).map(|i| json!(i)).collect(),
    };

    let result = suture.stitch(&input).unwrap();
    // [-4::2] on 8: start = 8 + (-4) = 4, end = 8, step = 2
    // Indices: 4, 6
    assert_eq!(result, json!({ "tail_stepped": [4, 6] }));
}

// ============================================================================
// Unstitch: whole-object round trip
// ============================================================================

#[test]
fn round_trip_whole_object_iteration() {
    let suture = request_suture(json!({
        "items[:]": "/data/[:]"
    }));

    let original = SimpleContainer {
        items: vec![json!({"a": 1}), json!({"b": 2}), json!({"c": 3})],
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
// Stitch: verify parse_first helper works with full JSON doc
// ============================================================================

#[test]
fn parse_first_helper_works() {
    let suture = parse_first(
        r#"{
            "name": "test_doc",
            "suture_sets": [{
                "name": "set1",
                "capture": "request",
                "sutures": [{ "items[:]": "/items/[:]" }]
            }]
        }"#,
    );
    assert!(suture.is_request());
    assert_eq!(suture.name(), "set1");
}

// ============================================================================
// Unstitch: nested object syntax round trip
// ============================================================================

#[test]
fn unstitch_nested_object_syntax() {
    let suture = request_suture(json!({
        "items[:]": {
            "name": "/data/[:]/name",
            "value": "/data/[:]/value"
        }
    }));

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
                NameValue {
                    name: "x".into(),
                    value: 1
                },
                NameValue {
                    name: "y".into(),
                    value: 2
                },
                NameValue {
                    name: "z".into(),
                    value: 3
                },
            ],
        }
    );
}

// ============================================================================
// Stitch: items[2:2] (empty range) produces nothing
// ============================================================================

#[test]
fn stitch_empty_range_produces_nothing() {
    let suture = request_suture(json!({
        "items[2:2]": "/empty/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!(1), json!(2), json!(3)],
    };

    let result = suture.stitch(&input).unwrap();
    // [2:2] -> no indices
    assert_eq!(result, json!({}));
}

// ============================================================================
// Stitch: items[0:] covers all elements (open-ended)
// ============================================================================

#[test]
fn stitch_open_ended_from_zero() {
    let suture = request_suture(json!({
        "items[0:]": "/all/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "all": ["a", "b", "c"] }));
}

// ============================================================================
// Stitch: items[:2] covers first two elements
// ============================================================================

#[test]
fn stitch_first_two_elements() {
    let suture = request_suture(json!({
        "items[:2]": "/first_two/[:]"
    }));

    let input = SimpleContainer {
        items: vec![json!("a"), json!("b"), json!("c"), json!("d")],
    };

    let result = suture.stitch(&input).unwrap();
    assert_eq!(result, json!({ "first_two": ["a", "b"] }));
}
