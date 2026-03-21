use std::{borrow::Cow, collections::HashMap};

use serde_json::Value;

/// A validated suture: one suture_set, compiled and ready to operate.
///
/// Use via [`Stitch`](super::Stitch) (Value layer) or [`Knit`](super::Knit) (streaming layer).
///
/// All string fields use `Cow<'static, str>`:
/// - **Compile-time** (`sutures_comptime`): `Cow::Borrowed(&'static str)`: zero allocation.
/// - **Runtime** (`sutures`): `Cow::Owned(String)`: standard heap allocation.
#[derive(Debug)]
pub struct Suture {
    pub(crate) id: Option<Cow<'static, str>>,
    pub(crate) name: Cow<'static, str>,
    pub(crate) description: Option<Cow<'static, str>>,
    pub(crate) version: Option<Cow<'static, str>>,
    pub(crate) binding: Bindings,
    pub(crate) constants: Vec<(Cow<'static, str>, Value)>,
}

/// Compiled mapping data, varying by capture direction.
///
/// Both variants hold a flat list of source → destination mappings plus
/// any constant injections. The `Binding` variant determines the
/// semantic direction:
///
/// - **Request**: source paths are struct-side, dest paths are JSON-side.
/// - **Response**: source paths are JSON-side, dest paths are struct-side.
///
/// Nested DSL objects are flattened during compilation by prepending the
/// parent path, so the mapping list is always flat.
#[derive(Debug)]
pub enum Bindings {
    /// struct → JSON (serialization path).
    ///
    /// Key: struct field name (from `Seam::fields()`).
    /// Value: strategy describing where to write and whether to recurse.
    ///
    /// At execution time, walk the struct via `Seam`, look up each field
    /// name in O(1), and apply the corresponding strategy.
    Request(TrieNode),
    /// JSON → struct (deserialization path).
    ///
    /// Key: JSON key (from the incoming payload).
    /// Value: strategy describing where to write and whether to recurse.
    ///
    /// At execution time, walk the JSON value's keys, look up each in O(1),
    /// and apply the corresponding strategy to populate the struct.
    Response(TrieNode),
}

#[derive(Debug)]
pub struct TrieNode {
    key: Cow<'static, str>,
    tasks: Vec<(Option<TrieNode>, BindingTaskType)>,
}

#[derive(Debug, Clone)]
pub enum BindingTaskType {
    /// serde on target
    Direct(Cow<'static, str>),
    /// Iterate on current item,
    Iterate {
        target: Cow<'static, str>,
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },
    /// Iterate Regex on current item,
    IteratePattern {
        pattern: Cow<'static, str>,
        target: Cow<'static, str>,
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

impl Suture {
    /// Returns the suture's id, if one was set.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns the suture's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description, if one was set.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the version, if one was set.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Returns a reference to the compiled binding.
    pub fn binding(&self) -> &Bindings {
        &self.binding
    }

    /// True when the binding captures request direction (struct → JSON).
    pub fn is_request(&self) -> bool {
        matches!(self.binding, Bindings::Request { .. })
    }

    /// True when the binding captures response direction (JSON → struct).
    pub fn is_response(&self) -> bool {
        matches!(self.binding, Bindings::Response(_))
    }
}

impl std::fmt::Display for Suture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.id, &self.version) {
            (Some(id), Some(v)) => write!(f, "{}@{} ({})", self.name, v, id),
            (Some(id), None) => write!(f, "{} ({})", self.name, id),
            (None, Some(v)) => write!(f, "{}@{}", self.name, v),
            (None, None) => write!(f, "{}", self.name),
        }
    }
}
