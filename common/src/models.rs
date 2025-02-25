use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use starknet_types_core::felt::Felt;

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct JWTResponse {
    #[serde_as(as = "DisplayFromStr")]
    pub jwt_token: String,
    pub expiration: u64,
    pub session_key: Option<VerifyingKey>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Unknown,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProverResult {
    pub proof: String,
    pub serialized_proof: Vec<Felt>,
    pub program_hash: Felt,
    pub program_output: Vec<Felt>,
    pub program_output_hash: Felt,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceFiles {
    pub private_input: String,
    pub public_input: String,
    pub memory: Vec<u8>,
    pub trace: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunResult {
    Pie(Vec<u8>),
    Trace(TraceFiles),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnosPieOutput {
    pub pie: Vec<u8>,
    pub program_output: Vec<Felt>,
    pub n_steps: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum JobResult {
    Prove(ProverResult),
    Run(RunResult),
    Snos(SnosPieOutput),
}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JobResponse {
    InProgress {
        id: u64,
        status: JobStatus,
    },
    Completed {
        result: JobResult,
        status: JobStatus,
    },
    Failed {
        error: String,
    },
}
