use crate::core::FileStream;
use actix_web::{
    get, post,
    web::{Data, Path, Payload},
    HttpRequest, HttpResponse, Responder,
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

#[post("/file")]
pub async fn upload(database: Data<Arc<RwLock<Database>>>, mut payload: Payload) -> impl Responder {
    let mut options = GridFsBucketOptions::default();
    options.bucket_name = Some("test".to_string());

    let bucket = database.read().unwrap().gridfs_bucket(options);

    let mut chunk_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        chunk_data.extend_from_slice(&chunk.unwrap());
    }

    let mut upload_stream = bucket.open_upload_stream("filename", None);
    upload_stream.write_all(&chunk_data).await.unwrap();

    match upload_stream.close().await {
        Ok(_) => "ok",
        Err(_) => "err",
    }
}

#[get("/file/{type}/{file_id}")]
pub async fn retrieve(
    request: HttpRequest,
    database: Data<Arc<RwLock<Database>>>,
    path: Path<(String, String)>,
) -> impl Responder {
    let (file_type, file_id) = path.into_inner();

    let obj_id = ObjectId::from_str(&file_id).unwrap();

    let mut options = GridFsBucketOptions::default();
    options.bucket_name = Some("test".to_string());

    let bucket = database.read().unwrap().gridfs_bucket(options);

    let file = bucket
        .find(doc! {"_id": obj_id}, None)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();

    let download_stream = bucket
        .open_download_stream(Bson::ObjectId(obj_id))
        .await
        .unwrap();

    let stream = FileStream::new(file, download_stream, request);

    let mut response = HttpResponse::Ok();

    response.streaming(stream)
}
