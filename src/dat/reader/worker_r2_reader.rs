use std::error::Error;
use worker::{Env, Range};

use super::async_file_reader::RangeReader;

/// Cloudflare Worker R2 implementation of RangeReader
/// Uses the Worker runtime's R2 API through environment bindings
pub struct WorkerR2RangeReader {
    env: Env,
    bucket_name: String,
    key: String,
}

impl WorkerR2RangeReader {
    pub fn new(env: Env, bucket_name: String, key: String) -> Self {
        Self {
            env,
            bucket_name,
            key,
        }
    }
}

impl RangeReader for WorkerR2RangeReader {
    fn read_range(
        &mut self,
        offset: u32,
        length: usize,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Box<dyn Error>>> + Send {
        let env = self.env.clone();
        let bucket_name = self.bucket_name.clone();
        let key = self.key.clone();

        async move {
            let bucket = env.bucket(&bucket_name).map_err(|e| -> Box<dyn Error> {
                format!("Failed to get R2 bucket: {:?}", e).into()
            })?;

            let range = Range {
                offset: Some(offset as usize),
                length: Some(length),
                suffix: None,
            };

            let object = bucket
                .get(&key)
                .range(range)
                .execute()
                .await
                .map_err(|e| -> Box<dyn Error> { format!("R2 get failed: {:?}", e).into() })?;

            match object {
                Some(obj) => {
                    let stream = obj
                        .body()
                        .ok_or_else(|| -> Box<dyn Error> { "No body in R2 object".into() })?;

                    let bytes = stream.bytes().await.map_err(|e| -> Box<dyn Error> {
                        format!("Failed to read bytes: {:?}", e).into()
                    })?;

                    Ok(bytes.to_vec())
                }
                None => Err("Object not found in R2 bucket".into()),
            }
        }
    }
}
