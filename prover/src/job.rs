use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Clone)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Serialize, Clone)]
pub struct Job {
    pub id: u64,
    pub status: JobStatus,
    pub result: Option<String>, // You can change this to any type based on your use case
}

pub type JobStore = Arc<Mutex<Vec<Job>>>;
