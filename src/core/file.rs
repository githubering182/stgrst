use super::_Range;
use actix_web::web::Bytes;
use futures::{AsyncReadExt, FutureExt, Stream};
use mongodb::GridFsDownloadStream;
use std::{
    io::Error,
    pin::Pin,
    task::{Context, Poll},
};

// TODO: properly align struct fields
pub struct FileStream {
    stream: GridFsDownloadStream,
    range: _Range,
    chunk_size: u64,
    offset: u64,
}

impl FileStream {
    pub fn new(stream: GridFsDownloadStream, chunk_size: u32, range: _Range) -> Self {
        Self {
            stream,
            range,
            chunk_size: chunk_size as u64,
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

        let read_size = match this.chunk_size {
            chunk_size if chunk_size + this.offset > this.range.read_length => {
                this.range.read_length - this.offset
            }
            chunk_size => chunk_size,
        };
        // TODO:
        // appread that pending on read will give pending as output and as
        // wrappers getting polled again it goes all over again
        // maybe i can await for poll of reading w loop or something
        // ex: loop {
        //     match res.poll_unpin(cx) {
        //         Poll::Pending => {
        //             continue;
        //         }
        //         Poll::Ready(Err(e)) => {
        //             return Poll::Ready(Some(Err(e)));
        //         }
        //         Poll::Ready(Ok(_)) => {
        //             this.offset += read_size;
        //             return Poll::Ready(Some(Ok(Bytes::from(buf))));
        //         }
        //     }
        // }
        // consider bffer allocation in other approach
        let mut buf = vec![0; read_size as usize];
        // let mut buf = Vec::with_capacity(read_size as usize);
        let mut res = this.stream.read_exact(&mut buf);

        match res.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Ok(_)) => {
                this.offset += read_size;
                Poll::Ready(Some(Ok(Bytes::from(buf))))
            }
        }
    }
}
