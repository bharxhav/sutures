// ---------------------------------------------------------------------------
// $id: https://schema.sutures.dev/v1.json
//
// These are the deserialized form of a `.sutures.json` file before
// compilation into the path tree.
// ---------------------------------------------------------------------------

use serde::Deserialize;

// ---- Root object ----------------------------------------------------------

/// Root object of a `.sutures.json` file.
///
/// Only `suture_sets` is needed for compilation.
/// The `name` field is required by the schema but not used after validation.
#[derive(Debug, Deserialize)]
pub(crate) struct SutureSchema {
    pub name: String,
    pub suture_sets: Vec<RawSutureSet>,
}

// ---- $defs/SutureSet ------------------------------------------------------

/// A named group of sutures targeting a single capture direction (request or response).
///
/// Schema: required `name`, `capture`, `sutures`.
/// Optional `id`, `description`, `version`.
///
/// When `capture` is `"request"`, sutures are `request_suture` objects.
/// When `capture` is `"response"`, sutures are `response_suture` objects.
///
/// `sutures` is kept as raw `Value` because individual suture objects have
/// dynamic keys (the terminal paths ARE the keys). Parsed into `RawSuture`
/// during compilation.
#[derive(Debug, Deserialize)]
pub(crate) struct RawSutureSet {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "capture")]
    pub capture_direction: Direction,
    pub sutures: Vec<serde_json::Value>,
}

/// Capture direction — `"request"` | `"response"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Request,
    Response,
}

// ---- $defs/json_terminal --------------------------------------------------

/// A JSON Pointer-like path starting with `/` that navigates into a JSON payload.
///
/// Schema pattern: `^/[A-Za-z0-9_$.\[\]:?/\`^()\-+\\*|]*$`
///
/// Examples: `/model`, `/choices/0/message/content`, `/data[:]`
pub(crate) type JsonTerminal = String;

// ---- $defs/struct_terminal ------------------------------------------------

/// A path starting with a letter that navigates into a typed structure.
///
/// Schema pattern: `^[A-Za-z][A-Za-z0-9_$.\[\]:?/\`^()\-+\\*|]*$`
///
/// Examples: `model`, `messages[:]`, `ChatResponse.content`
pub(crate) type StructTerminal = String;

// ---- $defs/request_suture & $defs/response_suture -------------------------

/// A single suture mapping object (recursive).
///
/// For **request** sutures (struct → JSON):
///   - keys are `struct_terminal`s
///   - values are `json_terminal`s
///   - `_` entries inject constants into the JSON output
///
/// For **response** sutures (JSON → struct):
///   - keys are `json_terminal`s
///   - values are `struct_terminal`s
///   - `_` entries inject constants into the struct
///
/// Schema `propertyNames`: `"_"` | terminal (direction-dependent).
#[derive(Debug)]
pub(crate) struct RawSuture {
    pub mappings: Vec<RawMapping>,
    pub constants: Vec<RawConstantEntry>,
}

/// One key → value pair within a suture (excluding `_`).
///
/// Schema `additionalProperties.oneOf` determines the value shape.
#[derive(Debug)]
pub(crate) struct RawMapping {
    /// The left-hand side: struct_terminal (request) or json_terminal (response).
    pub key: String,
    /// The right-hand side.
    pub value: RawMappingValue,
}

/// The value side of a mapping.
///
/// Schema `additionalProperties.oneOf`:
///   - a single terminal string
///   - an array of terminal strings (fan-out, minItems: 1)
///   - a nested suture object (recursion)
#[derive(Debug)]
pub(crate) enum RawMappingValue {
    /// Single terminal path.
    Terminal(String),
    /// Multiple terminal paths (fan-out).
    Terminals(Vec<String>),
    /// Nested suture object.
    Nested(RawSuture),
}

// ---- $defs/request_suture._.items & $defs/response_suture._.items ---------

/// One entry in a `_` constants array.
///
/// Schema: object with exactly 1 property (minProperties: 1, maxProperties: 1).
/// Key is a terminal, value is any valid JSON.
///
/// For request: key is `json_terminal`, value is injected into the JSON output.
/// For response: key is `struct_terminal`, value is injected into the struct.
#[derive(Debug)]
pub(crate) struct RawConstantEntry {
    pub terminal: String,
    pub value: serde_json::Value,
}
