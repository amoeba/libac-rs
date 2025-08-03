use futures::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};

use crate::dat::reader::{range_reader::RangeReader, types::dat_block::DatBlock};

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
