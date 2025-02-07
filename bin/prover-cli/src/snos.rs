use std::{fs, path::PathBuf};

use crate::fetch::{fetch_job_polling, fetch_job_sse};
use cairo_vm::types::layout_name::LayoutName::{self};
use clap::Parser;
use prover_sdk::{
    access_key::ProverAccessKey, sdk::ProverSDK, snos_input::SnosPieInput, JobResult, RunResult,
};
use url::Url;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct SnosRunner {
    #[arg(long, env)]
    pub prover_url: Url,
    #[arg(long, env)]
    pub compiled_os: PathBuf,
    #[arg(long, env)]
    pub block_number: u64,
    #[arg(long, env)]
    pub rpc_provider: String,
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

impl SnosRunner {
    pub async fn run(self) {
        let access_key = ProverAccessKey::from_hex_string(&self.prover_access_key.clone()).unwrap();
        let sdk = ProverSDK::new(self.prover_url.clone(), access_key)
            .await
            .unwrap();
        let compiled_os = fs::read(self.compiled_os).unwrap();

        let input = SnosPieInput {
            compiled_os,
            layout: LayoutName::all_cairo,
            full_output: self.full_output,
            block_number: self.block_number,
            rpc_provider: self.rpc_provider,
        };

        let job = sdk.snos_pie_gen(input).await.unwrap();
        if self.wait {
            let result = if self.sse {
                fetch_job_sse(sdk, job).await.unwrap()
            } else {
                fetch_job_polling(sdk, job).await.unwrap()
            };
            let path: std::path::PathBuf = self.program_output;
            let pie = handle_completed_job_response(result);
            let pie = match pie {
                RunResult::Pie(pie) => pie,
                _ => unreachable!("Expected a pie result, but got a run result"),
            };
            fs::write(path, pie).unwrap();
        }
    }
}

pub fn handle_completed_job_response(result: JobResult) -> RunResult {
    match result {
        JobResult::Prove(_) => {
            unreachable!("Expected a prove result, but got a run result",);
        }
        JobResult::Run(run_result) => run_result,
    }
}
