pub mod auth_errors;
pub mod authorizer;
pub mod register;

use authorizer::AuthorizationProvider;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::Signature;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sha2::Digest;
use sha2::Sha256;
use tokio::time::Instant;
use tracing::trace;

use crate::server::AppState;

//TODO: split into smaller functions
pub async fn signature_verification_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let headers = request.headers().clone();

    let signature_header = match headers.get("X-Signature") {
        Some(header) => header.to_str().map_err(|_| "Invalid header format"),
        None => Err("Missing x-signature header"),
    };
    let signature_hex = match signature_header {
        Ok(hex) => hex,
        Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
    };
    trace!("Received signature_hex: {}", signature_hex);

    let nonce_header = match headers.get("X-Nonce") {
        Some(header) => header.to_str().map_err(|_| "Invalid header format"),
        None => Err("Missing X-Nonce header"),
    };
    let nonce = match nonce_header {
        Ok(nonce) => nonce.parse::<u64>().map_err(|_| "Invalid nonce format"),
        Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
    };
    let nonce = match nonce {
        Ok(nonce) => nonce,
        Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
    };

    if !verify_nonce(&app_state, nonce).await {
        return (StatusCode::UNAUTHORIZED, "Invalid nonce").into_response();
    };

    let mut signature_bytes = [0u8; 64];
    if hex::decode_to_slice(signature_hex, &mut signature_bytes).is_err() {
        return (StatusCode::BAD_REQUEST, "Invalid signature format").into_response();
    }
    let signature = Signature::from_bytes(&signature_bytes);

    let timestamp_header = match headers.get("X-Timestamp") {
        Some(header) => header.to_str().map_err(|_| "Invalid header format"),
        None => Err("Missing X-Timestamp header"),
    };

    let timestamp_str = match timestamp_header {
        Ok(ts) => ts,
        Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
    };

    // Parse and validate timestamp
    let timestamp: DateTime<Utc> = match timestamp_str.parse() {
        Ok(ts) => ts,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid timestamp format").into_response(),
    };

    let now = Utc::now();
    if now.signed_duration_since(timestamp) > Duration::seconds(30) {
        return (StatusCode::UNAUTHORIZED, "Timestamp too old").into_response();
    }
    let (parts, body) = request.into_parts();

    let bytes = match body.collect().await {
        Ok(bytes) => bytes.to_bytes(),
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid body format").into_response(),
    };

    let value: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JSON body").into_response(),
    };
    let signed_data = json!(
        {
            "data": value,
            "timestamp": timestamp_str,
            "nonce": nonce
        }
    );
    cleanup_nonces(&app_state).await;

    let val_bytes = match serde_json::to_vec(&signed_data) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize JSON",
            )
                .into_response()
        }
    };

    let data_hash = Sha256::digest(val_bytes);
    match app_state
        .authorizer
        .is_authorized(signature, &data_hash)
        .await
    {
        Ok(true) => {
            trace!("Signature verified");
            let request = Request::from_parts(parts, Body::from(bytes));
            next.run(request).await.into_response()
        }
        Ok(false) => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Authorization check failed",
        )
            .into_response(),
    }
}

async fn verify_nonce(app_state: &AppState, nonce: u64) -> bool {
    let mut nonces = app_state.nonces.lock().await;
    if nonces.contains_key(&nonce) {
        return false; // Nonce already used
    }
    nonces.insert(nonce, Instant::now() + tokio::time::Duration::from_secs(30));
    true
}

async fn cleanup_nonces(app_state: &AppState) {
    let mut nonces = app_state.nonces.lock().await;
    let now = Instant::now();
    nonces.retain(|_, &mut expiry| expiry > now);
}
