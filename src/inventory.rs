use memflow::prelude::{ConnectorArgs, Inventory, OsArgs};
use pyo3::prelude::*;

use crate::{connector::PyConnector, os::PyOs, MemflowPyError};

#[pyclass(name = "Inventory")]
pub struct PyInventory(Inventory);

#[pymethods]
impl PyInventory {
    #[new]
    fn new(path: Option<&str>) -> PyResult<Self> {
        let inventory = match path {
            Some(path) => Inventory::scan_path(path).map_err(MemflowPyError::Memflow)?,
            None => Inventory::scan(),
        };

        Ok(Self(inventory))
    }

    fn connector(&self, name: &str, args: Option<&str>) -> PyResult<PyConnector> {
        Ok(PyConnector::new(
            self.0
                .create_connector(
                    name,
                    None,
                    args.and_then(|a| str::parse::<ConnectorArgs>(a).ok())
                        .as_ref(),
                )
                .map_err(MemflowPyError::Memflow)?,
        ))
    }

    fn os(&self, name: &str, connector: Option<PyConnector>, args: Option<&str>) -> PyResult<PyOs> {
        Ok(self
            .0
            .create_os(
                name,
                connector.map(|c| c.into()),
                args.and_then(|a| str::parse::<OsArgs>(a).ok()).as_ref(),
            )
            .map_err(MemflowPyError::Memflow)?
            .into())
    }
}
