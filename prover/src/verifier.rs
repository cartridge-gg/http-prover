use std::{path::PathBuf, process::Command};

use axum::{extract::Path, Json};

pub async fn handle_path(Json(path): Json<PathBuf>) -> Json<bool> {
    println!("{:?}", path);
    let mut command = Command::new("cpu_air_verifier");
    command.
        arg("--in_file=").
        arg(path.to_str().unwrap());
    let _status = command.status().unwrap();
    println!("{}", _status);
    Json(true)
}