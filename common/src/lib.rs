use ed25519_dalek::{ed25519::signature::Signer, Signature, SigningKey};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fmt::Debug;
pub mod models;
pub mod prover_input;
pub mod requests;
pub mod snos_input;

pub trait HttpProverData {
    fn to_json_value(&self) -> serde_json::Value;
    fn sign(&self, signing_key: SigningKey, timestamp: String, nonce: u64) -> String;
}
pub trait Signable: Serialize + Debug {}

impl<T: Serialize + Debug> Signable for T {}

pub fn sign_data<T: Signable>(
    input: &T,
    timestamp: &str,
    signing_key: &SigningKey,
    nonce: u64,
) -> String {
    let data_json = serde_json::to_value(input).expect("Serialization failed");
    let data_with_timestamp = json!({
        "data": data_json,
        "timestamp": timestamp,
        "nonce": nonce
    });
    let data_bytes =
        serde_json::to_vec(&data_with_timestamp).expect("Serialization to bytes failed");
    let hash = Sha256::digest(data_bytes);
    let signature: Signature = signing_key.sign(&hash);
    hex::encode(signature.to_bytes())
}
