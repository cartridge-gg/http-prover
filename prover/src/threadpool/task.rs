use crate::{
    errors::ProverError,
    threadpool::{layout_bridge::layout_bridge, prove::prove, snos::snos_pie_gen},
    utils::job::JobStore,
};

use std::sync::Arc;

use common::snos_input::SnosPieInput;
use tokio::sync::{broadcast::Sender, Mutex};

use super::{run::run, CairoVersionedInput};
use tracing::info;

pub struct TaskCommon {
    pub job_id: u64,
    pub job_store: JobStore,
    pub sse_tx: Arc<Mutex<Sender<String>>>,
}
impl TaskCommon {
    pub fn as_tuple(&self) -> (&u64, &JobStore, &Arc<Mutex<Sender<String>>>) {
        (&self.job_id, &self.job_store, &self.sse_tx)
    }
}

pub struct ProveParams {
    pub common: TaskCommon,
    pub program_input: CairoVersionedInput,
}

pub struct RunParams {
    pub common: TaskCommon,
    pub program_input: CairoVersionedInput,
}

pub struct LayoutBridgeParams {
    pub common: TaskCommon,
    pub proof: Vec<u8>,
}
pub struct SnosParams {
    pub common: TaskCommon,
    pub input: SnosPieInput,
}
pub enum Task {
    Run(RunParams),
    Prove(ProveParams),
    LayoutBridge(LayoutBridgeParams),
    Snos(SnosParams),
}

impl Task {
    pub fn extract_common(&self) -> (&u64, &JobStore, &Arc<Mutex<Sender<String>>>) {
        match self {
            Task::Prove(params) => params.common.as_tuple(),
            Task::Run(params) => params.common.as_tuple(),
            Task::LayoutBridge(params) => params.common.as_tuple(),
            Task::Snos(params) => params.common.as_tuple(),
        }
    }

    pub async fn execute(&self) -> Result<(), ProverError> {
        match self {
            Task::Prove(params) => {
                info!("Executing Prove task for job {}", params.common.job_id);
                prove(
                    params.common.job_id,
                    params.common.job_store.clone(),
                    params.program_input.clone(),
                    params.common.sse_tx.clone(),
                )
                .await
            }
            Task::Run(params) => {
                info!("Executing run task for job {}", params.common.job_id);
                run(
                    params.common.job_id,
                    params.common.job_store.clone(),
                    params.program_input.clone(),
                    params.common.sse_tx.clone(),
                )
                .await
            }
            Task::LayoutBridge(params) => {
                info!("Executing layout bridge for job {}", params.common.job_id);
                layout_bridge(&params.common, params.proof.clone()).await
            }
            Task::Snos(params) => {
                let program_input = params.input.clone();
                info!("Executing snos for job {}", params.common.job_id);
                snos_pie_gen(
                    params.common.job_id,
                    params.common.job_store.clone(),
                    program_input,
                    params.common.sse_tx.clone(),
                )
                .await
            }
        }
    }
}
