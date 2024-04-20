use crate::core::ArchiveJob;
use apalis::{
    prelude::Storage,
    redis::{connect, RedisStorage},
};
use rocket::{get, http::Status, State};

#[get("/<message>")]
pub async fn produce(_broker: &State<RedisStorage<ArchiveJob>>, message: &str) -> (Status, String) {
    let job = ArchiveJob {
        task: message.to_owned(),
    };

    let redis_conn = connect("redis://127.0.0.1/").await.unwrap();
    let mut redis = RedisStorage::<ArchiveJob>::new(redis_conn);

    let job_id = redis.push(job).await.unwrap();
    (Status::Created, job_id.to_string())
    // match job_id {
    //     Ok(job_id) => (Status::Created, job_id.to_string()),
    //     Err(_) => (Status::BadRequest, JobError::InternalError),
    // }
}
