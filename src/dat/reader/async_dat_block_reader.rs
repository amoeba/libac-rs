use futures::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};
use std::error::Error;
use std::io::{Cursor, Write};

#[derive(Debug)]
pub struct AsyncDatBlockReader {}

impl AsyncDatBlockReader {
    pub async fn read<R>(
        reader: &mut R,
        offset: u32,     // Initial offset to find the first 4-byte "next_address_pointer"
        size: u32,       // Total size of data to read
        block_size: u32, // Size of a "block unit" in the file. A block unit contains (block_size - 4) data bytes and a 4-byte pointer.
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>
    where
        R: AsyncRead + AsyncSeek + Unpin + Send + ?Sized,
    {
        reader.seek(SeekFrom::Start(offset as u64)).await?;

        let mut buffer = vec![0; size as usize];
        let mut writer = Cursor::new(&mut buffer);
        let mut left_to_read = size;

        let mut u32_bytes = [0u8; 4];
        reader.read_exact(&mut u32_bytes).await?;
        let mut next_block_header_location = u32::from_le_bytes(u32_bytes) as u64;

        while left_to_read > 0 {
            if left_to_read < block_size {
                let mut data_chunk = vec![0u8; left_to_read as usize];
                reader.read_exact(&mut data_chunk).await?;
                writer
                    .write_all(&data_chunk)
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?; // Handle error for write_all
                break; // Exit loop, all data read
            } else {
                let data_payload_size = (block_size - 4) as usize;
                let mut data_chunk = vec![0u8; data_payload_size];

                reader.read_exact(&mut data_chunk).await?;

                writer
                    .write_all(&data_chunk)
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

                left_to_read -= data_payload_size as u32;

                reader
                    .seek(SeekFrom::Start(next_block_header_location))
                    .await?;

                let mut u32_bytes = [0u8; 4];
                reader.read_exact(&mut u32_bytes).await?;
                next_block_header_location = u32::from_le_bytes(u32_bytes) as u64;
            }
        }
        Ok(buffer)
    }
}
