/// The `Seam` trait is implemented by `#[derive(Seam)]` on user structs.
/// It declares what fields the struct has and how to construct/deconstruct it.
pub trait Seam: Sized {}
