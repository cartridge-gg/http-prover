use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthorizerError {
    #[error("file read error")]
    FileAccessError(#[from] std::io::Error),
    #[error("invalid file format")]
    FormatError(#[from] serde_json::Error),
}
