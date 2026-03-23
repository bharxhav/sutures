mod compile;
mod knit;
mod schema;
mod stitch;
mod suture;
mod validate;

pub use schema::Direction;
pub use suture::{Bindings, Suture};

pub use suture::ConstantValue;

#[doc(hidden)]
pub use suture::{BindingTaskType, TrieNode};

use crate::error::Error;
use compile::compile;
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
    if schema.name.is_empty() {
        return Err(Error::Suture("root name must not be empty".into()));
    }
    if schema.suture_sets.is_empty() {
        return Err(Error::Suture("suture_sets must not be empty".into()));
    }
    Ok(schema.suture_sets.into_iter().map(compile).collect())
}
