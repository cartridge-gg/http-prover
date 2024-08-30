use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
use url::Url;

pub mod errors;
pub mod fetch;
pub mod prove;

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

#[derive(Parser, Debug, Clone)]
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
    #[arg(long, env, conflicts_with("program_input"))]
    pub program_input_path: Option<PathBuf>,
    #[arg(long, env)]
    pub program_input: Option<String>,
    #[arg(long, env)]
    pub program_output: PathBuf,
    #[arg(long, env)]
    pub prover_access_key: String,
    #[arg(long, env, default_value = "false")]
    pub wait: bool,
}
