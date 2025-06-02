use futures::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};

/// Trait for reading data from a specific offset/range
/// Can be implemented for files (using seek) or HTTP (using range requests)
pub trait RangeReader {
    /// Read exactly `length` bytes starting at `offset`
    fn read_range(
        &mut self,
        offset: u32,
        length: usize,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>> + Send;
}

/// File-based implementation of RangeReader using seek
pub struct FileRangeReader<R> {
    reader: R,
}

impl<R> FileRangeReader<R>
where
    R: AsyncRead + AsyncSeek + Unpin + Send,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> RangeReader for FileRangeReader<R>
where
    R: AsyncRead + AsyncSeek + Unpin + Send,
{
    fn read_range(
        &mut self,
        offset: u32,
        length: usize,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>> + Send {
        async move {
            // Seek to the position
            self.reader.seek(SeekFrom::Start(offset.into())).await?;

            // Read exactly the requested bytes
            let mut buffer = vec![0u8; length];
            self.reader.read_exact(&mut buffer).await?;

            Ok(buffer)
        }
    }
}

/// HTTP-based implementation using range requests
pub struct HttpRangeReader {
    url: String,
    client: reqwest::Client,
}

impl HttpRangeReader {
    pub fn new(client: reqwest::Client, url: String) -> Self {
        Self { url, client }
    }

    /// Convenience constructor that creates a default client
    pub fn with_default_client(url: String) -> Self {
        Self::new(reqwest::Client::new(), url)
    }
}

impl RangeReader for HttpRangeReader {
    fn read_range(
        &mut self,
        offset: u32,
        length: usize,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>> + Send {
        let end_byte = offset + length as u32 - 1;
        let range_header = format!("bytes={}-{}", offset, end_byte);
        let url = self.url.clone();
        let client = self.client.clone();

        async move {
            let response = client
                .get(&url)
                .header("Range", range_header)
                .send()
                .await?;

            // Check if the server supports range requests
            if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let bytes = response.bytes().await?;
                Ok(bytes.to_vec())
            } else if response.status().is_success() {
                // Server doesn't support ranges, but returned full content
                // We'll take just the part we need
                let bytes = response.bytes().await?;
                let start = offset as usize;
                let end = std::cmp::min(start + length, bytes.len());

                if start >= bytes.len() {
                    return Err("Offset beyond file size".into());
                }

                Ok(bytes[start..end].to_vec())
            } else {
                Err(format!("HTTP request failed with status: {}", response.status()).into())
            }
        }
    }
}

/// Represents a single block read from the file
#[derive(Debug)]
pub struct DatBlock {
    /// Pointer to the next block (8 bytes)
    pub next_block_offset: u32,
    /// The actual data content of this block
    pub data: Vec<u8>,
}

/// Contains the complete file data after reading all blocks
#[derive(Debug)]
pub struct DatFile {
    /// Combined data from all blocks
    pub buffer: Vec<u8>,
}

/// The main API entry point for reading block-based files
#[derive(Debug)]
pub struct DatFileReader {
    pub size: usize,
    pub block_size: usize,
    pub left_to_read: usize,
}

impl DatFileReader {
    pub fn new(size: usize, block_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        if block_size < 4 {
            return Err("block_size must be at least 4 bytes (for the pointer)".into());
        }
        Ok(Self {
            size,
            block_size,
            left_to_read: size,
        })
    }

    /// Read a file starting at the given offset for the specified total size
    pub async fn read_file<R>(
        &mut self,
        reader: &mut R,
        start_offset: u32,
    ) -> Result<DatFile, Box<dyn std::error::Error>>
    where
        R: RangeReader,
    {
        let mut buffer = Vec::with_capacity(self.size);
        let mut next_address = start_offset;

        while self.left_to_read > 0 {
            let block = self.read_block(reader, next_address).await?;
            buffer.extend_from_slice(&block.data);

            if self.left_to_read > 0 {
                next_address = block.next_block_offset
            }
        }

        Ok(DatFile { buffer })
    }

    /// Read a single block from the given offset
    ///
    /// A block is [ next_offset, data ]. For the last block, the next_offset
    /// is 0
    async fn read_block<R>(
        &mut self,
        reader: &mut R,
        offset: u32,
    ) -> Result<DatBlock, Box<dyn std::error::Error>>
    where
        R: RangeReader,
    {
        // Determine the size of the next read. This is either an entire block
        // when we have more than a block worth of data to read or whatever is
        // left to read (+ 4 bytes for the pointer).
        let next_read_size = std::cmp::min(self.block_size, self.left_to_read + 4);
        self.left_to_read -= next_read_size - 4;

        // Dispatch the read to the underlying read implementation
        let block_data = reader.read_range(offset, next_read_size).await?;

        if block_data.len() < 4 {
            return Err("Block too small to contain pointer".into());
        }

        // Create and return the DatBlock
        let next_offset_bytes: [u8; 4] = block_data[0..4].try_into()?;
        let next_block_offset = u32::from_le_bytes(next_offset_bytes);
        let data: Vec<u8> = block_data[4..].to_vec();

        Ok(DatBlock {
            next_block_offset: next_block_offset,
            data,
        })
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     use clap::{Parser, Subcommand};
//     use tokio::io::{AsyncWriteExt, stdout};

//     #[derive(Parser)]
//     #[command(name = "dat")]
//     #[command(about = "A CLI for reading block-based files")]
//     struct Cli {
//         #[command(subcommand)]
//         command: Commands,
//     }

//     #[derive(Subcommand)]
//     enum Commands {
//         /// File operations
//         File {
//             #[command(subcommand)]
//             action: FileCommands,
//         },
//         /// HTTP operations
//         Http {
//             #[command(subcommand)]
//             action: HttpCommands,
//         },
//     }

//     #[derive(Subcommand)]
//     enum FileCommands {
//         /// Read data from a block-structured file
//         Read {
//             /// Starting offset in the file
//             offset: u32,
//             /// Total size to read
//             file_size: usize,
//             /// Path to the file
//             file_path: String,
//             /// Maximum block size (default: 1024)
//             #[arg(long, default_value_t = 1024)]
//             block_size: usize,
//         },
//     }

//     #[derive(Subcommand)]
//     enum HttpCommands {
//         /// Read data from a block-structured file over HTTP
//         Read {
//             /// Starting offset in the file
//             offset: u32,
//             /// Total size to read
//             file_size: usize,
//             /// URL to the file
//             url: String,
//             /// Maximum block size (default: 1024)
//             #[arg(long, default_value_t = 1024)]
//             block_size: usize,
//         },
//     }

//     let cli = Cli::parse();

//     match cli.command {
//         Commands::File { action } => match action {
//             FileCommands::Read {
//                 offset,
//                 file_size,
//                 file_path,
//                 block_size,
//             } => {
//                 // Read from file
//                 let file = tokio::fs::File::open(&file_path).await?;
//                 let compat_file = tokio_util::compat::TokioAsyncReadCompatExt::compat(file);
//                 let mut file_reader = FileRangeReader::new(compat_file);
//                 let mut reader = DatFileReader::new(file_size, block_size)?;

//                 let result = reader.read_file(&mut file_reader, offset).await?;

//                 // Write to stdout
//                 let mut stdout = stdout();
//                 stdout.write_all(&result.buffer).await?;
//                 stdout.flush().await?;
//             }
//         },
//         Commands::Http { action } => match action {
//             HttpCommands::Read {
//                 offset,
//                 file_size,
//                 url,
//                 block_size,
//             } => {
//                 // Read from HTTP
//                 let client = reqwest::Client::new();
//                 let mut http_reader = HttpRangeReader::new(client, url);
//                 let mut reader = DatFileReader::new(file_size, block_size)?;

//                 let result = reader.read_file(&mut http_reader, offset).await?;

//                 // Write to stdout
//                 let mut stdout = stdout();
//                 stdout.write_all(&result.buffer).await?;
//                 stdout.flush().await?;
//             }
//         },
//     }

//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[async_std::test]
//     async fn test_simple_block_reading() {
//         use async_std::fs::File;
//         use async_std::io::prelude::*;

//         // Create test file with block structure
//         let test_file_path = "test_blocks.dat";

//         {
//             // Prepare the test file data
//             // Block 1: [next_offset: 20] [data: "hello"]
//             // Block 2: [next_offset: 0]  [data: "world"]
//             let mut test_data = Vec::new();

//             // Block 1 at offset 0: next=20, data="hello" (5 bytes, padded to 8)
//             test_data.extend_from_slice(&20u32.to_le_bytes()); // pointer to block 2
//             test_data.extend_from_slice(b"hello\0\0\0"); // 8 bytes data with padding

//             // Padding to reach offset 20
//             test_data.resize(20, 0);

//             // Block 2 at offset 20: next=0, data="world" (5 bytes, padded to 8)
//             test_data.extend_from_slice(&0u32.to_le_bytes()); // no next block
//             test_data.extend_from_slice(b"world\0\0\0"); // 8 bytes data with padding

//             // Write to file
//             let mut file = File::create(test_file_path).await.unwrap();
//             file.write_all(&test_data).await.unwrap();
//         }

//         // Now test reading from the actual file
//         let file = File::open(test_file_path).await.unwrap();
//         let mut file_reader = FileRangeReader::new(file);
//         let mut reader = DatFileReader::new(16, 16).unwrap(); // 8 bytes pointer + 8 bytes data

//         let result = reader.read_file(&mut file_reader, 0).await.unwrap(); // Read "hello" + "world"

//         // The result contains the first 10 bytes: "hello\0\0\0wo"
//         assert_eq!(&result.buffer[0..5], b"hello");
//         assert_eq!(&result.buffer[8..10], b"wo");
//         println!("Successfully read first 10 bytes across 2 blocks from real file");

//         // Clean up
//         async_std::fs::remove_file(test_file_path).await.unwrap();
//     }

//     #[tokio::test]
//     async fn test_http_reader_real() {
//         use tokio::fs::File;
//         use tokio::io::AsyncWriteExt;
//         use warp::Filter;

//         // Create test file with block structure
//         let test_file_path = "test_http_blocks.dat";

//         {
//             // Prepare the test file data
//             // Block 1: [next_offset: 20] [data: "hello"]
//             // Block 2: [next_offset: 0]  [data: "world"]
//             let mut test_data = Vec::new();

//             // Block 1 at offset 0: next=20, data="hello" (5 bytes, padded to 8)
//             test_data.extend_from_slice(&20u32.to_le_bytes()); // pointer to block 2
//             test_data.extend_from_slice(b"hello\0\0\0"); // 8 bytes data with padding

//             // Padding to reach offset 20
//             test_data.resize(20, 0);

//             // Block 2 at offset 20: next=0, data="world" (5 bytes, padded to 8)
//             test_data.extend_from_slice(&0u32.to_le_bytes()); // no next block
//             test_data.extend_from_slice(b"world\0\0\0"); // 8 bytes data with padding

//             // Write to file
//             let mut file = File::create(test_file_path).await.unwrap();
//             file.write_all(&test_data).await.unwrap();
//         }

//         // Start HTTP server in background serving the file
//         let file_route = warp::path("data.dat").and(warp::fs::file(test_file_path));

//         let (addr, server) = warp::serve(file_route).bind_ephemeral(([127, 0, 0, 1], 0)); // Bind to available port

//         let server_handle = tokio::spawn(server);

//         // Give server a moment to start
//         tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

//         // Now test our HTTP reader against the local server
//         let client = reqwest::Client::new();
//         let url = format!("http://127.0.0.1:{}/data.dat", addr.port());
//         let mut http_reader = HttpRangeReader::new(client, url);
//         let mut reader = DatFileReader::new(16, 16).unwrap();

//         let result = reader.read_file(&mut http_reader, 0).await.unwrap();

//         // The result contains the first 10 bytes: "hello\0\0\0wo"
//         assert_eq!(&result.buffer[0..5], b"hello");
//         assert_eq!(&result.buffer[8..10], b"wo");
//         println!("Successfully read first 10 bytes across 2 blocks via HTTP!");

//         // Clean up
//         server_handle.abort();
//         tokio::fs::remove_file(test_file_path).await.unwrap();
//     }
// }
