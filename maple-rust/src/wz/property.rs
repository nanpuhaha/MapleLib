use crate::wz::reader::WzBinaryReader;
use std::io::{Read, Seek, Result};
use flate2::read::ZlibDecoder;
use lz4_flex;

#[derive(Debug, Clone)]
pub enum WzImageProperty {
    Null(String),
    Short(String, i16),
    Int(String, i32),
    Long(String, i64),
    Float(String, f32),
    Double(String, f64),
    String(String, String),
    Vector(String, i32, i32),
    Canvas { name: String, properties: Vec<WzImageProperty>, decompressed_data: Vec<u8> },
    SubProperty { name: String, properties: Vec<WzImageProperty> },
    Convex { name: String, properties: Vec<WzImageProperty> },
    Sound { name: String, len_ms: u32, headerless_data: Vec<u8> },
    UOL { name: String, uol: String },
}

impl WzImageProperty {
    pub fn parse<R: Read + Seek>(reader: &mut WzBinaryReader<R>, base_offset: u32) -> Result<Self> {
        let name = reader.read_string_block(base_offset)?;
        let prop_type = reader.read_u8()?;

        match prop_type {
            0 => Ok(WzImageProperty::Null(name)),
            2 | 11 => Ok(WzImageProperty::Short(name, reader.read_i16()?)),
            3 | 19 => Ok(WzImageProperty::Int(name, reader.read_compressed_int()?)),
            20 => Ok(WzImageProperty::Long(name, reader.read_compressed_long()?)),
            4 => Ok(WzImageProperty::Float(name, reader.read_compressed_float()?)),
            5 => Ok(WzImageProperty::Double(name, reader.read_f64()?)),
            8 => Ok(WzImageProperty::String(name, reader.read_wz_string()?)),
            9 => { // Extended property
                let _ = reader.read_u32()?; // Skip 4 bytes, this is not an extended type
                let _ = reader.read_u32()?; // Skip 4 bytes
                Self::parse_extended(reader, name, base_offset)
            }
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown property type: {}", prop_type))),
        }
    }

    fn parse_extended<R: Read + Seek>(reader: &mut WzBinaryReader<R>, name: String, base_offset: u32) -> Result<Self> {
        let extended_name = reader.read_wz_string()?;
        match extended_name.as_str() {
            "Property" => {
                let mut properties = Vec::new();
                let count = reader.read_compressed_int()?;
                for _ in 0..count {
                    properties.push(WzImageProperty::parse(reader, base_offset)?);
                }
                Ok(WzImageProperty::SubProperty { name, properties })
            }
            "Canvas" => {
                let _unknown = reader.read_u8()?;
                let mut properties = Vec::new();
                if _unknown == 1 {
                    let count = reader.read_compressed_int()?;
                     for _ in 0..count {
                        properties.push(WzImageProperty::parse(reader, base_offset)?);
                    }
                }
                let width = reader.read_compressed_int()? as u32;
                let height = reader.read_compressed_int()? as u32;
                let format = reader.read_compressed_int()? as u32;
                let format2 = reader.read_u8()?;
                let _ = reader.read_u32()?; // Skip 4 bytes
                let data_len = reader.read_u32()? as usize - 1;
                let _ = reader.read_u8()?; // Skip 1 byte

                let decompressed_data = if data_len > 0 {
                    let compressed_bytes = reader.read_bytes(data_len)?;
                    let mut decompressed = Vec::new();

                    // format 1 = ZLIB, format 2 = LZ4
                    if format == 1 || format2 == 1 {
                        let mut decoder = ZlibDecoder::new(&compressed_bytes[..]);
                        decoder.read_to_end(&mut decompressed)?;
                    } else if format == 2 {
                        decompressed = lz4_flex::decompress(&compressed_bytes, (width * height * 4) as usize)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    } else {
                        decompressed = compressed_bytes;
                    }
                    decompressed
                } else {
                    Vec::new()
                };

                Ok(WzImageProperty::Canvas { name, properties, decompressed_data })
            }
            "Shape2D#Vector2D" => {
                let x = reader.read_compressed_int()?;
                let y = reader.read_compressed_int()?;
                Ok(WzImageProperty::Vector(name, x, y))
            }
            "Shape2D#Convex2D" => {
                let mut properties = Vec::new();
                let count = reader.read_compressed_int()?;
                for _ in 0..count {
                    properties.push(WzImageProperty::parse(reader, base_offset)?);
                }
                Ok(WzImageProperty::Convex { name, properties })
            }
            "Sound_DX8" => {
                let _unknown = reader.read_u8()?;
                let len_ms = reader.read_compressed_int()? as u32;
                let _ = reader.read_compressed_int()?; // some other len
                let _ = reader.read_u32()?; // offset
                let data_len = reader.read_u32()? as usize;
                let headerless_data = reader.read_bytes(data_len)?;
                Ok(WzImageProperty::Sound { name, len_ms, headerless_data })
            }
            "UOL" => {
                let _unknown = reader.read_u8()?;
                let uol = reader.read_wz_string()?;
                Ok(WzImageProperty::UOL { name, uol })
            }
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown extended property type: {}", extended_name))),
        }
    }
}