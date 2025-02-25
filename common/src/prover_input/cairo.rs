use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

use super::RunMode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CairoProverInput {
    pub program: CairoCompiledProgram,
    pub program_input: Vec<Felt>,
    pub layout: super::Layout,
    pub n_queries: Option<u32>,
    pub pow_bits: Option<u32>,
    pub run_mode: RunMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CairoCompiledProgram {
    //pub version: u64,
    pub type_declarations: serde_json::Value,
    pub libfunc_declarations: serde_json::Value,
    pub statements: serde_json::Value,
    pub funcs: serde_json::Value,
}
