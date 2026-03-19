pub mod error;
pub mod knit;
pub mod seam;
pub mod stitch;
pub mod v1;

// Re-export derive macros
#[cfg(feature = "derive")]
pub use sutures_derive::Seam;

// Re-export Seam types at crate root
pub use seam::Seam;

pub use knit::Knit;
pub use stitch::Stitch;

// Alias v1 as the default
pub use v1::{load, parse};
