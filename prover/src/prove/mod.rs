use axum::{routing::post, Router};
mod cairo0;
mod cairo1;

pub fn router() -> Router {
    Router::new()
        .route("/cairo0", post(cairo0::root))
        .route("/cairo1", post(cairo1::root))
}
