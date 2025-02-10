use crate::auth::jwt::Claims;
use crate::server::AppState;
use crate::threadpool::task::SnosParams;
use crate::threadpool::task::{Task, TaskCommon};
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::snos_input::SnosPieInput;
use serde_json::json;

pub async fn root(
    State(app_state): State<AppState>,
    _claims: Claims,
    Json(program_input): Json<SnosPieInput>,
) -> impl IntoResponse {
    let thread_pool = app_state.running_thread_pool.clone();
    let job_store = app_state.job_store.clone();
    let job_id = job_store.create_job().await;
    let thread = thread_pool.lock().await;
    let common = TaskCommon {
        job_id,
        job_store,
        sse_tx: app_state.sse_tx.clone(),
    };
    let snos_params = SnosParams {
        common,
        input: program_input.clone(),
    };
    let _ = thread
        .execute(Task::Snos(snos_params))
        .await
        .into_response();
    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}
