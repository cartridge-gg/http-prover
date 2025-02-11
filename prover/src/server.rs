use crate::auth::auth_errors::AuthorizerError;
use crate::auth::authorizer::{AuthorizationProvider, Authorizer, FileAuthorizer};
use crate::auth::register::register;
use crate::auth::signature_verification_middleware;
use crate::errors::ProverError;
use crate::layout_bridge::root;
use crate::sse::sse_handler;
use crate::threadpool::ThreadPool;
use crate::utils::job::{get_job, JobStore};
use crate::utils::shutdown::shutdown_signal;
use crate::verifier::verify_proof;
use crate::{prove, run, Args};
use axum::extract::DefaultBodyLimit;
use axum::middleware::{self};
use axum::{
    routing::{get, post},
    serve, Router,
};
use core::net::SocketAddr;
use ed25519_dalek::VerifyingKey;
use std::collections::HashMap;
use tokio::time::Instant;

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub job_store: JobStore,
    pub proving_thread_pool: Arc<Mutex<ThreadPool>>,
    pub running_thread_pool: Arc<Mutex<ThreadPool>>,
    pub nonces: Arc<Mutex<HashMap<u64, Instant>>>,
    pub authorizer: Authorizer,
    pub admin_keys: Vec<VerifyingKey>,
    pub sse_tx: Arc<Mutex<Sender<String>>>,
}

pub async fn start(args: Args) -> Result<(), ProverError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,rpc_client,hyper=off".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let authorizer =
        Authorizer::Persistent(FileAuthorizer::new(args.authorized_keys_path.clone()).await?);
    let mut admin_keys = Vec::new();
    for key in args.admin_keys {
        let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(key)
            .map_err(|e| AuthorizerError::PrefixHexConversionError(e.to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes.try_into()?)?;
        admin_keys.push(verifying_key);
        authorizer.authorize(verifying_key).await?;
    }

    for key in args.authorized_keys.iter() {
        let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(key)
            .map_err(|e| AuthorizerError::PrefixHexConversionError(e.to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes.try_into()?)?;
        authorizer.authorize(verifying_key).await?;
    }
    let (sse_tx, _) = broadcast::channel(200);
    let app_state = AppState {
        authorizer,
        job_store: JobStore::default(),
        proving_thread_pool: Arc::new(Mutex::new(ThreadPool::new(args.prove_workers))),
        running_thread_pool: Arc::new(Mutex::new(ThreadPool::new(args.run_workers))),
        nonces: Arc::new(Mutex::new(HashMap::new())),
        admin_keys,
        sse_tx: Arc::new(Mutex::new(sse_tx)),
    };

    async fn ok_handler() -> &'static str {
        "OK"
    }
    let open_routes = Router::new()
        .route("/", get(ok_handler))
        .route("/verify", post(verify_proof))
        .route("/get-job/:id", get(get_job))
        .route("/sse", get(sse_handler))
        .route("/register", post(register))
        .with_state(app_state.clone());

    let auth_routes = Router::new()
        .route("/layout-bridge", post(root))
        .with_state(app_state.clone())
        .nest("/prove", prove::router(app_state.clone()))
        .nest("/run", run::router(app_state.clone()))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            signature_verification_middleware,
        ));

    let app = Router::new()
        .merge(open_routes)
        .merge(auth_routes)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1000));

    let address: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(ProverError::AddressParse)?;

    let listener = TcpListener::bind(address).await?;

    info!("Listening on {}", address);

    let keys = args.authorized_keys.clone();
    info!("provided public keys {:?}", keys);

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(
            app_state.proving_thread_pool,
            app_state.running_thread_pool,
        ))
        .await?;

    Ok(())
}
