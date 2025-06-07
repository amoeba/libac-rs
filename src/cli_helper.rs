use std::error::Error;
use std::io::Seek;
use std::{
    fs::{self, File, create_dir},
    io::{Cursor, SeekFrom},
};

use libac_rs::dat::reader::dat_directory_entry::DatDirectoryEntry;
use libac_rs::dat::{
    enums::dat_file_type::DatFileType,
    file_types::{dat_file::DatFile, texture::Texture},
    reader::{dat_block_reader::DatBlockReader, dat_database::DatDatabase},
};

pub async fn index_dat(dat_file_path: &str) -> Result<DatDatabase, Box<dyn Error>> {
    let mut db_file = File::open(dat_file_path)?;
    let db = DatDatabase::read(&mut db_file)?;

    Ok(db)
}

pub async fn find_file_by_id(
    db: &DatDatabase,
    object_id: &str,
) -> Result<DatDirectoryEntry, Box<dyn Error>> {
    // TODO: Factor out into testable helper
    // Convert hex string to u32
    let parsed_id = if object_id.starts_with("0x") {
        u32::from_str_radix(&object_id[2..], 16)?
    } else {
        u32::from_str_radix(object_id, 16)?
    };

    println!("parsed_id: {}", parsed_id);
    let files = db.list_files(true)?;
    let target_file = files.iter().find(|file| file.object_id == parsed_id);

    match target_file {
        Some(file) => Ok(*file),
        None => {
            return Err(format!("Object ID {} not found in DAT file", object_id).into());
        }
    }
}

pub async fn extract_texture_by_id(
    dat_file_path: &str,
    object_id: &str,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    // Convert hex string to u32
    let parsed_id = if object_id.starts_with("0x") {
        u32::from_str_radix(&object_id[2..], 16)?
    } else {
        u32::from_str_radix(object_id, 16)?
    };

    // Read the database to find the file entry
    let mut db_file = File::open(dat_file_path)?;
    db_file.seek(SeekFrom::Start(0))?;
    let db = DatDatabase::read(&mut db_file)?;
    let files = db.list_files(true)?;

    // Find the file with matching object ID
    let target_file = files.iter().find(|file| file.object_id == parsed_id);

    let target_file = match target_file {
        Some(file) => file,
        None => {
            eprintln!("Object ID {} not found in DAT file", object_id);
            return Ok(());
        }
    };

    // Check if it's a texture
    if target_file.file_type() != DatFileType::Texture {
        eprintln!(
            "Object ID {} is not a texture (type: {:?})",
            object_id,
            target_file.file_type()
        );
        return Ok(());
    }

    // Set up export directory
    if !fs::exists(output_dir)? {
        create_dir(output_dir)?;
    }

    // Read the texture data
    let dat_file_buffer = DatBlockReader::read(
        &mut db_file,
        target_file.file_offset,
        target_file.file_size,
        db.header.block_size,
    )?;
    let mut reader = Cursor::new(dat_file_buffer);

    let dat_file: DatFile<Texture> = DatFile::read(&mut reader)?;
    let texture = dat_file.inner;

    // Export to PNG
    let output_path = format!("{}/{}.png", output_dir, object_id);
    texture.to_png(&output_path, 1)?;

    println!(
        "Extracted texture {} ({}x{}) to {}",
        object_id, texture.width, texture.height, output_path
    );
    println!(
        "File info: offset={}, size={}",
        target_file.file_offset, target_file.file_size
    );

    Ok(())
}
