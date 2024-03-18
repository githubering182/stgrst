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
    buffer: Vec<u8>,
    stream: GridFsDownloadStream,
    pub range: Range,
    pub file_name: String,
    chunk_size: u64,
    offset: u64,
}

impl FileStream {
    pub fn new(stream: GridFsDownloadStream, file: FilesCollectionDocument, range: Range) -> Self {
        Self {
            buffer: Vec::with_capacity(file.chunk_size_bytes as usize),
            stream,
            range,
            file_name: file.filename.unwrap_or(String::from("no_name")),
            chunk_size: file.chunk_size_bytes as u64,
            offset: 0,
        }
    }
}

impl Stream for FileStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.offset >= this.range.read_length {
            return Poll::Ready(None);
        }

        // let read_size = match this.chunk_size {
        //     chunk_size if chunk_size + this.offset > this.range.read_length => {
        //         this.range.read_length - this.offset
        //     }
        //     chunk_size => chunk_size,
        // };

        // let mut buffer = Vec::with_capacity(read_size as usize);
        let mut reader = this.stream.read_exact(&mut this.buffer);

        match reader.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Ok(_)) => {
                // this.offset += read_size;
                let buffer_data: Vec<u8> = this.buffer.drain(..).collect();
                this.offset += this.chunk_size;
                Poll::Ready(Some(Ok(Bytes::from(buffer_data))))
            }
        }
    }
}
