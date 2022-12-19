use memflow::prelude::{ConnectorArgs, Inventory, OsArgs, TargetInfo};
use pyo3::prelude::*;

use crate::{connector::PyConnector, os::PyOs, MemflowPyError};

#[pyclass(name = "Inventory")]
pub struct PyInventory(Inventory);

#[pymethods]
impl PyInventory {
    /// Creates a new inventory of plugins from the provided path.
    /// The path has to be a valid directory or the function will fail with an `Error::IO` error.
    ///
    /// If no path is provided the Inventory will query PATH, and an additional set of of directories (standard unix ones, if unix,
    /// and "HOME/.local/lib" on all OSes) for "memflow" directory, and if there is one, then
    /// search for libraries in there.
    #[new]
    fn new(path: Option<&str>) -> PyResult<Self> {
        let inventory = match path {
            Some(path) => Inventory::scan_path(path).map_err(MemflowPyError::Memflow)?,
            None => Inventory::scan(),
        };

        Ok(Self(inventory))
    }

    /// Adds a library directory to the inventory
    ///
    /// This function optionally applies additional filter to only scan potentially wanted files
    ///
    /// # Safety
    ///
    /// Same as previous functions - compiler can not guarantee the safety of
    /// third party library implementations.
    fn add_dir(&mut self, path: &str, filter: Option<&str>) -> PyResult<()> {
        match filter {
            Some(filter) => self
                .0
                .add_dir_filtered(path.into(), filter)
                .map_err(MemflowPyError::Memflow)?,
            None => self
                .0
                .add_dir(path.into())
                .map_err(MemflowPyError::Memflow)?,
        };

        Ok(())
    }

    /// Returns the names of all currently available connectors that can be used.
    fn available_connectors(&self) -> Vec<String> {
        self.0.available_connectors()
    }

    /// Returns the names of all currently available os plugins that can be used.
    fn available_os(&self) -> Vec<String> {
        self.0.available_os()
    }

    /// Returns the help string of the given Connector.
    ///
    /// This function returns an error in case the Connector was not found or does not implement the help feature.
    fn connector_help(&self, name: &str) -> PyResult<String> {
        Ok(self
            .0
            .connector_help(name)
            .map_err(MemflowPyError::Memflow)?)
    }

    /// Returns the help string of the given Os Plugin.
    ///
    /// This function returns an error in case the Os Plugin was not found or does not implement the help feature.
    fn os_help(&self, name: &str) -> PyResult<String> {
        Ok(self.0.os_help(name).map_err(MemflowPyError::Memflow)?)
    }

    /// Returns a list of all available targets of the connector.
    ///
    /// This function returns an error in case the connector does not implement this feature.
    fn connector_target_list(&self, name: &str) -> PyResult<Vec<PyTargetInfo>> {
        Ok(self
            .0
            .connector_target_list(name)
            .map_err(MemflowPyError::Memflow)?
            .into_iter()
            .map(PyTargetInfo::from)
            .collect())
    }

    // TODO:
    // Creates a new Connector / OS builder.
    // pub fn builder(&self) -> BuilderEmpty

    /// Tries to create a new instance for the library with the given name.
    /// The instance will be initialized with the args provided to this call.
    ///
    /// In case no library could be found this will throw an `Error::Library`.
    ///
    /// # Safety
    ///
    /// This function assumes all libraries were loaded with appropriate safety
    /// checks in place. This function is safe, but can crash if previous checks
    /// fail.
    fn create_connector(
        &self,
        name: &str,
        input: Option<PyOs>,
        args: Option<&str>,
    ) -> PyResult<PyConnector> {
        Ok(PyConnector::new(
            self.0
                .create_connector(
                    name,
                    input.map(|o| o.into()),
                    args.and_then(|a| str::parse::<ConnectorArgs>(a).ok())
                        .as_ref(),
                )
                .map_err(MemflowPyError::Memflow)?,
        ))
    }

    /// Create OS instance
    ///
    /// This is the primary way of building a OS instance.
    ///
    /// # Arguments
    ///
    /// * `name` - name of the target OS
    /// * `input` - connector to be passed to the OS
    /// * `args` - arguments to be passed to the OS
    fn create_os(
        &self,
        name: &str,
        input: Option<PyConnector>,
        args: Option<&str>,
    ) -> PyResult<PyOs> {
        Ok(self
            .0
            .create_os(
                name,
                input.map(|c| c.into()),
                args.and_then(|a| str::parse::<OsArgs>(a).ok()).as_ref(),
            )
            .map_err(MemflowPyError::Memflow)?
            .into())
    }

    // TODO:
    // Sets the maximum logging level in all plugins and updates the
    // internal [`PluginLogger`] in each plugin instance.
    // pub fn set_max_log_level(&self, level: LevelFilter)
}

#[derive(Clone)]
#[pyclass(name = "TargetInfo")]
pub struct PyTargetInfo(TargetInfo);

#[pymethods]
impl PyTargetInfo {
    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    fn __str__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

impl From<TargetInfo> for PyTargetInfo {
    fn from(ti: TargetInfo) -> Self {
        Self(ti)
    }
}

impl From<PyTargetInfo> for TargetInfo {
    fn from(py_info: PyTargetInfo) -> Self {
        py_info.0
    }
}
