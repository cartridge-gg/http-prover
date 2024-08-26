use std::path::PathBuf;
use std::str::FromStr;

use crate::config::generate;
use crate::errors::ProverError;
use crate::extractors::workdir::TempDirHandle;
use crate::job::{create_job, update_job_status, JobStatus, JobStore};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::process::Command;
use tempfile::TempDir;

pub async fn root(
    State(job_store): State<JobStore>,
    TempDirHandle(path): TempDirHandle,
) -> impl IntoResponse {
    let job_id = create_job(&job_store).await;
    tokio::spawn({
        async move {
            if let Err(e) = prove(job_id, job_store.clone(), path).await {
                update_job_status(job_id, &job_store, JobStatus::Failed, Some(e.to_string())).await;
            };
        }
    });

    (
        StatusCode::ACCEPTED,
        format!("Task started, job id: {}", job_id),
    )
}

pub async fn prove(job_id: u64, job_store: JobStore, dir: TempDir) -> Result<(), ProverError> {
    update_job_status(job_id, &job_store, JobStatus::Running, None).await;

    let path = dir.into_path();
    // let program_input_path: PathBuf = path.join("cairo/input.json");
    // let program_path: PathBuf = path.join("cairo/program.json");
    let proof_path: PathBuf = path.join("program_proof_cairo.json");
    let trace_file = path.join("program_trace.trace");
    let memory_file = path.join("program_memory.memory");
    let public_input_file = path.join("program_public_input.json");
    let private_input_file = path.join("program_private_input.json");
    let params_file = path.join("cpu_air_params.json");
    let config_file = PathBuf::from_str("config/cpu_air_prover_config.json")
        .map_err(|_| ProverError::ConfigMissing)?;

    let mut command = Command::new("cairo1-run");
    command
        .arg("--trace_file")
        .arg(&trace_file)
        .arg("--memory_file")
        .arg(&memory_file)
        .arg("--layout")
        .arg("recursive")
        .arg("--proof_mode")
        .arg("--air_public_input")
        .arg(&public_input_file)
        .arg("--air_private_input")
        .arg(&private_input_file)
        .arg("--args_file")
        .arg("/home/mateuszpc/dev/dojo_example/dojo_bin/resources/input.txt")
        .arg(
            "/home/mateuszpc/dev/dojo_example/dojo_bin/target/dev/binary_dojo_example.sierra.json",
        );

    let mut child = command.spawn().map_err(|_| ProverError::CairoRunFailed)?;
    let _status = child.wait().map_err(|_| ProverError::CairoRunFailed)?;

    generate(public_input_file.clone(), params_file.clone());

    let mut command_proof = Command::new("cpu_air_prover");
    command_proof
        .arg("--out_file")
        .arg(&proof_path)
        .arg("--private_input_file")
        .arg(&private_input_file)
        .arg("--public_input_file")
        .arg(&public_input_file)
        .arg("--prover_config_file")
        .arg(&config_file)
        .arg("--parameter_file")
        .arg(&params_file)
        .arg("-generate-annotations");

    let mut child_proof = command_proof
        .spawn()
        .map_err(|_| ProverError::CairoProofFailed)?;
    let status_proof = child_proof
        .wait()
        .map_err(|_| ProverError::CairoProofFailed)?;

    if status_proof.success() {
        update_job_status(
            job_id,
            &job_store,
            JobStatus::Completed,
            Some(format!("Proof generated, workdir: {}", path.display())),
        )
        .await;
    }
    Ok(())
}
