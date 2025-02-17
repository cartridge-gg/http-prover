use std::{fs, path::PathBuf};

use clap::Parser;
use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK, LayoutBridgeInput};
use url::Url;

use crate::{
    fetch::{fetch_job_polling, fetch_job_sse},
    prove::handle_completed_job_response,
};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct LayoutBridgeRunner {
    #[arg(long, env)]
    pub prover_url: Url,
    #[arg(long, env)]
    pub input: PathBuf,
    #[arg(long, env)]
    pub program_output: PathBuf,
    #[arg(long, env)]
    pub prover_access_key: String,
    #[arg(long, env, default_value = "false")]
    pub wait: bool,
    #[arg(long, env, default_value = "false")]
    pub sse: bool,
    #[arg(long, env, default_value = "false")]
    pub full_output: bool,
}

impl LayoutBridgeRunner {
    pub async fn run(self) {
        let access_key = ProverAccessKey::from_hex_string(&self.prover_access_key.clone()).unwrap();
        let sdk = ProverSDK::new(self.prover_url.clone(), access_key)
            .await
            .unwrap();
        let proof = fs::read(self.input).unwrap();
        let input = LayoutBridgeInput { proof };
        let job = sdk.layout_bridge(input).await.unwrap();
        if self.wait {
            let result = if self.sse {
                fetch_job_sse(sdk, job).await.unwrap()
            } else {
                fetch_job_polling(sdk, job).await.unwrap()
            };
            let path: std::path::PathBuf = self.program_output;
            let result = handle_completed_job_response(result);
            if self.full_output {
                std::fs::write(path, serde_json::to_string_pretty(&result).unwrap()).unwrap();
            } else {
                std::fs::write(path, result.proof).unwrap();
            }
        }
    }
}
