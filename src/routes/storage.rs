use crate::core::{DataBaseError, FileStream};
use actix_web::{
    error, get, post,
    web::{Data, Path, Payload},
    Error, HttpRequest, HttpResponse, Responder, ResponseError, Result,
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

#[get("/test")]
pub async fn test(rq: HttpRequest) -> impl Responder {
    let x = FileStream::parse_range(rq.headers(), 2014);
    println!("rq hs: {:?}", x);
    "ok"
}

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
        Err(_) => return Ok(HttpResponse::InternalServerError()),
    };

    let mut chunk_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        chunk_data.extend_from_slice(&chunk.unwrap());
    }

    let mut upload_stream = bucket.open_upload_stream("filename", None);

    upload_stream.write_all(&chunk_data).await?;
    upload_stream.close().await?;

    Ok(HttpResponse::Ok())
}

// TODO: rewrite to use results with ? operator
#[get("/file/{type}/{file_id}")]
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

    let stream = FileStream::new(file, download_stream, request);

    Ok(HttpResponse::Ok().streaming(stream))
}
