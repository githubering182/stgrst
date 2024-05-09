use crate::{
    core::{FileMeta, Range},
    BUCKET_CHUNK_SIZE, MAX_STREAM_LENGTH, MONGO_DB_NAME,
};
use futures::{AsyncWriteExt, StreamExt};
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

    pub async fn upload(&self, data: Data<'_>, meta: FileMeta<'_>) -> Result<String, String> {
        let bucket = self.bucket();

        let mut upload_stream = bucket.open_upload_stream(meta.name, None);
        let mut hasher = Md5::new();
        let mut stream = data.open(MAX_STREAM_LENGTH.gigabytes());
        let mut buffer = vec![0; self.read_size];

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

        let file_hash = self.get_hash_str(hasher);

        if self.search_by_hash(&file_hash).await.is_err() {
            _ = upload_stream.abort().await;
            _ = bucket.delete(upload_stream.id().clone()).await;
            return Err("Such file already exists".to_string());
        }

        _ = self.update_file(upload_stream.id(), file_hash, meta).await;

        Ok(upload_stream.id().to_string())
    }

    pub async fn retrieve(
        &self,
        _range: Range,
        file_id: &str,
    ) -> Result<(ContentType, ReaderStream<One<Compat<GridFsDownloadStream>>>), ()> {
        let bucket = self.bucket();
        let obj_id = ObjectId::from_str(file_id).unwrap();
        let _file = bucket.find(doc! {"_id": obj_id}, None).await.unwrap();

        let download_stream = bucket
            .open_download_stream(Bson::ObjectId(obj_id))
            .await
            .unwrap();

        Ok((
            ContentType::MP4,
            ReaderStream::one(download_stream.compat()),
        ))
    }
}

impl BucketService {
    async fn update_file<'r>(
        &self,
        file_id: &Bson,
        file_hash: String,
        meta: FileMeta<'r>,
    ) -> Result<(), ()> {
        let collection = self.collection();

        let new_meta = doc! {
            "metadata": doc! {
                "hash": file_hash,
                "type": meta._type,
                "extension": meta.extension
            }
        };
        _ = collection
            .update_one(doc! { "_id": file_id }, doc! { "$set":  new_meta }, None)
            .await;

        Ok(())
    }

    async fn search_by_hash(&self, hash: &String) -> Result<(), ()> {
        let bucket = self.bucket();

        match bucket.find(doc! {"metadata.hash": hash}, None).await {
            Err(_) => Ok(()),
            Ok(mut cursor) => match cursor.next().await {
                Some(_) => Err(()),
                None => Ok(()),
            },
        }
    }

    fn get_hash_str(&self, hasher: Md5) -> String {
        hasher
            .finalize()
            .into_iter()
            .map(|ch| ch.to_string())
            .collect::<Vec<String>>()
            .join("")
    }

    fn bucket(&self) -> GridFsBucket {
        let mut options = GridFsBucketOptions::default();
        options.bucket_name = Some(self.bucket_name.clone());

        self.database.gridfs_bucket(options)
    }

    fn collection(&self) -> Collection<Document> {
        let bucket_files = self.bucket_name.to_owned() + ".files";
        self.database.collection(bucket_files.as_str())
    }
}
