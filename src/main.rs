use std::{
    error::Error,
    fs::{self, File, create_dir},
    io::{Cursor, Seek, SeekFrom},
};

use libac_rs::dat::{
    file_types::{dat_file::DatFile, texture::Texture},
    reader::{
        dat_block_reader::DatBlockReader, dat_database::DatDatabase, dat_file_type::DatFileType,
        http_reader::HttpByteRangeReader,
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
async fn http_test() -> Result<(), Box<dyn Error>> {
    let url = "http://devd.io:8000/client_portal.dat";
    let mut reader = HttpByteRangeReader::new(url).await?;

    // TODO: Just implement async for DatBlockReader.
    //
    // Here's my icon info
    // file is DatDirectoryEntry { bit_flags: 196608, object_id: 100667226, file_offset: 885193728, file_size: 3096, date: 1370456463, iteration: 1458 }

    DatBlockReader::read_async(&mut reader, 885193728, 3096, 1024).await?;

    Ok(())
}

// #[tokio::main]
// async fn main() {
//     let _ = http_test().await;
// }
//
fn main() {
    example_extract_icon();
}
