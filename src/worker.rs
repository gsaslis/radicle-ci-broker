use crossbeam_channel::{Receiver, RecvError};

use radicle_term as term;

use crate::ci::CI;

#[derive(Debug)]
pub struct CIJob {
    project_name: String,
    patch_branch: String,
    patch_head: String,
    project_id: String,
    git_uri: String,
}

pub struct Worker<T: CI + Send> {
    pub(crate) id: usize,
    receiver: Receiver<CIJob>,
    ci: T,
}

impl<T: CI + Send> Worker<T> {
    pub fn new(id: usize, receiver: Receiver<CIJob>, ci: T) -> Self {
        Self { id, receiver, ci }
    }

    pub fn run(&mut self) -> Result<(), RecvError> {
        loop {
            let job = self.receiver.recv()?;
            self.process(job);
        }
    }

    fn process(&mut self, job: CIJob) {
        term::info!("[{}] Worker {} received job: {:?}", self.id, self.id, job);
        self.ci.setup(job.project_name, job.patch_branch, job.patch_head, &job.project_id, job.git_uri).unwrap();
        self.ci.run_pipeline(&job.project_id).unwrap();
    }
}

