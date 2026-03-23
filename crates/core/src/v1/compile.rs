use std::borrow::Cow;
use std::collections::VecDeque;

use super::schema::{Direction, RawSutureSet};
use super::suture::{BindingTaskType, Bindings, Suture, TrieNode};
use super::validate::{validate_constant, validate_key, validate_terminal};
use crate::error::Error;

// ===========================================================================
// Entry point
// ===========================================================================

/// Compile a single suture_set into a ready-to-use `Suture`.
pub(super) fn compile(set: RawSutureSet) -> Result<Suture, Error> {
    if set.name.is_empty() {
        return Err(Error::Suture("suture set name must not be empty".into()));
    }
    let (binding, constants) = compile_bindings(&set)?;

    Ok(Suture {
        id: set.id.map(Cow::Owned),
        name: Cow::Owned(set.name),
        description: set.description.map(Cow::Owned),
        version: set.version.map(Cow::Owned),
        binding,
        constants,
    })
}

type CompilationResult = (Bindings, Vec<(Cow<'static, str>, serde_json::Value)>);

// ===========================================================================
// Core loop
// ===========================================================================

/// One-pass compilation of a suture set into a binding trie + constants.
///
/// Iterates the suture set exactly once. For each entry the processing
/// pipeline runs as a flat state machine:
///
/// ```text
///   Classify → Validate → Split → Build chain → Merge into trie
/// ```
fn compile_bindings(suture_set: &RawSutureSet) -> Result<CompilationResult, Error> {
    let sep = terminal_separator(&suture_set.capture_direction);

    let mut root = TrieNode {
        key: Cow::Owned(suture_set.name.clone()),
        binding: BindingTaskType::Direct,
        targets: vec![],
        children: vec![],
    };
    let mut constants: Vec<(Cow<'static, str>, serde_json::Value)> = Vec::new();

    for (i, raw) in suture_set.sutures.iter().enumerate() {
        let ctx = format!("suture[{i}]");

        let obj = raw
            .as_object()
            .ok_or_else(|| Error::Suture(format!("{ctx}: must be an object")))?;

        // Work queue for flattening nested objects without recursion.
        // Uses VecDeque for FIFO ordering (process keys in declaration order).
        let mut queue: VecDeque<(String, serde_json::Value)> =
            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        while let Some((key, val)) = queue.pop_front() {
            // ── State 1: Classify ──
            if key == "_" {
                compile_constants(&val, &mut constants, &ctx, &suture_set.capture_direction)?;
                continue;
            }

            // ── State 2: Validate ──
            validate_key(&key, &suture_set.capture_direction)
                .map_err(|e| Error::Suture(format!("{ctx}: {e}")))?;

            // ── State 3: Split path ──
            // Strip exactly one leading '/' (response direction), then split
            // on the direction-dependent separator, respecting backtick regions.
            let path = key.strip_prefix('/').unwrap_or(&key);
            let segments: Vec<&str> = if path.is_empty() {
                vec![]
            } else {
                split_path(path, sep)
            };

            // ── State 4: Build chain + merge into trie ──
            match &val {
                serde_json::Value::String(target) => {
                    validate_terminal(target, &suture_set.capture_direction)
                        .map_err(|e| Error::Suture(format!("{ctx}.{key}: {e}")))?;

                    if segments.is_empty() {
                        root.targets.push(Cow::Owned(target.clone()));
                    } else {
                        let child = build_terminal_chain(&segments, target, &ctx)?;
                        merge_into(&mut root, child);
                    }
                }

                serde_json::Value::Array(arr) => {
                    if arr.is_empty() {
                        return Err(Error::Suture(format!(
                            "{ctx}.{key}: array value must not be empty"
                        )));
                    }
                    for item in arr {
                        let target = item.as_str().ok_or_else(|| {
                            Error::Suture(format!("{ctx}.{key}: array items must be strings"))
                        })?;
                        validate_terminal(target, &suture_set.capture_direction)
                            .map_err(|e| Error::Suture(format!("{ctx}.{key}: {e}")))?;

                        if segments.is_empty() {
                            root.targets.push(Cow::Owned(target.to_owned()));
                        } else {
                            let child = build_terminal_chain(&segments, target, &ctx)?;
                            merge_into(&mut root, child);
                        }
                    }
                }

                serde_json::Value::Object(obj) => {
                    for (child_key, child_val) in obj {
                        // Handle nested _ constants before path concatenation.
                        if child_key == "_" {
                            compile_constants(
                                child_val,
                                &mut constants,
                                &ctx,
                                &suture_set.capture_direction,
                            )?;
                        } else {
                            // Bracket-leading child keys (e.g. "[0].text") attach
                            // directly to the parent without an extra separator.
                            let full = if child_key.starts_with(sep) || child_key.starts_with('[') {
                                format!("{}{}", key, child_key)
                            } else {
                                format!("{}{}{}", key, sep, child_key)
                            };
                            queue.push_back((full, child_val.clone()));
                        }
                    }
                }

                _ => {
                    return Err(Error::Suture(format!(
                        "{ctx}.{key}: value must be a string, array, or object"
                    )));
                }
            }
        }
    }

    let binding = match suture_set.capture_direction {
        Direction::Request => Bindings::Request(root),
        Direction::Response => Bindings::Response(root),
    };

    Ok((binding, constants))
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Validate and collect constant entries from a `_` key.
fn compile_constants(
    val: &serde_json::Value,
    constants: &mut Vec<(Cow<'static, str>, serde_json::Value)>,
    ctx: &str,
    direction: &Direction,
) -> Result<(), Error> {
    let arr = val
        .as_array()
        .ok_or_else(|| Error::Suture(format!("{ctx}: '_' must be an array")))?;

    for entry in arr {
        let cobj = entry
            .as_object()
            .ok_or_else(|| Error::Suture(format!("{ctx}: constant entry must be an object")))?;
        if cobj.len() != 1 {
            return Err(Error::Suture(format!(
                "{ctx}: constant entry must have exactly one property"
            )));
        }
        let (terminal, v) = cobj.iter().next().unwrap();
        validate_terminal(terminal, direction)
            .map_err(|e| Error::Suture(format!("{ctx}: constant '{terminal}': {e}")))?;
        validate_constant(v).map_err(|e| Error::Suture(format!("{ctx}: {e}")))?;
        constants.push((Cow::Owned(terminal.clone()), v.clone()));
    }
    Ok(())
}

/// Build a chain of TrieNodes from path segments, bottom-up.
///
/// The leaf segment gets the RHS target. Intermediate segments get empty
/// targets — they exist only to describe traversal (Iterate, etc).
fn build_terminal_chain(segments: &[&str], target: &str, ctx: &str) -> Result<TrieNode, Error> {
    let leaf = segments
        .last()
        .ok_or_else(|| Error::Suture(format!("{ctx}: key produced no path segments")))?;

    let mut node = TrieNode {
        key: Cow::Owned(bare_ident(leaf).to_owned()),
        binding: resolve_binding(leaf),
        targets: vec![Cow::Owned(target.to_owned())],
        children: vec![],
    };

    // Walk backwards through intermediate segments, wrapping each as parent.
    for &seg in segments[..segments.len() - 1].iter().rev() {
        node = TrieNode {
            key: Cow::Owned(bare_ident(seg).to_owned()),
            binding: resolve_binding(seg),
            targets: vec![],
            children: vec![node],
        };
    }

    Ok(node)
}

/// Merge a child node into a parent's children list.
///
/// If a child with matching key and binding already exists, merge recursively.
/// Otherwise, append as a new child. Targets are accumulated (fan-out).
fn merge_into(parent: &mut TrieNode, child: TrieNode) {
    let existing = parent
        .children
        .iter_mut()
        .find(|c| c.key == child.key && c.binding == child.binding);

    if let Some(existing_node) = existing {
        existing_node.targets.extend(child.targets);
        for grandchild in child.children {
            merge_into(existing_node, grandchild);
        }
    } else {
        parent.children.push(child);
    }
}

// ===========================================================================
// Binding resolution
// ===========================================================================

/// Resolve a segment into a `BindingTaskType` by examining its syntax.
///
/// - Backtick regex `` `pat` `` → `IteratePattern`
/// - Brackets `[...]` (outside backticks) → `Iterate`
/// - Neither → `Direct`
fn resolve_binding(segment: &str) -> BindingTaskType {
    // ── Try regex pattern first ──
    if let Some(bt_start) = segment.find('`')
        && let Some(rel_end) = segment[bt_start + 1..].find('`')
    {
        let bt_end = rel_end + bt_start + 1;
        let pattern = segment[bt_start + 1..bt_end].to_owned();

        // Slice qualifier after closing backtick: `pat`[1:3]
        let after = &segment[bt_end + 1..];
        let (start, end, step) = if let Some(bs) = after.find('[') {
            if let Some(be) = after[bs..].find(']') {
                parse_range(&after[bs + 1..bs + be])
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        return BindingTaskType::IteratePattern {
            pattern: Cow::Owned(pattern),
            start,
            end,
            step,
        };
    }

    // ── Try bracket range (skip backtick regions) ──
    let bytes = segment.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'`' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'[' {
            let bracket_start = i + 1;
            i += 1;
            while i < bytes.len() && bytes[i] != b']' {
                i += 1;
            }
            if i < bytes.len() {
                let (start, end, step) = parse_range(&segment[bracket_start..i]);
                return BindingTaskType::Iterate { start, end, step };
            }
        }
        i += 1;
    }

    BindingTaskType::Direct
}

// ===========================================================================
// Reusable leaf utils
// ===========================================================================

/// Split a path on `sep`, respecting backtick-delimited regex regions.
///
/// Unlike `str::split`, this will not split inside backtick pairs, so a regex
/// pattern containing the separator character is kept intact.
fn split_path(path: &str, sep: char) -> Vec<&str> {
    let mut segments = Vec::new();
    let bytes = path.as_bytes();
    let sep_byte = sep as u8;
    let mut start = 0;
    let mut i = 0;
    let mut in_backtick = false;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            in_backtick = !in_backtick;
            i += 1;
            continue;
        }
        if !in_backtick && bytes[i] == sep_byte {
            segments.push(&path[start..i]);
            start = i + 1;
        }
        i += 1;
    }
    segments.push(&path[start..]);
    segments
}

/// Parse `start:end:step` from bracket content. Already validated.
fn parse_range(inner: &str) -> (Option<i64>, Option<i64>, Option<i64>) {
    let parts: Vec<&str> = inner.split(':').collect();
    match parts.len() {
        1 => {
            let n = parts[0].parse::<i64>().ok();
            (n, n.and_then(|v| v.checked_add(1)), Some(1))
        }
        2 => {
            let start = if parts[0].is_empty() {
                None
            } else {
                parts[0].parse().ok()
            };
            let end = if parts[1].is_empty() {
                None
            } else {
                parts[1].parse().ok()
            };
            (start, end, None)
        }
        3 => {
            let start = if parts[0].is_empty() {
                None
            } else {
                parts[0].parse().ok()
            };
            let end = if parts[1].is_empty() {
                None
            } else {
                parts[1].parse().ok()
            };
            let step = if parts[2].is_empty() {
                None
            } else {
                parts[2].parse().ok()
            };
            (start, end, step)
        }
        _ => (None, None, None),
    }
}

/// Return the bare identifier from a segment by stripping brackets and backticks.
///
/// `messages[:]`        → `messages`
/// `items[0]`           → `items`
/// `` `content_\\d+` `` → `content_\\d+` (the regex pattern)
fn bare_ident(key: &str) -> &str {
    // Regex segment — return the pattern as the identifier.
    if let Some(rest) = key.strip_prefix('`') {
        return rest.split('`').next().unwrap_or(rest);
    }
    key.split(['[', '`']).next().unwrap_or(key)
}

/// Direction-dependent path separator.
fn terminal_separator(dir: &Direction) -> char {
    match dir {
        Direction::Request => '.',
        Direction::Response => '/',
    }
}
