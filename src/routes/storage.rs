use crate::{core::Range, services::BucketService};
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
pub async fn upload(db: &State<Client>, bucket_name: &str, data: Data<'_>) -> (Status, String) {
    match BucketService::new(db, bucket_name).upload(data).await {
        Err(message) => (Status::Conflict, message),
        Ok(file_id) => (Status::Created, file_id),
    }
}

#[get("/<bucket_name>/<file_id>")]
pub async fn retrieve(
    range: Range,
    db: &State<Client>,
    bucket_name: &str,
    file_id: &str,
) -> (ContentType, ReaderStream<One<Compat<GridFsDownloadStream>>>) {
    BucketService::new(db, bucket_name)
        .retrieve(range, file_id)
        .await
}
