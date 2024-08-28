use std::{convert::Infallible, net::AddrParseError};
use thiserror::Error;

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
}
