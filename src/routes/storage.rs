use crate::services::BucketService;
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
    let result_id = BucketService::new(db, bucket_name)
        .upload(data)
        .await
        .unwrap();
    (Status::Created, result_id)
}

#[get("/<bucket_name>/<file_id>")]
pub async fn retrieve(
    db: &State<Client>,
    bucket_name: &str,
    file_id: &str,
) -> (ContentType, ReaderStream<One<Compat<GridFsDownloadStream>>>) {
    BucketService::new(db, bucket_name).retrieve(file_id).await
}
