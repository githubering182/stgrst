use crate::core::{ArchiveJob, JobError};
use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder, ResponseError, Result,
};
use apalis::{prelude::Storage, redis::RedisStorage};

#[get("/archive/{message}/")]
pub async fn produce(
    path: Path<String>,
    broker: Data<RedisStorage<ArchiveJob>>,
) -> Result<impl Responder, impl ResponseError> {
    let broker = &*broker.into_inner();

    let job = ArchiveJob {
        task: path.into_inner(),
    };
    let job_id = broker.clone().push(job).await;

    match job_id {
        Ok(job_id) => Ok(HttpResponse::Created().body(job_id.to_string())),
        Err(_) => Err(JobError::InternalError),
    }
}
