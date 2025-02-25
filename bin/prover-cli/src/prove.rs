use std::path::PathBuf;

use clap::Parser;
use prover_sdk::{
    access_key::ProverAccessKey, sdk::ProverSDK, Cairo0ProverInput, CairoCompiledProgram,
    CairoProverInput, JobResult, Layout, ProverResult, RunMode,
};
use url::Url;

use crate::{
    common::{validate_input, CairoVersion},
    fetch::{fetch_job_polling, fetch_job_sse},
};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Prove {
    #[arg(long, env)]
    pub prover_url: Url,
    #[arg(long, short, env, default_value = "v1")]
    pub cairo_version: CairoVersion,
    #[arg(long, short, env)]
    pub layout: Layout,
    #[arg(long, env)]
    pub program_path: PathBuf,
    #[arg(long, env)]
    pub program_input_path: PathBuf,
    #[arg(long, env)]
    pub program_output: PathBuf,
    #[arg(long, env)]
    pub prover_access_key: String,
    #[arg(long, env, default_value = "false")]
    pub wait: bool,
    #[arg(long, env, default_value = "false")]
    pub sse: bool,
    #[arg(long, env)]
    pub n_queries: Option<u32>,
    #[arg(long, env)]
    pub pow_bits: Option<u32>,
    #[arg(long, env, default_value = "trace")]
    pub run_mode: RunMode,
    #[arg(long, env, default_value = "false")]
    pub full_output: bool,
}
impl Prove {
    pub async fn run(self) {
        let access_key = ProverAccessKey::from_hex_string(&self.prover_access_key.clone()).unwrap();
        if matches!(self.run_mode, RunMode::Bootload) {
            assert!(self.layout.is_bootloadable(),"Invalid layout for bootloading, supported layouts for bootloader: recursive, recursive_with_poseidon, starknet, starknet_with_keccak")
        }
        let sdk = ProverSDK::new(self.prover_url.clone(), access_key)
            .await
            .unwrap();
        let job = prove(self.clone(), sdk.clone()).await;
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
pub fn handle_completed_job_response(result: JobResult) -> ProverResult {
    match result {
        JobResult::Prove(prove_result) => prove_result,
        JobResult::Run(_) | JobResult::Snos(_) => {
            unreachable!("Expected a prove result, but got a run result");
        }
    }
}

pub async fn prove(args: Prove, sdk: ProverSDK) -> u64 {
    match args.cairo_version {
        CairoVersion::V0 => {
            let program = std::fs::read(&args.program_path).unwrap();
            let input = std::fs::read(args.program_input_path).unwrap();
            let data = Cairo0ProverInput {
                program,
                layout: args.layout,
                program_input: input,
                pow_bits: args.pow_bits,
                n_queries: args.n_queries,
                run_mode: args.run_mode,
            };
            sdk.prove_cairo0(data).await.unwrap()
        }
        CairoVersion::V1 => {
            let program = std::fs::read_to_string(&args.program_path).unwrap();
            let input = std::fs::read_to_string(args.program_input_path).unwrap();
            let input = validate_input(&input);
            let program_serialized: CairoCompiledProgram = serde_json::from_str(&program).unwrap();
            let data = CairoProverInput {
                program: program_serialized,
                layout: args.layout,
                program_input: input,
                pow_bits: args.pow_bits,
                n_queries: args.n_queries,
                run_mode: args.run_mode,
            };
            sdk.prove_cairo(data).await.unwrap()
        }
    }
}
