use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateSignatureRequest {
    pub signature: Signature,
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateNonceRequest {
    pub public_key: String,
}
