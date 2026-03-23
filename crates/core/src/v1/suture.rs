use std::borrow::Cow;

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
    pub(crate) constants: Vec<(Cow<'static, str>, ConstantValue)>,
}

/// A scalar value for constant injection.
///
/// Replaces `serde_json::Value` in the compiled representation so that
/// downstream code (including proc-macro generated code) has no dependency
/// on `serde_json`.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Cow<'static, str>),
}

/// Compiled mapping trie, varying by capture direction.
///
/// The trie mirrors the source-side structure. Each node carries a binding
/// (Direct / Iterate / IteratePattern) that tells the runtime HOW to
/// traverse, and zero or more targets that tell it WHERE to write.
///
/// - **Request**: trie follows struct-side paths, targets are JSON-side.
/// - **Response**: trie follows JSON-side paths, targets are struct-side.
#[derive(Debug)]
pub enum Bindings {
    /// struct → JSON (serialization path).
    ///
    /// Walk the struct via `Seam`, match each field against the trie's
    /// children, and apply the corresponding binding strategy.
    Request(TrieNode),
    /// JSON → struct (deserialization path).
    ///
    /// Walk the JSON value's keys, match each against the trie's
    /// children, and apply the corresponding binding strategy.
    Response(TrieNode),
}

/// A node in the compiled mapping trie.
///
/// A node may have both `targets` and `children`. This means the node
/// captures the current value to the listed targets AND recurses into
/// children for sub-field extraction.
///
/// A single-index `[N]` is compiled as `Iterate { start: N, end: N+1, step: 1 }`
/// — a length-1 slice. The runtime treats it identically to a slice.
#[derive(Debug)]
pub struct TrieNode {
    pub(crate) key: Cow<'static, str>,
    pub(crate) binding: BindingTaskType,
    pub(crate) targets: Vec<Cow<'static, str>>,
    pub(crate) children: Vec<TrieNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingTaskType {
    /// Direct key lookup — serde on this field.
    Direct,
    /// Iterate array with optional pythonic slice range.
    Iterate {
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },
    /// Iterate keys matching a regex pattern with optional range.
    IteratePattern {
        pattern: Cow<'static, str>,
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

impl Suture {
    /// Returns the compiled constants list.
    pub fn constants(&self) -> &[(Cow<'static, str>, ConstantValue)] {
        &self.constants
    }
}

impl TrieNode {
    /// Returns the key for this trie node.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the binding task type.
    pub fn binding(&self) -> &BindingTaskType {
        &self.binding
    }

    /// Returns the target paths.
    pub fn targets(&self) -> &[Cow<'static, str>] {
        &self.targets
    }

    /// Returns the child nodes.
    pub fn children(&self) -> &[TrieNode] {
        &self.children
    }
}

impl Bindings {
    /// Returns a reference to the inner trie root.
    pub fn root(&self) -> &TrieNode {
        match self {
            Bindings::Request(root) | Bindings::Response(root) => root,
        }
    }
}

// ---------------------------------------------------------------------------
// Compile-time constructors (used by sutures_comptime proc-macro)
// ---------------------------------------------------------------------------

impl Suture {
    #[doc(hidden)]
    pub fn __comptime(
        id: Option<Cow<'static, str>>,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
        binding: Bindings,
        constants: Vec<(Cow<'static, str>, ConstantValue)>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            version,
            binding,
            constants,
        }
    }
}

impl TrieNode {
    #[doc(hidden)]
    pub fn __comptime(
        key: Cow<'static, str>,
        binding: BindingTaskType,
        targets: Vec<Cow<'static, str>>,
        children: Vec<TrieNode>,
    ) -> Self {
        Self {
            key,
            binding,
            targets,
            children,
        }
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
