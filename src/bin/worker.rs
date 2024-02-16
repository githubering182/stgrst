use anyhow::Result;
use apalis::prelude::*;
use apalis::redis::RedisStorage;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum EmailError {
    NoStorage,
    SomeError(&'static str),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Email {
    pub to: String,
}

impl Email {
    pub async fn handle_email(job: Email, _ctx: JobContext) {
        println!("Attempting to send email to {}", job.to);
    }
}

impl Job for Email {
    const NAME: &'static str = "apalis::Email";
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let mut storage = RedisStorage::connect("redis://127.0.0.1/").await?;
    storage
        .push(Email {
            to: String::from("recipient"),
        })
        .await?;
    Monitor::new()
        .register_with_count(2, move |index| {
            WorkerBuilder::new(format!("email-worker-{index}"))
                .with_storage(storage.clone())
                .build_fn(Email::handle_email)
        })
        .run()
        .await?;
    Ok(())
}
