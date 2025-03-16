use std::{fs::File, io::{BufWriter, Seek}};

use deku::DekuContainerRead;
use image::{DynamicImage, ImageBuffer, RgbaImage};
use libac_rs::{dat::file_types::texture::Texture, icon::Icon};

fn main() -> Result<(), std::io::Error>{
    // Offset: 254331904 (skip first two DWORDS for now)
    let mut file = File::options().read(true).open("../ACEmulator/ACE/Dats/client_portal.dat").unwrap();
    file.seek(std::io::SeekFrom::Start(254331904+8))?; // (skip first two DWORDS for now)

    let (_, texture) = Texture::from_reader((&mut file, 0))?;
    texture.to_png("test.png", 1)?;

    // Offset: 184571904
    file.seek(std::io::SeekFrom::Start(184592384+8))?;
    let (_, overlay_texture) = Texture::from_reader((&mut file, 0))?;
    overlay_texture.to_png("overlay.png", 1)?;

    // icon now?
    let icon = Icon {
        width: 32,
        height: 32,
        scale: 4,
        base: texture,
        underlay: Some(overlay_texture),
        overlay: None,
        overlay2: None,
        effect: None,
    };

    icon.export_to_file("icon.png")?;

    Ok(())
}
