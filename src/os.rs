use memflow::{
    os::{process::Pid, OsInner},
    prelude::OsInstanceArcBox,
    types::umem,
};
use pyo3::prelude::*;
use std::cell::RefCell;

use crate::process::{PyProcess, PyProcessInfo};
use crate::MemflowPyError;

#[derive(Clone)]
#[pyclass(name = "Os")]
pub struct PyOs(RefCell<OsInstanceArcBox<'static>>);

#[pymethods]
impl PyOs {
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
}

impl From<OsInstanceArcBox<'static>> for PyOs {
    fn from(inst: OsInstanceArcBox<'static>) -> Self {
        Self(RefCell::new(inst))
    }
}
