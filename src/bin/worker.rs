use anyhow::Result;
use apalis::prelude::{Monitor, WithStorage, WorkerBuilder, WorkerFactoryFn};
use apalis::redis::RedisStorage;
use env_logger::{init_from_env as init_logger_from_env, Env};
use storage::core::ArchiveJob;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger_from_env(Env::new().default_filter_or("info"));

    let broker = RedisStorage::connect("redis://127.0.0.1/").await?;

    // TODO: new version released - check
    Monitor::new()
        .register_with_count(2, move |index| {
            WorkerBuilder::new(format!("StorageWroker-{index}"))
                .with_storage(broker.clone())
                .build_fn(ArchiveJob::handle_job)
        })
        .run()
        .await?;

    Ok(())
}
