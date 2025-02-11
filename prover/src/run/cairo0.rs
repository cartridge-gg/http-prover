use crate::server::AppState;
use crate::threadpool::{
    task::{RunParams, Task, TaskCommon},
    CairoVersionedInput,
};
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::prover_input::Cairo0ProverInput;
use serde_json::json;

pub async fn root(
    State(app_state): State<AppState>,
    Json(program_input): Json<Cairo0ProverInput>,
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

    let execution_params = RunParams {
        common,
        program_input: CairoVersionedInput::Cairo0(program_input.clone()),
    };
    let _ = thread
        .execute(Task::Run(execution_params))
        .await
        .into_response();
    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}
