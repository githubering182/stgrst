use anyhow::Result;
use apalis::prelude::{Monitor, TokioExecutor, WorkerBuilder, WorkerFactoryFn};
use apalis::redis::{connect, RedisStorage};
use env_logger::{init_from_env as init_logger_from_env, Env};
use storage::core::ArchiveJob;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger_from_env(Env::new().default_filter_or("info"));

    let conn = connect("redis://127.0.0.1/").await?;
    let broker = RedisStorage::new(conn);

    let worker = WorkerBuilder::new("StorageWroker")
        .with_storage(broker.clone())
        .build_fn(ArchiveJob::handle_job);

    Monitor::<TokioExecutor>::new()
        .register_with_count(2, worker)
        .run()
        .await?;

    Ok(())
}
