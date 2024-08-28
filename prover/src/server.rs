use crate::extractors::workdir::TempDirHandle;
use crate::threadpool::ThreadPool;
use crate::utils::job::{get_job, JobStore};
use crate::utils::shutdown::shutdown_signal;
use crate::verifier::root;
use crate::{errors::ServerError, prove, Args};

use axum::{
    middleware,
    routing::{get, post},
    serve, Router,
};
use core::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[derive(Clone)]
pub struct AppState {
    pub job_store: JobStore,
    pub thread_pool: Arc<Mutex<ThreadPool>>,
}
pub async fn start(args: Args) -> Result<(), ServerError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = AppState {
        job_store: Arc::new(Mutex::new(Vec::new())),
        thread_pool: Arc::new(Mutex::new(ThreadPool::new(2))),
    };

    let app = Router::new()
        .route("/verify", post(root))
        .route("/get-job/:id", get(get_job))
        .with_state(app_state.clone())
        .nest("/prove", prove::router(app_state.clone()))
        .layer(middleware::from_extractor::<TempDirHandle>());

    let address: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(ServerError::AddressParse)?;
    
    let listener = TcpListener::bind(address).await?;

    trace!("Listening on {}", address);

    serve(listener, app)
    .with_graceful_shutdown(shutdown_signal(app_state.thread_pool))
    .await?;
    
    Ok(())
}
