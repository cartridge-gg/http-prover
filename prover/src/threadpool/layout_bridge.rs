use common::prover_input::{Cairo0ProverInput, Layout};
use serde_json::Value;
use tokio::fs;

use crate::errors::ProverError;

use super::{prove::prove, task::TaskCommon, CairoVersionedInput};

const LAYOUT_BRIDGE_PATH: &str = "layout_bridge.json";

pub async fn layout_bridge(common: &TaskCommon, proof: &str) -> Result<(), ProverError> {
    let program = fs::read(LAYOUT_BRIDGE_PATH).await?;
    let program_serialized: Value = serde_json::from_slice(&program).unwrap();

    let program_input: Value = serde_json::from_str(proof).unwrap();
    let input = Cairo0ProverInput {
        program: program_serialized,
        program_input,
        layout: Layout::RecursiveWithPoseidon,
        n_queries: None,
        pow_bits: None,
        bootload: true,
    };
    let input = CairoVersionedInput::Cairo0(input);
    prove(
        common.job_id,
        common.job_store.clone(),
        input,
        common.sse_tx.clone(),
    )
    .await
}
