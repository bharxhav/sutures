
/// A validated suture: one suture_set, compiled and ready to operate.
///
/// Use via [`Stitch`](super::Stitch) (Value layer) or [`Knit`](super::Knit) (streaming layer).
#[derive(Debug)]
pub struct Suture {
    pub(crate) id: Option<String>,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) binding: Binding,
}

/// Compiled mapping data, varying by capture direction.
///
/// - `Request`: struct field paths → JSON pointer paths (struct → JSON).
/// - `Response`: JSON pointer paths → struct field paths (JSON → struct).
#[derive(Debug)]
pub enum Binding {
    /// struct → JSON: keys are `.`-separated struct paths, values are `/`-prefixed JSON pointers.
    Request {
        todo!()
    },
    /// JSON → struct: keys are `/`-prefixed JSON pointers, values are `.`-separated struct paths.
    Response {
        todo!()
    },
}
