use crate::{
    errors::ServerError, prove, temp_dir_middleware::TempDirHandle, verifier::verify_proof, Args,
};
use axum::{
    middleware,
    response::IntoResponse,
    routing::{get, post},
    serve, Router,
};
use core::net::SocketAddr;
use tokio::net::TcpListener;
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

    let app = Router::new()
        .route("/", get(handler))
        .route("/verify", post(verify_proof))
        .nest("/prove", prove::router())
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
