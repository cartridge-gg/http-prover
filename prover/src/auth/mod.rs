pub mod jwt;
pub mod validation;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::validation::generate_nonce;
    use crate::auth::validation::is_public_key_authorized;
    use crate::auth::validation::validate_signature;
    use crate::prove::errors::ProveError;
    use crate::prove::models::GenerateNonceRequest;
    use crate::prove::models::ValidateSignatureRequest;
    use crate::server::AppState;
    use axum::body::Body;
    use axum::extract::Extension;
    use axum::extract::Query;
    use axum::extract::State;
    use axum::http::{Response, StatusCode};
    use axum::Json;
    use ed25519_dalek::ed25519::signature::Keypair;
    use ed25519_dalek::Signature;
    use ed25519_dalek::{Signer, SigningKey, Verifier};
    use rand::rngs::OsRng;
    use std::collections::HashMap;
    use std::env;
    use std::sync::Arc;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_generate_nonce() -> Result<(), ProveError> {
        let private_key_hex: String = env::var("PRIVATE_KEY")?;
        let private_key_bytes = hex::decode(&private_key_hex)?;
        let mut private_key_array = [0u8; 32];
        private_key_array.copy_from_slice(&private_key_bytes[..32]); // Copy the first 32 bytes

        let signing_key: SigningKey = SigningKey::from_bytes(&private_key_array);

        let public_key = signing_key.verifying_key();
        let encoded_verifying_key: Vec<u8> = public_key.to_bytes().to_vec();
        let public_key_hex: String = hex::encode(&encoded_verifying_key);

        let state = AppState {
            nonces: Arc::new(Mutex::new(HashMap::new())),
            prover_image_name: "sample".to_string(),
        };
        let params = GenerateNonceRequest {
            public_key: public_key_hex,
        };
        let result = generate_nonce(State(state.into()), Query(params)).await;

        assert!(result.is_ok());

        let response = result.unwrap();

        println!("{:?}", response.nonce);
        Ok(())
    }
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_is_public_key_authorized() {
        // Test with an authorized key
        let result = is_public_key_authorized(
            "authorized_keys.json",
            "05a257b53c49a28f2eb391653695e3ad2964ccec11fb30ca2b3d334187985501",
        )
        .await;
        assert!(result.is_ok());

        // Test with an unauthorized key
        let result = is_public_key_authorized("authorized_keys.json", "unauthorized_key").await;
        assert!(result.is_err());
    }
}
