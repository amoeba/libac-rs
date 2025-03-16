use std::{
    fs::File,
    io::{BufWriter, Seek},
};

use deku::DekuContainerRead;
use libac_rs::{dat::file_types::texture::Texture, icon::Icon};

fn main() -> Result<(), std::io::Error> {
    let offsets = [254331904, 184571904, 885193728];
    let mut file = File::options()
        .read(true)
        .open("../ACEmulator/ACE/Dats/client_portal.dat")
        .unwrap();
    for offset in offsets {
        file.seek(std::io::SeekFrom::Start(offset + 8))?; // (skip first two DWORDS for now)

        let (_, texture) = Texture::from_reader((&mut file, 0))?;

        // Offset: 184571904
        // file.seek(std::io::SeekFrom::Start(184592384+8))?;
        // let (_, overlay_texture) = Texture::from_reader((&mut file, 0))?;
        // overlay_texture.to_png("overlay.png", 1)?;

        // icon now?
        let icon = Icon {
            width: 32,
            height: 32,
            scale: 8,
            base: texture,
            underlay: None,
            overlay: None,
            overlay2: None,
            effect: None,
        };

        icon.export_to_file(&format!("{}.png", offset))?;
    }

    Ok(())
}
