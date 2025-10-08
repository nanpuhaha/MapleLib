use crate::wz::image::WzImage;
use crate::wz::reader::WzBinaryReader;
use std::io::{Read, Seek, SeekFrom, Result};
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct WzDirectory {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub directories: Vec<WzDirectory>,
    #[pyo3(get)]
    pub images: Vec<WzImage>,
    offset: u32,
    size: u32,
    checksum: u32,
}

impl WzDirectory {
    pub fn new(name: String) -> Self {
        Self {
            name,
            directories: Vec::new(),
            images: Vec::new(),
            offset: 0,
            size: 0,
            checksum: 0,
        }
    }

    pub fn parse_entries<R: Read + Seek>(&mut self, reader: &mut WzBinaryReader<R>, fstart: u32) -> Result<()> {
        let count = reader.read_compressed_int()?;
        for _ in 0..count {
            let type_byte = reader.read_u8()?;
            match type_byte {
                2 => { // Image
                    let name = reader.read_string_block(fstart)?;
                    let size = reader.read_compressed_int()? as u32;
                    let checksum = reader.read_compressed_int()? as u32;
                    let offset = reader.read_u32()?;
                    let image = WzImage::new(name, offset, size, checksum);
                    self.images.push(image);
                }
                3 | 4 => { // Directory
                    let name = reader.read_wz_string()?;
                    let size = reader.read_compressed_int()? as u32;
                    let checksum = reader.read_compressed_int()? as u32;
                    let offset = reader.read_u32()?;
                    let mut dir = WzDirectory::new(name);
                    dir.offset = offset;
                    dir.size = size;
                    dir.checksum = checksum;
                    self.directories.push(dir);
                }
                _ => {
                    // TODO: Handle list.wz (type 1) and other potential types
                }
            }
        }

        // Now recursively parse the sub-directories we just found.
        for dir in &mut self.directories {
            let current_pos = reader.pos()?;
            reader.seek(SeekFrom::Start(dir.offset as u64))?;
            dir.parse_entries(reader, fstart)?;
            reader.seek(SeekFrom::Start(current_pos))?;
        }

        Ok(())
    }

    pub fn parse_images_recursively<R: Read + Seek>(&mut self, reader: &mut WzBinaryReader<R>, base_offset: u32) -> Result<()> {
        for image in &mut self.images {
            let current_pos = reader.pos()?;
            reader.seek(SeekFrom::Start(image.offset as u64))?;
            image.parse(reader, base_offset)?;
            reader.seek(SeekFrom::Start(current_pos))?;
        }

        for dir in &mut self.directories {
            dir.parse_images_recursively(reader, base_offset)?;
        }

        Ok(())
    }
}