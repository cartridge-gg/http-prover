use std::str::FromStr;

use clap::{Parser, ValueEnum};
use prover_sdk::{Cairo0ProverInput, Cairo1ProverInput, ProverAccessKey, ProverSDK};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ProveError {
    #[error("Failed to read: {0}")]
    Read(#[from] tokio::io::Error),

    #[error("Failed to parse prover key")]
    DecodeKey(prefix_hex::Error),

    #[error("Failed to initialize or authenticate prover SDK")]
    Initialize(prover_sdk::ProverSdkErrors),

    #[error("Failed to parse input: {0}")]
    ParseInput(#[from] serde_json::Error),

    #[error("Failed to prove: {0}")]
    Prove(prover_sdk::ProverSdkErrors),
}

#[derive(Debug, Serialize, Deserialize, ValueEnum, Clone)]
pub enum CairoVersion {
    V0,
    V1,
}

impl FromStr for CairoVersion {
    type Err = String;

    fn from_str(input: &str) -> Result<CairoVersion, Self::Err> {
        match input {
            "v0" => Ok(CairoVersion::V0),
            "v1" => Ok(CairoVersion::V1),
            _ => Err(format!("Invalid Cairo version: {}", input)),
        }
    }
}

#[derive(Parser, Debug, Serialize, Deserialize)]
#[clap(author, version, about, long_about = None)]
pub struct CliInput {
    #[arg(short, long, env)]
    pub key: String,

    #[arg(short, long, env, default_value = "v1")]
    pub cairo_version: CairoVersion,

    #[arg(short, long, env)]
    pub url: Url,
}

pub async fn prove(args: CliInput, input: String) -> Result<String, ProveError> {
    let secret_key = ProverAccessKey::from_hex_string(&args.key).map_err(ProveError::DecodeKey)?;
    let sdk = ProverSDK::new(secret_key, args.url)
        .await
        .map_err(ProveError::Initialize)?;

    let proof = match args.cairo_version {
        CairoVersion::V0 => {
            let input: Cairo0ProverInput =
                serde_json::from_str(&input).map_err(ProveError::ParseInput)?;
            sdk.prove_cairo0(input).await.map_err(ProveError::Prove)?
        }
        CairoVersion::V1 => {
            let input: Cairo1ProverInput =
                serde_json::from_str(&input).map_err(ProveError::ParseInput)?;
            sdk.prove_cairo(input).await.map_err(ProveError::Prove)?
        }
    };

    let proof_json: serde_json::Value =
        serde_json::from_str(&proof).expect("Failed to parse result");

    Ok(serde_json::to_string_pretty(&proof_json).expect("Failed to serialize result"))
}
