use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use byteorder::{LittleEndian, ReadBytesExt};
use log::{debug, info};

const PAK_SIGNATURE: [u8; 4] = [
    0x4C,
    0x53,
    0x50,
    0x4B
];

#[derive(Debug)]
struct PackHeader {
    pub version: u32,
    pub file_list_offset: u64,
    pub file_list_size: u32,
    pub flags: u8,
    pub priority: u8,

    pub md5: Vec<u8>,
    pub num_parts: u16,
}

const NAME_SIZE: usize = 256;

#[derive(Debug)]
struct FileEntry {
    // 256
    pub name: String,
    // 4
    pub offset_in_file1: u32,
    // 2
    pub offset_in_file2: u16,
    // 1
    pub archive_part: u8,
    // 1
    pub flags: u8,
    // 4
    pub size_on_disk: u32,
    // 4
    pub uncompressed_size: u32,
}

impl PackHeader {
    fn from_reader(mut rdr: &mut impl Read, version: u32) -> io::Result<Self> {
        let file_list_offset = rdr.read_u64::<LittleEndian>()?;
        let file_list_size = rdr.read_u32::<LittleEndian>()?;
        let flags = rdr.read_u8()?;
        let priority = rdr.read_u8()?;

        let mut md5_buffer = vec![0u8; 16];
        rdr.read_exact(&mut md5_buffer)?;

        let num_parts = rdr.read_u16::<LittleEndian>()?;

        return Ok(PackHeader {
            version,
            file_list_offset,
            file_list_size,
            flags,
            priority,

            md5: md5_buffer,
            num_parts,
        });
    }
}

impl FileEntry {
    fn from_reader(rdr: &mut impl Read) -> io::Result<Self> {
        let mut name_buffer = vec![0u8; NAME_SIZE];
        rdr.read_exact(&mut name_buffer)?;

        let offset_in_file1 = rdr.read_u32::<LittleEndian>()?;
        let offset_in_file2 = rdr.read_u16::<LittleEndian>()?;
        let archive_part = rdr.read_u8()?;
        let flags = rdr.read_u8()?;
        let size_on_disk = rdr.read_u32::<LittleEndian>()?;
        let uncompressed_size = rdr.read_u32::<LittleEndian>()?;

        let filtered_name = name_buffer
            .into_iter()
            .filter(|x| *x != 0u8)
            .collect::<Vec<_>>();

        return Ok(Self {
            name: String::from_utf8(filtered_name).unwrap(),
            offset_in_file1,
            offset_in_file2,
            archive_part,
            flags,
            size_on_disk,
            uncompressed_size,
        });
    }
}

pub fn unpack_pak_file(path: &str) {
    let f = File::open(path.clone()).expect("file not found");

    let file_name = path.split("/").last().unwrap();
    let file_name_without_extension = file_name.replace(".pak", "");

    let base_path = path.replace(file_name, "") + "/" + file_name_without_extension.as_str();

    let mut reader = BufReader::new(f);

    // read signature
    let mut buf = vec![0u8; 4];
    reader.read_exact(&mut buf).unwrap();
    if buf != PAK_SIGNATURE {
        panic!("File is not a pak file (signature wrong)")
    }

    let pak_version = reader.read_u32::<LittleEndian>().unwrap();
    debug!("Pak version: {}", pak_version);

    let header = PackHeader::from_reader(&mut reader, pak_version).unwrap();

    reader.seek(SeekFrom::Start(header.file_list_offset)).expect("TODO: panic message");

    let num_files = reader.read_i32::<LittleEndian>().unwrap();
    let compressed_size = reader.read_i32::<LittleEndian>().unwrap();

    debug!("start file list: {:?}", header.file_list_offset);
    debug!("flags: {:?}", header.flags);
    debug!("num_files: {:?}", num_files);
    debug!("compressed_size: {:?}", compressed_size);

    let mut compressed_file_list = vec![0u8; compressed_size as usize];
    reader.read_exact(&mut compressed_file_list).unwrap();

    let single_entry_size = 272;
    let file_buffer_size = single_entry_size * num_files as usize;
    let uncompressed_list_buffer = lz4_flex::decompress(
        &compressed_file_list,
        file_buffer_size
    ).expect("Failed to decompress file list");

    let mut uncompressed_reader = BufReader::new(uncompressed_list_buffer.as_slice());
    let mut file_entries: Vec<FileEntry>  = vec![];

    for _ in 0..num_files {
        let entry = FileEntry::from_reader(&mut uncompressed_reader).unwrap();
        file_entries.push(entry);
    }

    file_entries.into_iter().for_each(|entry| {
        let pos_shifted = (entry.offset_in_file2 as u64) << 32;
        let file_pos_start = (entry.offset_in_file1 as u64 | pos_shifted);
        info!("unpacking file: {}", entry.name);
        debug!("Offset in file: {}", file_pos_start);
        debug!("Size on disk: {}", entry.size_on_disk);
        debug!("Uncompressed size: {}", entry.uncompressed_size);

        reader.seek(SeekFrom::Start(file_pos_start)).expect("Could not find position in file");

        let mut compressed_file = vec![0u8; entry.size_on_disk as usize];
        reader.read_exact(&mut compressed_file).unwrap();

        let uncompressed_file = if entry.uncompressed_size == 0 {
            compressed_file
        } else {
            lz4_flex::decompress(
                &compressed_file,
                entry.uncompressed_size as usize
            ).expect("Failed to decompress file")
        };

        let file_name = entry.name.split("/").last().unwrap();
        let path = entry.name.replace(file_name, "");
        std::fs::create_dir_all(format!("{}/{}", base_path, path)).unwrap();

        let mut file = File::create(format!("{}/{}/{}", base_path, path, file_name)).unwrap();
        file.write_all(uncompressed_file.as_slice()).unwrap();
    });
}