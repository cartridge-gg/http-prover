use std::{fs, path::PathBuf, sync::Arc};

use common::{
    models::{JobResult, JobStatus, RunResult},
    prover_input::{Cairo0ProverInput, CairoProverInput, Layout},
};
use tempfile::tempdir;
use tokio::{
    process::Command,
    sync::{broadcast::Sender, Mutex},
};
use tracing::trace;

use crate::{
    errors::ProverError,
    threadpool::utlis::{command_run, create_template, ProvePaths},
    utils::job::JobStore,
};

use super::utlis::{prepare_input, RunPaths};

#[derive(Clone)]
pub enum CairoVersionedInput {
    Cairo(CairoProverInput),
    Cairo0(Cairo0ProverInput),
}
impl CairoVersionedInput {
    pub fn get_parameters(&self) -> (Option<u32>, Option<u32>, bool) {
        match self {
            CairoVersionedInput::Cairo(input) => (input.n_queries, input.pow_bits, input.bootload),
            CairoVersionedInput::Cairo0(input) => (input.n_queries, input.pow_bits, input.bootload),
        }
    }
}
pub trait BootloaderPath {
    fn path(&self) -> Result<PathBuf, ProverError>;
}

impl BootloaderPath for Layout {
    fn path(&self) -> Result<PathBuf, ProverError> {
        match self {
            Layout::Recursive => Ok("bootloaders/recursive.json".into()),
            Layout::RecursiveWithPoseidon => Ok("bootloaders/recursive_with_poseidon.json".into()),
            Layout::Starknet => Ok("bootloaders/starknet.json".into()),
            Layout::StarknetWithKeccak => Ok("bootloaders/starknet_with_keccak.json".into()),
            Layout::Dex | Layout::Small => {
                Err(ProverError::CustomError("Invalid layout".to_string()))
            }
        }
    }
}

pub async fn run(
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
    let (_, _, bootload) = program_input.get_parameters();
    let result = program_input
        .prepare_and_run(&RunPaths::from(&paths), bootload)
        .await;
    trace!("Trace generated for job {}", job_id);
    let sender = sse_tx.lock().await;
    if result.is_ok() {
        let memory = fs::read(&paths.memory_file)?;
        let trace = fs::read(&paths.trace_file)?;
        let public_input = fs::read_to_string(paths.public_input_file)?;
        let private_input = fs::read_to_string(paths.private_input_file)?;
        let runner_result = RunResult {
            memory,
            trace,
            public_input,
            private_input,
            pie: None,
        };
        job_store
            .update_job_status(
                job_id,
                JobStatus::Completed,
                serde_json::to_string(&JobResult::Run(runner_result)).ok(),
            )
            .await;
        if sender.receiver_count() > 0 {
            sender
                .send(serde_json::to_string(&(JobStatus::Completed, job_id))?)
                .unwrap();
        }
    } else {
        result.unwrap();
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

impl CairoVersionedInput {
    pub async fn prepare_and_run(
        &self,
        paths: &'_ RunPaths<'_>,
        bootload: bool,
    ) -> Result<(), ProverError> {
        self.prepare(paths)?;
        self.run_internal(paths, bootload).await
    }
    fn prepare(&self, paths: &RunPaths<'_>) -> Result<(), ProverError> {
        match self {
            CairoVersionedInput::Cairo(input) => {
                let program = serde_json::to_string(&input.program)?;
                let input = prepare_input(&input.program_input);
                fs::write(paths.program, program)?;
                fs::write(paths.program_input_path, input)?;
            }
            CairoVersionedInput::Cairo0(input) => {
                fs::write(paths.program, serde_json::to_string(&input.program)?)?;
                fs::write(
                    paths.program_input_path.clone(),
                    serde_json::to_string(&input.program_input)?,
                )?;
            }
        }
        Ok(())
    }
    async fn run_internal(&self, paths: &RunPaths<'_>, bootload: bool) -> Result<(), ProverError> {
        match self {
            CairoVersionedInput::Cairo(input) => {
                trace!("Running cairo1-run");
                if bootload {
                    trace!("Generating PIE");

                    let command = paths.cairo1_pie_command(&input.layout.to_string());
                    command_run(command).await?;

                    trace!("PIE generated");

                    let pie_file_str = paths.pie_output.to_str().unwrap();
                    let program_input_file_str = paths.program_input_path.to_str().unwrap();
                    create_template(pie_file_str, program_input_file_str)?;

                    trace!("Running cairo-run to generate trace from PIE");
                    let command = paths.cairo0_run_command(&input.layout, true)?;
                    command_run(command).await
                } else {
                    trace!("Running cairo-run to generate trace");
                    let command = paths.cairo1_run_command(&input.layout.to_string());
                    command_run(command).await
                }
            }
            CairoVersionedInput::Cairo0(input) => {
                if bootload {
                    trace!("Generating PIE");
                    let command = paths.cairo0_pie_command(&input.layout.to_string());
                    command_run(command).await?;

                    trace!("PIE generated");
                    let pie_file_str = paths.pie_output.to_str().unwrap();
                    let program_input_file_str = paths.program_input_path.to_str().unwrap();
                    create_template(pie_file_str, program_input_file_str)?;

                    let command = paths.cairo0_run_command(&input.layout, true)?;
                    command_run(command).await
                } else {
                    let command = paths.cairo0_run_command(&input.layout, false)?;
                    command_run(command).await
                }
            }
        }
    }
}

impl RunPaths<'_> {
    pub fn cairo1_run_command(&self, layout: &str) -> Command {
        let mut command = Command::new("cairo1-run");
        command
            .arg("--trace_file")
            .arg(self.trace_file)
            .arg("--memory_file")
            .arg(self.memory_file)
            .arg("--layout")
            .arg(layout)
            .arg("--proof_mode")
            .arg("--air_public_input")
            .arg(self.public_input_file)
            .arg("--air_private_input")
            .arg(self.private_input_file)
            .arg("--args_file")
            .arg(self.program_input_path)
            .arg(self.program);
        command
    }
    pub fn cairo0_run_command(
        &self,
        layout: &Layout,
        bootloader: bool,
    ) -> Result<Command, ProverError> {
        let program = if bootloader && layout.is_bootloadable() {
            layout.path()?
        } else {
            self.program.clone()
        };
        let layout = layout.to_string();
        let mut command = Command::new("python");
        command
            .arg("cairo-lang/src/starkware/cairo/lang/scripts/cairo-run")
            .arg("--trace_file")
            .arg(self.trace_file)
            .arg("--memory_file")
            .arg(self.memory_file)
            .arg("--layout")
            .arg(layout)
            .arg("--proof_mode")
            .arg("--air_public_input")
            .arg(self.public_input_file)
            .arg("--air_private_input")
            .arg(self.private_input_file)
            .arg("--program_input")
            .arg(self.program_input_path)
            .arg("--program")
            .arg(program);
        Ok(command)
    }
    pub fn cairo0_pie_command(&self, layout: &str) -> Command {
        let mut command = Command::new("python");
        command
            .arg("cairo-lang/src/starkware/cairo/lang/scripts/cairo-run")
            .arg("--layout")
            .arg(layout)
            .arg("--program_input")
            .arg(self.program_input_path)
            .arg("--program")
            .arg(self.program)
            .arg("--cairo_pie_output")
            .arg(self.pie_output);
        command
    }
    pub fn cairo1_pie_command(&self, layout: &str) -> Command {
        let mut command = Command::new("cairo1-run");
        command
            .arg("--layout")
            .arg(layout)
            .arg("--args_file")
            .arg(self.program_input_path)
            .arg(self.program)
            .arg("--cairo_pie_output")
            .arg(self.pie_output)
            .arg("--append_return_values");
        command
    }
}
