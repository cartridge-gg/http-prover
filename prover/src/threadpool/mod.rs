use crate::{errors::ProverError, threadpool::prove::prove};
use common::{cairo0_prover_input::Cairo0ProverInput, cairo_prover_input::CairoProverInput};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::{
    spawn,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tracing::trace;
pub mod prove;
use crate::job::JobStore;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<(u64, JobStore, TempDir, CairoVersionedInput)>>,
}
pub enum CairoVersionedInput {
    Cairo(CairoProverInput),
    Cairo0(Cairo0ProverInput),
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
        job_store: JobStore,
        dir: TempDir,
        program_input: CairoVersionedInput,
    )-> Result<(), ProverError> {
        self.sender
            .as_ref()
            .unwrap()
            .send((job_id, job_store, dir, program_input))
            .await.unwrap();
        Ok(())}

    pub async fn shutdown(&mut self) -> Result<(), ProverError> {
        if let Some(sender) = self.sender.take() {
            drop(sender);
        }

        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                handle.await.unwrap();
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
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<(u64, JobStore, TempDir, CairoVersionedInput)>>>,
    ) -> Worker {
        let thread = spawn(async move {
            loop {
                let message = receiver.lock().await.recv().await;
                match message {
                    Some((job_id, job_store, dir, program_input)) => {
                        trace!("Worker {id} got a job; executing.");

                        if let Err(e) = prove(job_id, job_store, dir, program_input).await {
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
}
