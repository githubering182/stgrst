use crate::{
    core::{FileMeta, Range},
    BUCKET_CHUNK_SIZE, MAX_STREAM_LENGTH, MONGO_DB_NAME, SKIP_BUFFER_SIZE,
};
use futures::{AsyncReadExt as FuturesAsyncRead, AsyncWriteExt, StreamExt};
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
use std::{cmp::min, str::FromStr};
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
        range: Range,
        file_id: &str,
    ) -> Result<(ContentType, ReaderStream<One<Compat<GridFsDownloadStream>>>), ()> {
        let bucket = self.bucket();
        let obj_id = ObjectId::from_str(file_id).unwrap();
        let file = match bucket
            .find(doc! {"_id": obj_id}, None)
            .await
            .unwrap()
            .next()
            .await
        {
            Some(file) => file.unwrap(),
            None => unreachable!(),
        };

        let mut download_stream = match bucket.open_download_stream(Bson::ObjectId(obj_id)).await {
            Ok(stream) => stream,
            Err(_) => unreachable!(),
        };

        if range.partial {
            _ = self
                .skip_head(&mut download_stream, range.start as usize)
                .await;
        }

        let content_type = self.get_content_type(file.metadata.unwrap().get("extension"));

        Ok((content_type, ReaderStream::one(download_stream.compat())))
    }
}

impl BucketService {
    async fn skip_head(&self, stream: &mut GridFsDownloadStream, mut to: usize) -> Result<(), ()> {
        let mut skip_buffer = vec![0; SKIP_BUFFER_SIZE];

        while to > 0 {
            let read_size = min(to, SKIP_BUFFER_SIZE);
            match stream.read_exact(&mut skip_buffer[..read_size]).await {
                Ok(_) => to -= read_size,
                Err(_) => return Err(()),
            }
        }

        Ok(())
    }

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

    fn get_content_type(&self, file_extensin: Option<&Bson>) -> ContentType {
        match file_extensin {
            Some(Bson::String(extension)) => match extension.to_lowercase().as_str() {
                "mp4" => ContentType::MP4,
                "mov" => ContentType::MOV,
                "mpeg" | "mpg" => ContentType::MPEG,
                "jpg" | "jpeg" => ContentType::JPEG,
                "png" => ContentType::PNG,
                _ => ContentType::Bytes,
            },
            _ => ContentType::Bytes,
        }
    }
}
