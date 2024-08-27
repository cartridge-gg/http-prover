use common::cairo_prover_input::CairoProverInput;
use serde_json::Value;
use tracing::trace;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::{
    process::Command,
    spawn,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use crate::{
    config::generate, errors::ProverError, job::{update_job_status, JobStore}
};
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<(u64, JobStore, TempDir, CairoProverInput)>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel(100);

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub async fn execute(
        &self,
        job_id: u64,
        job_store: JobStore, // Ensure you're using the correct JobStore type
        dir: TempDir,
        program_input: CairoProverInput, // Ensure you're using the correct CairoProverInput type
    ) {
        self.sender
            .as_ref()
            .unwrap()
            .send((job_id, job_store, dir, program_input))
            .await
            .unwrap();
    }

    pub async fn shutdown(&mut self) {
        if let Some(sender) = self.sender.take() {
            drop(sender);
        }

        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                handle.await.unwrap();
            }
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<(u64, JobStore, TempDir, CairoProverInput)>>>,
    ) -> Worker {
        let thread = spawn(async move {
            loop {
                let message = receiver.lock().await.recv().await;
                match message {
                    Some((job_id, job_store, dir, program_input)) => {
                        trace!("Worker {id} got a job; executing.");

                        if let Err(e) = Worker::prove(job_id, job_store, dir, program_input).await {
                            eprintln!("Worker {id} encountered an error: {:?}", e);
                        }

                        trace!("Worker {id} finished the job.");
                    }
                    None => break,
                }
            }
        });

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }

    async fn prove(
        job_id: u64,
        job_store: JobStore, // Use the correct JobStore type
        dir: TempDir,
        program_input: CairoProverInput, // Use the correct CairoProverInput type
    ) -> Result<(), ProverError> {
        update_job_status(job_id, &job_store, crate::job::JobStatus::Running, None).await;
        let path = dir.into_path();
        let program_input_path: PathBuf = program_input.program_input_path;
        let program_path: PathBuf = path.join("program.json");
        let proof_path: PathBuf = path.join("program_proof_cairo.json");
        let trace_file = path.join("program_trace.trace");
        let memory_file = path.join("program_memory.memory");
        let public_input_file = path.join("program_public_input.json");
        let private_input_file = path.join("program_private_input.json");
        let params_file = path.join("cpu_air_params.json");
        let config_file = PathBuf::from_str("config/cpu_air_prover_config.json").unwrap();
        let program = serde_json::to_string(&program_input.program)?;
        let layout = program_input.layout;
        fs::write(&program_path, program.clone())?;

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
}
