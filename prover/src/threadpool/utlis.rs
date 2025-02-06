use starknet_crypto::Felt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::TempDir;
use tokio::process::Command;

use crate::errors::ProverError;

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

pub struct RunPaths<'a> {
    pub trace_file: &'a PathBuf,
    pub memory_file: &'a PathBuf,
    pub public_input_file: &'a PathBuf,
    pub private_input_file: &'a PathBuf,
    pub program_input_path: &'a PathBuf,
    pub program: &'a PathBuf,
    pub pie_output: &'a PathBuf,
}

impl<'a> RunPaths<'a> {
    pub fn new(
        trace_file: &'a PathBuf,
        memory_file: &'a PathBuf,
        public_input_file: &'a PathBuf,
        private_input_file: &'a PathBuf,
        program_input_path: &'a PathBuf,
        program: &'a PathBuf,
        pie_output: &'a PathBuf,
    ) -> Self {
        Self {
            trace_file,
            memory_file,
            public_input_file,
            private_input_file,
            program_input_path,
            program,
            pie_output,
        }
    }
}

impl<'a> From<&'a ProvePaths> for RunPaths<'a> {
    fn from(
        ProvePaths {
            trace_file,
            memory_file,
            public_input_file,
            private_input_file,
            program_input: program_input_path,
            program: program_path,
            pie_output,
            ..
        }: &'a ProvePaths,
    ) -> Self {
        Self {
            trace_file,
            memory_file,
            public_input_file,
            private_input_file,
            program_input_path,
            program: program_path,
            pie_output,
        }
    }
}

pub async fn command_run(mut command: Command) -> Result<(), ProverError> {
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = command.spawn()?;
    let output = child.wait_with_output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ProverError::CustomError(stderr.into()));
    }
    Ok(())
}

pub fn prepare_input(felts: &[Felt]) -> String {
    felts
        .iter()
        .fold("[".to_string(), |a, i| a + &i.to_string() + " ")
        .trim_end()
        .to_string()
        + "]"
}

#[test]
fn test_prepare_input() {
    assert_eq!("[]", prepare_input(&[]));
    assert_eq!("[1]", prepare_input(&[1.into()]));
    assert_eq!(
        "[1 2 3 4]",
        prepare_input(&[1.into(), 2.into(), 3.into(), 4.into()])
    );
}

pub fn create_template(pie_path: &str, file_path: &str) -> std::io::Result<()> {
    // Manually format the JSON string
    let json = format!(
        r#"{{
  "tasks": [
    {{
      "type": "CairoPiePath",
      "path": "{}",
      "use_poseidon": true
    }}
  ],
  "single_page": true
}}"#,
        pie_path
    );

    // Write the JSON string to the file
    fs::write(file_path, json)
}
