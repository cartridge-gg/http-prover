use crate::threadpool::ThreadPool;
use futures::join;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::info; // Import the logging macro

pub async fn shutdown_signal(
    proving_thread_pool: Arc<Mutex<ThreadPool>>,
    running_thread_pool: Arc<Mutex<ThreadPool>>,
) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            info!("Shutting down the server");
        },
        () = terminate => {
            info!("Shutting down the server due to termination signal");
        },
    }

    // Trigger thread pool shutdown
    info!("Shutting down the thread pool...");
    info!("Shutting down the thread pools...");

    let (proving_result, running_result) =
        {
            let mut proving_thread_pool = proving_thread_pool.lock().await;
            let mut running_thread_pool = running_thread_pool.lock().await;

            join!(
                async {
                    proving_thread_pool.shutdown().await.map_err(|e| {
                        eprintln!("Error during proving thread pool shutdown: {:?}", e)
                    })
                },
                async {
                    running_thread_pool.shutdown().await.map_err(|e| {
                        eprintln!("Error during running thread pool shutdown: {:?}", e)
                    })
                }
            )
        };

    info!("Proving thread pool shutdown: {:?}", proving_result);
    info!("Running thread pool shutdown: {:?}", running_result);
}
