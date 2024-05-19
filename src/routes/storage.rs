use crate::{
    core::{FileMeta, Range},
    services::BucketService,
};
use mongodb::{bson::doc, Client, GridFsDownloadStream};
use rocket::{
    get,
    http::{ContentType, Status},
    post,
    response::stream::{One, ReaderStream},
    Data, State,
};
use tokio_util::compat::Compat;

#[post("/<bucket_name>", data = "<data>")]
pub async fn upload(
    db: &State<Client>,
    bucket_name: &str,
    data: Data<'_>,
    meta: FileMeta<'_>,
) -> (Status, String) {
    let bucket_service = BucketService::new(db, bucket_name);
    match bucket_service.upload(data, meta).await {
        Ok(file_id) => (Status::Created, file_id),
        Err(message) => (Status::Conflict, message),
    }
}

#[get("/<bucket_name>/<file_id>")]
pub async fn retrieve(
    range: Range,
    db: &State<Client>,
    bucket_name: &str,
    file_id: &str,
) -> (
    Status,
    Option<(ContentType, ReaderStream<One<Compat<GridFsDownloadStream>>>)>,
) {
    let bucket_service = BucketService::new(db, bucket_name);
    match bucket_service.retrieve(range, file_id).await {
        Ok(result) => (Status::Ok, Some(result)),
        Err(_) => (Status::NotFound, None),
    }
}
