use crate::extractors::workdir::TempDirHandle;
use crate::job::JobStatus;
use crate::job::{Job, JobStore};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tempfile::TempDir;
use tokio::time::sleep;

pub async fn root(
    State(job_store): State<JobStore>,
    TempDirHandle(path): TempDirHandle,
) -> impl IntoResponse {
    let mut jobs = job_store.lock().await;
    let job_id = jobs.len() as u64;
    let new_job = Job {
        id: job_id,
        status: JobStatus::Pending,
        result: None,
    };
    jobs.push(new_job);
    drop(jobs);
    // Drop the lock

    tokio::spawn({
        async move {
            prove(job_id, job_store, path).await;
        }
    });

    (
        StatusCode::ACCEPTED,
        format!("Task started, job id: {}", job_id),
    )
}

pub async fn prove(job_id: u64, job_store: JobStore, dir: TempDir) {
    let mut jobs = job_store.lock().await;
    if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
        job.status = JobStatus::Running;
    }
    drop(jobs);
    // Release lock after updating the status to Running
    let path = dir.into_path();
    // Perform async work
    sleep(tokio::time::Duration::from_secs(20)).await;

    let mut jobs = job_store.lock().await;
    if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
        job.status = JobStatus::Completed;
        job.result = Some(format!(
            "Proof generated successfully, work dir {}",
            path.display()
        ));
    }
    // Release lock after updating the status to Completed and setting the result
}
