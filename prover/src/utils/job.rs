use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use common::models::JobStatus;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{auth::jwt::Claims, server::AppState};

#[derive(Serialize, Clone)]
pub struct Job {
    pub id: u64,
    pub status: JobStatus,
    pub result: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum JobResponse {
    InProgress { id: u64, status: JobStatus },
    Completed { result: String, status: JobStatus },
    Failed { error: String },
}

#[derive(Clone, Default)]
pub struct JobStore {
    jobs: Arc<Mutex<Vec<Job>>>,
}

impl JobStore {
    pub async fn create_job(&self) -> u64 {
        let mut jobs = self.jobs.lock().await;
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
    pub async fn update_job_status(&self, job_id: u64, status: JobStatus, result: Option<String>) {
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
            job.result = result;
        }
    }
    pub async fn get_job(&self, id: u64) -> Option<Job> {
        let jobs = self.jobs.lock().await;
        jobs.iter().find(|job| job.id == id).cloned()
    }
}

pub async fn get_job(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
    _claims: Claims,
) -> impl IntoResponse {
    if let Some(job) = app_state.job_store.get_job(id).await {
        let (status, response) = match job.status {
            JobStatus::Pending | JobStatus::Running => (
                StatusCode::OK,
                Json(JobResponse::InProgress {
                    id: job.id,
                    status: job.status.clone(),
                }),
            ),
            JobStatus::Completed => (
                StatusCode::OK,
                Json(JobResponse::Completed {
                    status: job.status.clone(),
                    result: job
                        .result
                        .clone()
                        .unwrap_or_else(|| "No result available".to_string()),
                }),
            ),
            JobStatus::Failed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JobResponse::Failed {
                    error: job
                        .result
                        .clone()
                        .unwrap_or_else(|| "Unknown error".to_string()),
                }),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JobResponse::Failed {
                    error: "Unknown error".to_string(),
                }),
            ),
        };
        (status, response).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(format!("Job with id {} not found", id)),
        )
            .into_response()
    }
}
