use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;

use crate::wz::constants::{WZ_GMSIV, WZ_MSEAIV, WZ_BMSCLASSIC};
use crate::wz::directory::WzDirectory;
use crate::wz::reader::WzBinaryReader;
use crate::wz::version::WzMapleVersion;
use crate::crypto::aes::create_wz_key_from_iv;

#[pyclass]
#[derive(Debug, Clone)]
pub struct WzHeader {
    #[pyo3(get)]
    pub ident: String,
    #[pyo3(get)]
    pub fsize: u64,
    #[pyo3(get)]
    pub fstart: u32,
    #[pyo3(get)]
    pub copyright: String,
}

impl WzHeader {
    pub fn new<R: Read + Seek>(reader: &mut WzBinaryReader<R>) -> io::Result<Self> {
        let ident = reader.read_string(4)?;
        if ident != "PKG1" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid WZ header"));
        }
        let fsize = reader.read_u64()?;
        let fstart = reader.read_u32()?;

        let copyright_len = fstart as u64 - reader.pos()?;
        let copyright = reader.read_string(copyright_len as usize)?;

        Ok(Self {
            ident,
            fsize,
            fstart,
            copyright,
        })
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct WzFile {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub header: Option<WzHeader>,
    #[pyo3(get)]
    pub wz_dir: Option<WzDirectory>,
    #[pyo3(get)]
    pub version: i16,
    #[pyo3(get)]
    pub maple_version: WzMapleVersion,
    #[pyo3(get)]
    pub version_hash: u32,
    wz_iv: [u8; 4],
}

#[pymethods]
impl WzFile {
    #[new]
    fn py_new(path: String, version: WzMapleVersion) -> PyResult<Self> {
        WzFile::new(path, version).map_err(|e| PyIOError::new_err(e.to_string()))
    }

    fn parse(&mut self) -> PyResult<()> {
        let file = File::open(&self.path).map_err(|e| PyIOError::new_err(e.to_string()))?;
        let mut reader = BufReader::new(file);
        self.parse_internal(&mut reader).map_err(|e| PyIOError::new_err(e.to_string()))
    }

    pub fn get_object(&self, path: &str) -> PyResult<Option<PyObject>> {
        if self.wz_dir.is_none() {
            return Ok(None);
        }

        let segments: Vec<&str> = path.split('/').collect();
        let mut current_dir = self.wz_dir.as_ref().unwrap();

        for (i, segment) in segments.iter().enumerate() {
            if i == segments.len() - 1 {
                // Last segment, could be a directory or an image
                if let Some(dir) = current_dir.directories.iter().find(|d| d.name.eq_ignore_ascii_case(segment)) {
                    return Ok(Some(dir.clone().into_py(pyo3::Python::acquire_gil())));
                }
                if let Some(img) = current_dir.images.iter().find(|i| i.name.eq_ignore_ascii_case(segment)) {
                    return Ok(Some(img.clone().into_py(pyo3::Python::acquire_gil())));
                }
            } else {
                if let Some(next_dir) = current_dir.directories.iter().find(|d| d.name.eq_ignore_ascii_case(segment)) {
                    current_dir = next_dir;
                } else {
                    return Ok(None); // Path not found
                }
            }
        }

        Ok(None)
    }
}

impl WzFile {
    pub fn new<P: AsRef<Path>>(path: P, version: WzMapleVersion) -> io::Result<Self> {
        let wz_iv = match version {
            WzMapleVersion::GMS => WZ_GMSIV,
            WzMapleVersion::EMS | WzMapleVersion::MSEA => WZ_MSEAIV,
            _ => WZ_BMSCLASSIC,
        };

        Ok(Self {
            path: path.as_ref().to_str().unwrap_or_default().to_string(),
            header: None,
            wz_dir: None,
            version: -1,
            maple_version: version,
            version_hash: 0,
            wz_iv,
        })
    }

    pub fn parse_internal<R: Read + Seek>(&mut self, reader: &mut R) -> io::Result<()> {
        let mut wz_reader = WzBinaryReader::new(reader, self.wz_iv, 0);

        let header = WzHeader::new(&mut wz_reader)?;
        let fstart = header.fstart;
        let file_path = self.path.clone();
        let file_name = Path::new(&file_path).file_name().unwrap_or_default().to_str().unwrap_or_default().to_string();
        self.header = Some(header);

        wz_reader.seek(SeekFrom::Start(fstart as u64))?;
        let wz_version_header = wz_reader.read_u16()?;

        if self.version == -1 {
            // Bruteforce the version
            for ver in 0..u16::MAX {
                let hash = self.calculate_version_hash(wz_version_header, ver as i16);
                if hash != 0 {
                     // A bit of a guess, but if the first directory name parses, it's probably the right version
                    let current_pos = wz_reader.pos()?;
                    wz_reader.set_hash(hash);
                    let mut test_dir = WzDirectory::new("test".to_string());
                    if test_dir.parse_entries(&mut wz_reader, fstart).is_ok() {
                        self.version = ver as i16;
                        self.version_hash = hash;
                        wz_reader.seek(SeekFrom::Start(current_pos))?;
                        break;
                    }
                    wz_reader.seek(SeekFrom::Start(current_pos))?;
                }
            }
            if self.version == -1 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Could not determine WZ version"));
            }
        } else {
            self.version_hash = self.calculate_version_hash(wz_version_header, self.version);
        }

        wz_reader.set_hash(self.version_hash);

        let mut root_dir = WzDirectory::new(file_name);
        root_dir.parse_entries(&mut wz_reader, fstart)?;

        // This is not lazy, it parses everything at once. Good for testing.
        // let wz_key = create_wz_key_from_iv(&self.wz_iv);
        // root_dir.parse_images_recursively(&mut wz_reader, fstart, &wz_key)?;

        self.wz_dir = Some(root_dir);

        Ok(())
    }

    fn calculate_version_hash(&self, wz_version_header: u16, game_version: i16) -> u32 {
        let version_str = game_version.to_string();
        let mut hash: u32 = 0;
        for ch in version_str.chars() {
            hash = (hash * 32) + (ch as u32) + 1;
        }

        let decrypted_version = !((hash >> 24) & 0xFF ^ (hash >> 16) & 0xFF ^ (hash >> 8) & 0xFF ^ hash & 0xFF) as u8;

        if wz_version_header as u8 == decrypted_version {
            return hash;
        }

        0
    }
}