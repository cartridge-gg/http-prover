use clap::{Parser, Subcommand};

pub mod run;
use run::CairoRunner;

pub mod prove;
use prove::Prove;

pub mod common;
pub mod errors;
pub mod fetch;

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,
}
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Prove cairo programs")]
    Prove(Prove),
    #[clap(about = "Run cairo programs")]
    Run(CairoRunner),
}

#[tokio::main]
pub async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Subcommands::Prove(prove) => {
            prove.run().await;
        }
        Subcommands::Run(run) => {
            run.run().await;
        }
    }
}
