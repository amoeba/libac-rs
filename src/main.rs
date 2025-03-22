use std::{
    error::Error,
    fs::File,
    io::{Seek, SeekFrom},
};

use libac_rs::dat::{
    dat_database::DatDatabase,
    file_types::{dat_file::DatFile, texture::Texture},
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
    let mut file = File::open("../libac-js/out.bin")?;
    file.seek(SeekFrom::Start(0))?;
    let db = DatDatabase::read(&mut file)?;

    println!("{:?}", db);

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    test_read_dat();
    Ok(())
}
