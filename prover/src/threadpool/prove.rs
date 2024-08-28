use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::TempDir;
use tokio::process::Command;

use crate::config::generate;
use crate::errors::ProverError;
use crate::job::{update_job_status, JobStore};

use super::CairoVersionedInput;

pub async fn prove(
    job_id: u64,
    job_store: JobStore,
    dir: TempDir,
    program_input: CairoVersionedInput,
) -> Result<(), ProverError> {
    update_job_status(job_id, &job_store, crate::job::JobStatus::Running, None).await;
    let path = dir.into_path();
    let program_path: PathBuf = path.join("program.json");
    let proof_path: PathBuf = path.join("program_proof_cairo.json");
    let trace_file = path.join("program_trace.trace");
    let memory_file = path.join("program_memory.memory");
    let public_input_file = path.join("program_public_input.json");
    let private_input_file = path.join("program_private_input.json");
    let params_file = path.join("cpu_air_params.json");
    let config_file = PathBuf::from_str("config/cpu_air_prover_config.json").unwrap();
    match program_input {
        CairoVersionedInput::Cairo(input) => {
            let program_input_path: PathBuf = input.program_input_path;
            let layout = input.layout;
            let program = serde_json::to_string(&input.program)?;
            fs::write(&program_path, program.clone())?;
            cairo_run(
                trace_file,
                memory_file,
                layout,
                public_input_file.clone(),
                private_input_file.clone(),
                program_input_path,
                program_path,
            )
            .await?;
        }
        CairoVersionedInput::Cairo0(input) => {
            let program_input_path: PathBuf = input.program_input_path;
            let layout = input.layout;
            let program = serde_json::to_string(&input.program)?;
            fs::write(&program_path, program.clone())?;

            cairo0_run(
                trace_file,
                memory_file,
                layout,
                public_input_file.clone(),
                private_input_file.clone(),
                program_input_path,
                program_path,
            )
            .await?;
        }
    }

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

    let mut child_proof = command_proof.spawn()?;
    let status_proof = child_proof.wait().await?;
    let result = fs::read_to_string(&proof_path)?;
    let proof: Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    if status_proof.success() {
        update_job_status(
            job_id,
            &job_store,
            crate::job::JobStatus::Completed,
            Some(final_result),
        )
        .await;
    } else {
        update_job_status(
            job_id,
            &job_store,
            crate::job::JobStatus::Failed,
            Some(final_result),
        )
        .await;
    }
    Ok(())
}

pub async fn cairo0_run(
    trace_file: PathBuf,
    memory_file: PathBuf,
    layout: String,
    public_input_file: PathBuf,
    private_input_file: PathBuf,
    program_input_path: PathBuf,
    program_path: PathBuf,
) -> Result<(), ProverError> {
    let mut command = Command::new("cairo-run");
    command
        .arg("--trace_file")
        .arg(&trace_file)
        .arg("--memory_file")
        .arg(&memory_file)
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input")
        .arg(&public_input_file)
        .arg("--air_private_input")
        .arg(&private_input_file)
        .arg("--program_input")
        .arg(&program_input_path)
        .arg("--program")
        .arg(&program_path);

    let mut child = command.spawn()?;
    let _status = child.wait().await?;
    Ok(())
}
pub async fn cairo_run(
    trace_file: PathBuf,
    memory_file: PathBuf,
    layout: String,
    public_input_file: PathBuf,
    private_input_file: PathBuf,
    program_input_path: PathBuf,
    program_path: PathBuf,
) -> Result<(), ProverError> {
    let mut command = Command::new("cairo1-run");
    command
        .arg("--trace_file")
        .arg(&trace_file)
        .arg("--memory_file")
        .arg(&memory_file)
        .arg("--layout")
        .arg(layout)
        .arg("--proof_mode")
        .arg("--air_public_input")
        .arg(&public_input_file)
        .arg("--air_private_input")
        .arg(&private_input_file)
        .arg("--args_file")
        .arg(&program_input_path)
        .arg(&program_path);

    let mut child = command.spawn()?;
    let _status = child.wait().await?;
    Ok(())
}
