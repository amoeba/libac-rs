use std::{
    error::Error,
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
};

use libac_rs::dat::{
    dat_database::DatDatabase,
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

fn main() -> Result<(), Box<dyn Error>> {
    test_read_dat()?;
    Ok(())
}
