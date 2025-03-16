use std::{fs::File, io::{BufWriter, Seek}};

use deku::DekuContainerRead;
use image::{DynamicImage, ImageBuffer, RgbaImage};
use libac_rs::dat::file_types::texture::Texture;

fn main() -> Result<(), std::io::Error>{
    // Offset: 254331904 (skip first two DWORDS for now)
    let mut file = File::options().read(true).open("../ACEmulator/ACE/Dats/client_portal.dat").unwrap();
    file.seek(std::io::SeekFrom::Start(254331904+8))?; // (skip first two DWORDS for now)
    println!("{:?}", file);

    let (_, texture) = Texture::from_reader((&mut file, 0))?;
    texture.to_png("test.png", 1)?;

    Ok(())
}
