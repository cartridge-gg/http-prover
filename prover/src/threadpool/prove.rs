use super::CairoVersionedInput;
use crate::errors::ProverError;
use crate::threadpool::utlis::{ProvePaths, RunPaths};
use crate::utils::{config::Template, job::JobStore};
use cairo_proof_parser::json_parser::proof_from_annotations;
use cairo_proof_parser::output::ExtractOutputResult;
use cairo_proof_parser::program::{CairoVersion, ExtractProgramResult};
use cairo_proof_parser::{self, ProofJSON};
use common::models::{JobStatus, ProverResult};
use serde_json::Value;
use std::fs;

use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tracing::trace;

#[allow(clippy::too_many_arguments)]
pub async fn prove(
    job_id: u64,
    job_store: JobStore,
    program_input: CairoVersionedInput,
    sse_tx: Arc<Mutex<Sender<String>>>,
) -> Result<(), ProverError> {
    let dir = tempdir()?;
    job_store
        .update_job_status(job_id, JobStatus::Running, None)
        .await;

    let paths = ProvePaths::new(dir);
    let (n_queries, pow_bits, bootload) = program_input.get_parameters();
    program_input
        .prepare_and_run(&RunPaths::from(&paths), bootload, job_id)
        .await?;

    Template::generate_from_public_input_file(&paths.public_input_file, n_queries, pow_bits)?
        .save_to_file(&paths.params_file)?;

    trace!("Running prover");
    let start = tokio::time::Instant::now();
    let prove_status = paths.prove_command().spawn()?.wait().await?;
    let elapsed = start.elapsed();
    trace!(
        "Prover finished in {:?} ms for job: {}",
        elapsed.as_millis(),
        job_id
    );
    let result = fs::read_to_string(&paths.proof_path)?;
    let proof: Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    let sender = sse_tx.lock().await;

    if prove_status.success() {
        let cairo_version = match program_input {
            CairoVersionedInput::Cairo(_) => CairoVersion::Cairo,
            CairoVersionedInput::Cairo0(_) => CairoVersion::Cairo0,
        };

        let prover_result = prover_result(&final_result, cairo_version, bootload)?;

        job_store
            .update_job_status(
                job_id,
                JobStatus::Completed,
                serde_json::to_string_pretty(&prover_result).ok(),
            )
            .await;
        if sender.receiver_count() > 0 {
            sender
                .send(serde_json::to_string(&(JobStatus::Completed, job_id))?)
                .unwrap();
        }
    } else {
        job_store
            .update_job_status(job_id, JobStatus::Failed, Some(final_result))
            .await;
        if sender.receiver_count() > 0 {
            sender
                .send(serde_json::to_string(&(JobStatus::Failed, job_id))?)
                .unwrap();
        }
    }
    Ok(())
}

fn prover_result(
    proof: &str,
    cairo_version: CairoVersion,
    bootload: bool,
) -> Result<ProverResult, ProverError> {
    let proof_json = serde_json::from_str::<ProofJSON>(proof)?;
    let proof_from_annotations = proof_from_annotations(proof_json)?;
    let ExtractProgramResult { program_hash, .. } =
        if cairo_version == CairoVersion::Cairo0 || bootload {
            proof_from_annotations.extract_program(CairoVersion::Cairo0)?
        } else {
            proof_from_annotations.extract_program(CairoVersion::Cairo)?
        };
    let ExtractOutputResult {
        program_output,
        program_output_hash,
    } = proof_from_annotations.extract_output()?;
    let serialized_proof = proof_from_annotations.to_felts();
    let prover_result = ProverResult {
        proof: proof.to_string(),
        program_hash,
        program_output,
        program_output_hash,
        serialized_proof,
    };
    Ok(prover_result)
}
