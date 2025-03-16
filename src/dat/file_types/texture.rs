use std::{fs::File, io::BufWriter};

use deku::{DekuRead, DekuWrite};
use image::{DynamicImage, ImageBuffer, RgbaImage};

use crate::dat::enums::surface_pixel_format::SurfacePixelFormat;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct Texture {
    pub unknown: i32, // This is sometimes 6? Seems used somehow.
    pub width: i32,
    pub height: i32,
    pub format: SurfacePixelFormat,
    pub length: i32,
    #[deku(count = "length")]
    pub data: Vec<u8>,
    pub default_palette_id: Option<u32>, // TODO: Not fully hooked up
}

impl Texture {
    pub fn export(&self) -> Result<Vec<u8>, std::io::Error> {
        match self.format {
            SurfacePixelFormat::PFID_R8G8B8 => {
                // TODO: This is untested (PFID_A8R8G8B8 is tested)
                let result = self
                    .data
                    .chunks_exact(3)
                    .flat_map(|chunk| {
                        // [B,G,R] -> [R,G,B]
                        [chunk[2], chunk[1], chunk[0]]
                    })
                    .collect();

                Ok(result)
            }
            SurfacePixelFormat::PFID_A8R8G8B8 => {
                let result = self
                    .data
                    .chunks_exact(4)
                    .flat_map(|chunk| {
                        // [B,G,R,A] -> [R,G,B,A]
                        [chunk[2], chunk[1], chunk[0], chunk[3]]
                    })
                    .collect();

                Ok(result)
            }
            _ => todo!(),
        }
    }

    pub fn to_image(&self, scale: u32) -> Result<DynamicImage, std::io::Error> {
        let buf = self.export()?;
        let img: RgbaImage = ImageBuffer::from_raw(self.width as u32, self.height as u32, buf)
            .expect("Failed to create ImageBuffer");

        Ok(DynamicImage::ImageRgba8(img).resize(
            (self.width as u32) * scale,
            (self.height as u32) * scale,
            image::imageops::FilterType::Lanczos3,
        ))
    }
    pub fn to_png(&self, path: &str, scale: u32) -> Result<(), std::io::Error> {
        let image = self.to_image(scale)?;
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        image
            .write_to(&mut writer, image::ImageFormat::Png)
            .expect("Failed to write image.");

        Ok(())
    }
}
