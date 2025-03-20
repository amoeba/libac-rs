use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::io;
use std::io::{Read, Result};

use super::texture::Texture;

// TODO: Implement more readers
#[derive(Debug)]
pub enum DatFileType {
    Texture(Texture),
}

pub trait DatFileRead: Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self>;
}

#[derive(Debug)]
pub struct DatFile {
    pub id: i32,
    pub inner: DatFileType,
}

impl DatFile {
    pub fn read<T: DatFileRead, R: Read>(reader: &mut R) -> Result<Self> {
        let id = reader.read_i32::<LittleEndian>()?;

        let inner = match std::any::type_name::<T>() {
            "libac_rs::dat::file_types::texture::Texture" => {
                DatFileType::Texture(Texture::read(reader)?)
            }
            x => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported type: {}", x),
                ));
            }
        };

        Ok(Self {
            id: id,
            inner: inner,
        })
    }
}
