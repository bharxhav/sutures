mod compile;
mod knit;
mod schema;
mod stitch;
mod suture;

pub use knit::*;
pub use schema::Direction;
pub use stitch::*;
pub use suture::*;

use crate::error::Error;
use compile::{compile_suture_set, parse_schema};
use serde_json::Value;

/// Parse a `.sutures.json` string into compiled sutures.
///
/// Deserializes the JSON, then compiles each suture_set independently.
/// Returns one `Result<Suture>` per suture_set in the file.
pub fn parse(input: &str) -> Result<Vec<Result<Suture, Error>>, Error> {
    let value: Value = serde_json::from_str(input).map_err(Error::Parse)?;
    load(value)
}

/// Compile a sutures manifest from a JSON value.
/// Returns one `Result<Suture>` per suture_set.
pub fn load(input: Value) -> Result<Vec<Result<Suture, Error>>, Error> {
    let schema = parse_schema(input)?;
    Ok(schema
        .suture_sets
        .into_iter()
        .map(compile_suture_set)
        .collect())
}
