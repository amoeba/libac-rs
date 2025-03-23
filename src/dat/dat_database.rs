use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
pub enum DatFileType {
    Texture,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct DatFile {
    pub bit_flags: u32,
    pub object_id: u32,
    pub file_offset: u32,
    pub file_size: u32,
    pub date: u32,
    pub iteration: u32,
}

impl DatFile {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<DatFile, Box<dyn Error>> {
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

const DAT_HEADER_OFFSET: u64 = 0x140;

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
    version_minor: u32,
}

impl DatDatabaseHeader {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<DatDatabaseHeader, Box<dyn Error>> {
        reader.seek(SeekFrom::Start(DAT_HEADER_OFFSET))?;

        let file_type = reader.read_u32::<LittleEndian>()?;
        let block_size = reader.read_u32::<LittleEndian>()?;
        let file_size = reader.read_u32::<LittleEndian>()?;
        let data_set = reader.read_u32::<LittleEndian>()?;
        let data_subset = reader.read_u32::<LittleEndian>()?;
        let free_head = reader.read_u32::<LittleEndian>()?;
        let free_tail = reader.read_u32::<LittleEndian>()?;
        let free_count = reader.read_u32::<LittleEndian>()?;
        let btree = reader.read_u32::<LittleEndian>()?;
        let new_lru = reader.read_u32::<LittleEndian>()?;
        let old_lru = reader.read_u32::<LittleEndian>()?;
        let use_lru = reader.read_u32::<LittleEndian>()? == 1;
        let master_map_id = reader.read_u32::<LittleEndian>()?;
        let engine_pack_version = reader.read_u32::<LittleEndian>()?;
        let game_pack_version = reader.read_u32::<LittleEndian>()?;
        let mut version_major = vec![0; 16];
        reader.read_exact(&mut version_major)?;
        let version_minor = reader.read_u32::<LittleEndian>()?;

        Ok(DatDatabaseHeader {
            file_type,
            block_size,
            file_size,
            data_set,
            data_subset,
            free_head,
            free_tail,
            free_count,
            btree,
            new_lru,
            old_lru,
            use_lru,
            master_map_id,
            engine_pack_version,
            game_pack_version,
            version_major,
            version_minor,
        })
    }
}

#[derive(Debug)]
pub struct DatReader {}

impl DatReader {
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

        while size > 0 {
            if left_to_read < block_size {
                let mut data: Vec<u8> = vec![0; left_to_read as usize];
                reader.read_exact(&mut data)?;
                writer.write_all(&data)?;

                break;
            } else {
                let mut data: Vec<u8> = vec![0; (block_size as usize) - 4];
                reader.read_exact(&mut data)?;
                writer.write_all(&data)?;

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
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<DatDirectoryHeader, Box<dyn Error>> {
        let mut branches = vec![0; 62];

        for i in 0..branches.len() {
            branches[i] = reader.read_u32::<LittleEndian>()?;
        }

        let entry_count = reader.read_u32::<LittleEndian>()?;

        let mut entries = vec![];

        for _ in 0..entry_count {
            entries.push(DatFile::read(reader)?);
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
pub struct DatDirectory {
    header: DatDirectoryHeader,
    directories: Vec<DatDirectory>,
}

impl DatDirectory {
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        offset: u32,
        block_size: u32,
    ) -> Result<DatDirectory, Box<dyn Error>> {
        // Read DatDirectoryHeader
        let header_buf =
            DatReader::read(reader, offset, DAT_DIRECTORY_HEADER_OBJECT_SIZE, block_size)?;
        let mut header_reader = Cursor::new(header_buf);
        let header = DatDirectoryHeader::read(&mut header_reader)?;

        let mut directories: Vec<DatDirectory> = Vec::new();

        // Recurse only if we're not a leaf
        if header.branches[0] != 0 {
            for i in 0..header.entry_count + 1 {
                let dir = DatDirectory::read(reader, header.branches[i as usize], block_size)?;
                directories.push(dir);
            }
        }

        Ok(DatDirectory {
            header,
            directories,
        })
    }

    fn list_files(&self, files_list: &mut Vec<DatFile>) -> Result<(), Box<dyn Error>> {
        println!("list_files; list is {}", files_list.len());
        for i in 0..self.directories.len() {
            self.directories[i].list_files(files_list)?;
        }

        // TODO: Make sure this is right
        for i in 0..self.header.entries.len() {
            // Debugging
            println!(
                "entry_count: {}; entries_len: {}",
                self.header.entry_count,
                self.header.entries.len()
            );
            files_list.push(self.header.entries[i as usize]);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DatDatabase {
    header: DatDatabaseHeader,
    root_dir: DatDirectory,
}

impl DatDatabase {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<DatDatabase, Box<dyn Error>> {
        let header: DatDatabaseHeader = DatDatabaseHeader::read(reader)?;
        let root_dir = DatDirectory::read(reader, header.btree, header.block_size)?;

        Ok(DatDatabase { header, root_dir })
    }

    pub fn list_files(&self) -> Result<Vec<DatFile>, Box<dyn Error>> {
        let mut files_list: Vec<DatFile> = Vec::new();
        self.root_dir.list_files(&mut files_list)?;

        Ok(files_list)
    }
}
