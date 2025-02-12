use axum::Json;
use tempfile::tempdir;

use std::process::Command;

pub async fn verify_proof(Json(proof): Json<String>) -> Json<bool> {
    let dir = if let Ok(dir) = tempdir() {
        dir
    } else {
        eprintln!("Failed to create tempdir");
        return Json(false);
    };

    // Define the path for the proof file
    let file = dir.into_path().join("proof");

    // Write the proof string to the file
    if let Err(e) = std::fs::write(&file, proof) {
        eprintln!("Failed to write proof to file: {}", e);
        return Json(false);
    }

    // Create the command to run the verifier
    let mut command = Command::new("cpu_air_verifier");
    command.arg("--in_file").arg(&file);

    // Execute the command and capture the status
    let status = command.status();

    // Remove the proof file
    if let Err(e) = std::fs::remove_file(&file) {
        eprintln!("Failed to remove proof file: {}", e);
    }

    // Check if the command was successful
    match status {
        Ok(exit_status) => Json(exit_status.success()),
        Err(e) => {
            eprintln!("Failed to execute verifier: {}", e);
            Json(false)
        }
    }
}
