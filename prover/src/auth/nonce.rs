use super::authorizer::AuthorizationProvider;
use crate::errors::ProverError;
use crate::server::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use bytes::{Bytes, BytesMut};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{io, ops::Deref, str::FromStr};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateNonceRequest {
    pub public_key: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateNonceResponse {
    #[serde_as(as = "DisplayFromStr")]
    pub nonce: Nonce,
    pub expiration: usize,
}

#[derive(Debug, Clone)]
pub struct Nonce(Bytes);

impl Nonce {
    pub fn new(size: usize) -> Self {
        let mut bytes = BytesMut::zeroed(size);
        rand::thread_rng().fill_bytes(bytes.as_mut());
        Self(bytes.into())
    }
}
impl FromStr for Nonce {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            prefix_hex::decode::<Vec<u8>>(s)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
                .into(),
        ))
    }
}

impl std::fmt::Display for Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", prefix_hex::encode(self.0.to_vec()))
    }
}

impl Deref for Nonce {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub async fn generate_nonce(
    State(state): State<AppState>,
    Query(params): Query<GenerateNonceRequest>,
) -> Result<Json<GenerateNonceResponse>, ProverError> {
    if params.public_key.trim().is_empty() {
        return Err(ProverError::CustomError("Public key is empty".to_string()));
        //TODO: Add proper error
    }
    if !state
        .authorizer
        .is_authorized(&params.public_key)
        .await
        .unwrap()
    {
        return Err(ProverError::CustomError("Unauthorized".to_string())); //TODO: Add proper error
    }
    //TODO: add key authorization here
    let message_expiration_time: usize = state.message_expiration_time;
    //TODO: unharcoded expiration time
    let nonce: Nonce = Nonce::new(32);
    let noce_string = nonce.to_string();
    let mut nonces = state.nonces.lock().await;
    let formatted_key = params.public_key.trim().to_lowercase();
    nonces.insert(noce_string, formatted_key);
    Ok(Json(GenerateNonceResponse {
        nonce,
        expiration: message_expiration_time,
    }))
}
