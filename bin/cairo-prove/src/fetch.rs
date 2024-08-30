use std::time::Duration;

use prover_sdk::sdk::ProverSDK;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use tracing::info;

use crate::errors::ProveErrors;

#[derive(Serialize, Clone)] //TODO: Move to common models to avoid duplicating definitions
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Serialize, Clone)] //TODO: Move to common models to avoid duplicating definitions
pub struct Job {
    pub id: u64,
    pub status: JobStatus,
    pub result: Option<String>, // You can change this to any type based on your use case
}
#[derive(Deserialize)] //TODO: Move to common models to avoid duplicating definitions
pub struct JobId {
    pub job_id: u64,
}

pub async fn fetch_job(sdk: ProverSDK, job: String) -> Result<String, ProveErrors> {
    let job: JobId = serde_json::from_str(&job)?;
    info!("Fetching job: {}", job.job_id);
    loop {
        let response = sdk.get_job(job.job_id).await?;
        let response = response.text().await?;
        let json_response: Value = serde_json::from_str(&response)?;
        if let Some(status) = json_response.get("status").and_then(Value::as_str) {
            if status == "Completed" {
                return Ok(json_response
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or_else(|| "No result found")
                    .to_string());
            }
        }
        info!(
            "Job is not completed yet. Waiting for 10 seconds, job status {}",
            json_response.get("status").unwrap()
        );
        sleep(Duration::from_secs(10)).await;
    }
}
