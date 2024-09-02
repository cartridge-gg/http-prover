use crate::{
    auth::jwt::Claims,
    extractors::workdir::TempDirHandle,
    server::AppState,
    utils::job::{create_job, update_job_status, JobStatus, JobStore},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::process::Command;
use tempfile::TempDir;

pub async fn root(
    State(app_state): State<AppState>,
    TempDirHandle(dir): TempDirHandle,
    _claims: Claims,
    Json(proof): Json<String>,
) -> impl IntoResponse {
    let job_id = create_job(&app_state.job_store).await;
    let job_store = app_state.job_store.clone();
    tokio::spawn({
        async move {
            if let Err(e) = verify_proof(job_id, job_store.clone(), dir, proof).await {
                update_job_status(job_id, &job_store, JobStatus::Failed, Some(e)).await;
            }
        }
    });

    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}

pub async fn verify_proof(
    job_id: u64,
    job_store: JobStore,
    dir: TempDir,
    proof: String,
) -> Result<(), String> {
    update_job_status(job_id, &job_store, JobStatus::Running, None).await;

    // Define the path for the proof file
    let path = dir.into_path();
    let file = path.join("proof");

    // Write the proof string to the file
    std::fs::write(&file, &proof).map_err(|e| format!("Failed to write proof to file: {}", e))?;

    // Create the command to run the verifier
    let mut command = Command::new("cpu_air_verifier");
    command.arg("--in_file").arg(&file);

    // Execute the command and capture the status
    let status = command
        .status()
        .map_err(|e| format!("Failed to execute verifier: {}", e))?;

    // Remove the proof file
    std::fs::remove_file(&file).map_err(|e| format!("Failed to remove proof file: {}", e))?;

    // Check if the command was successful

    update_job_status(
        job_id,
        &job_store,
        JobStatus::Completed,
        Some(status.success().to_string()),
    )
    .await;
    Ok(())
}
