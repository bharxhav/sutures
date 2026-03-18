/// Structural descriptor for a type (struct or enum).
///
/// Implemented by `#[derive(Seam)]`.
/// Returns a static, ordered slice of fields — zero heap, fully compile-time.
pub trait Seam {
    const IS_ENUM: bool;
    const IS_ANON_STRUCT: bool;
    fn fields() -> &'static [SeamField];
}

/// A single field or variant in a Seam-described type.
#[derive(Debug, Clone, Copy)]
pub struct SeamField {
    pub name: &'static str,
    pub ty: SeamFieldType,
}

/// What kind of value a field holds.
#[derive(Debug, Clone, Copy)]
pub enum SeamFieldType {
    /// Leaf Terminal: serde handles ser/deser (primitives, String, Vec<primitive>, Tuples, etc.)
    Terminal,
    /// Inline named fields (e.g. enum variant body). Not a standalone type.
    AnonymousStruct(fn() -> &'static [SeamField]),
    /// A named struct that implements Seam.
    Struct(fn() -> &'static [SeamField]),
    /// A named enum that implements Seam.
    Enum(fn() -> &'static [SeamField]),
}
