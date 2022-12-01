// Provide dummy api for testing from python.
// TODO: Add builder for dummy OS.

use memflow::{
    dummy::{DummyMemory, DummyOs},
    os::OsInner,
    prelude::OsInstance,
    types::size,
};
use pyo3::prelude::*;
// Used for trait_obj
use cglue::arc::CArc;
use cglue::*;

use crate::os::PyOs;

#[pymodule]
pub fn register_dummy_module(_py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(_py, "dummy")?;
    child_module.add_function(wrap_pyfunction!(os, child_module)?)?;
    parent_module.add_submodule(child_module)?;
    Ok(())
}

/// Retrieve dummy OS with an empty process.
#[pyfunction]
fn os() -> PyOs {
    let mem = DummyMemory::new(size::mb(64));
    let mut os = DummyOs::new(mem);
    let pid = os.alloc_process(size::mb(60), &[]);
    let mut prc = os.process_by_pid(pid).unwrap();
    prc.proc.add_modules(10, size::kb(1));
    group_obj!((os, CArc::default()) as OsInstance).into()
}
