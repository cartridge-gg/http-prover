use crate::extractors::workdir::TempDirHandle;
use crate::job::{create_job, update_job_status, JobStatus, JobStore};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tempfile::TempDir;
use tokio::time::sleep;

pub async fn root(
    State(job_store): State<JobStore>,
    TempDirHandle(path): TempDirHandle,
) -> impl IntoResponse {
    let job_id = create_job(&job_store).await;
    tokio::spawn({
        async move {
            if let Err(e) = prove(job_id, job_store.clone(), path).await{
                update_job_status(job_id, &job_store, JobStatus::Failed, Some(e)).await;
            };
        }
    });

    (
        StatusCode::ACCEPTED,
        format!("Task started, job id: {}", job_id),
    )
}

pub async fn prove(job_id: u64, job_store: JobStore, dir: TempDir) -> Result<(), String> {
    update_job_status(job_id, &job_store, JobStatus::Running, None).await;
    let path = dir.into_path();
    sleep(tokio::time::Duration::from_secs(20)).await;

    update_job_status(job_id, &job_store, JobStatus::Completed, Some(format!("Proof generated, workdir: {}",path.display()))).await;
    Ok(())
}
