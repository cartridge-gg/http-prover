use crate::auth::jwt::Claims;
use crate::server::AppState;
use crate::threadpool::task::LayoutBridgeParams;
use crate::threadpool::task::{Task, TaskCommon};
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::prover_input::LayoutBridgeInput;
use serde_json::json;

pub async fn root(
    State(app_state): State<AppState>,
    _claims: Claims,
    Json(program_input): Json<LayoutBridgeInput>,
) -> impl IntoResponse {
    let thread_pool = app_state.thread_pool.clone();
    let job_store = app_state.job_store.clone();
    let job_id = job_store.create_job().await;
    let thread = thread_pool.lock().await;
    let task_base = TaskCommon {
        job_id,
        job_store,
        sse_tx: app_state.sse_tx.clone(),
    };

    let layout_bridge_params = LayoutBridgeParams {
        common: task_base,
        proof: program_input.proof,
    };

    let _ = thread
        .execute(Task::LayoutBridge(layout_bridge_params))
        .await
        .into_response();

    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}
