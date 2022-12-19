use crate::{internal::InternalDT, MemflowPyError};
use memflow::{
    prelude::{
        ArchitectureIdent, IntoProcessInstanceArcBox, MemoryView, ModuleInfo, Process, ProcessInfo,
        ProcessState,
    },
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
        let dt: InternalDT = ty.try_into()?;

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

    fn module_by_name(&mut self, name: &str) -> PyResult<PyModuleInfo> {
        Ok(self
            .0
            .module_by_name(name)
            .map_err(MemflowPyError::Memflow)?
            .into())
    }

    fn info(&self) -> PyProcessInfo {
        self.0.info().clone().into()
    }

    fn __str__(&self) -> String {
        self.info().__str__()
    }
}

#[derive(Clone)]
#[pyclass(name = "ProcessInfo")]
pub struct PyProcessInfo(ProcessInfo);

#[pymethods]
impl PyProcessInfo {
    #[new]
    fn new(
        address: umem,
        pid: u32,
        state: PyProcessState,
        name: &str,
        path: &str,
        command_line: &str,
        sys_arch: PyArchitectureIdent,
        proc_arch: PyArchitectureIdent,
    ) -> Self {
        Self(ProcessInfo {
            address: address.into(),
            pid,
            state: state.into(),
            name: name.into(),
            path: path.into(),
            command_line: command_line.into(),
            sys_arch: sys_arch.into(),
            proc_arch: proc_arch.into(),
        })
    }

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

    #[getter]
    fn state(&self) -> PyProcessState {
        self.0.state.clone().into()
    }

    #[getter]
    fn path(&self) -> String {
        self.0.path.to_string()
    }

    #[getter]
    fn command_line(&self) -> String {
        self.0.command_line.to_string()
    }

    #[getter]
    fn sys_arch(&self) -> PyArchitectureIdent {
        self.0.sys_arch.into()
    }

    #[getter]
    fn proc_arch(&self) -> PyArchitectureIdent {
        self.0.proc_arch.into()
    }

    fn __repr__(&self) -> String {
        format!(
            r#"ProcessInfo(address={:#04x}, pid={}, state={}, name="{}", path="{}", command_line="{}", sys_arch={}, proc_arch={})"#,
            self.address(),
            self.pid(),
            self.state().__repr__(),
            self.name(),
            self.path(),
            self.command_line(),
            self.sys_arch().__repr__(),
            self.proc_arch().__repr__()
        )
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
    #[new]
    fn new(
        name: &str,
        address: i32,
        base: umem,
        size: u64,
        path: &str,
        process_addr: umem,
        arch: PyArchitectureIdent,
    ) -> Self {
        Self(ModuleInfo {
            address: address.into(),
            name: name.into(),
            base: base.into(),
            size,
            path: path.into(),
            parent_process: process_addr.into(),
            arch: arch.into(),
        })
    }

    /// Returns the address of the module header.
    ///
    /// # Remarks
    ///
    /// On Windows this will be the address where the [`PEB`](https://docs.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-peb) entry is stored.
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

    #[getter]
    fn parent_process(&self) -> umem {
        self.0.parent_process.to_umem()
    }

    #[getter]
    fn arch(&self) -> PyArchitectureIdent {
        self.0.arch.into()
    }

    fn __repr__(&self) -> String {
        format!(
            r#"ModuleInfo(address={:#04x}, name="{}", base={:#04x}, size={:#04x}, path="{}", parent_process={}, arch={})"#,
            self.address(),
            self.name(),
            self.base(),
            self.size(),
            self.path(),
            self.parent_process(),
            self.arch().__repr__()
        )
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

#[derive(Clone)]
#[pyclass(name = "ArchitectureIdent")]
pub struct PyArchitectureIdent(ArchitectureIdent);

#[pymethods]
impl PyArchitectureIdent {
    #[new]
    fn new(
        arch: &str,
        page_size: Option<usize>,
        address_extensions: Option<bool>,
    ) -> PyResult<Self> {
        let ident = match arch {
            "X86_64" => ArchitectureIdent::X86(64, address_extensions.unwrap_or_default()),
            "X86" => ArchitectureIdent::X86(32, address_extensions.unwrap_or_default()),
            "AArch64" => {
                ArchitectureIdent::AArch64(page_size.unwrap_or_else(|| memflow::types::size::kb(4)))
            }
            "Unknown" => {
                ArchitectureIdent::Unknown(page_size.unwrap_or_else(|| memflow::types::size::kb(4)))
            }
            _ => Err(MemflowPyError::InvalidArch(arch.to_string()))?,
        };

        Ok(Self(ident))
    }

    fn __repr__(&self) -> String {
        match self.0 {
            ArchitectureIdent::Unknown(page_size) => {
                format!(r#"ArchitectureIdent("Unknown", page_size={})"#, page_size)
            }
            ArchitectureIdent::X86(bitness, address_extensions) => match bitness {
                64 if address_extensions => {
                    r#"ArchitectureIdent("X86", address_extensions=True)"#.to_owned()
                }
                32 if address_extensions => {
                    r#"ArchitectureIdent("X86", address_extensions=True)"#.to_owned()
                }
                64 => r#"ArchitectureIdent("X86_64")"#.to_owned(),
                32 => r#"ArchitectureIdent("X86")"#.to_owned(),
                _ => unreachable!("bitness should only be 32bit or 64bit"),
            },
            ArchitectureIdent::AArch64(page_size) => {
                format!(
                    r#"ArchitectureIdent("AArch64", page_size={:#04x})"#,
                    page_size
                )
            }
        }
    }
}

impl From<ArchitectureIdent> for PyArchitectureIdent {
    fn from(ai: ArchitectureIdent) -> Self {
        Self(ai)
    }
}

impl From<PyArchitectureIdent> for ArchitectureIdent {
    fn from(py_ident: PyArchitectureIdent) -> Self {
        py_ident.0
    }
}

#[derive(Clone)]
#[pyclass(name = "ProcessState")]
pub struct PyProcessState(ProcessState);

#[pymethods]
impl PyProcessState {
    #[new]
    fn new(alive: bool, exit_code: Option<i32>) -> Self {
        let state = match alive {
            true => ProcessState::Alive,
            false if exit_code.is_some() => ProcessState::Dead(exit_code.unwrap()),
            _ => ProcessState::Unknown,
        };

        Self(state)
    }

    fn __repr__(&self) -> String {
        match self.0 {
            ProcessState::Unknown => "ProcessState(alive=False)".to_owned(),
            ProcessState::Alive => "ProcessState(alive=True)".to_owned(),
            ProcessState::Dead(exit_code) => format!("ProcessState(alive=false, {})", exit_code),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl From<ProcessState> for PyProcessState {
    fn from(ps: ProcessState) -> Self {
        Self(ps)
    }
}

impl From<PyProcessState> for ProcessState {
    fn from(py_state: PyProcessState) -> Self {
        py_state.0
    }
}
