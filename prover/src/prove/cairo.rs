use std::path::PathBuf;
use std::str::FromStr;

use crate::config::generate;
use crate::errors::ProverError;
use crate::extractors::workdir::TempDirHandle;
use crate::job::{create_job, update_job_status, JobStatus, JobStore};
use crate::server::AppState;
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::cairo_prover_input::CairoProverInput;
use serde_json::{json, Value};
use std::process::Command;
use tempfile::TempDir;
use tokio::fs;

pub async fn root(
    State(app_state): State<AppState>,
    TempDirHandle(path): TempDirHandle,
    Json(program_input): Json<CairoProverInput>,
) -> impl IntoResponse {
    let thread_pool = app_state.thread_pool.clone();
    let job_store = app_state.job_store.clone();
    let job_id = create_job(&job_store).await;
    let thread = thread_pool.lock().await;
    thread.execute(job_id, job_store, path, program_input).await;
    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}