use super::RetrieveQuery;
use crate::core::_Range;
use crate::core::{DataBaseError, FileStream};
use actix_web::{
    get,
    http::header::{ContentDisposition, ContentRange, ContentRangeSpec, ACCEPT_RANGES},
    post,
    web::{Data, Path, Payload, Query},
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
#[post("/file/{bucket}/")]
pub async fn upload(
    path: Path<String>,
    database: Data<Arc<RwLock<Database>>>,
    mut payload: Payload,
) -> Result<impl Responder> {
    let bucket = path.into_inner();
    let mut options = GridFsBucketOptions::default();
    options.bucket_name = Some(bucket);

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
#[get("/file/{bucket}/{file_id}/")]
pub async fn retrieve(
    request: HttpRequest,
    database: Data<Arc<RwLock<Database>>>,
    path: Path<(String, String)>,
    archive_query: Query<RetrieveQuery>,
) -> Result<impl Responder, impl ResponseError> {
    let (bucket, file_id) = path.into_inner();

    let obj_id = match ObjectId::from_str(&file_id) {
        Ok(obj_id) => obj_id,
        Err(_) => return Err(DataBaseError::InternalError),
    };

    let mut options = GridFsBucketOptions::default();
    options.bucket_name = Some(bucket);

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
    let range = _Range::new(request.headers(), file.length);

    let mut response = HttpResponse::Ok();
    match archive_query.archive {
        Some(archive) if archive => {
            response.append_header(ContentDisposition::attachment(
                file.filename.as_ref().unwrap().clone(),
            ));
        }
        _ => {
            response.append_header(ContentRange(ContentRangeSpec::Bytes {
                range: Some((range.start, range.end)),
                instance_length: Some(range.read_length),
            }));
        }
    }
    response.streaming(FileStream::new(
        download_stream,
        file.chunk_size_bytes,
        range,
    ));

    drop(file);
    Ok(response)
}
