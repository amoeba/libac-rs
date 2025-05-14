use futures::FutureExt;
use futures::future::BoxFuture;
use futures::io::{AsyncRead, AsyncSeek, SeekFrom};
use futures::task::{Context, Poll};
use std::error::Error;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::pin::Pin;

use tokio::runtime::Handle;

use super::async_http_chunk_reader::AsyncHttpChunkReader; // For the simplified adapter

pub struct AsyncHttpIoAdapter {
    inner: AsyncHttpChunkReader, // Direct ownership
    read_future: Option<BoxFuture<'static, Result<(), Box<dyn Error + Send + Sync>>>>,
    seek_future: Option<BoxFuture<'static, Result<u64, Box<dyn Error + Send + Sync>>>>,
    temp_buf: Vec<u8>,
}
impl AsyncHttpIoAdapter {
    pub fn new(reader: AsyncHttpChunkReader) -> Self {
        Self {
            inner: reader,
            read_future: None,
            seek_future: None,
            temp_buf: Vec::new(),
        }
    }
}

// Helper to convert generic Box<dyn Error> to std::io::Error
fn to_io_error(e: Box<dyn Error + Send + Sync>) -> IoError {
    // Attempt to downcast to IoError first
    if let Some(io_err) = e.downcast_ref::<IoError>() {
        // This requires IoError to be created with `new` rather than `other`.
        // For simplicity, just wrap it.
        return IoError::new(IoErrorKind::Other, format!("{}", e));
    }
    IoError::new(IoErrorKind::Other, e.to_string())
}

impl AsyncRead for AsyncHttpIoAdapter {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<IoResult<usize>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let buf_len = buf.len();

        self.temp_buf = vec![0u8; buf_len];
        let read_fut = self.inner.read(&mut self.temp_buf);
        self.read_future = Some(Box::pin(async {
            read_fut.await?;
            Ok(())
        }));

        let fut = self.read_future.as_mut().unwrap();
        match fut.as_mut().poll(cx) {
            Poll::Ready(Ok(())) => match fut.as_mut().poll(cx) {
                Poll::Ready(Ok(())) => {
                    self.read_future = None;
                    buf[..self.temp_buf.len()].copy_from_slice(&self.temp_buf);
                    Poll::Ready(Ok(self.temp_buf.len()))
                }
                Poll::Ready(Err(e)) => {
                    self.read_future = None;
                    Poll::Ready(Err(to_io_error(e)))
                }
                Poll::Pending => Poll::Pending,
            },
            Poll::Ready(Err(e)) => {
                self.read_future = None;
                Poll::Ready(Err(to_io_error(e)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncSeek for AsyncHttpIoAdapter {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<IoResult<u64>> {
        if self.seek_future.is_none() {
            let inner = &mut self.inner;
            self.seek_future = Some(Box::pin(inner.seek(pos)));
        }

        let fut = self.seek_future.as_mut().unwrap();
        match fut.as_mut().poll(cx) {
            Poll::Ready(Ok(pos)) => {
                self.seek_future = None;
                Poll::Ready(Ok(pos))
            }
            Poll::Ready(Err(e)) => {
                self.seek_future = None;
                Poll::Ready(Err(to_io_error(e)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Required for AsyncRead/AsyncSeek if passing &mut R
impl Unpin for AsyncHttpIoAdapter {}
