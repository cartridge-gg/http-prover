use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Clone)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Serialize, Clone)]
pub struct Job {
    pub id: u64,
    pub status: JobStatus,
    pub result: Option<String>, // You can change this to any type based on your use case
}

pub type JobStore = Arc<Mutex<Vec<Job>>>;

pub async fn create_job(job_store: &JobStore) -> u64 {
    let mut jobs = job_store.lock().await;
    let job_id = jobs.len() as u64;
    let new_job = Job {
        id: job_id,
        status: JobStatus::Pending,
        result: None,
    };
    jobs.push(new_job);
    drop(jobs);
    job_id
}

pub async fn update_job_status(
    job_id: u64,
    job_store: &JobStore,
    status: JobStatus,
    result: Option<String>,
) {
    let mut jobs = job_store.lock().await;
    if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
        job.status = status;
        job.result = result;
    }
    drop(jobs);
}
pub async fn check_job_state_by_id(
    Path(id): Path<u64>,
    State(job_store): State<JobStore>,
) -> impl IntoResponse {
    let jobs = job_store.lock().await;
    if let Some(job) = jobs.iter().find(|job| job.id == id) {
        (StatusCode::OK, serde_json::to_string(job).unwrap())
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("Job with id {} not found", id),
        )
    }
}
