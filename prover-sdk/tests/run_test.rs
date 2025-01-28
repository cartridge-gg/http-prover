use core::panic;

use common::models::JobResult;
use helpers::fetch_job;
use prover_sdk::{
    access_key::ProverAccessKey, sdk::ProverSDK, CairoCompiledProgram, CairoProverInput, Layout,
};
use starknet_types_core::felt::Felt;
use url::Url;
mod helpers;
#[tokio::test]
async fn test_cairo_run() {
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let url = std::env::var("PROVER_URL").unwrap();
    let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
    let url = Url::parse(&url).unwrap();
    let sdk = ProverSDK::new(url, access_key).await.unwrap();
    let program = std::fs::read_to_string("../examples/cairo/fibonacci_compiled.json").unwrap();
    let program: CairoCompiledProgram = serde_json::from_str(&program).unwrap();
    let program_input_string = std::fs::read_to_string("../examples/cairo/input.json").unwrap();
    let mut program_input: Vec<Felt> = Vec::new();
    for part in program_input_string.split(',') {
        let felt = Felt::from_dec_str(part).unwrap();
        program_input.push(felt);
    }
    let data = CairoProverInput {
        program,
        layout: Layout::Recursive,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
        bootload: false,
    };
    let job = sdk.run_cairo(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    match result {
        JobResult::Prove(_prove_result) => {
            panic!("Expected run result, got prove result");
        }
        JobResult::Run(run_result) => {
            // Validate private input
            assert!(
                !run_result.private_input.is_empty(),
                "Private input is empty"
            );
            assert!(
                run_result.private_input.len() > 0,
                "Unexpected private input length"
            );

            // Validate public input
            assert!(!run_result.public_input.is_empty(), "Public input is empty");
            assert!(
                run_result.public_input.len() > 0,
                "Unexpected public input length"
            );

            // Validate memory
            assert!(!run_result.memory.is_empty(), "Memory is empty");
            assert!(run_result.memory.len() > 0, "Unexpected memory length");

            // Validate trace
            assert!(!run_result.trace.is_empty(), "Trace is empty");
            assert!(run_result.trace.len() > 0, "Unexpected trace length");
        }
    }
}
