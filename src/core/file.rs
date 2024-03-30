use super::Range;
use actix_web::web::Bytes;
use futures::{AsyncReadExt, FutureExt, Stream};
use mongodb::{gridfs::FilesCollectionDocument, GridFsDownloadStream};
use std::{
    io::Error,
    pin::Pin,
    task::{Context, Poll},
};

// TODO: properly align struct fields
pub struct FileStream {
    stream: GridFsDownloadStream,
    pub range: Range,
    pub file_name: String,
    chunk_size: u64,
}

impl FileStream {
    pub fn new(stream: GridFsDownloadStream, file: FilesCollectionDocument, range: Range) -> Self {
        Self {
            stream,
            range,
            file_name: file.filename.unwrap_or(String::from("no_name")),
            chunk_size: file.chunk_size_bytes as u64,
        }
    }
}

impl Stream for FileStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        let mut buffer = vec![0; this.chunk_size as usize];
        let mut poll = this.stream.read(&mut buffer);

        match poll.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e))),
            Poll::Ready(Ok(0)) => return Poll::Ready(None),
            Poll::Ready(Ok(read)) => {
                buffer.truncate(read);
                return Poll::Ready(Some(Ok(Bytes::from(buffer))));
            }
        }
    }
}
