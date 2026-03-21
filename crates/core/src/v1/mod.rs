mod knit;
mod optimize;
mod schema;
mod stitch;
mod suture;

pub use schema::Direction;
pub use suture::{Binding, Suture};

use crate::error::Error;
use optimize::compile_suture_set;
use schema::SutureSchema;
use serde_json::Value;

/// Parse a `.sutures.json` string into compiled sutures.
pub fn parse(input: &str) -> Result<Vec<Result<Suture, Error>>, Error> {
    let value: Value = serde_json::from_str(input).map_err(Error::Parse)?;
    load(value)
}

/// Compile a sutures manifest from a JSON value.
pub fn load(input: Value) -> Result<Vec<Result<Suture, Error>>, Error> {
    let schema: SutureSchema = serde_json::from_value(input).map_err(Error::Parse)?;
    Ok(schema
        .suture_sets
        .into_iter()
        .map(compile_suture_set)
        .collect())
}
