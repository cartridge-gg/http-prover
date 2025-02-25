use crate::server::AppState;
use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::KeepAlive, Sse},
};
use common::models::JobStatus;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tracing::info;

#[derive(Deserialize, Serialize)]
pub struct JobParams {
    job_id: u64,
}
pub async fn sse_handler(
    State(state): State<AppState>,
    Query(params): Query<JobParams>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    info!("SSE handler connected");
    let mut rx = state.sse_tx.lock().await.subscribe();
    let job_id = params.job_id;

    let job_status = state
        .job_store
        .get_job(job_id)
        .await
        .map_or(JobStatus::Unknown, |j| j.status);

    let stream = stream! {
        if matches!(job_status.clone(), JobStatus::Completed | JobStatus::Failed) {
            yield Ok(axum::response::sse::Event::default().data(serde_json::to_string(&(job_status, job_id)).unwrap()));
            return;
        }
        while let Ok(message) = rx.recv().await {
            match serde_json::from_str::<(JobStatus, u64)>(&message) {
                Ok((status, received_job_id)) => {
                    if job_id == received_job_id {
                        info!("Sending message: {}", message);
                        yield Ok(axum::response::sse::Event::default().data(message));
                        // If the job is completed or failed, break the loop to stop sending events
                        if matches!(status, JobStatus::Completed | JobStatus::Failed) {
                            info!("Job {} completed or failed, stopping SSE.", received_job_id);
                            break;
                        }
                    } else {
                        info!("Ignoring message for job {} as it doesn't match requested job IDs ", received_job_id);
                    }
                }
                Err(e) => println!("Failed to deserialize job status: {}", e),
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}
