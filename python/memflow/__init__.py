from .memflow import *


class CDataTypeMeta(type):
    def __mul__(self, length):
        return ARRAY(self, length)


# TODO: Move to rust
class Structure(object, metaclass=CDataTypeMeta):
    def __new__(cls, *args, **kwargs):
        # self._endianness_ = "what to put here?"
        # unpack *args to kwargs if not already present
        for idx, val in enumerate(args):
            field_name = cls._fields_[idx][0]
            if not kwargs.get(field_name):
                kwargs[field_name] = val
        # Set field and offset attributes on the class
        for field_name, field_val in kwargs.items():
            setattr(cls, field_name, field_val)
        return super(Structure, cls).__new__(cls)

    def __eq__(self, other):
        if self.__class__ == other.__class__:
            return self.__dict__ == other.__dict__
        else:
            raise TypeError("Comparing object is not of the same structure type.")

    def __str__(self):
        return repr(self)

    def __repr__(self):
        fields_strs = []
        for field in self._fields_:
            field_name = field[0]
            # Skip "private" fields (i.e. _pad_0x0)
            if field_name.startswith("_"):
                continue
            fields_strs.append(f"{field_name}={getattr(self, field_name)}")
        return "{}({})".format(self.__class__.__name__, ", ".join(fields_strs))


# TODO: Move to rust
class Array:
    def __init__(self, *args):
        self._vals_ = args

    def __getitem__(self, key):
        return self._vals_[key]

    def __len__(self):
        return self._length_

    def __str__(self):
        return repr(self)

    def __repr__(self):
        fields_strs = []
        for field in self._fields_:
            field_name = field[0]
            fields_strs.append(f"{field_name}={getattr(self, field_name)}")
        return "ARRAY({}, {})({})".format(
            self._type_.__name__, len(self), ", ".join(self._vals_)
        )


# Cache for memflow array types.
mf_arr_types = dict()


def ARRAY(target_type, len):
    global mf_arr_types
    if len not in mf_arr_types:
        mf_arr_types[len] = dict()
    if target_type not in mf_arr_types[len]:
        arr_type = type(
            f"{target_type.__name__}_MF_Array_{len}",
            (Array,),
            dict(_type_=target_type, _length_=len),
        )
        mf_arr_types[len][target_type] = arr_type
    return mf_arr_types[len][target_type]


class CDataType(object, metaclass=CDataTypeMeta):
    # Preserve ctypes shorthand type
    _type_ = ""


# Fill in c data types
class c_short(CDataType):
    _type_ = "h"


class c_ushort(CDataType):
    _type_ = "H"


class c_long(CDataType):
    _type_ = "l"


class c_ulong(CDataType):
    _type_ = "L"


class c_int(CDataType):
    _type_ = "i"


class c_uint(CDataType):
    _type_ = "I"


class c_float(CDataType):
    _type_ = "f"


class c_double(CDataType):
    _type_ = "d"


class c_longdouble(CDataType):
    _type_ = "g"


class c_longlong(CDataType):
    _type_ = "q"


class c_ulonglong(CDataType):
    _type_ = "Q"


class c_ubyte(CDataType):
    _type_ = "B"


class c_byte(CDataType):
    _type_ = "b"


class c_char(CDataType):
    _type_ = "c"


class c_bool(CDataType):
    _type_ = "?"


class c_wchar(CDataType):
    _type_ = "u"


# Fill in specifically-sized types
c_int8 = c_byte
c_uint8 = c_ubyte
for kind in [c_short, c_int, c_long, c_longlong]:
    if sizeof(kind) == 2:
        c_int16 = kind
    elif sizeof(kind) == 4:
        c_int32 = kind
    elif sizeof(kind) == 8:
        c_int64 = kind
for kind in [c_ushort, c_uint, c_ulong, c_ulonglong]:
    if sizeof(kind) == 2:
        c_uint16 = kind
    elif sizeof(kind) == 4:
        c_uint32 = kind
    elif sizeof(kind) == 8:
        c_uint64 = kind
del kind
# Corresponds to memflow `umem` type, guaranteed to hold a whole address of introspection target.
umem = c_uint64


class Pointer(Structure):
    _fields_ = [("addr", umem)]

    def __init__(self, addr):
        self.addr = addr

    def __str__(self):
        return f"{self._type_.__name__} @ {hex(self.addr)}"


# Cache for memflow pointer types.
mf_lp_types = dict()


def POINTER(target_type, byteness):
    global mf_lp_types
    if byteness not in mf_lp_types:
        mf_lp_types[byteness] = dict()
    if target_type not in mf_lp_types[byteness]:
        ptr_type = type(
            f"MF_LP_{target_type.__name__}",
            (Pointer,),
            dict(
                _type_=target_type,
                _byteness_=byteness,
            ),
        )
        mf_lp_types[byteness][target_type] = ptr_type
    return mf_lp_types[byteness][target_type]


def POINTER64(target_type):
    return POINTER(target_type, 8)


def POINTER32(target_type):
    return POINTER(target_type, 4)
