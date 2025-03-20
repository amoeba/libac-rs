use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::io::{Read, Result};

use super::texture::Texture;

// TODO: Implement more readers
#[derive(Debug)]
pub enum DatFileType {
    Texture(Texture),
}

pub trait DatFileRead: IntoDatFileType + Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self>;
}

pub trait IntoDatFileType {
    fn into(self) -> DatFileType;
}

#[derive(Debug)]
pub struct DatFile {
    pub id: i32,
    pub inner: DatFileType,
}

impl DatFile {
    pub fn read<T: DatFileRead, R: Read>(reader: &mut R) -> Result<Self> {
        let id = reader.read_i32::<LittleEndian>()?;
        let inner = T::read(reader)?.into();

        Ok(Self { id, inner })
    }
}
