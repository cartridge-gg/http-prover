use cairo_prove::prove::prove;
use cairo_prove::{fetch::fetch_job, Args};
use clap::Parser;
use prover_sdk::access_key::ProverAccessKey;
use prover_sdk::sdk::ProverSDK;
#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let access_key = ProverAccessKey::from_hex_string(
        "0x8c844ac75da32b52e4a98582ab4c7ed5f1dee417b37a7bf9306135fca51d90b4",
    )
    .unwrap();
    let sdk = ProverSDK::new(args.prover_url.clone(), access_key)
        .await
        .unwrap();
    let job = prove(args.clone(), sdk.clone()).await.unwrap();
    if args.wait {
        let job = fetch_job(sdk, job).await.unwrap();
        let path: std::path::PathBuf = args.program_output;
        std::fs::write(path, job).unwrap();
    }
}
