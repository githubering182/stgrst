use actix_cors::Cors;
use actix_web::{
    get,
    middleware::Logger,
    post,
    web::{Bytes, BytesMut, Data, Path, Payload},
    App, HttpResponse, HttpServer, Responder,
};
use futures::{
    io::ReadExact, AsyncReadExt, AsyncWriteExt, FutureExt, Stream, StreamExt, TryStreamExt,
};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    gridfs::FilesCollectionDocument,
    options::{ClientOptions, GridFsBucketOptions, GridFsUploadOptions},
    results::InsertOneResult,
    Client, Database, GridFsDownloadStream,
};
use std::{
    io::{Error, Read},
    pin::Pin,
    str::FromStr,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

#[get("/file/{type}/{file_id}")]
async fn retrieve(
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

    let stream = FileStream::new(file, download_stream);

    let mut response = HttpResponse::Ok();

    response.streaming(stream)
}

struct FileStream {
    stream: GridFsDownloadStream,
    length: u64,
    chunk_size: u64,
    offset: u64,
}

impl FileStream {
    fn new(file: FilesCollectionDocument, stream: GridFsDownloadStream) -> Self {
        Self {
            stream,
            length: file.length,
            chunk_size: file.chunk_size_bytes as u64,
            offset: 0,
        }
    }
}

impl Stream for FileStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.offset >= this.length {
            return Poll::Ready(None);
        }

        let read_size = match this.chunk_size {
            chunk_size if chunk_size + this.offset > this.length => {
                let chunk_size = this.length - this.offset;
                chunk_size
            }
            chunk_size => chunk_size,
        };

        this.offset += read_size;

        let mut buf = vec![0; read_size as usize];
        let mut res = this.stream.read_exact(&mut buf);

        match res.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(_)) => Poll::Ready(None),
            Poll::Ready(Ok(_)) => Poll::Ready(Some(Ok(Bytes::from(buf)))),
        }
    }
}

#[post("/file")]
async fn upload(database: Data<Arc<RwLock<Database>>>, mut payload: Payload) -> impl Responder {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let database = Arc::new(RwLock::new(client.database("storage_rs")));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .supports_credentials()
            .allowed_headers(vec![
                "Content-Type",
                "Authorization",
                "Access-Control-Allow-Origin",
            ])
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"]);
        App::new()
            .app_data(Data::new(database.clone()))
            .wrap(Logger::default())
            .wrap(cors)
            .service(upload)
            .service(retrieve)
    })
    .workers(4)
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
