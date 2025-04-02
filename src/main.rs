use std::{
    error::Error,
    fs::{self, File, create_dir},
    io::{Cursor, Seek, SeekFrom},
};

use libac_rs::dat::{
    file_types::{dat_file::DatFile, texture::Texture},
    reader::{dat_database::DatDatabase, dat_file_type::DatFileType, dat_reader::DatReader},
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
        let dat_file_buffer = DatReader::read(
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
                    texture.to_png(&format!("./export/{}.png", dat_file.id), 1)?;
                }
            }
            DatFileType::Unknown => {
                // Doing nothing for now
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    example_extract_icon()?;

    Ok(())
}
