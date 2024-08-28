use std::{convert::Infallible, net::AddrParseError};
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Server(#[from] std::io::Error),

    #[error(transparent)]
    AddressParse(#[from] AddrParseError),
}
#[derive(Debug, Error)]
pub enum ProverError {
    #[error("Cairo run failed")]
    CairoRunFailed,
    #[error("Cairo proof failed")]
    CairoProofFailed,
    #[error("Failed to update job status")]
    UpdateJobStatusFailed,
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
}
impl<T> From<SendError<T>> for ProverError {
    fn from(err: SendError<T>) -> ProverError {
        ProverError::SendError(err.to_string())
    }
}
impl IntoResponse for ProverError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            ProverError::CairoRunFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cairo run failed",
            ),
            ProverError::CairoProofFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cairo proof failed",
            ),
            ProverError::UpdateJobStatusFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update job status",
            ),
            ProverError::FileWriteError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "File write operation failed",
            ),
            ProverError::Parse(_) => (
                StatusCode::BAD_REQUEST,
                "Failed to parse JSON",
            ),
            ProverError::InfallibleError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An infallible error occurred",
            ),
            ProverError::CustomError(msg) => (
                StatusCode::BAD_REQUEST,
                msg.as_str(),
            ),
            // Assume you added this variant
            ProverError::SendError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg.as_str(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
