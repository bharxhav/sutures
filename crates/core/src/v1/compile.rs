use std::borrow::Cow;

use super::schema::{Direction, RawSutureSet};
use super::suture::{BindingTaskType, Bindings, Suture, TrieNode};
use super::validate::{validate_constant, validate_key, validate_terminal};
use crate::error::Error;

// ===========================================================================
// Entry point
// ===========================================================================

/// Compile a single suture_set into a ready-to-use `Suture`.
pub(super) fn compile(set: RawSutureSet) -> Result<Suture, Error> {
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
        tasks: vec![],
    };
    let mut constants: Vec<(Cow<'static, str>, serde_json::Value)> = Vec::new();

    for (i, raw) in suture_set.sutures.iter().enumerate() {
        let ctx = format!("suture[{i}]");

        let obj = raw
            .as_object()
            .ok_or_else(|| Error::Suture(format!("{ctx}: must be an object")))?;

        // Work queue for flattening nested objects without recursion.
        let mut queue: Vec<(String, serde_json::Value)> =
            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        while let Some((key, val)) = queue.pop() {
            // ── State 1: Classify ──
            if key == "_" {
                compile_constants(&val, &mut constants, &ctx)?;
                continue;
            }

            // ── State 2: Validate ──
            validate_key(&key, &suture_set.capture_direction)
                .map_err(|e| Error::Suture(format!("{ctx}: {e}")))?;

            // ── State 3: Split path ──
            let segments: Vec<&str> = key
                .trim_start_matches('/')
                .split(sep)
                .filter(|s| !s.is_empty())
                .collect();

            // ── State 4: Build chain + merge into trie ──
            match &val {
                serde_json::Value::String(target) => {
                    validate_terminal(target, &suture_set.capture_direction)
                        .map_err(|e| Error::Suture(format!("{ctx}.{key}: {e}")))?;

                    let (child, binding) = build_terminal_chain(&segments, target, &ctx)?;

                    merge_terminal_chains(&mut root, child, binding);
                }

                serde_json::Value::Array(arr) => {
                    for item in arr {
                        let target = item.as_str().ok_or_else(|| {
                            Error::Suture(format!("{ctx}.{key}: array items must be strings"))
                        })?;
                        validate_terminal(target, &suture_set.capture_direction)
                            .map_err(|e| Error::Suture(format!("{ctx}.{key}: {e}")))?;

                        let (child, binding) = build_terminal_chain(&segments, target, &ctx)?;

                        merge_terminal_chains(&mut root, child, binding);
                    }
                }

                serde_json::Value::Object(obj) => {
                    for (child_key, child_val) in obj {
                        // Handle nested _ constants before path concatenation.
                        if child_key == "_" {
                            compile_constants(child_val, &mut constants, &ctx)?;
                        } else {
                            let full = if child_key.starts_with(sep) {
                                format!("{}{}", key, child_key)
                            } else {
                                format!("{}{}{}", key, sep, child_key)
                            };
                            queue.push((full, child_val.clone()));
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
        validate_constant(v).map_err(|e| Error::Suture(format!("{ctx}: {e}")))?;
        constants.push((Cow::Owned(terminal.clone()), v.clone()));
    }
    Ok(())
}

/// Build a chain of TrieNodes from path segments, bottom-up.
///
/// The leaf segment gets the RHS target in its binding. Intermediate segments
/// get empty targets — they exist only to describe traversal (Iterate, etc).
///
/// Returns `(first_child_node, binding_from_parent_to_first_child)`.
fn build_terminal_chain(
    segments: &[&str],
    target: &str,
    ctx: &str,
) -> Result<(TrieNode, BindingTaskType), Error> {
    let leaf = segments
        .last()
        .ok_or_else(|| Error::Suture(format!("{ctx}: key produced no path segments")))?;

    let leaf_binding = resolve_binding(leaf, target.to_owned());
    let mut child_node = TrieNode {
        key: Cow::Owned(bare_ident(leaf).to_owned()),
        tasks: vec![],
    };
    let mut child_binding = leaf_binding;

    // Walk backwards through intermediate segments
    for &seg in segments[..segments.len() - 1].iter().rev() {
        let node = TrieNode {
            key: Cow::Owned(bare_ident(seg).to_owned()),
            tasks: vec![(Some(child_node), child_binding)],
        };
        let binding = resolve_binding(seg, String::new());
        child_node = node;
        child_binding = binding;
    }

    Ok((child_node, child_binding))
}

/// Merge a child node + binding into a parent's task list.
///
/// If a task with a matching key and compatible binding already exists,
/// the child's tasks are recursively merged into the existing node.
/// Otherwise, a new task is appended.
fn merge_terminal_chains(parent: &mut TrieNode, child: TrieNode, binding: BindingTaskType) {
    let existing = parent.tasks.iter_mut().find(|(node_opt, b)| {
        if let Some(node) = node_opt {
            node.key == child.key && is_same_binding_task(b, &binding)
        } else {
            false
        }
    });

    if let Some((node_opt, _)) = existing {
        let existing_node = node_opt.as_mut().unwrap();
        for (grandchild, gb) in child.tasks {
            if let Some(gc) = grandchild {
                merge_terminal_chains(existing_node, gc, gb);
            } else {
                existing_node.tasks.push((None, gb));
            }
        }
    } else {
        parent.tasks.push((Some(child), binding));
    }
}

/// Check if two bindings represent the same traversal strategy.
///
/// - `Direct`: targets must match (different targets = separate fan-out tasks)
/// - `Iterate`: ranges must match (target ignored — intermediates have empty target)
/// - `IteratePattern`: pattern + ranges must match
fn is_same_binding_task(a: &BindingTaskType, b: &BindingTaskType) -> bool {
    match (a, b) {
        (BindingTaskType::Direct(t1), BindingTaskType::Direct(t2)) => t1 == t2,
        (
            BindingTaskType::Iterate {
                start: s1,
                end: e1,
                step: st1,
                ..
            },
            BindingTaskType::Iterate {
                start: s2,
                end: e2,
                step: st2,
                ..
            },
        ) => s1 == s2 && e1 == e2 && st1 == st2,
        (
            BindingTaskType::IteratePattern {
                pattern: p1,
                start: s1,
                end: e1,
                step: st1,
                ..
            },
            BindingTaskType::IteratePattern {
                pattern: p2,
                start: s2,
                end: e2,
                step: st2,
                ..
            },
        ) => p1 == p2 && s1 == s2 && e1 == e2 && st1 == st2,
        _ => false,
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
fn resolve_binding(segment: &str, target: String) -> BindingTaskType {
    // ── Try regex pattern first ──
    if let Some(bt_start) = segment.find('`') {
        if let Some(rel_end) = segment[bt_start + 1..].find('`') {
            let bt_end = rel_end + bt_start + 1;
            let pattern = segment[bt_start + 1..bt_end].to_owned();

            // Range qualifier after closing backtick: `pat`[1:3]
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
                target: Cow::Owned(target),
                start,
                end,
                step,
            };
        }
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
                return BindingTaskType::Iterate {
                    target: Cow::Owned(target),
                    start,
                    end,
                    step,
                };
            }
        }
        i += 1;
    }

    BindingTaskType::Direct(Cow::Owned(target))
}

// ===========================================================================
// Reusable leaf utils
// ===========================================================================

/// Parse `start:end:step` from bracket content. Already validated.
fn parse_range(inner: &str) -> (Option<i64>, Option<i64>, Option<i64>) {
    let parts: Vec<&str> = inner.split(':').collect();
    match parts.len() {
        1 => {
            let n = parts[0].parse::<i64>().ok();
            (n, n.map(|v| v + 1), Some(1))
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

/// Return the bare identifier from a key by stripping brackets and backticks.
///
/// `messages[:]`  → `messages`
/// `items[0]`     → `items`
/// `` `pat`[:] `` → `` `` (empty — regex keys are the entire key)
fn bare_ident(key: &str) -> &str {
    key.split(|c: char| c == '[' || c == '`')
        .next()
        .unwrap_or(key)
}

/// Direction-dependent path separator.
fn terminal_separator(dir: &Direction) -> char {
    match dir {
        Direction::Request => '.',
        Direction::Response => '/',
    }
}
