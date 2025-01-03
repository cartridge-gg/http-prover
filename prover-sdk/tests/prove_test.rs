use std::fs;

use common::prover_input::*;
use helpers::fetch_job;
use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};
use serde_json::Value;

use starknet_types_core::felt::Felt;

use url::Url;
mod helpers;

#[tokio::test]
async fn test_cairo_prove_bootloader() {
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
    let layout = "recursive".to_string();
    let data = CairoProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
        bootload: true,
    };
    let job = sdk.prove_cairo(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    fs::write("../cairo1_boot_rec_proof.json",result.proof.clone()).unwrap();
    // //Values calculated using https://github.com/HerodotusDev/integrity
    assert_eq!(result.serialized_proof.len(), 3117);
    assert_eq!(
        result.program_hash,
        Felt::from_hex("0x59c0bac6bc951237009c9c51711516b8a06d7b18acc37d53563f6ad4014d978")
            .unwrap()
    );
    assert_eq!(
        result.program_output_hash,
        Felt::from_hex("0x7216da28237d9a39cbb472112b92178ecaf8440bba0edb0ad15bb1e93742a54")
            .unwrap()
    );
    let result = sdk.clone().verify(result.proof).await;
    assert!(result.is_ok(), "Failed to verify proof");
    assert_eq!("true", result.unwrap());
}

#[tokio::test]
async fn test_cairo_prove() {
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
    let layout = "recursive".to_string();
    let data = CairoProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
        bootload: false,
    };
    let job = sdk.prove_cairo(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    // //Values calculated using https://github.com/HerodotusDev/integrity
    assert_eq!(result.serialized_proof.len(), 2469);
    assert_eq!(
        result.program_hash,
        Felt::from_hex("0x4fa9237f64ebf68987209abe5058c32961131e3e5f16383d832dba7a6a5c937")
            .unwrap()
    );
    assert_eq!(
        result.program_output_hash,
        Felt::from_hex("0x4dd3006138ea865d5610d65e02fd227b7436acc95e4b2ec30f5a21356ea6205")
            .unwrap()
    );
    let result = sdk.clone().verify(result.proof).await;
    assert!(result.is_ok(), "Failed to verify proof");
    assert_eq!("true", result.unwrap());
}

#[tokio::test]
async fn test_cairo0_prove_bootloader() {
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let url = std::env::var("PROVER_URL").unwrap();
    let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
    let url = Url::parse(&url).unwrap();
    let sdk = ProverSDK::new(url, access_key).await.unwrap();
    let program = std::fs::read_to_string("../examples/cairo0/fibonacci_compiled.json").unwrap();
    let program: Cairo0CompiledProgram = serde_json::from_str(&program).unwrap();
    let program_input_string = std::fs::read_to_string("../examples/cairo0/input.json").unwrap();
    let program_input: Value = serde_json::from_str(&program_input_string).unwrap();
    let layout = "recursive".to_string();
    let data = Cairo0ProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
        bootload: true
    };
    let job = sdk.prove_cairo0(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    fs::write("proof.json",result.proof.clone()).unwrap();
    // //Values calculated using https://github.com/HerodotusDev/integrity
    assert_eq!(result.serialized_proof.len(), 3195);
    assert_eq!(
        result.program_hash,
        Felt::from_hex("0x59c0bac6bc951237009c9c51711516b8a06d7b18acc37d53563f6ad4014d978")
            .unwrap()
    );
    // assert_eq!(result.program_output.len(), 2);
    assert_eq!(
        result.program_output_hash,
        Felt::from_hex("0x275566ba7fa277b9939f8e3392400f151887d28c907e14e0178bde14cc66f1e")
            .unwrap()
    );

    let result = sdk.clone().verify(result.proof).await.unwrap();
    assert_eq!("true", result);
}
#[tokio::test]
async fn test_cairo0_prove() {
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let url = std::env::var("PROVER_URL").unwrap();
    let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
    let url = Url::parse(&url).unwrap();
    let sdk = ProverSDK::new(url, access_key).await.unwrap();
    let program = std::fs::read_to_string("../examples/cairo0/fibonacci_compiled.json").unwrap();
    let program: Cairo0CompiledProgram = serde_json::from_str(&program).unwrap();
    let program_input_string = std::fs::read_to_string("../examples/cairo0/input.json").unwrap();
    let program_input: Value = serde_json::from_str(&program_input_string).unwrap();
    let layout = "recursive".to_string();
    let data = Cairo0ProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
        bootload: false,
    };
    let job = sdk.prove_cairo0(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    //Values calculated using https://github.com/HerodotusDev/integrity
    assert_eq!(result.serialized_proof.len(), 2303);
    assert_eq!(
        result.program_hash,
        Felt::from_hex("0x7ac5582e353f8750487838481a46b5429ef84b2f18f909aaab9388f1fe0a28b")
            .unwrap()
    );
    assert_eq!(result.program_output.len(), 2);
    assert_eq!(
        result.program_output_hash,
        Felt::from_hex("0x60cbf4532b874a9a19557a55b45663831f71e21438525174b82842a1fab0ec4")
            .unwrap()
    );

    let result = sdk.clone().verify(result.proof).await.unwrap();
    assert_eq!("true", result);
}


#[tokio::test]
async fn test_cairo_multi_prove() {
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
    let layout = "recursive".to_string();
    let data = CairoProverInput {
        program,
        layout,
        program_input,
        n_queries: Some(16),
        pow_bits: Some(20),
        bootload: false,
    };
    let job1 = sdk.prove_cairo(data.clone()).await.unwrap();
    let job2 = sdk.prove_cairo(data.clone()).await.unwrap();
    let job3 = sdk.prove_cairo(data.clone()).await.unwrap();
    let result = fetch_job(sdk.clone(), job1).await;
    let result = sdk.clone().verify(result.unwrap().proof).await.unwrap();
    assert_eq!("true", result);
    let result = fetch_job(sdk.clone(), job2).await;
    let result = sdk.clone().verify(result.unwrap().proof).await.unwrap();
    assert_eq!("true", result);
    let result = fetch_job(sdk.clone(), job3).await;
    let result = sdk.clone().verify(result.unwrap().proof).await.unwrap();
    assert_eq!("true", result);
}
