use clap::Parser;
use prover::{errors::ServerError, server::start, Args};
#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let args = Args::parse();
    println!("args: {:?}", args);
    start(args).await?;
    Ok(())
}
