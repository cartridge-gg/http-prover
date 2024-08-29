pub mod auth_errors;
pub mod authorizer;
pub mod jwt;
pub mod nonce;
pub mod validation;
use crate::server::AppState;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use nonce::generate_nonce;

pub fn auth(app_state: AppState) -> Router {
    Router::new()
        .route("/auth", get(generate_nonce))
        .with_state(app_state.clone())
}

pub async fn validate_signature() -> impl IntoResponse {
    //mock implementation
    "signature"
}
