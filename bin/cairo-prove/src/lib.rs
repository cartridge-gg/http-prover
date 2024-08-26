use std::{path::PathBuf, str::FromStr};
use common::{cairo0_prover_input::{Cairo0CompiledProgram, Cairo0ProverInput}, cairo_prover_input::{CairoCompiledProgram, CairoProverInput}};
use serde_json::Value;
use thiserror::Error;
use clap::{Parser, ValueEnum};
use prover_sdk::sdk::ProverSDK;
use serde::{Deserialize, Serialize};
use url::Url;
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env)]
    pub prover_url: Url,
    #[arg(long, short, env, default_value = "v1")]
    pub cairo_version: CairoVersion,
    #[arg(long, short, env)]
    pub layout: String,
    #[arg(long, env)]
    pub program_path: PathBuf,
    #[arg(long, env)]
    pub program_input_path: PathBuf,
    #[arg(long, env)]
    pub program_output: Option<PathBuf>,
}

pub async fn prove(args: Args) -> Result<(),ProveErrors> {
    let prover_url = args.prover_url.clone();
    let sdk = ProverSDK::new(prover_url)?;
    // let proof = std::fs::read_to_string("/home/mateuszpc/dev/dojo_example/proof.json").unwrap();
    // let result = sdk.verify(proof).await?;
    // println!("Result: {}", result);
    let program = std::fs::read_to_string(&args.program_path).unwrap();
    let program_input = std::fs::read_to_string(&args.program_input_path).unwrap();
    let proof = match args.cairo_version {
        CairoVersion::V0 => {
            let program_serialized:Cairo0CompiledProgram = serde_json::from_str(&program).unwrap();
            let data = Cairo0ProverInput{
                program: program_serialized,
                layout:args.layout,
                program_input:Value::default(),
            };
            sdk.prove_cairo0(data).await?
        }
        CairoVersion::V1 => {
            let program_serialized:CairoCompiledProgram = serde_json::from_str(&program).unwrap();
            let data = CairoProverInput{
                program: program_serialized,
                layout:args.layout,
                program_input_path:args.program_input_path,
            };
            sdk.prove_cairo(data).await?
        }
    };
    println!("Proof: {}", proof);
    Ok(())
}
#[derive(Debug, Error)]
pub enum ProveErrors {
    #[error(transparent)]
    SdkErrors(#[from] prover_sdk::errors::SdkErrors),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Prover response error: {0}")]
    ProveResponseError(String),
    #[error("Missing program input")]
    MissingProgramInput,
}
