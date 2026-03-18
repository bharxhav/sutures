/// Errors produced by sutures operations.
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Suture(String),
    Stitch(String),
    Unstitch(String),
    Knit(String),
    Unknit(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::Suture(msg) => write!(f, "suture error: {msg}"),
            Self::Stitch(msg) => write!(f, "stitch error: {msg}"),
            Self::Unstitch(msg) => write!(f, "unstitch error: {msg}"),
            Self::Knit(msg) => write!(f, "knit error: {msg}"),
            Self::Unknit(msg) => write!(f, "unknit error: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Parse(e)
    }
}
