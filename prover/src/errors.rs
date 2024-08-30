use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{convert::Infallible, net::AddrParseError};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::auth::auth_errors::{AuthError, AuthorizerError};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Server(#[from] std::io::Error),

    #[error(transparent)]
    AddressParse(#[from] AddrParseError),
}
#[derive(Debug, Error)]
pub enum ProverError {
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    FileWriteError(#[from] std::io::Error),
    #[error(transparent)]
    InfallibleError(#[from] Infallible),
    #[error("{0}")]
    CustomError(String),
    #[error("Failed to send message{0}")]
    SendError(String),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("Internal server error{0}")]
    InternalServerError(String),
    #[error(transparent)]
    Authorizer(#[from] AuthorizerError),
}
impl<T> From<SendError<T>> for ProverError {
    fn from(err: SendError<T>) -> ProverError {
        ProverError::SendError(err.to_string())
    }
}
impl IntoResponse for ProverError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            ProverError::FileWriteError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ProverError::Parse(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ProverError::InfallibleError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ProverError::CustomError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ProverError::SendError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ProverError::Auth(e) => match e {
                AuthError::InvalidToken => (StatusCode::BAD_REQUEST, e.to_string()),
                AuthError::MissingAuthorizationHeader => (StatusCode::BAD_REQUEST, e.to_string()),
                AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, e.to_string()),
            },
            ProverError::InternalServerError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            ProverError::Authorizer(authorizer_error) => match authorizer_error {
                AuthorizerError::FileAccessError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                AuthorizerError::FormatError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
                AuthorizerError::MissingAuthorizationHeader => {
                    (StatusCode::BAD_REQUEST, authorizer_error.to_string())
                }
                AuthorizerError::PrefixHexConversionError(e) => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                AuthorizerError::VerifyingKeyError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
                AuthorizerError::DataError(_e) => (StatusCode::INTERNAL_SERVER_ERROR,"Conversion to Vec<u8> failed".to_string()),
            },
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
