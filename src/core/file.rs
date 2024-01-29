use actix_web::{
    http::header::{HeaderMap, Range},
    web::Bytes,
    HttpRequest,
};
use futures::{AsyncReadExt, FutureExt, Stream};
use mongodb::{gridfs::FilesCollectionDocument, GridFsDownloadStream};
use std::{
    io::Error,
    pin::Pin,
    task::{Context, Poll},
};

pub struct FileStream {
    stream: GridFsDownloadStream,
    range: Range,
    length: u64,
    chunk_size: u64,
    offset: u64,
}

impl FileStream {
    pub fn new(
        file: FilesCollectionDocument,
        stream: GridFsDownloadStream,
        request: HttpRequest,
    ) -> Self {
        Self {
            stream,
            range: FileStream::parse_range(request.headers(), file.length),
            length: file.length,
            chunk_size: file.chunk_size_bytes as u64,
            offset: 0,
        }
    }

    pub fn parse_range(headers: &HeaderMap, file_length: u64) -> Range {
        let (start, mut end) = match headers.get("range") {
            None => (0, file_length),
            Some(range) => {
                let mut range_split = range.to_str().unwrap().split("=");
                let h_type = range_split.next();
                let h_data = range_split.next();

                match h_type.is_none() || h_data.is_none() {
                    true => (0, file_length),
                    false if h_type.unwrap() != "bytes" => (0, file_length),
                    _ => {
                        let parsed: Vec<&str> = h_data.unwrap().split("-").collect();
                        (
                            parsed[0].parse::<u64>().unwrap_or(0),
                            parsed[1].parse::<u64>().unwrap_or(255 * 1024),
                        )
                    }
                }
            }
        };

        if end >= file_length {
            end = file_length - 1;
        }

        Range::bytes(start, end)
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
