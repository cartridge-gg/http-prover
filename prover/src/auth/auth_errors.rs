use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthorizerError {
    #[error("file read error")]
    FileAccessError(#[from] std::io::Error),
    #[error("invalid file format")]
    FormatError(#[from] serde_json::Error),
    #[error("Missing authorization header")]
    MissingAuthorizationHeader,
}
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing authorization header")]
    MissingAuthorizationHeader,

    #[error("Unauthorized")]
    Unauthorized,
}
