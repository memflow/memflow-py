use std::ffi::*;
use std::mem::size_of;

use indexmap::IndexMap;
use memflow::types::umem;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};

use crate::MemflowPyError;

/// Please stick to explicit widths, no c_int nonsense!
#[derive(Clone, Debug)]
pub enum InternalDT {
    /// Represents the C signed char datatype, and interprets the value as small integer.
    Byte,
    /// Represents the C unsigned char datatype, it interprets the value as small integer.
    UByte,
    /// Represents the C char datatype, and interprets the value as a single character.
    Char,
    /// Represents the C wchar_t datatype, and interprets the value as a single character unicode string.
    WideChar,
    /// Represents the C double datatype.
    Double,
    /// Represents the C long double datatype. On platforms where sizeof(long double) == sizeof(double) it is an alias to c_double.
    /// For more info see: https://github.com/rust-lang/rust-bindgen/issues/1549
    LongDouble,
    /// Represents the C float datatype.
    Float,
    /// Represents the C signed short datatype. no overflow checking is done.
    Short,
    /// Represents the C unsigned short datatype. no overflow checking is done.
    UShort,
    /// Represents the C signed int datatype. no overflow checking is done. On platforms where sizeof(int) == sizeof(long) it is an alias to c_long.
    Int,
    /// Represents the C unsigned int datatype. no overflow checking is done. On platforms where sizeof(int) == sizeof(long) it is an alias for c_ulong.
    UInt,
    /// Represents the C signed long datatype.
    Long,
    /// Represents the C unsigned long datatype.
    ULong,
    /// Represents the C signed long long datatype.
    LongLong,
    /// Represents the C unsigned long long datatype.
    ULongLong,
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
            InternalDT::Byte => Ok(i8::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::UByte => Ok(u8::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::Char => Ok(PyBytes::new(py, &bytes).to_object(py)),
            InternalDT::WideChar => todo!(),
            InternalDT::Double => Ok(c_double::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::LongDouble => todo!(),
            InternalDT::Float => Ok(c_float::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::Short => Ok(c_short::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::UShort => Ok(c_ushort::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::Int => Ok(c_int::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::UInt => Ok(c_uint::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::Long => Ok(c_long::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::ULong => Ok(c_ulong::from_le_bytes(bytes[..].try_into()?).to_object(py)),
            InternalDT::LongLong => {
                Ok(c_longlong::from_le_bytes(bytes[..].try_into()?).to_object(py))
            }
            InternalDT::ULongLong => {
                Ok(c_ulonglong::from_le_bytes(bytes[..].try_into()?).to_object(py))
            }
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
            InternalDT::Byte => Ok(obj.extract::<i8>(py)?.to_le_bytes().to_vec()),
            InternalDT::UByte => Ok(obj.extract::<u8>(py)?.to_le_bytes().to_vec()),
            InternalDT::Char => Ok(PyBytes::try_from_exact(obj.as_ref(py))
                .unwrap()
                .as_bytes()
                .to_owned()),
            // OS widechar encoding.
            InternalDT::WideChar => todo!(),
            InternalDT::Double => Ok(obj.extract::<c_double>(py)?.to_le_bytes().to_vec()),
            InternalDT::LongDouble => todo!(),
            InternalDT::Float => Ok(obj.extract::<c_float>(py)?.to_le_bytes().to_vec()),
            InternalDT::Short => Ok(obj.extract::<c_short>(py)?.to_le_bytes().to_vec()),
            InternalDT::UShort => Ok(obj.extract::<c_ushort>(py)?.to_le_bytes().to_vec()),
            InternalDT::Int => Ok(obj.extract::<c_int>(py)?.to_le_bytes().to_vec()),
            InternalDT::UInt => Ok(obj.extract::<c_uint>(py)?.to_le_bytes().to_vec()),
            InternalDT::Long => Ok(obj.extract::<c_long>(py)?.to_le_bytes().to_vec()),
            InternalDT::ULong => Ok(obj.extract::<c_ulong>(py)?.to_le_bytes().to_vec()),
            InternalDT::LongLong => Ok(obj.extract::<c_longlong>(py)?.to_le_bytes().to_vec()),
            InternalDT::ULongLong => Ok(obj.extract::<c_ulonglong>(py)?.to_le_bytes().to_vec()),
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
            InternalDT::Byte => size_of::<c_schar>(),
            InternalDT::UByte => size_of::<c_uchar>(),
            InternalDT::Char => size_of::<c_char>(),
            InternalDT::WideChar => size_of::<c_char>() * 2,
            InternalDT::Short => size_of::<c_short>(),
            InternalDT::UShort => size_of::<c_ushort>(),
            InternalDT::Double => size_of::<c_double>(),
            InternalDT::LongDouble => size_of::<c_double>() * 2,
            InternalDT::Float => size_of::<c_float>(),
            InternalDT::Int => size_of::<c_int>(),
            InternalDT::UInt => size_of::<c_uint>(),
            InternalDT::Long => size_of::<c_long>(),
            InternalDT::ULong => size_of::<c_ulong>(),
            InternalDT::LongLong => size_of::<c_longlong>(),
            InternalDT::ULongLong => size_of::<c_ulonglong>(),
            InternalDT::Pointer32(_) => 4,
            InternalDT::Pointer64(_) => 8,
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
                    "c_byte" => Self::Byte,
                    "c_ubyte" | "c_bool" => Self::UByte,
                    "c_char" => Self::Char,
                    "c_char_p" => todo!("add c_char_p support"),
                    "c_wchar" => Self::WideChar,
                    "c_wchar_p" => todo!("add c_wchar_p support"),
                    "c_double" => Self::Double,
                    "c_longdouble" => Self::LongDouble,
                    "c_float" => Self::Float,
                    "c_short" => Self::Short,
                    "c_ushort" => Self::UShort,
                    "c_int" => Self::Int,
                    "c_uint" => Self::UInt,
                    "c_long" => Self::Long,
                    "c_ulong" => Self::ULong,
                    "c_longlong" => Self::LongLong,
                    "c_ulonglong" => Self::ULongLong,
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
                            .ok_or_else(|| MemflowPyError::NoType(field_name.clone()))?
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
