use common::prover_input::{Cairo0ProverInput, Layout};
use serde_json::Value;
use tokio::fs;

use crate::errors::ProverError;

use super::{prove::prove, task::TaskCommon, CairoVersionedInput};

const LAYOUT_BRIDGE_PATH: &str = "layout_bridge.json";

pub async fn layout_bridge(common: &TaskCommon, proof: &str) -> Result<(), ProverError> {
    let program = fs::read(LAYOUT_BRIDGE_PATH).await?;

    let program_input: Value = serde_json::from_str(proof)?;
    let input = Cairo0ProverInput {
        program,
        program_input,
        layout: Layout::RecursiveWithPoseidon,
        n_queries: None,
        pow_bits: None,
        run_mode: common::prover_input::RunMode::Bootload,
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
