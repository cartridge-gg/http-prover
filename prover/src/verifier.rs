use crate::{
    extractors::workdir::TempDirHandle,
    job::{create_job, update_job_status, JobStatus, JobStore},
    server::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::process::Command;
use tempfile::TempDir;

pub async fn root(
    State(app_state): State<AppState>,
    TempDirHandle(dir): TempDirHandle,
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

    (
        StatusCode::ACCEPTED,
        format!("Task started, job id: {}", job_id),
    )
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
    if status.success() {
        update_job_status(
            job_id,
            &job_store,
            JobStatus::Completed,
            Some(format!(
                "Proof verified successfully, exit status: {}",
                status
            )),
        )
        .await;
        Ok(())
    } else {
        Err(format!("Verifier failed with exit status: {}", status))
    }
}
