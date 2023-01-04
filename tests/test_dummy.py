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
    test_struct = TEST((1, 2), -2, POINTER64(POINT)(proc_info.address + 0x7777))
    proc.write(proc_info.address, TEST, test_struct)
    proc.write(proc_info.address + 0x7777, POINT, POINT(55, 3.14))

    # Test reading a structure.
    test_works = proc.read(proc_info.address, TEST)
    assert test_works.two == -2

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


class TEST_OFFSETS(Structure):
    _fields_ = [("one", (c_uint32 * 2)), ("two", c_int64)]
    _offsets_ = [(0x8, "two_offset", c_int64)]

    def __str__(self):
        return f"TEST_OFFSETS = {self.one}, {self.two}"


def test_offsets():
    my_os = dummy.os()

    # Test writing new `TEST` structure.
    test_struct = TEST_OFFSETS((1, 2), 2, two_offset=2)
    my_os.phys_write(0, TEST_OFFSETS, test_struct)

    # Test reading a structure with offsets.
    test_works = my_os.phys_read(0, TEST_OFFSETS)
    assert test_works.two_offset == 2


def test_string():
    my_os = dummy.os()
    proc_info = my_os.process_info_list()[0]
    proc = my_os.process_from_info(proc_info)

    # Test writing a string using `bytes`
    proc.write(proc_info.address, c_char * 8, bytes("it works", "utf-8"))
    # Test reading a char null terminated string.
    test_works = proc.read_char_string(proc_info.address)
    assert test_works == "it works"
