pub mod auth;
pub mod prove;
pub mod server;
pub mod verifier;
use std::path::PathBuf;

use clap::{arg, Parser, ValueHint};

/// Command line arguments for the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env, default_value = "0.0.0.0")]
    pub host: String,
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,
    #[arg(long, short, env)]
    pub jwt_secret_key: String,
    #[arg(long, short, env, default_value = "3600")]
    pub message_expiration_time: u32,
    #[arg(long, short, env, default_value = "3600")]
    pub session_expiration_time: u32,
    #[arg(long, env, value_hint = ValueHint::FilePath)]
    pub authorized_keys_path: Option<PathBuf>,
    #[arg(long, env)]
    pub authorized_keys: Option<Vec<String>>,
}
