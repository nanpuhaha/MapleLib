use crate::wz::property::WzImageProperty;
use crate::wz::reader::WzBinaryReader;
use std::io::{Read, Seek, Result};
use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyclass]
#[derive(Debug, Clone)]
pub struct WzImage {
    #[pyo3(get)]
    pub name: String,
    properties: Vec<WzImageProperty>,
    offset: u32,
    size: u32,
    checksum: u32,
    parsed: bool,
}

#[pymethods]
impl WzImage {
    #[getter]
    fn properties(&self, py: Python) -> PyResult<PyObject> {
        let py_props = PyList::empty(py);
        for prop in &self.properties {
            py_props.append(prop.to_py_object(py)?)?;
        }
        Ok(py_props.into())
    }
}

impl WzImage {
    pub fn new(name: String, offset: u32, size: u32, checksum: u32) -> Self {
        Self {
            name,
            properties: Vec::new(),
            offset,
            size,
            checksum,
            parsed: false,
        }
    }

    pub fn parse<R: Read + Seek>(&mut self, reader: &mut WzBinaryReader<R>, base_offset: u32) -> Result<()> {
        if self.parsed {
            return Ok(());
        }

        let check_byte = reader.read_u8()?;
        if check_byte != 0x73 {
            // Not a valid image, or maybe a different format
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid image header byte"));
        }

        let _ = reader.read_wz_string()?; // Should be "Property" or something similar

        let property_count = reader.read_compressed_int()?;
        for _ in 0..property_count {
            let property = WzImageProperty::parse(reader, base_offset)?;
            self.properties.push(property);
        }

        self.parsed = true;
        Ok(())
    }
}