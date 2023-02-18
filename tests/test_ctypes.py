from memflow import Process, ProcessInfo, ProcessState, Inventory, Os, dummy
from ctypes import *


class POINT(Structure):
    _fields_ = [("x", c_uint32), ("y", c_float)]

    def __str__(self):
        return f"POINT = {self.x}, {self.y}"


def test_basic():
    proc = dummy.quick_process(4096, bytes([0x8]))
    proc_address = proc.info().address

    # Test writing new `TEST` structure.
    proc.write(proc_address, POINT, POINT(55, 3.14))

    # Test reading a structure.
    test_works = proc.read(proc_address, POINT)
    assert test_works.x == 55
