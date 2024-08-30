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
        "0x49940f2f29f0722efdf40e20a9bc4e2657920c97587771f43e33be39e3555234",
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
