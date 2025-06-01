use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
pub struct DatBlockReader {}

impl DatBlockReader {
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        offset: u32,
        size: u32,
        block_size: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        reader.seek(SeekFrom::Start(offset as u64))?;
        let mut buffer = vec![0; size as usize];
        let mut writer = Cursor::new(&mut buffer);
        let mut left_to_read = size;
        let mut next_address = reader.read_u32::<LittleEndian>()?;
        while left_to_read > 0 {
            if left_to_read < block_size {
                let mut data: Vec<u8> = vec![0; left_to_read as usize];

                reader.read_exact(&mut data)?;
                println!("read chunk {:?}", data);

                writer.write_all(&data)?;
                break;
            } else {
                let mut data: Vec<u8> = vec![0; (block_size as usize) - 4];

                reader.read_exact(&mut data)?;
                println!("read chunk {:?}", data);

                writer.write_all(&data)?;
                reader.seek(SeekFrom::Start(next_address as u64))?;
                next_address = reader.read_u32::<LittleEndian>()?;
                left_to_read -= block_size - 4;
            }
        }
        Ok(buffer)
    }
}
