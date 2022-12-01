use memflow::prelude::Inventory;
use pyo3::{prelude::*, types::PyTuple};

use crate::{connector::PyConnector, os::PyOs, MemflowPyError};

#[pyclass(name = "Inventory")]
pub struct PyInventory(Inventory);

#[pymethods]
impl PyInventory {
    #[new]
    #[args(py_args = "*")]
    fn new(py_args: &PyTuple) -> PyResult<Self> {
        if !py_args.is_empty() {
            Ok(Self(
                Inventory::scan_path(py_args.get_item(0)?.extract::<&str>()?)
                    .map_err(MemflowPyError::Memflow)?,
            ))
        } else {
            Ok(Self(Inventory::scan()))
        }
    }

    fn connector(&self, name: &str) -> PyResult<PyConnector> {
        // TODO: Add support for connector args.
        Ok(PyConnector::new(
            self.0
                .create_connector(name, None, None)
                .map_err(MemflowPyError::Memflow)?,
        ))
    }

    fn os(&self, name: &str, connector: Option<PyConnector>) -> PyResult<PyOs> {
        // TODO: Add support for os args.
        Ok(self
            .0
            .create_os(name, connector.map(|c| c.into()), None)
            .map_err(MemflowPyError::Memflow)?
            .into())
    }
}
