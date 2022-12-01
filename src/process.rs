use crate::{internal::InternalDT, MemflowPyError};
use memflow::{
    prelude::{IntoProcessInstanceArcBox, MemoryView, ModuleInfo, Process, ProcessInfo},
    types::umem,
};
use pyo3::{exceptions::PyException, prelude::*};

#[derive(Clone)]
#[pyclass(name = "Process")]
pub struct PyProcess(IntoProcessInstanceArcBox<'static>);

impl PyProcess {
    pub fn new(inst: IntoProcessInstanceArcBox<'static>) -> Self {
        Self(inst)
    }
}

#[pymethods]
impl PyProcess {
    fn read(&mut self, addr: umem, ty: PyObject) -> PyResult<PyObject> {
        let dt: InternalDT = ty.try_into()?;

        let bytes = self
            .0
            .read_raw(addr.into(), dt.size())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(bytes)?)
    }

    fn read_ptr(&mut self, ptr_inst: PyObject) -> PyResult<PyObject> {
        let addr: umem = Python::with_gil(|py| ptr_inst.getattr(py, "addr")?.extract(py))?;
        let dt: InternalDT = Python::with_gil(|py| ptr_inst.getattr(py, "_type_")?.try_into())?;

        let bytes = self
            .0
            .read_raw(addr.into(), dt.size())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(bytes)?)
    }

    fn write(&mut self, addr: umem, ty: PyObject, value: PyObject) -> PyResult<()> {
        let dt: InternalDT = ty.try_into().unwrap(); // TODO: Why does ? not get a valid error propagated!!!

        self.0
            .write_raw(addr.into(), &dt.py_to_bytes(value)?)
            .map_err(|e| PyException::new_err(format!("failed to write bytes {}", e)))?;

        Ok(())
    }

    fn module_info_list(&mut self) -> PyResult<Vec<PyModuleInfo>> {
        Ok(self
            .0
            .module_list()
            .map_err(MemflowPyError::Memflow)?
            .into_iter()
            .map(PyModuleInfo::from)
            .collect())
    }

    fn module_from_name(&mut self, name: &str) -> PyResult<PyModuleInfo> {
        Ok(self
            .0
            .module_by_name(name)
            .map_err(MemflowPyError::Memflow)?
            .into())
    }

    fn __str__(&self) -> String {
        let info = self.0.info();
        format!("{} ({})", info.name, info.pid)
    }
}

#[derive(Clone)]
#[pyclass(name = "ProcessInfo")]
pub struct PyProcessInfo(ProcessInfo);

#[pymethods]
impl PyProcessInfo {
    #[getter]
    fn address(&self) -> umem {
        self.0.address.to_umem()
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn pid(&self) -> u32 {
        self.0.pid
    }

    fn __str__(&self) -> String {
        format!("{} ({}) @ {:#04x}", self.name(), self.pid(), self.address())
    }
}

impl From<ProcessInfo> for PyProcessInfo {
    fn from(pi: ProcessInfo) -> Self {
        Self(pi)
    }
}

impl From<PyProcessInfo> for ProcessInfo {
    fn from(py_info: PyProcessInfo) -> Self {
        py_info.0
    }
}

#[derive(Clone)]
#[pyclass(name = "ModuleInfo")]
pub struct PyModuleInfo(ModuleInfo);

#[pymethods]
impl PyModuleInfo {
    #[getter]
    fn address(&self) -> umem {
        self.0.address.to_umem()
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn base(&self) -> umem {
        self.0.base.to_umem()
    }

    #[getter]
    fn size(&self) -> u64 {
        self.0.size
    }

    #[getter]
    fn path(&self) -> String {
        self.0.path.to_string()
    }

    fn __str__(&self) -> String {
        format!("{} @ {:#04x}", self.name(), self.base())
    }
}

impl From<ModuleInfo> for PyModuleInfo {
    fn from(mi: ModuleInfo) -> Self {
        Self(mi)
    }
}

impl From<PyModuleInfo> for ModuleInfo {
    fn from(py_info: PyModuleInfo) -> Self {
        py_info.0
    }
}
