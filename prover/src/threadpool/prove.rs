use super::run::RunPaths;
use super::CairoVersionedInput;
use crate::errors::ProverError;
use crate::utils::{config::Template, job::JobStore};
use cairo_proof_parser::json_parser::proof_from_annotations;
use cairo_proof_parser::output::ExtractOutputResult;
use cairo_proof_parser::program::{CairoVersion, ExtractProgramResult};
use cairo_proof_parser::{self, ProofJSON};
use common::models::{JobStatus, ProverResult};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::{tempdir, TempDir};
use tokio::process::Command;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tracing::trace;

#[allow(clippy::too_many_arguments)]
pub async fn prove(
    job_id: u64,
    job_store: JobStore,
    program_input: CairoVersionedInput,
    sse_tx: Arc<Mutex<Sender<String>>>,
    n_queries: Option<u32>,
    pow_bits: Option<u32>,
    bootload: bool,
) -> Result<(), ProverError> {
    let dir = tempdir()?;
    job_store
        .update_job_status(job_id, JobStatus::Running, None)
        .await;

    let paths = ProvePaths::new(dir);

    program_input
        .prepare_and_run(&RunPaths::from(&paths), bootload)
        .await?;

    Template::generate_from_public_input_file(&paths.public_input_file, n_queries, pow_bits)?
        .save_to_file(&paths.params_file)?;

    trace!("Running prover");

    let prove_status = paths.prove_command().spawn()?.wait().await?;
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

#[derive(Debug, Clone)]
pub(super) struct ProvePaths {
    pub(super) program_input: PathBuf,
    pub(super) program: PathBuf,
    pub(super) proof_path: PathBuf,
    pub(super) trace_file: PathBuf,
    pub(super) memory_file: PathBuf,
    pub(super) public_input_file: PathBuf,
    pub(super) private_input_file: PathBuf,
    pub(super) params_file: PathBuf,
    pub(super) config_file: PathBuf,
    pub(super) pie_output: PathBuf,
}

impl ProvePaths {
    pub fn new(base_dir: TempDir) -> Self {
        let path = base_dir.into_path();
        Self {
            program_input: path.join("program_input.json"),
            program: path.join("program.json"),
            proof_path: path.join("program_proof_cairo.json"),
            trace_file: path.join("program_trace.trace"),
            memory_file: path.join("program_memory.memory"),
            public_input_file: path.join("program_public_input.json"),
            private_input_file: path.join("program_private_input.json"),
            params_file: path.join("cpu_air_params.json"),
            config_file: PathBuf::from_str("config/cpu_air_prover_config.json").unwrap(),
            pie_output: path.join("program_pie_output.zip"),
        }
    }
    pub fn prove_command(&self) -> Command {
        let mut command = Command::new("cpu_air_prover");
        command
            .arg("--out_file")
            .arg(&self.proof_path)
            .arg("--private_input_file")
            .arg(&self.private_input_file)
            .arg("--public_input_file")
            .arg(&self.public_input_file)
            .arg("--prover_config_file")
            .arg(&self.config_file)
            .arg("--parameter_file")
            .arg(&self.params_file)
            .arg("-generate-annotations");
        command
    }
}
