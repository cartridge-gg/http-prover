// use common::cairo_prover_input::{CairoCompiledProgram, CairoProverInput};
// use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};
// use starknet_types_core::felt::Felt;
// use url::Url;

// #[tokio::test]
// async fn test_cairo_prove() {
//     let private_key = std::env::var("PRIVATE_KEY").unwrap();
//     let url = std::env::var("PROVER_URL").unwrap();
//     let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
//     let url = Url::parse(&url).unwrap();
//     let sdk = ProverSDK::new(url, access_key).await.unwrap();
//     let program = std::fs::read_to_string("../examples/cairo/fibonacci_compiled.json").unwrap();
//     let program:CairoCompiledProgram = serde_json::from_str(&program).unwrap();
//     let program_input_string = std::fs::read_to_string("../examples/cairo/input.json").unwrap();
//     let mut program_input:Vec<Felt> = Vec::new();
//     for part in program_input_string.split(',') {
//         let felt = Felt::from_dec_str(part).unwrap();
//         program_input.push(felt);
//     }
//     let layout = "recursive".to_string();
//     let data = CairoProverInput{
//         program,
//         layout,
//         program_input
//     };
//     let proof = sdk.prove_cairo(data).await.unwrap();
//     println!("{:?}", proof);
//     let job_id = 0;
//     let response = sdk.get_job(0).await.unwrap();
//     for i in 0..10 {
//         tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
//     }

// }

// #[tokio::test]
// async fn test_cairo0_prove() {
//     let private_key = std::env::var("PRIVATE_KEY").unwrap();
//     let url = std::env::var("PROVER_URL").unwrap();
//     let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
//     let url = Url::parse(&url).unwrap();
//     let _sdk = ProverSDK::new(url, access_key).await.unwrap();
// }
