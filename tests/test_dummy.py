from memflow import *
from ctypes import *


class POINT(Structure):
    _fields_ = [("x", c_uint32), ("y", c_float)]

    def __str__(self):
        return f"POINT = {self.x}, {self.y}"


class TEST(Structure):
    _fields_ = [("one", (c_uint32 * 2)), ("two", c_int64), ("ptr", POINTER64(POINT))]

    def __str__(self):
        return f"TEST = {self.one}, {self.two}, {self.ptr}"


def test_basic():
    my_os = dummy.os()
    proc_info = my_os.process_info_list()[0]
    proc = my_os.process_from_info(proc_info)

    # Test writing new `TEST` structure.
    test_struct = TEST((1, 2), 2, POINTER64(POINT)(proc_info.address + 0x7777))
    proc.write(proc_info.address, TEST, test_struct)
    proc.write(proc_info.address + 0x7777, POINT, POINT(55, 3.14))

    # Test reading a structure.
    test_works = proc.read(proc_info.address, TEST)
    assert test_works.two == 2

    # Test reading through a pointer.
    point_works = proc.read_ptr(test_works.ptr)
    assert point_works.x == 55


def test_os_phys_rw():
    my_os = dummy.os()

    # Test writing new `TEST` structure.
    test_struct = TEST((1, 2), 2, POINTER64(POINT)(0x7777))
    my_os.phys_write(0, TEST, test_struct)
    my_os.phys_write(0x7777, POINT, POINT(55, 3.14))

    # Test reading a structure.
    test_works = my_os.phys_read(0, TEST)
    assert test_works.two == 2

    # Test reading through a pointer.
    point_works = my_os.phys_read_ptr(test_works.ptr)
    assert point_works.x == 55
