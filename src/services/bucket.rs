use crate::{core::Range, BUCKET_CHUNK_SIZE, MAX_STREAM_LENGTH, MONGO_DB_NAME};
use futures::AsyncWriteExt;
use md5::{Digest, Md5};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    options::GridFsBucketOptions,
    Client, Collection, Database, GridFsBucket, GridFsDownloadStream,
};
use rocket::{
    data::ToByteUnit,
    http::ContentType,
    response::stream::{One, ReaderStream},
    tokio::io::AsyncReadExt,
    Data, State,
};
use std::str::FromStr;
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};

// TODO: impl bucket realease with closing?
pub struct BucketService {
    database: Database,
    bucket_name: String,
    read_size: usize,
}

impl BucketService {
    pub fn new(db: &State<Client>, bucket_name: &str) -> Self {
        let database = db.database(MONGO_DB_NAME);

        Self {
            database,
            bucket_name: bucket_name.into(),
            read_size: BUCKET_CHUNK_SIZE,
        }
    }

    pub async fn upload(&self, data: Data<'_>) -> Result<String, String> {
        let bucket = self.bucket();
        let collection = self.collection();

        let mut buffer = vec![0; self.read_size];
        let mut upload_stream = bucket.open_upload_stream("temp_file", None);
        let mut stream = data.open(MAX_STREAM_LENGTH.gigabytes());
        let mut hasher = Md5::new();

        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(read) if read < self.read_size => buffer.truncate(read),
                _ => (),
            }
            _ = upload_stream.write_all(&buffer).await;
            hasher.update(&buffer);
        }

        _ = upload_stream.close().await;

        let file_hash = hasher
            .finalize()
            .into_iter()
            .map(|ch| ch.to_string())
            .collect::<Vec<String>>()
            .join("");

        if self.search_by_hash(&file_hash).await.is_err() {
            _ = upload_stream.abort().await;
            _ = bucket.delete(upload_stream.id().clone()).await;
            return Err("Such file already exists".to_string());
        }

        _ = collection
            .update_one(
                doc! { "_id": upload_stream.id() },
                doc! { "$set": doc! {"metadata": file_hash} },
                None,
            )
            .await;

        Ok(upload_stream.id().to_string())
    }

    pub async fn retrieve(
        &self,
        _range: Range,
        file_id: &str,
    ) -> (ContentType, ReaderStream<One<Compat<GridFsDownloadStream>>>) {
        let bucket = self.bucket();
        let obj_id = ObjectId::from_str(file_id).unwrap();
        let cursor = bucket.find(doc! {"_id": obj_id}, None).await;

        let _file = cursor.unwrap();

        let download_stream = bucket
            .open_download_stream(Bson::ObjectId(obj_id))
            .await
            .unwrap();

        (
            ContentType::MP4,
            ReaderStream::one(download_stream.compat()),
        )
    }
}

impl BucketService {
    async fn search_by_hash(&self, hash: &String) -> Result<(), ()> {
        let bucket = self.bucket();
        match bucket.find(doc! {"meta.hash": hash}, None).await {
            Err(_) => Ok(()),
            Ok(_) => Err(()),
        }
    }
    fn bucket(&self) -> GridFsBucket {
        let mut options = GridFsBucketOptions::default();
        options.bucket_name = Some(self.bucket_name.clone());

        self.database.gridfs_bucket(options)
    }
    fn collection(&self) -> Collection<Document> {
        self.database.collection(self.bucket_name.as_str())
    }
}
