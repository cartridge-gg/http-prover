use cairo_prove::prove::prove;
use cairo_prove::{fetch::fetch_job, Args};
use clap::Parser;
use prover_sdk::sdk_builder::ProverSDKBuilder;
use prover_sdk::sdk::ProverSDK;
use url::Url;
use ed25519_dalek::VerifyingKey;
#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let sdk = ProverSDK::new(args.prover_url.clone()).unwrap();
    let sdk_builder:ProverSDKBuilder = ProverSDKBuilder::new(args.prover_url.join("/auth").unwrap());
    let key = VerifyingKey::from_bytes(&[0;32]).unwrap();
    let nonce = sdk_builder.get_nonce(&key).await.unwrap();
    println!("nonce: {}", nonce);
    // let job = prove(args.clone(),sdk.clone()).await.unwrap();
    // if args.wait {
    //     let job = fetch_job(sdk, job).await.unwrap();
    //     let path: std::path::PathBuf = args.program_output;
    //     std::fs::write(path, job).unwrap();
    // }
}
