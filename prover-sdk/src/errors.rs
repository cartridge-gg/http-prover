use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkErrors {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Prover response error: {0}")]
    ProveResponseError(String),
}
