use crate::{errors::ProverError, threadpool::prove::prove, utils::job::JobStore};

use std::sync::Arc;

use tokio::sync::{broadcast::Sender, Mutex};

use tracing::info;

use super::{run::run, CairoVersionedInput};

pub struct TaskCommon {
    pub job_id: u64,
    pub job_store: JobStore,
    pub sse_tx: Arc<Mutex<Sender<String>>>,
}

pub struct ProveParams {
    pub common: TaskCommon,
    pub program_input: CairoVersionedInput,
    pub n_queries: Option<u32>,
    pub pow_bits: Option<u32>,
    pub bootload: bool,
}

pub struct RunParams {
    pub common: TaskCommon,
    pub program_input: CairoVersionedInput,
}

pub enum Task {
    Run(RunParams),
    Prove(ProveParams),
}

impl Task {
    pub fn extract_common(&self) -> (&u64, &JobStore, &Arc<Mutex<Sender<String>>>) {
        match self {
            Task::Prove(params) => (
                &params.common.job_id,
                &params.common.job_store,
                &params.common.sse_tx,
            ),
            Task::Run(params) => (
                &params.common.job_id,
                &params.common.job_store,
                &params.common.sse_tx,
            ),
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
                    params.n_queries,
                    params.pow_bits,
                    params.bootload,
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
        }
    }
}
