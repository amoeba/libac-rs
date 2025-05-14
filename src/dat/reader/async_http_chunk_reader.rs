use futures::io::SeekFrom;
use std::error::Error;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use reqwest::{
    Client,
    header::{HeaderValue, RANGE},
};

#[derive(Clone)]
pub struct AsyncHttpChunkReader {
    client: Client,
    url: String,
    current_pos: u64,
    total_size: Option<u64>, // Cached total size
}

impl AsyncHttpChunkReader {
    pub async fn new(client: Client, url: String) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Optionally, get total size with a HEAD request for robust SeekFrom::End
        let head_resp = client.head(&url).send().await?;
        if !head_resp.status().is_success() {
            return Err(Box::new(IoError::new(
                IoErrorKind::Other,
                format!("HTTP HEAD Error: {}", head_resp.status()),
            )));
        }
        let total_size = head_resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        Ok(AsyncHttpChunkReader {
            client,
            url,
            current_pos: 0,
            total_size,
        })
    }

    async fn ensure_total_size(&mut self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        if let Some(size) = self.total_size {
            return Ok(size);
        }
        // Fetch and cache
        let head_resp = self.client.head(&self.url).send().await?;
        if !head_resp.status().is_success() {
            return Err(Box::new(IoError::new(
                IoErrorKind::Other,
                format!("HTTP HEAD Error: {}", head_resp.status()),
            )));
        }
        let size = head_resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| {
                IoError::new(
                    IoErrorKind::NotFound,
                    "Content-Length header missing or invalid",
                )
            })?;
        self.total_size = Some(size);
        Ok(size)
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if buf.is_empty() {
            return Ok(());
        }

        let read_offset = self.current_pos;
        let read_len = buf.len() as u64;

        if read_len == 0 {
            return Ok(());
        }

        let range_end = read_offset + read_len - 1;
        let range_header_val = format!("bytes={}-{}", read_offset, range_end);

        let response = self
            .client
            .get(&self.url)
            .header(RANGE, HeaderValue::from_str(&range_header_val)?)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            return Err(Box::new(IoError::new(
                IoErrorKind::UnexpectedEof,
                format!(
                    "Range not satisfiable: {} (offset: {}, len: {}) trying to read from URL: {}",
                    response.status(),
                    read_offset,
                    read_len,
                    self.url
                ),
            )));
        }
        if !response.status().is_success() {
            return Err(Box::new(IoError::new(
                IoErrorKind::Other,
                format!("HTTP Error: {} for URL: {}", response.status(), self.url),
            )));
        }

        let fetched_bytes = response.bytes().await?;
        if fetched_bytes.len() != buf.len() {
            return Err(Box::new(IoError::new(
                IoErrorKind::UnexpectedEof,
                format!(
                    "HTTP range request did not return the exact number of bytes requested. Expected {}, got {}. URL: {}",
                    buf.len(),
                    fetched_bytes.len(),
                    self.url
                ),
            )));
        }

        buf.copy_from_slice(&fetched_bytes);
        self.current_pos += read_len; // Advance cursor
        Ok(())
    }

    pub async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let new_abs_pos: u64 = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(offset) => if offset >= 0 {
                self.current_pos.checked_add(offset as u64)
            } else {
                self.current_pos.checked_sub((-offset) as u64)
            }
            .ok_or_else(|| {
                IoError::new(
                    IoErrorKind::InvalidInput,
                    "Seek resulted in an invalid position (overflow/underflow).",
                )
            })?,
            SeekFrom::End(offset_from_end) => {
                let total = self.ensure_total_size().await?;
                if offset_from_end >= 0 {
                    total.checked_add(offset_from_end as u64)
                } else {
                    total.checked_sub((-offset_from_end) as u64)
                }
                .ok_or_else(|| {
                    IoError::new(
                        IoErrorKind::InvalidInput,
                        "Seek from end resulted in an invalid position (overflow/underflow).",
                    )
                })?
            }
        };
        self.current_pos = new_abs_pos;
        Ok(self.current_pos)
    }
}
