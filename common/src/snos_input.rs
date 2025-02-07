use cairo_vm::types::layout_name::LayoutName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnosPieInput {
    pub compiled_os: Vec<u8>,
    pub block_number: u64,
    pub rpc_provider: String,
    pub layout: LayoutName,
    pub full_output: bool,
}
