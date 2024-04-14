use crate::core::ArchiveJob;
use apalis::{prelude::Storage, redis::RedisStorage};
use rocket::{get, http::Status, State};

#[get("/<message>")]
pub async fn produce(_broker: &State<RedisStorage<ArchiveJob>>, message: &str) -> (Status, String) {
    let job = ArchiveJob {
        task: message.to_owned(),
    };
    let mut redis = RedisStorage::<ArchiveJob>::connect("redis://127.0.0.1/")
        .await
        .unwrap();
    let job_id = redis.push(job).await.unwrap();
    (Status::Created, job_id.to_string())
    // match job_id {
    //     Ok(job_id) => (Status::Created, job_id.to_string()),
    //     Err(_) => (Status::BadRequest, JobError::InternalError),
    // }
}
