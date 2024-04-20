use crate::core::Range;
use futures::AsyncWriteExt;
use md5::{Digest, Md5};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    options::GridFsBucketOptions,
    Client, GridFsBucket, GridFsDownloadStream,
};
use rocket::{
    data::ToByteUnit,
    http::ContentType,
    response::stream::{One, ReaderStream},
    tokio::io::AsyncReadExt,
    Data, State,
};
use std::io::Result;
use std::str::FromStr;
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};

pub struct BucketService {
    bucket: GridFsBucket,
    read_size: usize,
}

// TODO: impl bucket realease with closing?
impl BucketService {
    pub fn new(db: &State<Client>, bucket_name: &str) -> Self {
        let mut options = GridFsBucketOptions::default();
        options.bucket_name = Some(bucket_name.to_owned());

        // TODO: find out what happes here. will several coroutines use
        // same connection and is it blocking for each other?
        let bucket = db.database("storage_rs").gridfs_bucket(options);

        Self {
            bucket,
            read_size: 512 * 1024,
        }
    }

    pub async fn upload(&self, data: Data<'_>) -> Result<String> {
        let mut upload_stream = self.bucket.open_upload_stream("filename", None);

        let mut stream = data.open(10.gigabytes());
        let mut buffer = vec![0; self.read_size];
        let mut hasher = Md5::new();

        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(read) if read < self.read_size => buffer.truncate(read),
                _ => (),
            }

            upload_stream.write_all(&buffer).await.unwrap();
            hasher.update(&buffer);
        }

        let _file_hash = hasher.finalize();

        upload_stream.close().await.unwrap();

        Ok(upload_stream.id().to_string())
    }

    pub async fn retrieve(
        &self,
        _range: Range,
        file_id: &str,
    ) -> (ContentType, ReaderStream<One<Compat<GridFsDownloadStream>>>) {
        let obj_id = ObjectId::from_str(file_id).unwrap();
        let cursor = self.bucket.find(doc! {"_id": obj_id}, None).await;

        let _file = cursor.unwrap();

        let download_stream = self
            .bucket
            .open_download_stream(Bson::ObjectId(obj_id))
            .await
            .unwrap();

        (
            ContentType::MP4,
            ReaderStream::one(download_stream.compat()),
        )
    }
}
