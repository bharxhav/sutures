
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

impl Suture {
    /// Returns the suture's id, if one was set.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns the suture's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description, if one was set.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the version, if one was set.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Returns a reference to the compiled binding.
    pub fn binding(&self) -> &Binding {
        &self.binding
    }

    /// True when the binding captures request direction (struct → JSON).
    pub fn is_request(&self) -> bool {
        matches!(self.binding, Binding::Request { .. })
    }

    /// True when the binding captures response direction (JSON → struct).
    pub fn is_response(&self) -> bool {
        matches!(self.binding, Binding::Response { .. })
    }
}

impl std::fmt::Display for Suture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.id, &self.version) {
            (Some(id), Some(v)) => write!(f, "{}@{} ({})", self.name, v, id),
            (Some(id), None) => write!(f, "{} ({})", self.name, id),
            (None, Some(v)) => write!(f, "{}@{}", self.name, v),
            (None, None) => write!(f, "{}", self.name),
        }
    }
}
