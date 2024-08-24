use crate::{
    errors::ServerError, job::JobStore, prove, temp_dir_middleware::TempDirHandle,
    verifier::verify_proof, Args,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    serve, Router,
};
use core::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub async fn start(args: Args) -> Result<(), ServerError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let job_store: JobStore = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .route("/", get(handler))
        .route("/verify", post(verify_proof))
        .route("/job-status/:id", get(check_job_state_by_id))
        .with_state(job_store.clone())
        .nest("/prove", prove::router(job_store))
        .layer(middleware::from_extractor::<TempDirHandle>());
    let address: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(ServerError::AddressParse)?;
    let listener = TcpListener::bind(address).await?;
    trace!("Listening on {}", address);
    serve(listener, app).await?;
    Ok(())
}

async fn handler() -> impl IntoResponse {
    "Hello, World!"
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
