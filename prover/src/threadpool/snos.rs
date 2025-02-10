use std::sync::Arc;

use cairo_vm::{vm::runners::cairo_pie::CairoPie, Felt252};
use common::{
    models::{JobResult, JobStatus, SnosPieOutput},
    snos_input::SnosPieInput,
};
use prove_block::get_memory_segment;
use prove_block::prove_block;
use tempfile::tempdir;
use tokio::{
    fs,
    sync::{broadcast::Sender, Mutex},
};
use tracing::info;

use crate::{errors::ProverError, utils::job::JobStore};

pub async fn snos_pie_gen(
    job_id: u64,
    job_store: JobStore,
    program_input: SnosPieInput,
    sse_tx: Arc<Mutex<Sender<String>>>,
) -> Result<(), ProverError> {
    let dir = tempdir()?;
    let snos_pie_path = dir.into_path().join("snos_pie.zip");
    job_store
        .update_job_status(job_id, JobStatus::Running, None)
        .await;
    info!("Generating snos pie for job {}", job_id);
    let start = tokio::time::Instant::now();
    let result = prove_block(
        &program_input.compiled_os,
        program_input.block_number,
        &program_input.rpc_provider,
        program_input.layout,
        program_input.full_output,
    )
    .await
    .map_err(|e| ProverError::CustomError(e.to_string()))?;

    let elapsed = start.elapsed();
    info!("Snos pie generation for job {} took {:?}", job_id, elapsed);

    let sender = sse_tx.lock().await;

    if result.0.run_validity_checks().is_ok() {
        let pie = result.0.clone();
        let steps = pie.extract_steps();
        info!("Pie for job {}, have steps: {}", job_id, steps);
        let output = pie.extract_output();
        result.0.write_zip_file(&snos_pie_path)?;
        let pie = fs::read(&snos_pie_path).await?;
        let snos_pie = SnosPieOutput {
            pie,
            n_steps: steps,
            program_output: output,
        };
        job_store
            .update_job_status(
                job_id,
                JobStatus::Completed,
                Some(serde_json::to_string(&JobResult::Snos(snos_pie))?),
            )
            .await;
        if sender.receiver_count() > 0 {
            sender
                .send(serde_json::to_string(&(JobStatus::Completed, job_id))?)
                .unwrap();
        }
    } else {
        info!("Failed to generate snos pie for job {}", job_id);
        job_store
            .update_job_status(job_id, JobStatus::Failed, None)
            .await;
        if sender.receiver_count() > 0 {
            sender
                .send(serde_json::to_string(&(JobStatus::Failed, job_id))?)
                .unwrap();
        }
    }
    Ok(())
}

trait SnosPie {
    fn extract_output(&self) -> Vec<Felt252>;
    fn extract_steps(&self) -> usize;
}
impl SnosPie for CairoPie {
    fn extract_output(&self) -> Vec<Felt252> {
        let output_segment_index = 2_usize;
        let output_segment = get_memory_segment(self, output_segment_index);
        let output: Vec<cairo_vm::Felt252> = output_segment
            .iter()
            .map(|(_key, value)| value.get_int().unwrap())
            .collect::<Vec<_>>();
        output
    }
    fn extract_steps(&self) -> usize {
        self.execution_resources.n_steps
    }
}
