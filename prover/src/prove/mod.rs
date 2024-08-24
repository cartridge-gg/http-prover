use axum::{routing::post, Router};

use crate::job::JobStore;
mod cairo;
mod cairo0;

pub fn router(job_store: JobStore) -> Router {
    Router::new()
        .route("/cairo0", post(cairo0::root))
        .route("/cairo1", post(cairo::root))
        .with_state(job_store)
}
