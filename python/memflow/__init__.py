from .memflow import *
from ctypes import Structure, c_uint32, c_uint64
from _ctypes import _Pointer

lp32_types = dict()
lp64_types = dict()


def __str__ptr(self):
    return f"{self._type_.__name__} @ {hex(self.addr)}"


class Pointer64(_Pointer):
    ...


class Pointer32(_Pointer):
    ...


def POINTER32(target_type):
    global lp32_types
    if target_type not in lp32_types:
        ptr_type = type(
            f"LP32_{target_type.__name__}",
            (Structure,),
            dict(
                __base__=Pointer32,
                _type_=target_type,
                _fields_=[("addr", c_uint32)],
                __str__=__str__ptr,
            ),
        )
        lp32_types[target_type] = ptr_type
    return lp32_types[target_type]


def POINTER64(target_type):
    global lp64_types
    if target_type not in lp64_types:
        ptr_type = type(
            f"LP64_{target_type.__name__}",
            (Structure,),
            dict(
                __base__=Pointer64,
                _type_=target_type,
                _fields_=[("addr", c_uint64)],
                __str__=__str__ptr,
            ),
        )
        lp64_types[target_type] = ptr_type
    return lp64_types[target_type]
