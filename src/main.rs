use std::{
    error::Error,
    fs::{self, File, create_dir},
    io::{Cursor, Seek, SeekFrom},
};

use libac_rs::dat::{
    file_types::{dat_file::DatFile, texture::Texture},
    reader::{
        async_dat_block_reader::AsyncDatBlockReader, async_http_chunk_reader::AsyncHttpChunkReader,
        async_http_io_adapter::AsyncHttpIoAdapter, dat_block_reader::DatBlockReader,
        dat_database::DatDatabase, dat_directory_entry::DatDirectoryEntry,
        dat_file_type::DatFileType,
    },
};

fn example_extract_icon() -> Result<(), Box<dyn Error>> {
    let mut db_file = File::open("../ACEmulator/ACE/Dats/client_portal.dat")?;
    db_file.seek(SeekFrom::Start(0))?;
    let db = DatDatabase::read(&mut db_file)?;

    let files = db.list_files(true)?;

    // Set up export dir
    if !fs::exists("export")? {
        create_dir("export")?;
    }

    for file in files {
        let dat_file_buffer = DatBlockReader::read(
            &mut db_file,
            file.file_offset,
            file.file_size,
            db.header.block_size,
        )?;
        let mut reader = Cursor::new(dat_file_buffer);

        match file.file_type() {
            DatFileType::Texture => {
                let dat_file: DatFile<Texture> = DatFile::read(&mut reader)?;
                let texture = dat_file.inner;

                if texture.width == 32 && texture.height == 32 {
                    println!("file is {:?}", file);
                    texture.to_png(&format!("./export/{}.png", dat_file.id), 1)?;
                    break;
                }
            }
            DatFileType::Unknown => {
                // Doing nothing for now
            }
        }
    }

    Ok(())
}

// Run a server that can fulfill this with `simple-http-server` crate
// async fn http_test() -> Result<(), Box<dyn Error>> {
//     let url = "http://devd.io:8000/client_portal.dat";
//     let mut reader = HttpByteRangeReader::new(url).await?;

//     // TODO: Just implement async for DatBlockReader.
//     //
//     // Here's my icon info
//     // file is DatDirectoryEntry { bit_flags: 196608, object_id: 100667226, file_offset: 885193728, file_size: 3096, date: 1370456463, iteration: 1458 }

//     DatBlockReader::read_async(&mut reader, 885193728, 3096, 1024).await?;

//     Ok(())
// }

struct DatBlock {
    length: usize,
    buffer: Vec<u8>,
}

struct DatDatabaseReader {
    file: tokio::fs::File,
}

impl DatDatabaseReader {
    pub fn new(file: tokio::fs::File) -> Self {
        Self { file }
    }

    // async fn read_block(&self, reader: R, block_size: usize) -> DatBlock {
    //     // loop enough times to fill buffer `buffer of length `length`,
    //     // as many times as needed to fill with blocks of size `block_size`
    //     // TODO
    //     DatBlock {}
    // }
}

// #[tokio::main]
// async fn main() -> Result<(), std::io::Error> {
//     // Theoretical async API I need to write
//     let file = tokio::fs::File::open("../ACEmulator/ACE/Dats/client_portal.dat").await?;

//     // TODO: Create a new DatDatabaseReader instance over `file`
//     let reader = DatDatabaseReader::new(file);

//     // After instantaited, the reader holds a list of DatDirectoryEntry
//     // Like this:
//     let entry = DatDirectoryEntry {
//         bit_flags: 196608,
//         object_id: 100667226,
//         file_offset: 885193728,
//         file_size: 3096,
//         date: 1370456463,
//         iteration: 1458,
//     };

//     // We can read an entry. This involves one or more async reads to a DatBlockReader
//     // Each dat_file is made up of one or more blocks and blocks are not contiguous
//     let dat_file: Texture = reader.read_file::<Texture>(&entry).await?;

//     Ok(())
// }

// fn main() {
//     // example_extract_icon();
// }
//
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_url = "http://localhost:8000/client_portal.dat";
    let client = reqwest::Client::new();
    let mut http_chunk_reader = AsyncHttpChunkReader::new(client, file_url.to_string()).await?;

    // let mut adapter = AsyncHttpIoAdapter::new(http_chunk_reader);

    // Example file
    let entry = DatDirectoryEntry {
        bit_flags: 196608,
        object_id: 100667226,
        file_offset: 885193728,
        file_size: 3096,
        date: 1370456463,
        iteration: 1458,
    };
    println!(
        "Attempting to read {} bytes of data, starting with pointer at offset {}, block unit size {}.",
        entry.file_size, entry.file_offset, 1024
    );
    let mut buf = vec![0u8; entry.file_size as usize];
    http_chunk_reader.read(&mut buf).await?;

    // match AsyncDatBlockReader::read(&mut adapter, entry.file_offset, entry.file_size, 1024).await {
    //     Ok(data_buffer) => {
    //         println!(
    //             "Successfully read {} bytes from the stream.",
    //             data_buffer.len()
    //         );
    //         // Print first few bytes as hex for verification
    //         print!("Data (hex): ");
    //         for (i, byte) in data_buffer.iter().enumerate().take(32) {
    //             // Print up to 32 bytes
    //             print!("{:02X} ", byte);
    //             if i > 0 && (i + 1) % 16 == 0 {
    //                 println!(); // Newline every 16 bytes
    //             }
    //         }
    //         println!();
    //         if data_buffer.len() > 32 {
    //             println!("... (and {} more bytes)", data_buffer.len() - 32);
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("Error reading from DatBlockReader: {}", e);
    //         // If the error is an IoError, you might get more specific kinds
    //         if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
    //             eprintln!("IO Error Kind: {:?}", io_err.kind());
    //         }
    //     }
    // }

    Ok(())
}
