use clap::{Parser, Subcommand};

pub mod run;
use run::CairoRunner;

pub mod prove;
use prove::Prove;

pub mod config;
use config::ConfigGenerator;

pub mod layout_bridge;
use layout_bridge::LayoutBridgeRunner;

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
    #[clap(about = "Generate config based on provided public input")]
    Config(ConfigGenerator),
    #[clap(about = "Generate prove from layout bridge")]
    LayoutBridge(LayoutBridgeRunner),
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
        Subcommands::Config(config) => {
            config.run();
        }
        Subcommands::LayoutBridge(layout_bridge) => {
            layout_bridge.run().await;
        }
    }
}
