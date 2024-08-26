use std::net::AddrParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("server error")]
    Server(#[from] std::io::Error),

    #[error("failed to parse address")]
    AddressParse(#[from] AddrParseError),
}
#[derive(Debug, Error)]
pub enum ProverError{
    #[error("Prover config missing")]
    ConfigMissing,
    #[error("Cairo run failed")]
    CairoRunFailed,
    #[error("Cairo proof failed")]
    CairoProofFailed,
    #[error("Failed to update job status")]
    UpdateJobStatusFailed,
    #[error("failed to parse result")]
    Parse(#[from] serde_json::Error),
    #[error("File read error")]
    FileWriteError(#[from] std::io::Error),
}