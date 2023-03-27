// Provide dummy api for testing from python.

use memflow::{
    dummy::{DummyMemory, DummyOs},
    prelude::{IntoProcessInstance, Os, OsInstance, PhysicalMemory, Pid},
    types::umem,
};
use pyo3::{exceptions::PyException, prelude::*};
// Used for trait_obj
use cglue::arc::CArc;
use cglue::*;

use crate::{internal::InternalDT, os::PyOs, process::PyProcess, MemflowPyError};

#[pymodule]
pub fn register_dummy_module(_py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(_py, "dummy")?;
    child_module.add_class::<PyDummyMemory>()?;
    child_module.add_class::<PyDummyOs>()?;
    child_module.add_function(wrap_pyfunction!(quick_process, child_module)?)?;
    parent_module.add_submodule(child_module)?;
    Ok(())
}

#[pyfunction]
fn quick_process(virt_size: usize, buffer: &[u8]) -> PyProcess {
    PyProcess::new(group_obj!(
        (DummyOs::quick_process(virt_size, buffer), CArc::default()) as IntoProcessInstance
    ))
}

#[derive(Clone)]
#[pyclass(name = "DummyOs")]
pub struct PyDummyOs(DummyOs);

#[pymethods]
impl PyDummyOs {
    #[new]
    fn new(memory: PyDummyMemory) -> Self {
        DummyOs::new(memory.into()).into()
    }

    fn retrieve_os(&mut self) -> PyOs {
        group_obj!((self.0.clone(), CArc::default()) as OsInstance).into()
    }

    fn alloc_process(&mut self, size: usize) -> Pid {
        self.0.alloc_process(size, &[])
    }

    fn alloc_process_with_module(&mut self, size: usize) -> Pid {
        self.0.alloc_process_with_module(size, &[])
    }

    fn add_modules_for_process(&mut self, pid: Pid, count: usize, min_size: usize) -> PyResult<()> {
        self.0
            .process_by_pid(pid)
            .map_err(MemflowPyError::Memflow)?
            .proc
            .add_modules(count, min_size);

        Ok(())
    }
}

impl From<DummyOs> for PyDummyOs {
    fn from(dummy_os: DummyOs) -> Self {
        Self(dummy_os)
    }
}

impl From<PyDummyOs> for DummyOs {
    fn from(py_dummy_os: PyDummyOs) -> Self {
        py_dummy_os.0
    }
}

#[derive(Clone)]
#[pyclass(name = "DummyMemory")]
pub struct PyDummyMemory(DummyMemory);

#[pymethods]
impl PyDummyMemory {
    #[new]
    fn new(size: usize) -> Self {
        DummyMemory::new(size).into()
    }

    fn read(&mut self, addr: umem, ty: PyObject) -> PyResult<PyObject> {
        let dt: InternalDT = ty.try_into()?;
        let mut raw: Vec<u8> = vec![0; dt.size()];

        self.0
            .phys_read_into(addr.into(), raw.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(raw)?)
    }

    fn read_ptr(&mut self, ptr_inst: PyObject) -> PyResult<PyObject> {
        let addr: umem = Python::with_gil(|py| ptr_inst.getattr(py, "addr")?.extract(py))?;
        let dt: InternalDT = Python::with_gil(|py| ptr_inst.getattr(py, "_type_")?.try_into())?;
        let mut raw: Vec<u8> = vec![0; dt.size()];

        self.0
            .phys_read_into(addr.into(), raw.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(raw)?)
    }

    fn write(&mut self, addr: umem, ty: PyObject, value: PyObject) -> PyResult<()> {
        let dt: InternalDT = ty.try_into()?;

        self.0
            .phys_write(addr.into(), dt.py_to_bytes(value)?.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to write bytes {}", e)))?;

        Ok(())
    }
}

impl From<DummyMemory> for PyDummyMemory {
    fn from(dm: DummyMemory) -> Self {
        Self(dm)
    }
}

impl From<PyDummyMemory> for DummyMemory {
    fn from(py_dm: PyDummyMemory) -> Self {
        py_dm.0
    }
}
