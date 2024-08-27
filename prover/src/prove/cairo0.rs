use crate::config::generate;
use crate::errors::ProverError;
use crate::extractors::workdir::TempDirHandle;
use crate::job::{create_job, update_job_status, JobStatus, JobStore};
use crate::server::AppState;
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::cairo0_prover_input::Cairo0ProverInput;
use serde_json::json;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use tempfile::TempDir;
use tokio::fs;

pub async fn root(
    State(app_state): State<AppState>,
    TempDirHandle(path): TempDirHandle,
    Json(program_input): Json<Cairo0ProverInput>,
) -> impl IntoResponse {
    
    let job_store = app_state.job_store.clone();
    let job_id = create_job(&job_store).await;
    tokio::spawn({
        async move {
            if let Err(e) = prove(job_id, job_store.clone(), path, program_input).await {
                update_job_status(job_id, &job_store, JobStatus::Failed, Some(e.to_string())).await;
            };
        }
    });

    let body = json!({
        "message": "Task started",
        "job_id": job_id
    });
    (StatusCode::ACCEPTED, body.to_string())
}

pub async fn prove(
    job_id: u64,
    job_store: JobStore,
    dir: TempDir,
    program_input: Cairo0ProverInput,
) -> Result<(), ProverError> {
    update_job_status(job_id, &job_store, JobStatus::Running, None).await;
    let path = dir.into_path();
    let program_input_path: PathBuf = path.join("input.json");
    let program_path: PathBuf = path.join("program.json");
    let proof_path: PathBuf = path.join("program_proof_cairo.json");
    let trace_file = path.join("program_trace.trace");
    let memory_file = path.join("program_memory.memory");
    let public_input_file = path.join("program_public_input.json");
    let private_input_file = path.join("program_private_input.json");
    let params_file = path.join("cpu_air_params.json");
    let config_file = PathBuf::from_str("config/cpu_air_prover_config.json")?;

    let input = serde_json::to_string(&program_input.program_input)?;
    let program = serde_json::to_string(&program_input.program)?;
    let layout = program_input.layout;
    fs::write(&program_input_path, input.clone()).await?;
    fs::write(&program_path, program.clone()).await?;
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
    let _status = child.wait()?;
    generate(public_input_file.clone(), params_file.clone());

    let mut command_proof = Command::new("cpu_air_prover");
    command_proof
        .arg("--public_input_file")
        .arg(&public_input_file)
        .arg("--private_input_file")
        .arg(&private_input_file)
        .arg("--prover_config_file")
        .arg(&config_file)
        .arg("--parameter_file")
        .arg(&params_file)
        .arg("-generate_annotations")
        .arg("--out_file")
        .arg(&proof_path);

    let mut child_proof = command_proof.spawn()?;
    let status_proof = child_proof.wait()?;
    let result = fs::read_to_string(&proof_path).await?;
    let proof: Value = serde_json::from_str(&result)?;
    let final_result = serde_json::to_string_pretty(&proof)?;
    if status_proof.success() {
        update_job_status(
            job_id,
            &job_store,
            JobStatus::Completed,
            Some(format!("{}", final_result)),
        )
        .await;
    }
    Ok(())
}
