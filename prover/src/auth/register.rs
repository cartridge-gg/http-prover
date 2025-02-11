use axum::{extract::State, response::IntoResponse, Json};
use common::requests::AddKeyRequest;

use crate::{errors::ProverError, server::AppState};

use super::{auth_errors::AuthError, authorizer::AuthorizationProvider};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AddKeyRequest>,
) -> Result<impl IntoResponse, ProverError> {
    if !state.admin_keys.contains(&payload.authority) {
        return Err(ProverError::Auth(AuthError::Unauthorized));
    }
    payload
        .authority
        .verify_strict(payload.new_key.as_bytes(), &payload.signature)?;
    state.authorizer.authorize(payload.new_key).await?;
    Ok(())
}
