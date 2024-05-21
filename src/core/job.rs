use apalis::prelude::Job;
use serde::{Deserialize, Serialize};
use std::{thread::sleep, time::Duration};

#[derive(Debug, Deserialize, Serialize)]
pub struct ArchiveJob {
    pub task: String,
}

impl ArchiveJob {
    pub async fn handle_job(self) {
        println!("Recieved new job: {}", self.task);

        sleep(Duration::from_secs(60));

        println!("Job over: {}", self.task);
    }
}

impl Job for ArchiveJob {
    const NAME: &'static str = "storage::ArchiveJob";
}
