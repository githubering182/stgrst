use apalis::prelude::{Job, JobContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ArchiveJob {
    pub task: String,
}

impl ArchiveJob {
    pub async fn handle_job(job: ArchiveJob, _ctx: JobContext) {
        println!("Recieved new job: {}", job.task);
    }
}

impl Job for ArchiveJob {
    const NAME: &'static str = "storage::ArchiveJob";
}
