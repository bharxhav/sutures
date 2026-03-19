use crate::error::Error;
use crate::seam::Seam;

/// Value-layer operations on a compiled [`Suture`](crate::v1::Suture).
///
/// `stitch` builds a `serde_json::Value` tree from a struct.
/// `unstitch` extracts a struct from a `serde_json::Value` tree.
///
/// Both walk the compiled path tree and index into Value directly —
/// no serde Deserialize/Serialize machinery involved.
pub trait Stitch {
    /// Request direction: struct → Value.
    /// Walks the compiled tree, reads struct fields via Seam, places values at JSON paths.
    fn stitch<T: Seam>(&self, input: &T) -> Result<serde_json::Value, Error>;

    /// Response direction: Value → struct.
    /// Walks the compiled tree, extracts values from JSON paths, constructs struct via Seam.
    fn unstitch<T: Seam>(&self, input: &serde_json::Value) -> Result<T, Error>;
}
