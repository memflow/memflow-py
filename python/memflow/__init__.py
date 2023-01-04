from .memflow import *
from ctypes import Structure, c_uint64
from _ctypes import _Pointer

# Corresponds to memflow `umem` type.
umem = c_uint64
# Cache for memflow pointer types.
mf_lp_types = dict()


class MFPointer(Structure):
    _fields_ = [("addr", umem)]

    def __init__(self, addr):
        self.addr = addr
        print(f"self: {self}")

    def __str__(self):
        return f"{self._type_.__name__} @ {hex(self.addr)}"


def MF_POINTER(target_type, byteness):
    global mf_lp_types
    if target_type not in mf_lp_types:
        ptr_type = type(
            f"MF_LP_{target_type.__name__}",
            (MFPointer,),
            dict(
                _type_=target_type,
                _byteness_=byteness,
            ),
        )
        mf_lp_types[target_type] = ptr_type
    return mf_lp_types[target_type]


def POINTER64(target_type):
    return MF_POINTER(target_type, 8)


def POINTER32(target_type):
    return MF_POINTER(target_type, 4)
