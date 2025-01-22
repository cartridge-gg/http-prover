use cairo_prove::errors::ProveErrors;
use cairo_prove::prove::prove;
use cairo_prove::{
    fetch::{fetch_job_polling, fetch_job_sse},
    Args,
};
use clap::Parser;
use prover_sdk::access_key::ProverAccessKey;
use prover_sdk::sdk::ProverSDK;
use prover_sdk::LayoutBridgeOrBootload;
#[tokio::main]
pub async fn main() -> Result<(), ProveErrors> {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let access_key = ProverAccessKey::from_hex_string(&args.prover_access_key.clone())?;
    if !args.layout.is_bootloadable() && matches!(args.run_option, LayoutBridgeOrBootload::Bootload)
    {
        return Err(ProveErrors::Custom("Invalid layout for bootloading, supported layouts for bootloader: recursive, recursive_with_poseidon, starknet, starknet_with_keccak".to_string()));
    }
    let sdk = ProverSDK::new(args.prover_url.clone(), access_key).await?;
    let job = prove(args.clone(), sdk.clone()).await?;
    if args.wait {
        let job = if args.sse {
            fetch_job_sse(sdk, job).await?
        } else {
            fetch_job_polling(sdk, job).await?
        };
        let path: std::path::PathBuf = args.program_output;
        std::fs::write(path, serde_json::to_string_pretty(&job)?)?;
    }
    Ok(())
}
