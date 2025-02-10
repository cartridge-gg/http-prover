use axum::{routing::post, Router};

use crate::server::AppState;
mod cairo;
mod cairo0;
mod snos;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/cairo0", post(cairo0::root))
        .route("/cairo", post(cairo::root))
        .route("/snos", post(snos::root))
        .with_state(app_state)
}
