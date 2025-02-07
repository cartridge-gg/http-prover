pub mod auth;
pub mod errors;
pub mod layout_bridge;
pub mod prove;
pub mod run;
pub mod server;
pub mod sse;
pub mod threadpool;
pub mod utils;
pub mod verifier;
use std::path::PathBuf;

use clap::{arg, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env, default_value = "0.0.0.0")]
    pub host: String,
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,
    #[arg(long, short, env, default_value = "3600")]
    pub message_expiration_time: usize,
    #[arg(long, short, env, default_value = "3600")]
    pub session_expiration_time: usize,
    #[arg(long, short, env)]
    pub jwt_secret_key: String,
    #[arg(long, env, default_value = "authorized_keys.json")]
    pub authorized_keys_path: PathBuf,
    #[arg(long, env, value_delimiter = ',')]
    pub authorized_keys: Vec<String>,
    #[arg(long, env, default_value = "4")]
    pub prove_workers: usize,
    #[arg(long, env, default_value = "8")]
    pub run_workers: usize,
    #[arg(long, env, value_delimiter = ',')]
    pub admin_keys: Vec<String>,
}
