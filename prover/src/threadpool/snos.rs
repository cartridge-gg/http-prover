use std::sync::Arc;

use common::{models::JobStatus, snos_input::SnosPieInput};
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
    );
    let result = result.await.unwrap();
    let elapsed = start.elapsed();
    info!("Snos pie generation for job {} took {:?}", job_id, elapsed);
    let sender = sse_tx.lock().await;

    if result.0.run_validity_checks().is_ok() {
        result.0.write_zip_file(&snos_pie_path)?;
        let pie = fs::read(&snos_pie_path).await?;
        job_store
            .update_job_status(
                job_id,
                JobStatus::Completed,
                Some(serde_json::to_string(&common::models::JobResult::Run(
                    common::models::RunResult::Pie(pie),
                ))?),
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
