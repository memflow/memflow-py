from memflow import Process, ProcessInfo, ProcessState, Inventory, Os, dummy
from ctypes import *


class POINT(Structure):
    _fields_ = [("x", c_uint32), ("y", c_float)]

    def __str__(self):
        return f"POINT = {self.x}, {self.y}"


def test_basic():
    my_os = dummy.os()
    proc_info = my_os.process_info_list()[0]
    proc = my_os.process_from_info(proc_info)

    # Test writing new `TEST` structure.
    proc.write(proc_info.address, POINT, POINT(55, 3.14))

    # Test reading a structure.
    test_works = proc.read(proc_info.address, POINT)
    assert test_works.x == 55
