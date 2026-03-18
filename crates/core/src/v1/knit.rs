use crate::error::Error;
use crate::seam::Seam;

/// Streaming-layer operations on a compiled [`Suture`](super::Suture).
///
/// `knit` serializes a struct directly to bytes, guided by the compiled path tree.
/// `unknit` deserializes bytes directly into a struct via `DeserializeSeed`,
/// skipping the intermediate `serde_json::Value` allocation entirely.
///
/// Same compiled tree as [`Stitch`](super::Stitch), different traversal strategy.
pub trait Knit {
    /// Request direction: struct → bytes.
    /// Walks the compiled tree, reads struct fields via Seam, writes JSON directly.
    fn knit<T: Seam>(&self, input: &T) -> Result<Vec<u8>, Error>;

    /// Response direction: bytes → struct.
    /// Single-pass `DeserializeSeed` traversal — never materializes the full JSON tree.
    fn unknit<T: Seam>(&self, input: &[u8]) -> Result<T, Error>;
}
