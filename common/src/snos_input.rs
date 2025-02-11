use cairo_vm::types::layout_name::LayoutName;
use serde::{Deserialize, Serialize};

use crate::{sign_data, HttpProverData};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnosPieInput {
    pub compiled_os: Vec<u8>,
    pub block_number: u64,
    pub rpc_provider: String,
    pub layout: LayoutName,
    pub full_output: bool,
}

impl HttpProverData for SnosPieInput {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn sign(
        &self,
        signing_key: ed25519_dalek::SigningKey,
        timestamp: String,
        nonce: u64,
    ) -> String {
        sign_data(self, &timestamp, &signing_key, nonce)
    }
}
