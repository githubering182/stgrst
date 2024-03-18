use crate::core::{DataBaseError, FileStream, Range};
use actix_web::{
    http::header::HeaderMap,
    web::{Data, Payload},
    Result,
};
use futures::{AsyncWriteExt, StreamExt};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    options::GridFsBucketOptions,
    Client, GridFsBucket,
};
use std::str::FromStr;

pub struct BucketService {
    bucket: GridFsBucket,
}

// TODO: impl bucket realease with closing?
impl BucketService {
    pub fn new(db: Data<Client>, bucket_name: String) -> Self {
        let mut options = GridFsBucketOptions::default();
        options.bucket_name = Some(bucket_name);

        // TODO: find out what happes here. will several coroutines use
        // same connection and is it blocking for each other?
        let bucket = db.database("storage_rs").gridfs_bucket(options);

        Self { bucket }
    }

    pub async fn upload(&self, mut payload: Payload) -> Result<String> {
        let mut upload_stream = self.bucket.open_upload_stream("filename", None);

        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            upload_stream.write_all(&chunk).await?;
        }

        upload_stream.close().await?;

        Ok(upload_stream.id().to_string())
    }

    pub async fn retrieve(
        &self,
        headers: &HeaderMap,
        file_id: String,
    ) -> Result<FileStream, DataBaseError> {
        let obj_id = match ObjectId::from_str(&file_id) {
            Ok(obj_id) => obj_id,
            Err(_) => return Err(DataBaseError::InternalError),
        };
        let cursor = self.bucket.find(doc! {"_id": obj_id}, None).await;

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

        let range = Range::new(headers, file.length);

        let download_stream = match self
            .bucket
            .open_download_stream(Bson::ObjectId(obj_id))
            .await
        {
            Ok(download_stream) => download_stream,
            Err(_) => return Err(DataBaseError::NotFoundError),
        };

        Ok(FileStream::new(download_stream, file, range))
    }
}
