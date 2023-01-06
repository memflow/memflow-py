use std::array::TryFromSliceError;

use internal::InternalDT;
use pyo3::{exceptions::PyException, prelude::*};
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
    #[error(transparent)]
    Memflow(#[from] memflow::error::Error),
    #[error(transparent)]
    Python(#[from] PyErr),
    #[error("the python type `{0}` is not a valid type")]
    InvalidType(String),
    #[error("no python type found for `{0}`")]
    NoType(String),
    #[error(transparent)]
    ByteCast(#[from] TryFromSliceError),
    #[error("Python object missing attribute `{0}`")]
    MissingAttribute(String),
    #[error("The cglue object `{0}` is missing impl for `{1}`")]
    MissingCGlueImpl(String, String),
    #[error("The arch {0} is not valid")]
    InvalidArch(String),
}

impl From<MemflowPyError> for PyErr {
    fn from(err: MemflowPyError) -> Self {
        match err {
            MemflowPyError::Python(e) => e,
            _ => PyException::new_err(err.to_string()),
        }
    }
}

#[pyfunction]
fn sizeof(ty: PyObject) -> PyResult<usize> {
    let dt: InternalDT = ty.try_into()?;
    Ok(dt.size())
}

#[pymodule]
#[pyo3(name = "memflow")]
fn memflow_py(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    dummy::register_dummy_module(_py, m)?;
    m.add_function(wrap_pyfunction!(sizeof, m)?)?;
    m.add_class::<inventory::PyInventory>()?;
    m.add_class::<process::PyProcess>()?;
    m.add_class::<process::PyProcessInfo>()?;
    m.add_class::<process::PyModuleInfo>()?;
    m.add_class::<process::PyArchitectureIdent>()?;
    m.add_class::<process::PyProcessState>()?;
    m.add_class::<os::PyOs>()?;
    m.add_class::<process::PyProcess>()?;
    Ok(())
}
