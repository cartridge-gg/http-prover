use crate::auth::jwt::Claims;
use crate::server::AppState;
use crate::threadpool::{
    task::{ProveParams, Task, TaskCommon},
    CairoVersionedInput,
};
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::prover_input::CairoProverInput;
use serde_json::json;

pub async fn root(
    State(app_state): State<AppState>,
    _claims: Claims,
    Json(program_input): Json<CairoProverInput>,
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
    let execution_params = ProveParams {
        common: task_base,
        program_input: CairoVersionedInput::Cairo(program_input.clone()),
        n_queries: program_input.clone().n_queries,
        pow_bits: program_input.pow_bits,
        run_option: program_input.run_option,
    };
    let _ = thread
        .execute(Task::Prove(execution_params))
        .await
        .into_response();

    let body = json!({
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}
