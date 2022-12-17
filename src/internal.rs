use std::ffi::{self, c_double, c_float};
use std::mem::size_of;

use indexmap::IndexMap;
use memflow::types::umem;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};

use crate::MemflowPyError;

/// Please stick to explicit widths, no c_int nonsense!
#[derive(Clone, Debug)]
pub enum InternalDT {
    /// Represents the C char datatype, and interprets the value as a single character. The constructor accepts an optional string initializer, the length of the string must be exactly one character.
    Char,
    /// Represents the C double datatype. The constructor accepts an optional float initializer.
    Double,
    /// Represents the C long double datatype. The constructor accepts an optional float initializer. On platforms where sizeof(long double) == sizeof(double) it is an alias to c_double.
    /// For more info see: https://github.com/rust-lang/rust-bindgen/issues/1549
    LongDouble,
    /// Represents the C float datatype. The constructor accepts an optional float initializer.
    Float,
    /// Represents the C 8-bit signed int datatype. Usually an alias for c_byte.
    Int8,
    /// Represents the C 16-bit signed int datatype. Usually an alias for c_short.
    Int16,
    /// Represents the C 32-bit signed int datatype. Usually an alias for c_int.
    Int32,
    /// Represents the C 64-bit signed int datatype. Usually an alias for c_longlong.
    Int64,
    /// Represents the C 8-bit unsigned int datatype. Usually an alias for c_ubyte.
    UInt8,
    /// Represents the C 16-bit unsigned int datatype. Usually an alias for c_ushort.
    UInt16,
    /// Represents the C 32-bit unsigned int datatype. Usually an alias for c_uint.
    UInt32,
    /// Represents the C 64-bit unsigned int datatype. Usually an alias for c_ulonglong.
    UInt64,
    /// Represents the C wchar_t datatype, and interprets the value as a single character unicode string. The constructor accepts an optional string initializer, the length of the string must be exactly one character.
    WideChar,
    /// Native pointer type.
    Pointer(PyObject),
    /// Backed by the python function `POINTER32`.
    Pointer32(PyObject),
    /// Backed by the python function `POINTER64`.
    Pointer64(PyObject),
    // Backed by the ctypes (ctype * size) syntax.
    Array(PyObject, Box<InternalDT>, u32),
    /// Any python class with a ctypes _fields_ attribute.
    Structure(PyObject, IndexMap<String, InternalDT>),
}

impl InternalDT {
    pub fn py_from_bytes(&self, mut bytes: Vec<u8>) -> crate::Result<PyObject> {
        Python::with_gil(|py| match self {
            InternalDT::Char => Ok(PyBytes::new(py, &bytes).to_object(py)),
            InternalDT::Double => {
                Ok(ffi::c_float::from_le_bytes(bytes[..].try_into()?).to_object(py))
            }
            InternalDT::LongDouble => todo!(),
            InternalDT::Float => {
                Ok(ffi::c_float::from_le_bytes(bytes[..].try_into()?).to_object(py))
            }
            InternalDT::Int8 => Ok(i8::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::Int16 => Ok(i16::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::Int32 => Ok(i32::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::Int64 => Ok(i64::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::UInt8 => Ok(u8::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::UInt16 => Ok(u16::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::UInt32 => Ok(u32::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::UInt64 => Ok(u64::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::WideChar => todo!(),
            InternalDT::Pointer(_) => {
                todo!("Need to store address in an attribute to access later")
            }
            InternalDT::Pointer32(class) => {
                Ok(class.call1(py, (u32::from_le_bytes(bytes[..].try_into()?),))?)
            }
            InternalDT::Pointer64(class) => {
                Ok(class.call1(py, (u64::from_le_bytes(bytes[..].try_into()?),))?)
            }
            InternalDT::Array(class, dt, _) => Ok(class.call1(
                py,
                PyTuple::new(
                    py,
                    bytes
                        .chunks(dt.size())
                        .into_iter()
                        .map(|w| dt.py_from_bytes(w.to_vec()).unwrap()),
                ),
            )?),
            InternalDT::Structure(class, dts) => {
                let class_inst = class.call0(py)?;
                dts.into_iter()
                    .try_for_each::<_, crate::Result<()>>(|(name, dt)| {
                        let val = dt.py_from_bytes(bytes.drain(..dt.size()).collect())?;
                        class_inst.setattr(py, name.as_str(), val)?;
                        Ok(())
                    })?;
                Ok(class_inst)
            }
        })
    }

    pub fn py_to_bytes(&self, obj: PyObject) -> crate::Result<Vec<u8>> {
        Python::with_gil(|py| match self {
            InternalDT::Char => Ok(PyBytes::try_from_exact(obj.as_ref(py))
                .unwrap()
                .as_bytes()
                .to_owned()),
            InternalDT::Double => Ok(obj.extract::<c_double>(py)?.to_le_bytes().to_vec()),
            InternalDT::LongDouble => todo!(),
            InternalDT::Float => Ok(obj.extract::<c_float>(py)?.to_le_bytes().to_vec()),
            InternalDT::Int8 => Ok(obj.extract::<i8>(py)?.to_le_bytes().to_vec()),
            InternalDT::Int16 => Ok(obj.extract::<i16>(py)?.to_le_bytes().to_vec()),
            InternalDT::Int32 => Ok(obj.extract::<i32>(py)?.to_le_bytes().to_vec()),
            InternalDT::Int64 => Ok(obj.extract::<i64>(py)?.to_le_bytes().to_vec()),
            InternalDT::UInt8 => Ok(obj.extract::<u8>(py)?.to_le_bytes().to_vec()),
            InternalDT::UInt16 => Ok(obj.extract::<u16>(py)?.to_le_bytes().to_vec()),
            InternalDT::UInt32 => Ok(obj.extract::<u32>(py)?.to_le_bytes().to_vec()),
            InternalDT::UInt64 => Ok(obj.extract::<u64>(py)?.to_le_bytes().to_vec()),
            // OS widechar encoding.
            InternalDT::WideChar => todo!(),
            InternalDT::Pointer(_) => todo!(),
            InternalDT::Pointer32(_) => Ok(obj
                .getattr(py, "addr")?
                .extract::<umem>(py)?
                .to_le_bytes()
                .to_vec()),
            InternalDT::Pointer64(_) => Ok(obj
                .getattr(py, "addr")?
                .extract::<umem>(py)?
                .to_le_bytes()
                .to_vec()),
            InternalDT::Array(_, dt, len) => {
                let mut bytes = Vec::new();
                for i in 0..*len {
                    let item_obj = obj.call_method1(py, "__getitem__", (i,))?;
                    bytes.append(&mut dt.py_to_bytes(item_obj)?);
                }
                Ok(bytes)
            }
            // NOTE: The passed object is not checked to be type of structure.
            InternalDT::Structure(_, dts) => {
                let mut bytes = Vec::new();
                dts.into_iter()
                    .try_for_each::<_, crate::Result<()>>(|(name, dt)| {
                        if let Ok(val_obj) = obj.getattr(py, name.as_str()) {
                            bytes.append(&mut dt.py_to_bytes(val_obj)?);
                            Ok(())
                        } else {
                            Err(MemflowPyError::MissingAttribute(name.to_owned()))
                        }
                    })?;
                Ok(bytes)
            }
        })
    }

    pub fn size(&self) -> usize {
        match self {
            InternalDT::Double => size_of::<ffi::c_double>(),
            // TODO: I have a feeling this wont end well.
            InternalDT::LongDouble => size_of::<ffi::c_double>() * 2,
            InternalDT::Float => size_of::<ffi::c_float>(),
            InternalDT::Char | InternalDT::Int8 | InternalDT::UInt8 => 1,
            InternalDT::Int16 | InternalDT::UInt16 | InternalDT::WideChar => 2,
            InternalDT::Int32 | InternalDT::UInt32 | InternalDT::Pointer32(_) => 4,
            InternalDT::Int64 | InternalDT::UInt64 | InternalDT::Pointer64(_) => 8,
            InternalDT::Pointer(_) => size_of::<usize>(),
            InternalDT::Array(_, dt, len) => dt.size() * (*len as usize),
            InternalDT::Structure(_, dts) => dts.iter().map(|(_, dt)| dt.size()).sum(),
        }
    }
}

impl TryFrom<PyObject> for InternalDT {
    type Error = MemflowPyError;

    fn try_from(value: PyObject) -> Result<Self, Self::Error> {
        let base_name: String = Python::with_gil(|py| {
            let base_obj: PyObject = value.getattr(py, "__base__")?.extract(py)?;
            base_obj.getattr(py, "__name__")?.extract(py)
        })?;

        match base_name.as_str() {
            "_SimpleCData" => {
                let name: String =
                    Python::with_gil(|py| value.getattr(py, "__name__")?.extract(py))?;
                let dt = match name.as_str() {
                    "c_char" => Self::Char,
                    "c_char_p" => todo!("add c_char_p support"),
                    "c_double" => Self::Double,
                    "c_longdouble" => Self::LongDouble,
                    "c_float" => Self::Float,
                    "c_int8" | "c_byte" => Self::Int8,
                    "c_int16" | "c_short" => Self::Int16,
                    "c_int32" | "c_long" | "c_int" => Self::Int32,
                    "c_int64" | "c_longlong" => Self::Int64,
                    "c_uint8" | "c_bool" | "c_ubyte" => Self::UInt8,
                    "c_uint16" | "c_ushort" => Self::UInt16,
                    "c_uint32" | "c_ulong" | "c_uint" => Self::UInt32,
                    "c_uint64" | "c_ulonglong" => Self::UInt64,
                    "c_wchar" => Self::WideChar,
                    "c_wchar_p" => todo!("add c_wchar_p support"),
                    name => unreachable!("unknown SimpleCData type: {}", name),
                };
                Ok(dt)
            }
            "_Pointer" => Ok(InternalDT::Pointer(value)),
            "Pointer32" => Ok(InternalDT::Pointer32(value)),
            "Pointer64" => Ok(InternalDT::Pointer64(value)),
            "Array" => {
                let (len, ty_obj) = Python::with_gil::<_, crate::Result<(u32, PyObject)>>(|py| {
                    Ok((
                        value.getattr(py, "_length_")?.extract(py)?,
                        value.getattr(py, "_type_")?.extract(py)?,
                    ))
                })?;
                Ok(InternalDT::Array(value, Box::new(ty_obj.try_into()?), len))
            }
            "Structure" => {
                let fields = Python::with_gil(|py| {
                    value
                        .getattr(py, "_fields_")?
                        .extract::<Vec<Vec<PyObject>>>(py)
                })?;

                let dt_fields = fields
                    .into_iter()
                    .map(|field| {
                        let mut it = field.into_iter();
                        let field_name = it.next().unwrap().to_string();
                        let field_type: InternalDT = it
                            .next()
                            .ok_or(MemflowPyError::NoType(field_name.clone()))?
                            .try_into()?;
                        Ok((field_name, field_type))
                    })
                    .collect::<Result<IndexMap<String, InternalDT>, MemflowPyError>>()?;

                Ok(Self::Structure(value, dt_fields))
            }
            _ => Err(MemflowPyError::InvalidType(base_name)),
        }
    }
}
