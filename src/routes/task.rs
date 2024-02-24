use crate::core::{ArchiveJob, JobError};
use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder, ResponseError, Result,
};
use apalis::{prelude::Storage, redis::RedisStorage};
use std::sync::{Arc, Mutex};

#[get("/archive/{message}")]
pub async fn produce(
    path: Path<String>,
    broker: Data<Arc<Mutex<RedisStorage<ArchiveJob>>>>,
) -> Result<impl Responder, impl ResponseError> {
    let job_id = match broker.lock() {
        Ok(mut b) => {
            b.push(ArchiveJob {
                task: path.into_inner(),
            })
            .await
        }
        Err(_) => return Err(JobError::ConnectionError),
    };

    match job_id {
        Ok(job_id) => Ok(HttpResponse::Created().body(job_id.to_string())),
        Err(_) => Err(JobError::InternalError),
    }
}
