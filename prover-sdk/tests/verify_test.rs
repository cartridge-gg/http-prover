use common::prover_input::*;
use helpers::{fetch_job, handle_completed_job_response};
use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};
use starknet_types_core::felt::Felt;
use url::Url;

mod helpers;

#[tokio::test]
async fn test_verify_invalid_proof() {
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let url = std::env::var("PROVER_URL").unwrap();
    let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
    let url = Url::parse(&url).unwrap();
    let sdk = ProverSDK::new(url, access_key).await.unwrap();
    let result = sdk.clone().verify("wrong proof".to_string()).await;
    assert!(result.is_ok(), "Failed to verify proof");
    assert_eq!("false", result.unwrap());
}

#[tokio::test]
async fn test_verify_valid_proof() {
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
    let job = sdk.clone().prove_cairo(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    let result = handle_completed_job_response(result);

    let result = sdk.clone().verify(result.proof).await;
    assert!(result.is_ok(), "Failed to verify proof");
    assert_eq!("true", result.unwrap());
}
