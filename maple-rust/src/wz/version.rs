use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WzMapleVersion {
    GMS,
    EMS,
    BMS,
    CLASSIC,
    CUSTOM,
    GETFROMZLZ,
    GENERATE,
    UNKNOWN,
}