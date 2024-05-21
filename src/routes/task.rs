use std::str::FromStr;

use crate::core::ArchiveJob;
use apalis::{
    prelude::{Storage, TaskId},
    redis::{connect, RedisStorage},
};
use rocket::{get, http::Status, State};

#[get("/<message>")]
pub async fn produce_task(message: &str) -> (Status, String) {
    let job = ArchiveJob {
        task: message.to_owned(),
    };

    let redis_conn = connect("redis://127.0.0.1/").await.unwrap();
    let mut redis = RedisStorage::<ArchiveJob>::new(redis_conn);

    match redis.push(job).await {
        Ok(job_id) => (Status::Created, job_id.to_string()),
        Err(_) => (Status::BadRequest, "error".to_string()),
    }
}

#[get("/check/<task_id>")]
pub async fn check_task(
    broker: &State<RedisStorage<ArchiveJob>>,
    task_id: &str,
) -> (Status, &'static str) {
    let task_id = TaskId::from_str(task_id).unwrap();
    let request = match broker.fetch_by_id(&task_id).await {
        Ok(task) => task,
        Err(_) => return (Status::BadRequest, "404"),
    };

    match request {
        Some(request) => {
            let task = request.inner();
            println!("task: {:?}", task);
            (Status::Ok, "ok")
        }
        None => unreachable!(),
    }
}
