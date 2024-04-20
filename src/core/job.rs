use apalis::prelude::Job;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ArchiveJob {
    pub task: String,
}

impl ArchiveJob {
    pub async fn handle_job(job: ArchiveJob) {
        println!("Recieved new job: {}", job.task);
    }
}

impl Job for ArchiveJob {
    const NAME: &'static str = "storage::ArchiveJob";
}
