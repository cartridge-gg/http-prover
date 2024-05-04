use reqwest::Error as ReqwestError;
use thiserror::Error;
use hex::FromHexError;
use url::ParseError;

#[derive(Debug, Error)]
pub enum ProverSdkErrors {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] ReqwestError),

    #[error("JSON parsing failed: {0}")]
    JsonParsingFailed(String),

    #[error("Validate signature failed: {0}")]
    ValidateSignatureResponseError(String),

    #[error("Prove request failed: {0}")]
    ProveRequestFailed(String),

    #[error("Prove request failed: {0}")]
    ProveResponseError(String),

    #[error("JSON parsing failed: {0}")]
    ReqwestBuildError(String),
    
    #[error("Failed to serialize")]
    SerdeError(#[from] serde_json::Error),

    #[error("Nonce not found in the response")]
    NonceNotFound,

    #[error("JWT token not found in the response")]
    JwtTokenNotFound,

    #[error("Reading input file failed")]
    ReadFileError(#[from] std::io::Error),

    #[error("Expiration date not found")]
    ExpirationNotFound,

    #[error("Signing key not found, possibly build without auth.")]
    SigningKeyNotFound,

    #[error("Nonce request failed: {0}")]
    NonceRequestFailed(String),

    #[error("Validate signature request failed: {0}")]
    ValidateSignatureRequestFailed(String),

    #[error("Failed to parse to url")]
    UrlParseError(#[from] ParseError),

    #[error("Failed to decode hex")]
    FromHexError(#[from] FromHexError),
}