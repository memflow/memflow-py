use memflow::{
    os::{process::Pid, OsInner},
    prelude::{MemoryView, OsInstanceArcBox, PhysicalMemory},
    types::umem,
};
use pyo3::{exceptions::PyException, prelude::*};
use std::cell::RefCell;

use crate::{
    internal::InternalDT,
    process::{PyModuleInfo, PyProcess, PyProcessInfo},
    MemflowPyError,
};

#[derive(Clone)]
#[pyclass(name = "Os")]
pub struct PyOs(RefCell<OsInstanceArcBox<'static>>);

#[pymethods]
impl PyOs {
    #[getter]
    fn arch(&mut self) -> String {
        self.0.borrow_mut().info().arch.to_string()
    }

    #[getter]
    fn base(&mut self) -> umem {
        self.0.borrow_mut().info().base.to_umem()
    }

    #[getter]
    fn size(&mut self) -> u64 {
        self.0.borrow_mut().info().size
    }

    pub fn process_info_list(&mut self) -> PyResult<Vec<PyProcessInfo>> {
        Ok(self
            .0
            .borrow_mut()
            .process_info_list()
            .map_err(MemflowPyError::Memflow)?
            .into_iter()
            .map(PyProcessInfo::from)
            .collect())
    }

    pub fn process_from_info(&mut self, info: PyProcessInfo) -> PyResult<PyProcess> {
        let t = self.0.borrow_mut().clone();
        Ok(PyProcess::new(t.into_process_by_info(info.into()).unwrap()))
    }

    pub fn process_from_addr(&mut self, addr: umem) -> PyResult<PyProcess> {
        let t = self.0.borrow_mut().clone();
        Ok(PyProcess::new(
            t.into_process_by_address(addr.into()).unwrap(),
        ))
    }

    pub fn process_from_pid(&mut self, pid: Pid) -> PyResult<PyProcess> {
        let t = self.0.borrow_mut().clone();
        Ok(PyProcess::new(t.into_process_by_pid(pid).unwrap()))
    }

    pub fn process_from_name(&mut self, name: &str) -> PyResult<PyProcess> {
        let t = self.0.borrow_mut().clone();
        Ok(PyProcess::new(t.into_process_by_name(name).unwrap()))
    }

    fn module_info_list(&mut self) -> PyResult<Vec<PyModuleInfo>> {
        Ok(self
            .0
            .borrow_mut()
            .module_list()
            .map_err(MemflowPyError::Memflow)?
            .into_iter()
            .map(PyModuleInfo::from)
            .collect())
    }

    fn module_from_name(&mut self, name: &str) -> PyResult<PyModuleInfo> {
        Ok(self
            .0
            .borrow_mut()
            .module_by_name(name)
            .map_err(MemflowPyError::Memflow)?
            .into())
    }

    fn read(&mut self, addr: umem, ty: PyObject) -> PyResult<PyObject> {
        let dt: InternalDT = ty.try_into()?;

        let bytes = self
            .0
            .borrow_mut()
            .as_mut_impl_memoryview()
            .ok_or_else(|| {
                MemflowPyError::MissingCGlueImpl("Os".to_owned(), "MemoryView".to_owned())
            })?
            .read_raw(addr.into(), dt.size())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(bytes)?)
    }

    fn read_ptr(&mut self, ptr_inst: PyObject) -> PyResult<PyObject> {
        let addr: umem = Python::with_gil(|py| ptr_inst.getattr(py, "addr")?.extract(py))?;
        let dt: InternalDT = Python::with_gil(|py| ptr_inst.getattr(py, "_type_")?.try_into())?;

        let bytes = self
            .0
            .borrow_mut()
            .as_mut_impl_memoryview()
            .ok_or_else(|| {
                MemflowPyError::MissingCGlueImpl("Os".to_owned(), "MemoryView".to_owned())
            })?
            .read_raw(addr.into(), dt.size())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(bytes)?)
    }

    fn write(&mut self, addr: umem, ty: PyObject, value: PyObject) -> PyResult<()> {
        let dt: InternalDT = ty.try_into()?;

        self.0
            .borrow_mut()
            .as_mut_impl_memoryview()
            .ok_or_else(|| {
                MemflowPyError::MissingCGlueImpl("Os".to_owned(), "MemoryView".to_owned())
            })?
            .write_raw(addr.into(), &dt.py_to_bytes(value)?)
            .map_err(|e| PyException::new_err(format!("failed to write bytes {}", e)))?;

        Ok(())
    }

    fn phys_read(&mut self, addr: umem, ty: PyObject) -> PyResult<PyObject> {
        let dt: InternalDT = ty.try_into()?;
        let mut raw: Vec<u8> = vec![0; dt.size()];

        self.0
            .borrow_mut()
            .as_mut_impl_physicalmemory()
            .ok_or_else(|| {
                MemflowPyError::MissingCGlueImpl("Os".to_owned(), "PhysicalMemory".to_owned())
            })?
            .phys_read_into(addr.into(), raw.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(raw)?)
    }

    fn phys_read_ptr(&mut self, ptr_inst: PyObject) -> PyResult<PyObject> {
        let addr: umem = Python::with_gil(|py| ptr_inst.getattr(py, "addr")?.extract(py))?;
        let dt: InternalDT = Python::with_gil(|py| ptr_inst.getattr(py, "_type_")?.try_into())?;
        let mut raw: Vec<u8> = vec![0; dt.size()];

        self.0
            .borrow_mut()
            .as_mut_impl_physicalmemory()
            .ok_or_else(|| {
                MemflowPyError::MissingCGlueImpl("Os".to_owned(), "PhysicalMemory".to_owned())
            })?
            .phys_read_into(addr.into(), raw.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(raw)?)
    }

    fn phys_write(&mut self, addr: umem, ty: PyObject, value: PyObject) -> PyResult<()> {
        let dt: InternalDT = ty.try_into()?;

        self.0
            .borrow_mut()
            .as_mut_impl_physicalmemory()
            .ok_or_else(|| {
                MemflowPyError::MissingCGlueImpl("Os".to_owned(), "PhysicalMemory".to_owned())
            })?
            .phys_write(addr.into(), dt.py_to_bytes(value)?.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to write bytes {}", e)))?;

        Ok(())
    }
}

impl From<OsInstanceArcBox<'static>> for PyOs {
    fn from(inst: OsInstanceArcBox<'static>) -> Self {
        Self(RefCell::new(inst))
    }
}

impl From<PyOs> for OsInstanceArcBox<'static> {
    fn from(value: PyOs) -> Self {
        value.0.into_inner()
    }
}
