use pyo3::prelude::*;

mod wz;
mod crypto;

use wz::version::WzMapleVersion;
use wz::file::WzFile;
use wz::directory::WzDirectory;
use wz::image::WzImage;

/// A Python module implemented in Rust.
#[pymodule]
fn maple_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<WzMapleVersion>()?;
    m.add_class::<WzFile>()?;
    m.add_class::<WzDirectory>()?;
    m.add_class::<WzImage>()?;
    Ok(())
}