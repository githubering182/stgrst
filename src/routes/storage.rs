use crate::core::{DataBaseError, FileStream};
use actix_web::{
    get,
    http::header::ContentDisposition,
    post,
    web::{Data, Path, Payload},
    HttpRequest, HttpResponse, Responder, ResponseError, Result,
};
use futures::{AsyncWriteExt, StreamExt};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    options::GridFsBucketOptions,
    Database,
};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

// TODO: rewrite to use results with ? operator
#[post("/file")]
pub async fn upload(
    database: Data<Arc<RwLock<Database>>>,
    mut payload: Payload,
) -> Result<impl Responder> {
    let mut options = GridFsBucketOptions::default();
    options.bucket_name = Some("test".to_string());

    let bucket = match database.read() {
        Ok(db) => db.gridfs_bucket(options),
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let mut upload_stream = bucket.open_upload_stream("filename", None);

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        upload_stream.write_all(&chunk).await?;
    }

    upload_stream.close().await?;

    Ok(HttpResponse::Created().body(upload_stream.id().to_string()))
}

// TODO: rewrite to use results with ? operator
#[get("/file/{type}/{file_id}/")]
pub async fn retrieve(
    request: HttpRequest,
    database: Data<Arc<RwLock<Database>>>,
    path: Path<(String, String)>,
) -> Result<impl Responder, impl ResponseError> {
    let (file_type, file_id) = path.into_inner();

    let obj_id = match ObjectId::from_str(&file_id) {
        Ok(obj_id) => obj_id,
        Err(_) => return Err(DataBaseError::InternalError),
    };

    let mut options = GridFsBucketOptions::default();
    options.bucket_name = Some("test".to_string());

    let bucket = match database.read() {
        Ok(db) => db.gridfs_bucket(options),
        Err(_) => return Err(DataBaseError::NotFoundError),
    };

    let cursor = bucket.find(doc! {"_id": obj_id}, None).await;

    let file = match cursor {
        Ok(mut cursor) => {
            if let Some(file) = cursor.next().await {
                file.unwrap()
            } else {
                return Err(DataBaseError::NotFoundError);
            }
        }
        Err(_) => return Err(DataBaseError::NotFoundError),
    };

    let download_stream = match bucket.open_download_stream(Bson::ObjectId(obj_id)).await {
        Ok(download_stream) => download_stream,
        Err(_) => return Err(DataBaseError::NotFoundError),
    };

    let response = HttpResponse::Ok()
        .append_header(ContentDisposition::attachment("some.zip"))
        .streaming(FileStream::new(file, download_stream, request));

    Ok(response)
}
