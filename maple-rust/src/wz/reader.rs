use std::io::{Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct WzBinaryReader<R: Read + Seek> {
    reader: R,
    wz_iv: [u8; 4],
    version_hash: u32,
}

impl<R: Read + Seek> WzBinaryReader<R> {
    pub fn new(reader: R, wz_iv: [u8; 4], version_hash: u32) -> Self {
        Self {
            reader,
            wz_iv,
            version_hash,
        }
    }

    pub fn set_hash(&mut self, hash: u32) {
        self.version_hash = hash;
    }

    pub fn read_u8(&mut self) -> std::io::Result<u8> {
        self.reader.read_u8()
    }

    pub fn read_i8(&mut self) -> std::io::Result<i8> {
        self.reader.read_i8()
    }

    pub fn read_i16(&mut self) -> std::io::Result<i16> {
        self.reader.read_i16::<LittleEndian>()
    }

    pub fn read_u16(&mut self) -> std::io::Result<u16> {
        self.reader.read_u16::<LittleEndian>()
    }

    pub fn read_i32(&mut self) -> std::io::Result<i32> {
        self.reader.read_i32::<LittleEndian>()
    }

    pub fn read_u32(&mut self) -> std::io::Result<u32> {
        self.reader.read_u32::<LittleEndian>()
    }

    pub fn read_i64(&mut self) -> std::io::Result<i64> {
        self.reader.read_i64::<LittleEndian>()
    }

    pub fn read_u64(&mut self) -> std::io::Result<u64> {
        self.reader.read_u64::<LittleEndian>()
    }

    pub fn read_f32(&mut self) -> std::io::Result<f32> {
        self.reader.read_f32::<LittleEndian>()
    }

    pub fn read_f64(&mut self) -> std::io::Result<f64> {
        self.reader.read_f64::<LittleEndian>()
    }

    pub fn read_bytes(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0; len];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn read_string(&mut self, len: usize) -> std::io::Result<String> {
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn read_null_terminated_string(&mut self) -> std::io::Result<String> {
        let mut bytes = Vec::new();
        loop {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn read_compressed_int(&mut self) -> std::io::Result<i32> {
        let b = self.read_i8()?;
        if b == -128 {
            self.read_i32()
        } else {
            Ok(b as i32)
        }
    }

    pub fn read_compressed_long(&mut self) -> std::io::Result<i64> {
        let b = self.read_i8()?;
        if b == -128 {
            self.read_i64()
        } else {
            Ok(b as i64)
        }
    }

    pub fn read_compressed_float(&mut self) -> std::io::Result<f32> {
        let b = self.read_i8()?;
        if b == -128 {
            self.read_f32()
        } else {
            Ok(0.0)
        }
    }

    pub fn read_wz_string(&mut self) -> std::io::Result<String> {
        let len_byte = self.read_i8()?;
        let len = if len_byte < 0 {
            if len_byte == -128 {
                self.read_i32()? as usize
            } else {
                -len_byte as usize
            }
        } else {
            if len_byte == 127 {
                self.read_i32()? as usize
            } else {
                len_byte as usize
            }
        };

        if len == 0 {
            return Ok(String::new());
        }

        if len > 4096 { // Sanity check
             return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "String length too large"));
        }

        let is_unicode = len_byte < 0;
        let final_len = if is_unicode { len * 2 } else { len };

        if final_len > 4096 {
             return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "String final length too large"));
        }

        let string_bytes = self.read_bytes(final_len)?;

        if is_unicode {
            let mut key: u16 = 0xAAAA;
            let mut utf16_bytes = Vec::with_capacity(len * 2);
            for i in 0..len {
                let b1 = string_bytes[i*2] ^ (key as u8);
                let b2 = string_bytes[i*2 + 1] ^ ((key >> 8) as u8);
                utf16_bytes.push(b1);
                utf16_bytes.push(b2);
                key = key.wrapping_add(1);
            }
            let utf16_chars: Vec<u16> = utf16_bytes
                .chunks_exact(2)
                .map(|a| u16::from_le_bytes([a[0], a[1]]))
                .collect();
            Ok(String::from_utf16_lossy(&utf16_chars))
        } else {
            let mut key: u8 = 0xAA;
            let mut ascii_bytes = Vec::with_capacity(len);
            for byte in string_bytes {
                ascii_bytes.push(byte ^ key);
                key = key.wrapping_add(1);
            }
            Ok(String::from_utf8_lossy(&ascii_bytes).to_string())
        }
    }

    pub fn read_string_block(&mut self, base_offset: u32) -> std::io::Result<String> {
        let type_byte = self.read_u8()?;
        match type_byte {
            0 | 0x73 => self.read_wz_string(),
            1 | 0x1B => {
                let string_offset = self.read_u32()?;
                let current_pos = self.pos()?;
                self.seek(SeekFrom::Start(base_offset as u64 + string_offset as u64))?;
                let result = self.read_wz_string()?;
                self.seek(SeekFrom::Start(current_pos))?;
                Ok(result)
            }
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid string type in block")),
        }
    }

    pub fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }

    pub fn pos(&mut self) -> std::io::Result<u64> {
        self.reader.stream_position()
    }
}