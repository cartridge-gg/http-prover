use axum::response::IntoResponse;

pub async fn root() -> impl IntoResponse {
    "Hello, World from Cairo1!"
}