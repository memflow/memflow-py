use std::array::TryFromSliceError;

use pyo3::prelude::*;
use thiserror::Error;

pub(crate) mod connector;
pub(crate) mod dummy;
pub(crate) mod internal;
pub(crate) mod inventory;
pub(crate) mod os;
pub(crate) mod process;

pub type Result<T> = std::result::Result<T, MemflowPyError>;

#[derive(Error, Debug)]
pub enum MemflowPyError {
    #[error("memflow error")]
    Memflow(#[from] memflow::error::Error),
    #[error("python error")]
    Python(#[from] PyErr),
    #[error("the python type `{0}` is not a valid type")]
    InvalidType(String),
    #[error("no python type found for `{0}`")]
    NoType(String),
    #[error("byte cast error")]
    ByteCast(#[from] TryFromSliceError),
    #[error("Python object missing attribute `{0}`")]
    MissingAttribute(String),
}

impl From<MemflowPyError> for PyErr {
    fn from(err: MemflowPyError) -> Self {
        match err {
            MemflowPyError::Python(e) => e,
            // TODO: These dont seem to be going through.
            _ => err.into(),
        }
    }
}

#[pymodule]
#[pyo3(name = "memflow")]
fn memflow_py(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    dummy::register_dummy_module(_py, m)?;
    m.add_class::<inventory::PyInventory>()?;
    m.add_class::<process::PyProcess>()?;
    m.add_class::<process::PyProcessInfo>()?;
    m.add_class::<os::PyOs>()?;
    m.add_class::<process::PyProcess>()?;
    Ok(())
}
