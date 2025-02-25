use super::{Layout, RunMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cairo0ProverInput {
    pub program: Vec<u8>,
    pub program_input: Vec<u8>,
    pub layout: Layout,
    pub n_queries: Option<u32>,
    pub pow_bits: Option<u32>,
    pub run_mode: RunMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cairo0CompiledProgram {
    //TODO: Why this fails to serialize and deserialize layout bridge correctly
    pub attributes: Vec<String>,
    pub builtins: Vec<String>,
    pub compiler_version: String,
    pub data: Vec<String>,
    pub debug_info: serde_json::Value,
    pub hints: serde_json::Value,
    pub identifiers: serde_json::Value,
    pub main_scope: String,
    pub prime: String,
    pub reference_manager: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutBridgeInput {
    pub proof: Vec<u8>,
}
