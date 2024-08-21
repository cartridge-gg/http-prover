use crate::server::AppState;
use axum::{routing::post, Router};
mod cairo0;
mod cairo;
pub mod errors;
pub mod models;

pub fn router(app_state: &AppState) -> Router {
    Router::new()
        .route("/cairo0", post(cairo0::root))
        .route("/cairo", post(cairo::root))
        .with_state(app_state.clone())
}
