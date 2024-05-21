use apalis::prelude::{Monitor, TokioExecutor, WorkerBuilder, WorkerFactoryFn};
use apalis::redis::{connect, RedisStorage};
use storage::core::ArchiveJob;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let conn = connect("redis://127.0.0.1/").await.unwrap();
    let broker = RedisStorage::new(conn);

    let worker = WorkerBuilder::new("StorageWroker")
        .with_storage(broker)
        .build_fn(ArchiveJob::handle_job);

    Monitor::<TokioExecutor>::new()
        .register_with_count(2, worker)
        .run()
        .await
        .unwrap();

    Ok(())
}
