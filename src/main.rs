use std::{
    error::Error,
    fs::{self, File, create_dir},
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

use libac_rs::dat::{
    dat_database::{DatDatabase, DatReader},
    file_types::{
        dat_file::{DatFile, DatFileRead},
        texture::Texture,
    },
};

fn read_example_texture() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("../libac-js/out.bin")?;
    file.seek(SeekFrom::Start(0))?;

    let file: DatFile<Texture> = DatFile::read(&mut file)?;
    let texture = file.inner;

    println!("{:?}", texture);

    Ok(())
}

fn test_read_dat() -> Result<(), Box<dyn Error>> {
    let mut db_file = File::open("../ACEmulator/ACE/Dats/client_portal.dat")?;
    db_file.seek(SeekFrom::Start(0))?;
    let db = DatDatabase::read(&mut db_file)?;

    let files = db.list_files(true)?;
    println!("{:?}", files.len());

    // WIP: Iterate over files
    for file in files {
        match file.file_type() {
            libac_rs::dat::dat_database::DatFileType::Texture => {
                println!("texture");

                // WIP
                db_file.seek(SeekFrom::Start(file.file_offset as u64))?;
                let mut buffer = vec![0; file.file_size as usize];
                db_file.read_exact(&mut buffer)?;
                let mut reader = Cursor::new(buffer);
                let tex: DatFile<Texture> = DatFile::read(&mut reader)?;
                println!("{:?}", tex);

                break;
            }
            libac_rs::dat::dat_database::DatFileType::Unknown => {
                // println!();
            }
        }
    }

    Ok(())
}

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
            libac_rs::dat::dat_database::DatFileType::Texture => {
                let dat_file: DatFile<Texture> = DatFile::read(&mut reader)?;
                let texture = dat_file.inner;

                if texture.width == 32 && texture.height == 32 {
                    texture.to_png(&format!("./export/{}.png", dat_file.id), 1)?;
                }
            }
            libac_rs::dat::dat_database::DatFileType::Unknown => {
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
