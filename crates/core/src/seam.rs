/// A field in a Seam described struct.
///
/// `children: None` → leaf, serde handles serialization/deserialization.
/// `children: Some(f)` → nested Seam struct, call `f()` to recurse.
#[derive(Debug, Clone, Copy)]
pub struct SeamField {
    pub name: &'static str,
    pub children: Option<fn() -> &'static [SeamField]>,
}

/// Structural descriptor for a named-field struct.
///
/// Implemented by `#[derive(Seam)]`.
/// Returns a static, ordered slice of fields — zero heap, fully compile-time.
pub trait Seam {
    fn fields() -> &'static [SeamField];
}
