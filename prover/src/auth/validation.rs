use crate::{errors::ProverError, server::AppState};
use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, HeaderValue},
    response::IntoResponse,
    Json,
};
use common::{models::JWTResponse, requests::ValidateSignatureRequest};
use ed25519_dalek::Verifier;

use super::jwt::{encode_jwt, Keys};
pub const COOKIE_NAME: &str = "jwt_token";

pub async fn validate_signature(
    State(state): State<AppState>,
    Json(payload): Json<ValidateSignatureRequest>,
) -> Result<impl IntoResponse, ProverError> {
    tracing::info!("Validating signature");
    let nonces = state.nonces.lock().await;
    let public_key = nonces.get(&payload.nonce);
    let public_key = match public_key {
        Some(public_key) => public_key,
        None => {
            return Err(ProverError::CustomError(
                "Public key for nonce not found".to_string(),
            ))
        }
    };
    tracing::info!("Public key found for nonce");
    let encoded_public_key = prefix_hex::encode(public_key.to_bytes());
    let verification = match public_key.verify(payload.nonce.as_bytes(), &payload.signature) {
        Ok(_) => true,
        Err(_) => false,
    };
    if !verification {
        return Err(ProverError::CustomError("Signature is invalid".to_string()));
    }
    tracing::info!("Signature is valid");
    let expiration =
        chrono::Utc::now() + chrono::Duration::seconds(state.message_expiration_time as i64);
    let token = encode_jwt(
        &encoded_public_key,
        expiration.timestamp() as usize,
        Keys::new(state.jwt_secret_key.clone().as_bytes()),
    )?;
    tracing::info!("JWT token generated");
    let cookie_value = format!(
        "{}={}; HttpOnly; Secure; Path=/; Max-Age={}",
        COOKIE_NAME, token, state.session_expiration_time
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|_| ProverError::CustomError("Invalid cookie value".to_string()))?,
    );
    tracing::info!("Cookie set");
    Ok((
        headers,
        Json(JWTResponse {
            jwt_token: token,
            expiration:expiration.timestamp() as u64,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use axum::extract::State;
    use axum::Json;
    use common::requests::ValidateSignatureRequest;
    use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;
    use tokio::sync::Mutex;

    use crate::{
        auth::{authorizer::Authorizer, nonce::Nonce, validate_signature},
        errors::ProverError,
        server::AppState,
        threadpool::ThreadPool,
    };

    fn generate_signing_key() -> SigningKey {
        SigningKey::generate(&mut OsRng)
    }

    fn generate_verifying_key(signing_key: &SigningKey) -> VerifyingKey {
        signing_key.verifying_key()
    }

    #[tokio::test]
    async fn test_valid_signature() {
        let nonce = Nonce::new(32);
        let nonce_string = nonce.to_string();
        let private_key = generate_signing_key();
        let public_key = generate_verifying_key(&private_key);
        let signature = private_key.sign(nonce_string.as_bytes());
        let payload = ValidateSignatureRequest {
            nonce: nonce_string.clone(),
            signature,
        };
        let nonces: Arc<Mutex<HashMap<String, VerifyingKey>>> =
            Arc::new(Mutex::new(HashMap::new()));
        nonces.lock().await.insert(nonce_string.clone(), public_key);

        let app_state = AppState {
            jwt_secret_key: "secret".to_string(),
            job_store: Default::default(),
            message_expiration_time: 100,
            session_expiration_time: 100,
            thread_pool: Arc::new(Mutex::new(ThreadPool::new(1))),
            nonces,
            authorizer: Authorizer::Open,
        };

        let result = validate_signature(State(app_state), Json(payload)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_signature() {
        let nonce = Nonce::new(32);
        let nonce_string = nonce.to_string();
        let signing_private_key = generate_signing_key();
        let false_private_key = generate_signing_key();
        let false_public_key = generate_verifying_key(&false_private_key);
        let signature = signing_private_key.sign(nonce_string.as_bytes());
        let payload = ValidateSignatureRequest {
            nonce: nonce_string.clone(),
            signature,
        };
        let nonces: Arc<Mutex<HashMap<String, VerifyingKey>>> =
            Arc::new(Mutex::new(HashMap::new()));
        nonces
            .lock()
            .await
            .insert(nonce_string.clone(), false_public_key);

        let app_state = AppState {
            jwt_secret_key: "secret".to_string(),
            job_store: Default::default(),
            message_expiration_time: 100,
            session_expiration_time: 100,
            thread_pool: Arc::new(Mutex::new(ThreadPool::new(1))),
            nonces,
            authorizer: Authorizer::Open,
        };

        let result = validate_signature(State(app_state), Json(payload)).await;
        assert!(result.is_err());
        if let Err(ProverError::CustomError(message)) = result {
            assert_eq!(message, "Signature is invalid".to_string());
        } else {
            panic!("Unexpected error type");
        }
    }

    #[tokio::test]
    async fn test_nonce_not_found() {
        let nonce = Nonce::new(32);
        let nonce_string = nonce.to_string();
        let private_key = generate_signing_key();
        let signature = private_key.sign(nonce_string.as_bytes());
        let payload = ValidateSignatureRequest {
            nonce: nonce_string.clone(),
            signature,
        };
        let nonces: Arc<Mutex<HashMap<String, VerifyingKey>>> =
            Arc::new(Mutex::new(HashMap::new())); // Empty nonces map

        let app_state = AppState {
            jwt_secret_key: "secret".to_string(),
            job_store: Default::default(),
            message_expiration_time: 100,
            session_expiration_time: 100,
            thread_pool: Arc::new(Mutex::new(ThreadPool::new(1))),
            nonces,
            authorizer: Authorizer::Open,
        };

        let result = validate_signature(State(app_state), Json(payload)).await;
        let _error_message = match result {
            Err(ProverError::CustomError(message)) => message,
            _ => panic!("Unexpected result"),
        };
    }

    #[tokio::test]
    async fn test_missing_signature() {
        let nonce = Nonce::new(32);
        let nonce_string = nonce.to_string();
        let public_key = generate_verifying_key(&generate_signing_key());

        let payload = ValidateSignatureRequest {
            nonce: nonce_string.clone(),
            signature: Signature::from_bytes(&[0; 64]), // Invalid signature
        };
        let nonces: Arc<Mutex<HashMap<String, VerifyingKey>>> =
            Arc::new(Mutex::new(HashMap::new()));
        nonces.lock().await.insert(nonce_string.clone(), public_key);

        let app_state = AppState {
            jwt_secret_key: "secret".to_string(),
            job_store: Default::default(),
            message_expiration_time: 100,
            session_expiration_time: 100,
            thread_pool: Arc::new(Mutex::new(ThreadPool::new(1))),
            nonces,
            authorizer: Authorizer::Open,
        };

        let result = validate_signature(State(app_state), Json(payload)).await;
        assert!(result.is_err());
        if let Err(ProverError::CustomError(message)) = result {
            assert_eq!(message, "Signature is invalid".to_string());
        } else {
            panic!("Unexpected error type");
        }
    }
}
