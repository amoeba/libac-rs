use std::{error::Error, io::SeekFrom};

use reqwest::{Client, header};

pub trait AsyncRead {
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> impl std::future::Future<Output = std::io::Result<usize>> + Send;
}

pub trait AsyncSeek {
    fn seek(
        &mut self,
        pos: std::io::SeekFrom,
    ) -> impl std::future::Future<Output = std::io::Result<u64>> + Send;
}

pub struct HttpByteRangeReader {
    url: String,
    client: reqwest::Client,
    position: u64,
    content_length: u64,
}

impl HttpByteRangeReader {
    pub async fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();
        let resp = client.head(url).send().await?;
        let headers = resp.headers();

        // Error out if Accept-Ranges header is missing
        headers
            .get(header::ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Accept-Ranges header missing")
            })?;

        let content_length = headers
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Content-Length missing")
            })?;

        Ok(Self {
            url: url.to_string(),
            client,
            position: 0,
            content_length,
        })
    }
}

impl AsyncRead for HttpByteRangeReader {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let range = format!(
            "bytes={}-{}",
            self.position,
            self.position + buf.len() as u64 - 1
        );

        let resp = self
            .client
            .get(&self.url)
            .header(header::RANGE, range)
            .send()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let len = bytes.len();
        println!("Received {} bytes", len);
        buf[..len].copy_from_slice(&bytes);
        self.position += len as u64;

        Ok(len)
    }
}

impl AsyncSeek for HttpByteRangeReader {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => {
                if offset > 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek past end not supported",
                    ));
                }
                self.content_length.saturating_add(offset as u64)
            }
            SeekFrom::Current(offset) => {
                if offset >= 0 {
                    self.position.saturating_add(offset as u64)
                } else {
                    self.position.saturating_sub((-offset) as u64)
                }
            }
        };

        if new_pos > self.content_length {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Seek beyond content length",
            ));
        }

        self.position = new_pos;
        Ok(self.position)
    }
}
