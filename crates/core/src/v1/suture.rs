use std::borrow::Cow;

use super::schema::ConstantValue;

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
    pub(crate) binding: Binding,
}

/// Compiled mapping data, varying by capture direction.
///
/// The two variants use fundamentally different data structures because
/// serde's two directions have different drivers:
///
/// - **Serialization** (Request): the **struct** drives: it pushes fields
///   in declaration order via `serialize_field`. A flat `Vec<FieldMapping>`
///   suffices; the struct's own nesting IS the trie.
///
/// - **Deserialization** (Response): the **input JSON** drives: `MapAccess`
///   yields keys from the wire in arbitrary order. A `PathNode` trie keyed
///   by JSON keys routes each key to the correct struct field.
#[derive(Debug)]
pub enum Binding {
    /// struct → JSON (serialization path).
    ///
    /// Flat field list walked in struct declaration order.
    /// Serde recurses into nested structs automatically.
    Request {
        /// Struct fields → JSON targets, in declaration order.
        fields: Vec<StructFieldProxy>,
        /// Extra key-value pairs injected into the JSON output
        /// (not backed by any struct field). Written in `SerializeStruct::end()`.
        constants: Vec<(Cow<'static, str>, ConstantValue)>,
    },
    /// JSON → struct (deserialization path).
    ///
    /// Trie keyed by JSON object keys at each nesting depth.
    /// `MapAccess::next_key()` yields keys, trie routes them.
    Response {
        /// JSON keys → struct field resolution.
        trie: JsonPathProxy,
        /// Struct fields filled with fixed values when missing from
        /// the JSON input (resolved via serde's `expr_is_missing` path).
        constants: Vec<(Cow<'static, str>, ConstantValue)>,
    },
}

// ---------------------------------------------------------------------------
// Request: flat field mappings (struct drives)
// ---------------------------------------------------------------------------

/// One struct field's mapping at the current nesting level.
#[derive(Debug)]
pub struct StructFieldProxy {
    /// Struct field name (the `key` serde passes to `serialize_field`).
    pub(crate) name: Cow<'static, str>,
    /// Where to write the value in the JSON output.
    pub(crate) action: StructFieldProxyAction,
}

/// What to do with a struct field's value during serialization.
#[derive(Debug)]
pub enum StructFieldProxyAction {
    /// Write to a single JSON pointer path.
    Write(Cow<'static, str>),
    /// Fan-out: write the same value to multiple JSON pointer paths.
    WriteFanOut(Vec<Cow<'static, str>>),
    /// Nested struct -> push a JSON path prefix, then serde recurses
    /// into the inner struct's `Serialize` impl with sub-mappings.
    Descend(Cow<'static, str>, Vec<StructFieldProxy>),
}

// ---------------------------------------------------------------------------
// Response: path trie (JSON drives)
// ---------------------------------------------------------------------------

/// A node in the compiled path trie for deserialization.
///
/// Children are stored as a `Vec` for linear scan — JSON objects
/// typically have 5–20 keys, where linear beats hashing.
#[derive(Debug)]
pub struct JsonPathProxy {
    pub(crate) children: Vec<(Cow<'static, str>, JsonPathProxyAction)>,
}

/// What to do when a JSON key matches during deserialization.
#[derive(Debug)]
pub enum JsonPathProxyAction {
    /// Extract the value and assign it to a struct field.
    Extract(Cow<'static, str>),
    /// Fan-in: extract the value and assign to multiple struct fields.
    ExtractMany(Vec<Cow<'static, str>>),
    /// Descend into a nested JSON object -> contains more mappings.
    Descend(JsonPathProxy),
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
    pub fn binding(&self) -> &Binding {
        &self.binding
    }

    /// True when the binding captures request direction (struct → JSON).
    pub fn is_request(&self) -> bool {
        matches!(self.binding, Binding::Request { .. })
    }

    /// True when the binding captures response direction (JSON → struct).
    pub fn is_response(&self) -> bool {
        matches!(self.binding, Binding::Response { .. })
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
