use byteorder::{LittleEndian, ReadBytesExt};
use futures::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
pub struct DatBlockReader {}

impl DatBlockReader {
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        offset: u32,
        size: u32,
        block_size: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        reader.seek(SeekFrom::Start(offset as u64))?;
        let mut buffer = vec![0; size as usize];
        let mut writer = Cursor::new(&mut buffer);
        let mut left_to_read = size;
        let mut next_address = reader.read_u32::<LittleEndian>()?;
        while left_to_read > 0 {
            if left_to_read < block_size {
                let mut data: Vec<u8> = vec![0; left_to_read as usize];
                reader.read_exact(&mut data)?;
                writer.write_all(&data)?;
                break;
            } else {
                let mut data: Vec<u8> = vec![0; (block_size as usize) - 4];
                reader.read_exact(&mut data)?;
                writer.write_all(&data)?;
                reader.seek(SeekFrom::Start(next_address as u64))?;
                next_address = reader.read_u32::<LittleEndian>()?;
                left_to_read -= block_size - 4;
            }
        }
        Ok(buffer)
    }

    // New async implementation
    pub async fn read_async<R>(
        reader: &mut R,
        offset: u32,
        size: u32,
        block_size: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>>
    where
        R: AsyncRead + AsyncSeek + Unpin,
    {
        reader.seek(SeekFrom::Start(offset as u64)).await?;
        let mut buffer = vec![0; size as usize];
        let mut writer = Cursor::new(&mut buffer);
        let mut left_to_read = size;

        // Read the first next_address
        let mut next_address = read_u32_le(reader).await?;

        while left_to_read > 0 {
            if left_to_read < block_size {
                // Last block - read remaining data
                let mut data: Vec<u8> = vec![0; left_to_read as usize];
                reader.read_exact(&mut data).await?;
                writer.write_all(&data)?; // Note: write is synchronous
                break;
            } else {
                // Read block data (minus the 4 bytes already read for next_address)
                let mut data: Vec<u8> = vec![0; (block_size as usize) - 4];
                reader.read_exact(&mut data).await?;
                writer.write_all(&data)?; // Note: write is synchronous

                // Seek to next block
                reader.seek(SeekFrom::Start(next_address as u64)).await?;

                // Read next block's address
                next_address = read_u32_le(reader).await?;

                left_to_read -= block_size - 4;
            }
        }

        Ok(buffer)
    }
}

// Helper function to read a u32 in little endian order asynchronously
async fn read_u32_le<R>(reader: &mut R) -> Result<u32, std::io::Error>
where
    R: AsyncRead + Unpin,
{
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).await?;
    Ok(u32::from_le_bytes(buf))
}

// WIP: Alternative BlockReader that gets passed buffers of size block_size

#[derive(Debug)]
struct AsyncBlockReader {
    bytes_read: usize,
    buffer: Vec<u8>,
}

impl AsyncBlockReader {
    pub fn create(size: usize) -> Self {
        Self {
            bytes_read: 0,
            buffer: vec![0; size],
        }
    }
    pub fn read_block(&mut self) -> usize {
        // Return how much we should read
        0
    }
}
