use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom},
};

#[derive(Debug)]
pub enum DatFileType {
    Texture,
    Unknown,
}

#[derive(Debug)]
pub struct DatFile {
    pub bit_flags: u32,
    pub object_id: u32,
    pub file_offset: u32,
    pub file_size: u32,
    pub date: u32,
    pub iteration: u32,
}

impl DatFile {
    pub fn read<R: Read + Seek>(mut reader: R) -> Result<DatFile, Box<dyn Error>> {
        Ok(DatFile {
            bit_flags: reader.read_u32::<LittleEndian>()?,
            object_id: reader.read_u32::<LittleEndian>()?,
            file_offset: reader.read_u32::<LittleEndian>()?,
            file_size: reader.read_u32::<LittleEndian>()?,
            date: reader.read_u32::<LittleEndian>()?,
            iteration: reader.read_u32::<LittleEndian>()?,
        })
    }

    pub fn file_type(&self) -> DatFileType {
        match self.object_id {
            0x06000000..=0x07FFFFFF => DatFileType::Texture,
            _ => DatFileType::Unknown,
        }
    }
}

#[derive(Debug)]
enum DatDatabaseType {
    Portal,
    Cell,
}

const DAT_HEADER_OFFSET: i64 = 0x140;

#[derive(Debug)]
pub struct DatDatabaseHeader {
    file_type: u32,
    block_size: u32,
    file_size: u32,
    data_set: u32,
    data_subset: u32,
    free_head: u32,
    free_tail: u32,
    free_count: u32,
    btree: u32,
    new_lru: u32,
    old_lru: u32,
    use_lru: bool,
    master_map_id: u32,
    engine_pack_version: u32,
    game_pack_version: u32,
    version_major: Vec<u8>,
    version_minor: u8,
}

impl DatDatabaseHeader {
    pub fn read<R: Read + Seek>(mut reader: R) -> Result<DatDatabaseHeader, Box<dyn Error>> {
        reader.seek(SeekFrom::Current(DAT_HEADER_OFFSET))?;

        let mut version_major_value = vec![0u8; 4];
        reader.read_exact(&mut version_major_value)?;

        Ok(DatDatabaseHeader {
            file_type: reader.read_u32::<LittleEndian>()?,
            block_size: reader.read_u32::<LittleEndian>()?,
            file_size: reader.read_u32::<LittleEndian>()?,
            data_set: reader.read_u32::<LittleEndian>()?,
            data_subset: reader.read_u32::<LittleEndian>()?,
            free_head: reader.read_u32::<LittleEndian>()?,
            free_tail: reader.read_u32::<LittleEndian>()?,
            free_count: reader.read_u32::<LittleEndian>()?,
            btree: reader.read_u32::<LittleEndian>()?,
            new_lru: reader.read_u32::<LittleEndian>()?,
            old_lru: reader.read_u32::<LittleEndian>()?,
            use_lru: reader.read_u32::<LittleEndian>()? == 1,
            master_map_id: reader.read_u32::<LittleEndian>()?,
            engine_pack_version: reader.read_u32::<LittleEndian>()?,
            game_pack_version: reader.read_u32::<LittleEndian>()?,
            version_major: version_major_value,
            version_minor: reader.read_u8()?,
        })
    }
}

#[derive(Debug)]
pub struct DatReader {}

impl DatReader {
    pub fn read<R: Read + Seek>(
        mut reader: R,
        offset: u32,
        size: u32,
        block_size: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        reader.seek(SeekFrom::Current(offset as i64))?;
        let mut buffer = vec![0; block_size as usize];

        let mut left_to_read = size;
        let mut next_address = reader.read_u32::<LittleEndian>()?;
        let mut buffer_offset = 0;

        while size > 0 {
            if size < block_size {
                let mut data: Vec<u8> = vec![0; left_to_read as usize];
                reader.read_exact(&mut data);
                buffer[buffer_offset..buffer_offset + data.len()].copy_from_slice(&data);

                break;
            } else {
                let mut data: Vec<u8> = vec![0; (block_size as usize) - 4];
                reader.read_exact(&mut data);
                buffer[buffer_offset..buffer_offset + data.len()].copy_from_slice(&data);

                buffer_offset += (block_size as usize) - 4;
                reader.seek(SeekFrom::Start(next_address as u64))?;
                next_address = reader.read_u32::<LittleEndian>()?;
                left_to_read -= block_size - 4;
            }
        }
        Ok(buffer)
    }
}

#[derive(Debug)]
pub struct DatDirectoryHeader {
    branches: Vec<u32>,
    entry_count: u32,
    entries: Vec<DatFile>,
}

impl DatDirectoryHeader {
    pub fn read<R: Read + Seek>(mut reader: R) -> Result<DatDirectoryHeader, Box<dyn Error>> {
        let mut branches = vec![0; 62];

        for i in 0..62 {
            branches[i] = reader.read_u32::<LittleEndian>()?;
        }

        let entry_count = reader.read_u32::<LittleEndian>()?;

        let mut entries = vec![];

        for _ in 0..entries.len() {
            entries.push(DatFile::read(&mut reader)?);
        }

        Ok(DatDirectoryHeader {
            branches,
            entry_count,
            entries,
        })
    }
}

const DAT_DIRECTORY_HEADER_OBJECT_SIZE: u32 = 0x6B4;

#[derive(Debug)]
pub struct DatDirectory {}

impl DatDirectory {
    pub fn read<R: Read + Seek>(
        mut reader: R,
        offset: u32,
        block_size: u32,
    ) -> Result<DatDirectory, Box<dyn Error>> {
        // Read DatDirectoryHeader
        let header_buf =
            DatReader::read(reader, offset, DAT_DIRECTORY_HEADER_OBJECT_SIZE, block_size)?;
        let header_reader = Cursor::new(header_buf);
        let header = DatDirectoryHeader::read(header_reader)?;

        Ok(DatDirectory {})
    }

    pub fn read_all<R: Read + Seek>(&self, reader: R) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct DatDatabase {
    header: DatDatabaseHeader,
    files: Option<Vec<DatFile>>,
}

impl DatDatabase {
    pub fn read<R: Read + Seek>(mut reader: R) -> Result<DatDatabase, Box<dyn Error>> {
        let header: DatDatabaseHeader = Self::read_header(&mut reader)?;

        let root_dir = DatDirectory::read(reader.by_ref(), header.btree, header.block_size)?;
        root_dir.read_all(reader)?;

        Ok(DatDatabase {
            header,
            files: Some(vec![]), // TODO: Maybe don't use Option or pre-allocation
        })
    }

    fn read_header<R: Read + Seek>(reader: R) -> Result<DatDatabaseHeader, Box<dyn Error>> {
        DatDatabaseHeader::read(reader)
    }
}
