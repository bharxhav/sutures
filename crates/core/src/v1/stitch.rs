// Stitch (Value-layer) implementation for v1::Suture.
//
// Forward walk: traverse the compiled trie against a source Value, placing
// extracted values at the flat target paths in the output.
//
// Reverse walk: read from flat target paths in the source, write into the
// output following the trie structure. Used when the trie describes the
// *opposite* side from the one being read.

use std::borrow::Cow;

use regex::Regex;
use serde_json::{Map, Value};

use super::suture::{BindingTaskType, Bindings, ConstantValue, Suture, TrieNode};
use crate::error::Error;
use crate::seam::Seam;
use crate::stitch::Stitch;

// ===========================================================================
// Trait implementation
// ===========================================================================

impl Stitch for Suture {
    fn stitch<T: Seam + serde::Serialize>(&self, input: &T) -> Result<Value, Error> {
        let src = serde_json::to_value(input)?;
        let mut dst = Value::Object(Map::new());

        match &self.binding {
            Bindings::Request(root) => {
                // Forward: walk struct (trie), extract values, place at JSON targets.
                emit_root_targets(root, &src, &mut dst, &[], '/')?;
                forward_walk(root, &src, &mut dst, &mut Vec::new(), '/')?;
                inject_constants(&mut dst, &self.constants, '/')?;
            }
            Bindings::Response(root) => {
                // Reverse: read struct via struct-side targets, write JSON via trie.
                let leaves = collect_leaves(root, &[]);
                reverse_walk(&leaves, &src, &mut dst, '.')?;
            }
        }

        Ok(dst)
    }

    fn unstitch<T: Seam + serde::de::DeserializeOwned>(
        &self,
        input: &Value,
    ) -> Result<T, Error> {
        let mut dst = Value::Object(Map::new());

        match &self.binding {
            Bindings::Request(root) => {
                // Reverse: read JSON via JSON-side targets, write struct via trie.
                let leaves = collect_leaves(root, &[]);
                reverse_walk(&leaves, input, &mut dst, '/').map_err(stitch_to_unstitch)?;
            }
            Bindings::Response(root) => {
                // Forward: walk JSON (trie), extract values, place at struct targets.
                emit_root_targets(root, input, &mut dst, &[], '.')
                    .map_err(stitch_to_unstitch)?;
                forward_walk(root, input, &mut dst, &mut Vec::new(), '.')
                    .map_err(stitch_to_unstitch)?;
                inject_constants(&mut dst, &self.constants, '.')
                    .map_err(stitch_to_unstitch)?;
            }
        }

        serde_json::from_value(dst).map_err(Error::Parse)
    }
}

fn stitch_to_unstitch(e: Error) -> Error {
    match e {
        Error::Stitch(msg) => Error::Unstitch(msg),
        other => other,
    }
}

/// Emit a root node's own targets (handles the edge case where the root
/// itself carries targets, e.g. mapping `/` in a response suture).
fn emit_root_targets(
    root: &TrieNode,
    src: &Value,
    dst: &mut Value,
    indices: &[usize],
    target_sep: char,
) -> Result<(), Error> {
    for target in &root.targets {
        set_at_path(dst, target, src.clone(), indices, target_sep)?;
    }
    Ok(())
}

// ===========================================================================
// Forward walk — traverse the trie against the source value
// ===========================================================================

fn forward_walk(
    node: &TrieNode,
    src: &Value,
    dst: &mut Value,
    indices: &mut Vec<usize>,
    target_sep: char,
) -> Result<(), Error> {
    for child in &node.children {
        match &child.binding {
            BindingTaskType::Direct => {
                let val = match src.get(&*child.key) {
                    Some(v) => v,
                    None => continue,
                };
                for target in &child.targets {
                    set_at_path(dst, target, val.clone(), indices, target_sep)?;
                }
                if !child.children.is_empty() {
                    forward_walk(child, val, dst, indices, target_sep)?;
                }
            }
            BindingTaskType::Iterate { start, end, step } => {
                // Empty key means standalone bracket (e.g. [:]  as its own
                // path segment in `/items/[:]`). Operate on src directly.
                let arr_val = if child.key.is_empty() {
                    Some(src)
                } else {
                    src.get(&*child.key)
                };
                let arr = match arr_val.and_then(Value::as_array) {
                    Some(a) => a,
                    None => continue,
                };
                // Push the *enumeration* index (0, 1, 2…) so the write side
                // always produces a dense array, regardless of source slice.
                for (enum_idx, src_idx) in
                    slice_indices(arr.len(), *start, *end, *step).into_iter().enumerate()
                {
                    let elem = &arr[src_idx];
                    indices.push(enum_idx);
                    for target in &child.targets {
                        set_at_path(dst, target, elem.clone(), indices, target_sep)?;
                    }
                    if !child.children.is_empty() {
                        forward_walk(child, elem, dst, indices, target_sep)?;
                    }
                    indices.pop();
                }
            }
            BindingTaskType::IteratePattern {
                pattern,
                start,
                end,
                step,
            } => {
                // IteratePattern matches keys at the *current* source level.
                let obj = match src.as_object() {
                    Some(o) => o,
                    None => continue,
                };
                let re = compile_regex(pattern)?;
                let matching: Vec<_> = obj.iter().filter(|(k, _)| re.is_match(k)).collect();
                for (enum_idx, src_idx) in
                    slice_indices(matching.len(), *start, *end, *step).into_iter().enumerate()
                {
                    let (_, val) = &matching[src_idx];
                    indices.push(enum_idx);
                    for target in &child.targets {
                        set_at_path(dst, target, (*val).clone(), indices, target_sep)?;
                    }
                    if !child.children.is_empty() {
                        forward_walk(child, val, dst, indices, target_sep)?;
                    }
                    indices.pop();
                }
            }
        }
    }
    Ok(())
}

// ===========================================================================
// Reverse walk — read from flat target paths, write via trie structure
// ===========================================================================

/// A single leaf mapping extracted from the compiled trie.
struct LeafMapping {
    /// Trie path segments from root to leaf (key + binding at each level).
    trie_segments: Vec<(Cow<'static, str>, BindingTaskType)>,
    /// The flat target path string.
    target: Cow<'static, str>,
}

/// Flatten the trie into leaf mappings for the reverse walk.
fn collect_leaves(
    node: &TrieNode,
    path: &[(Cow<'static, str>, BindingTaskType)],
) -> Vec<LeafMapping> {
    let mut result = Vec::new();

    // Include this node's own targets.
    for target in &node.targets {
        result.push(LeafMapping {
            trie_segments: path.to_vec(),
            target: target.clone(),
        });
    }

    for child in &node.children {
        let mut child_path = path.to_vec();
        child_path.push((child.key.clone(), child.binding.clone()));
        result.extend(collect_leaves(child, &child_path));
    }

    result
}

/// Reverse walk: for each leaf, read from source via target path, write to
/// output via trie path.
fn reverse_walk(
    leaves: &[LeafMapping],
    src: &Value,
    dst: &mut Value,
    read_sep: char,
) -> Result<(), Error> {
    for leaf in leaves {
        let read_segments = parse_target_segments(&leaf.target, read_sep);
        let mut results = Vec::new();
        extract_values(src, &read_segments, 0, &mut Vec::new(), &mut results);

        for (value, indices) in results {
            let mut iter_pos = 0;
            set_via_trie_path(dst, &leaf.trie_segments, value, &indices, &mut iter_pos)?;
        }
    }
    Ok(())
}

/// Walk the source value along parsed target segments, collecting leaf values
/// and the iteration indices accumulated along the way.
fn extract_values(
    src: &Value,
    segments: &[TargetSegment<'_>],
    seg_idx: usize,
    indices: &mut Vec<usize>,
    results: &mut Vec<(Value, Vec<usize>)>,
) {
    if seg_idx >= segments.len() {
        results.push((src.clone(), indices.clone()));
        return;
    }

    match &segments[seg_idx] {
        TargetSegment::Key(key) => {
            if let Some(val) = src.get(*key) {
                extract_values(val, segments, seg_idx + 1, indices, results);
            }
        }
        TargetSegment::Index(idx) => {
            let resolved = if *idx < 0 {
                src.as_array().map(|a| (a.len() as i64 + idx) as usize)
            } else {
                Some(*idx as usize)
            };
            if let Some(val) = resolved.and_then(|i| src.get(i)) {
                extract_values(val, segments, seg_idx + 1, indices, results);
            }
        }
        TargetSegment::Slice { start, end, step } => {
            if let Some(arr) = src.as_array() {
                for (enum_idx, src_idx) in
                    slice_indices(arr.len(), *start, *end, *step).into_iter().enumerate()
                {
                    indices.push(enum_idx);
                    extract_values(&arr[src_idx], segments, seg_idx + 1, indices, results);
                    indices.pop();
                }
            }
        }
    }
}

/// Set a value in the output by navigating the trie path segments.
fn set_via_trie_path(
    dst: &mut Value,
    segments: &[(Cow<'static, str>, BindingTaskType)],
    value: Value,
    indices: &[usize],
    iter_pos: &mut usize,
) -> Result<(), Error> {
    if segments.is_empty() {
        *dst = value;
        return Ok(());
    }

    let (key, binding) = &segments[0];
    let rest = &segments[1..];

    match binding {
        BindingTaskType::Direct => {
            ensure_object(dst);
            let obj = dst.as_object_mut().unwrap();
            let entry = obj.entry(&**key).or_insert(Value::Null);
            set_via_trie_path(entry, rest, value, indices, iter_pos)
        }
        BindingTaskType::Iterate { start, end, step } => {
            // Empty key = standalone bracket segment — iterate on dst directly.
            let arr_val = if key.is_empty() {
                dst
            } else {
                ensure_object(dst);
                let obj = dst.as_object_mut().unwrap();
                obj.entry(&**key).or_insert(Value::Null)
            };

            let idx = trie_write_index(start, end, step, indices, iter_pos)?;
            ensure_array_len(arr_val, idx + 1);
            let arr = arr_val.as_array_mut().unwrap();
            set_via_trie_path(&mut arr[idx], rest, value, indices, iter_pos)
        }
        BindingTaskType::IteratePattern { .. } => Err(Error::Stitch(
            "IteratePattern on the write side is not supported".into(),
        )),
    }
}

/// Determine the write index for an Iterate trie segment in reverse walk.
///
/// - Single fixed index `[N]` → use N directly.
/// - Full or range iteration → consume the next index from the read side.
fn trie_write_index(
    start: &Option<i64>,
    end: &Option<i64>,
    _step: &Option<i64>,
    indices: &[usize],
    iter_pos: &mut usize,
) -> Result<usize, Error> {
    // Single fixed index: Iterate { start: N, end: N+1, step: 1 }.
    if let (Some(s), Some(e)) = (start, end) {
        if *e == *s + 1 && *s >= 0 {
            return Ok(*s as usize);
        }
    }
    // Full or range iteration: consume from read-side indices.
    let idx = indices.get(*iter_pos).copied().ok_or_else(|| {
        Error::Stitch("iteration index mismatch in reverse walk".into())
    })?;
    *iter_pos += 1;
    Ok(idx)
}

// ===========================================================================
// Target path parsing
// ===========================================================================

enum TargetSegment<'a> {
    /// Named key — navigate into an object.
    Key(&'a str),
    /// Fixed array index (may be negative for read-side resolution).
    Index(i64),
    /// Pythonic slice — on the read side iterates, on the write side consumes
    /// an index from the iteration context.
    Slice {
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },
}

/// Parse a flat target path string into navigable segments.
///
/// JSON targets (sep = '/'): `/messages/[:]/content`
/// Struct targets (sep = '.'): `messages[:].content`
fn parse_target_segments<'a>(path: &'a str, sep: char) -> Vec<TargetSegment<'a>> {
    let path = if sep == '/' {
        path.strip_prefix('/').unwrap_or(path)
    } else {
        path
    };

    if path.is_empty() {
        return vec![];
    }

    let mut segments = Vec::new();

    for raw in path.split(sep).filter(|s| !s.is_empty()) {
        // Standalone bracket expression: [:], [0], [1:3]
        if raw.starts_with('[') {
            if let Some(inner) = raw.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                push_bracket_segment(&mut segments, inner);
            }
            continue;
        }

        // Key with optional bracket suffix: items[:], items[0]
        if let Some(bracket_pos) = raw.find('[') {
            let key = &raw[..bracket_pos];
            if !key.is_empty() {
                segments.push(TargetSegment::Key(key));
            }
            if let Some(end_bracket) = raw[bracket_pos..].find(']') {
                let inner = &raw[bracket_pos + 1..bracket_pos + end_bracket];
                push_bracket_segment(&mut segments, inner);
            }
        } else {
            segments.push(TargetSegment::Key(raw));
        }
    }

    segments
}

fn push_bracket_segment<'a>(segments: &mut Vec<TargetSegment<'a>>, inner: &str) {
    if inner.contains(':') {
        let parts: Vec<&str> = inner.split(':').collect();
        let parse_opt = |s: &str| -> Option<i64> {
            if s.is_empty() { None } else { s.parse().ok() }
        };
        let start = parts.first().and_then(|s| parse_opt(s));
        let end = parts.get(1).and_then(|s| parse_opt(s));
        let step = parts.get(2).and_then(|s| parse_opt(s));
        segments.push(TargetSegment::Slice { start, end, step });
    } else if let Ok(n) = inner.parse::<i64>() {
        segments.push(TargetSegment::Index(n));
    }
}

// ===========================================================================
// Path navigation helpers
// ===========================================================================

fn set_at_path(
    dst: &mut Value,
    path: &str,
    value: Value,
    indices: &[usize],
    sep: char,
) -> Result<(), Error> {
    let segments = parse_target_segments(path, sep);
    let mut iter_pos = 0;
    set_at_segments(dst, &segments, value, indices, &mut iter_pos)
}

fn set_at_segments(
    dst: &mut Value,
    segments: &[TargetSegment<'_>],
    value: Value,
    indices: &[usize],
    iter_pos: &mut usize,
) -> Result<(), Error> {
    if segments.is_empty() {
        *dst = value;
        return Ok(());
    }

    match &segments[0] {
        TargetSegment::Key(key) => {
            ensure_object(dst);
            let obj = dst.as_object_mut().unwrap();
            let entry = obj.entry(key.to_string()).or_insert(Value::Null);
            set_at_segments(entry, &segments[1..], value, indices, iter_pos)
        }
        TargetSegment::Index(idx) => {
            if *idx < 0 {
                return Err(Error::Stitch(
                    "negative index on write side is not supported".into(),
                ));
            }
            let idx = *idx as usize;
            ensure_array_len(dst, idx + 1);
            let arr = dst.as_array_mut().unwrap();
            set_at_segments(&mut arr[idx], &segments[1..], value, indices, iter_pos)
        }
        TargetSegment::Slice { .. } => {
            let idx = indices.get(*iter_pos).copied().ok_or_else(|| {
                Error::Stitch("more iterators in target path than iteration depth".into())
            })?;
            *iter_pos += 1;
            ensure_array_len(dst, idx + 1);
            let arr = dst.as_array_mut().unwrap();
            set_at_segments(&mut arr[idx], &segments[1..], value, indices, iter_pos)
        }
    }
}

// ===========================================================================
// Constants
// ===========================================================================

fn inject_constants(
    dst: &mut Value,
    constants: &[(Cow<'static, str>, ConstantValue)],
    sep: char,
) -> Result<(), Error> {
    for (path, cval) in constants {
        let value = constant_to_value(cval);
        set_at_path(dst, path, value, &[], sep)?;
    }
    Ok(())
}

fn constant_to_value(cv: &ConstantValue) -> Value {
    match cv {
        ConstantValue::Null => Value::Null,
        ConstantValue::Bool(b) => Value::Bool(*b),
        ConstantValue::Int(i) => Value::Number((*i).into()),
        ConstantValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ConstantValue::String(s) => Value::String(s.to_string()),
    }
}

// ===========================================================================
// Pythonic slice indices
// ===========================================================================

/// Compute concrete indices for a Python-style slice over a collection of
/// length `len`. Mirrors `slice.indices(len)` in CPython.
fn slice_indices(
    len: usize,
    start: Option<i64>,
    end: Option<i64>,
    step: Option<i64>,
) -> Vec<usize> {
    if len == 0 {
        return vec![];
    }

    let len_i = len as i64;
    let step = step.unwrap_or(1);
    if step == 0 {
        return vec![];
    }

    // Special case: single index [N] — compiled as (N, N+1, 1).
    // Handles negative indices correctly (e.g. [-1] → last element).
    if step == 1 {
        if let (Some(s), Some(e)) = (start, end) {
            if e == s + 1 {
                let resolved = if s < 0 { s + len_i } else { s };
                return if resolved >= 0 && resolved < len_i {
                    vec![resolved as usize]
                } else {
                    vec![]
                };
            }
        }
    }

    let resolve = |val: i64| -> i64 {
        if val < 0 { val + len_i } else { val }
    };

    let mut result = Vec::new();

    if step > 0 {
        let s = resolve(start.unwrap_or(0)).clamp(0, len_i);
        let e = resolve(end.unwrap_or(len_i)).clamp(0, len_i);
        let mut i = s;
        while i < e {
            result.push(i as usize);
            i += step;
        }
    } else {
        let s = resolve(start.unwrap_or(len_i - 1)).clamp(-1, len_i - 1);
        let e = resolve(end.unwrap_or(-len_i - 1)).clamp(-1, len_i - 1);
        let mut i = s;
        while i > e {
            result.push(i as usize);
            i += step;
        }
    }

    result
}

// ===========================================================================
// Misc helpers
// ===========================================================================

fn ensure_object(val: &mut Value) {
    if !val.is_object() {
        *val = Value::Object(Map::new());
    }
}

fn ensure_array_len(val: &mut Value, min_len: usize) {
    if !val.is_array() {
        *val = Value::Array(Vec::new());
    }
    let arr = val.as_array_mut().unwrap();
    while arr.len() < min_len {
        arr.push(Value::Null);
    }
}

fn compile_regex(pattern: &str) -> Result<Regex, Error> {
    Regex::new(&format!("^{pattern}$"))
        .map_err(|e| Error::Stitch(format!("invalid regex: {e}")))
}
