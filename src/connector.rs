use memflow::{
    prelude::{ConnectorInstanceArcBox, PhysicalMemory},
    types::umem,
};
use pyo3::{exceptions::PyException, prelude::*};

use crate::internal::InternalDT;

#[derive(Clone)]
#[pyclass(name = "Connector")]
pub struct PyConnector(ConnectorInstanceArcBox<'static>);

impl PyConnector {
    pub fn new(inst: ConnectorInstanceArcBox<'static>) -> Self {
        Self(inst)
    }
}

#[pymethods]
impl PyConnector {
    #[getter]
    fn max_address(&mut self) -> umem {
        self.0.metadata().max_address.to_umem()
    }

    #[getter]
    fn real_size(&mut self) -> umem {
        self.0.metadata().real_size
    }

    #[getter]
    fn readonly(&mut self) -> bool {
        self.0.metadata().readonly
    }

    #[getter]
    fn ideal_batch_size(&mut self) -> u32 {
        self.0.metadata().ideal_batch_size
    }

    fn phys_read(&mut self, addr: umem, ty: PyObject) -> PyResult<PyObject> {
        let dt: InternalDT = ty.try_into()?;
        let mut raw: Vec<u8> = vec![0; dt.size()];

        self.0
            .phys_read_into(addr.into(), raw.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(raw)?)
    }

    fn phys_read_ptr(&mut self, ptr_inst: PyObject) -> PyResult<PyObject> {
        let addr: umem = Python::with_gil(|py| ptr_inst.getattr(py, "addr")?.extract(py))?;
        let dt: InternalDT = Python::with_gil(|py| ptr_inst.getattr(py, "_type_")?.try_into())?;
        let mut raw: Vec<u8> = vec![0; dt.size()];

        self.0
            .phys_read_into(addr.into(), raw.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to read bytes {}", e)))?;

        Ok(dt.py_from_bytes(raw)?)
    }

    fn phys_write(&mut self, addr: umem, ty: PyObject, value: PyObject) -> PyResult<()> {
        let dt: InternalDT = ty.try_into()?;

        self.0
            .phys_write(addr.into(), dt.py_to_bytes(value)?.as_mut_slice())
            .map_err(|e| PyException::new_err(format!("failed to write bytes {}", e)))?;

        Ok(())
    }
}

impl From<ConnectorInstanceArcBox<'static>> for PyConnector {
    fn from(inst: ConnectorInstanceArcBox<'static>) -> Self {
        Self(inst)
    }
}

impl From<PyConnector> for ConnectorInstanceArcBox<'static> {
    fn from(value: PyConnector) -> Self {
        value.0
    }
}
