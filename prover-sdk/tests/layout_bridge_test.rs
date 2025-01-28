use common::prover_input::*;
use helpers::{fetch_job, handle_completed_job_response};
use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};

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
    let proof = std::fs::read_to_string("../examples/layout_bridge/dynamic.json").unwrap();
    let data = LayoutBridgeInput { proof };
    let job = sdk.layout_bridge(data).await.unwrap();
    let result = fetch_job(sdk.clone(), job).await;
    assert!(result.is_some());
    let result = result.unwrap();
    let result = handle_completed_job_response(result);

    // //Values calculated using https://github.com/HerodotusDev/integrity
    assert_eq!(result.serialized_proof.len(), 4512);
    assert_eq!(
        result.program_hash,
        Felt::from_hex("0x4894ac4898d489ee2622fcfcd06a7c05d75ae113d6bc2c2842bcafe34af845e")
            .unwrap()
    );
    assert_eq!(
        result.program_output_hash,
        Felt::from_hex("0x4410fe800b4a3d761f9640c2030b34b4430a890e0e3e9e52261e1d15eaf95ab")
            .unwrap()
    );
    let result = sdk.clone().verify(result.proof).await;
    assert!(result.is_ok(), "Failed to verify proof");
    assert_eq!("true", result.unwrap());
}
