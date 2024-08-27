use cairo_prove::prove::prove;
use cairo_prove::{fetch::fetch_job, Args};
use clap::Parser;
use prover_sdk::sdk::ProverSDK;
#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let sdk = ProverSDK::new(args.prover_url.clone()).unwrap();
    let job = prove(args.clone(),sdk.clone()).await.unwrap();
    if args.wait {
        let job = fetch_job(sdk, job).await.unwrap();
        let path: std::path::PathBuf = args.program_output;
        std::fs::write(path, job).unwrap();
    }
}
