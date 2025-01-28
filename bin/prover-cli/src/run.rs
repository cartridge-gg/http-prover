use std::path::PathBuf;

use clap::Parser;
use prover_sdk::{
    access_key::ProverAccessKey, sdk::ProverSDK, Cairo0ProverInput, CairoCompiledProgram,
    CairoProverInput, JobResult, Layout, RunResult,
};
use serde_json::Value;
use tokio::fs;
use url::Url;

use crate::{
    common::{validate_input, CairoVersion},
    fetch::{fetch_job_polling, fetch_job_sse},
};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct CairoRunner {
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
    pub prover_access_key: String,
    #[arg(long, env, default_value = "false")]
    pub wait: bool,
    #[arg(long, env, default_value = "false")]
    pub sse: bool,
    #[arg(long, env)]
    pub proof_dir: PathBuf,
    #[arg(long, env, default_value = "false")]
    pub bootload: bool,
}
impl CairoRunner {
    pub async fn run(self) {
        let access_key = ProverAccessKey::from_hex_string(&self.prover_access_key.clone()).unwrap();
        let sdk = ProverSDK::new(self.prover_url.clone(), access_key)
            .await
            .unwrap();
        let job = cairo_runner(self.clone(), sdk.clone()).await;
        if self.wait {
            let result = if self.sse {
                fetch_job_sse(sdk, job).await.unwrap()
            } else {
                fetch_job_polling(sdk, job).await.unwrap()
            };
            let result = handle_completed_job_response(result);
            if !(self.proof_dir).exists() {
                fs::create_dir(&self.proof_dir).await.unwrap();
            }
            let path: std::path::PathBuf = self.proof_dir;

            let trace_path = path.join("trace");
            let memory_path = path.join("memory");
            fs::write(trace_path.clone(), result.trace).await.unwrap();
            fs::write(memory_path.clone(), result.memory).await.unwrap();

            let canonical_trace_path = fs::canonicalize(&trace_path).await.unwrap();
            let canonical_memory_path = fs::canonicalize(&memory_path).await.unwrap();

            let public_inputs_path = path.join("public_inputs");
            let private_inputs_path = path.join("private_inputs");

            fs::write(public_inputs_path, result.public_input)
                .await
                .unwrap();
            fs::write(private_inputs_path, result.private_input)
                .await
                .unwrap();

            println!(
                "Proof files are saved, change the paths in private input for those:
            trace: {}
            memory: {}",
                canonical_trace_path.to_str().unwrap(),
                canonical_memory_path.to_str().unwrap()
            );
        }
    }
}

pub async fn cairo_runner(args: CairoRunner, sdk: ProverSDK) -> u64 {
    match args.cairo_version {
        CairoVersion::V0 => {
            let program = std::fs::read(&args.program_path).unwrap();
            let input = std::fs::read_to_string(args.program_input_path).unwrap();
            let program_input: Value = serde_json::from_str(&input).unwrap();
            let data = Cairo0ProverInput {
                program,
                layout: args.layout,
                program_input,
                pow_bits: None,
                n_queries: None,
                bootload: args.bootload,
            };
            sdk.run_cairo0(data).await.unwrap()
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
                pow_bits: None,
                n_queries: None,
                bootload: args.bootload,
            };
            sdk.run_cairo(data).await.unwrap()
        }
    }
}

pub fn handle_completed_job_response(result: JobResult) -> RunResult {
    match result {
        JobResult::Prove(_) => {
            panic!("Expected a prove result, but got a run result",);
        }
        JobResult::Run(run_result) => run_result,
    }
}
