use crate::errors::ProverError;

use std::sync::Arc;
use task::Task;
use tokio::{
    spawn,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tracing::{error, trace};

pub mod layout_bridge;
pub mod prove;
pub mod run;
pub mod task;
pub mod utlis;

pub use run::CairoVersionedInput;

type ReceiverType = Arc<Mutex<mpsc::Receiver<Task>>>;
type SenderType = Option<mpsc::Sender<Task>>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: SenderType,
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

    pub async fn execute(&self, task: Task) -> Result<(), ProverError> {
        self.sender
            .as_ref()
            .ok_or(ProverError::CustomError(
                "Thread pool is shutdown".to_string(),
            ))?
            .send(task)
            .await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), ProverError> {
        if let Some(sender) = self.sender.take() {
            drop(sender); // Dropping the sender signals that no more messages will be sent
        }

        // Wait for each worker to finish its current task
        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                if let Err(e) = handle.await {
                    eprintln!("Error waiting for worker: {:?}", e);
                    return Err(ProverError::CustomError(format!("Worker error: {:?}", e)));
                }
            }
        }

        Ok(())
    }
}

struct Worker {
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: ReceiverType) -> Worker {
        let thread = spawn(async move {
            loop {
                let message = receiver.lock().await.recv().await;
                match message {
                    Some(task) => {
                        trace!("Worker {id} got a job; executing.");

                        let (job_id, job_store, sse_tx) = task.extract_common();

                        let job_result = task.execute().await;
                        if let Err(e) = job_result {
                            job_store
                                .update_job_status(
                                    *job_id,
                                    common::models::JobStatus::Failed,
                                    Some(e.to_string()),
                                )
                                .await;

                            let sender = sse_tx.clone();
                            let sender = sender.lock().await;
                            if sender.receiver_count() > 0 {
                                let _ = sender.send(
                                    serde_json::to_string(&(
                                        common::models::JobStatus::Failed,
                                        job_id,
                                    ))
                                    .unwrap(),
                                );
                            }
                            error!("Worker {id} encountered an error in job {job_id}: {:?}", e);
                        }

                        trace!("Worker {id} finished the job.");
                    }
                    None => {
                        trace!("Worker {id} detected shutdown signal.");
                        break;
                    }
                }
            }
        });

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}
