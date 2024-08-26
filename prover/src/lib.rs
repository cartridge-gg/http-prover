pub mod config;
pub mod errors;
pub mod extractors;
pub mod job;
pub mod prove;
pub mod server;
pub mod verifier;
use clap::{arg, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env, default_value = "0.0.0.0")]
    pub host: String,
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,
    #[arg(long, short, env, default_value = "3600")]
    pub message_expiration_time: u32,
    #[arg(long, short, env, default_value = "3600")]
    pub session_expiration_time: u32,
}
