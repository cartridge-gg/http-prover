use common::models::{JobResponse, JobResult};
use prover_sdk::{sdk::ProverSDK, ProverResult};

pub async fn fetch_job(sdk: ProverSDK, job: u64) -> Option<JobResult> {
    println!("Job ID: {}", job);
    sdk.sse(job).await.unwrap();
    let response = sdk.get_job(job).await.unwrap();
    let response = response.text().await.unwrap();
    let json_response: JobResponse = serde_json::from_str(&response).unwrap();

    if let JobResponse::Completed { result, .. } = json_response {
        return Some(result);
    }
    None
}

pub fn handle_completed_job_response(result: JobResult) -> ProverResult {
    match result {
        JobResult::Prove(prove_result) => prove_result,
        JobResult::Run(run_result) => {
            panic!(
                "Expected a prove result, but got a run result: {:?}",
                run_result
            );
        }
    }
}
