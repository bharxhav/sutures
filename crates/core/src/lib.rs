pub mod error;
pub mod seam;
pub mod v1;

// Re-export the derive macro
#[cfg(feature = "derive")]
pub use sutures_derive::Seam;

// Re-export Seam trait at crate root
pub use seam::Seam;

// Alias v1 as the default
pub use v1::load;
