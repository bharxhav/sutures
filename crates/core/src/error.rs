/// Errors produced by sutures operations.
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Suture(String),
    Stitch(String),
    Unstitch(String),
}
