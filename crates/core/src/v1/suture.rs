use super::schema::Direction;

/// A validated suture — one suture_set, compiled and ready to operate.
///
/// Use via [`Stitch`](super::Stitch) (Value layer) or [`Knit`](super::Knit) (streaming layer).
#[derive(Debug)]
pub struct Suture {
    pub(crate) direction: Direction,
}
