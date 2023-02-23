from memflow import *


class POINT(Structure):
    _fields_ = [("x", c_uint32), ("y", c_float)]

    def __str__(self):
        return f"POINT = {self.x}, {self.y}"


class TEST(Structure):
    _fields_ = [("one", (c_uint32 * 2)), ("two", c_int64), ("ptr", POINTER64(POINT))]

    def __str__(self):
        return f"TEST = {self.one}, {self.two}, {self.ptr}"


def test_basic():
    proc = dummy.quick_process(4096, bytes([0x8]))
    proc_address = proc.info().address

    # Test writing new `TEST` structure.
    test_struct = TEST((1, 2), -2, POINTER64(POINT)(proc_address + 0x777))
    proc.write(proc_address, TEST, test_struct)
    proc.write(proc_address + 0x777, POINT, POINT(55, 3.14))

    # Test reading a structure.
    test_works = proc.read(proc_address, TEST)
    assert test_works.two == -2

    # Test reading through a pointer.
    point_works = proc.read_ptr(test_works.ptr)
    assert point_works.x == 55


def test_os_phys_rw():
    my_os = dummy.DummyOs(dummy.DummyMemory(4096)).retrieve_os()

    # Test writing new `TEST` structure.
    test_struct = TEST((1, 2), 2, POINTER64(POINT)(0x777))
    my_os.phys_write(0, TEST, test_struct)
    my_os.phys_write(0x777, POINT, POINT(55, 3.14))

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
    my_os = dummy.DummyOs(dummy.DummyMemory(4096)).retrieve_os()

    # Test writing new `TEST` structure.
    test_struct = TEST_OFFSETS((1, 2), 2, two_offset=2)
    my_os.phys_write(0, TEST_OFFSETS, test_struct)

    # Test reading a structure with offsets.
    test_works = my_os.phys_read(0, TEST_OFFSETS)
    assert test_works.two_offset == 2


class NESTED_OFFSET_TEST(Structure):
    _fields_ = [("inner", TEST_OFFSETS)]


# TODO: Test `_anonymous_` make sure it can unnest a struct field into struct
def test_nested_offsets():
    my_os = dummy.DummyOs(dummy.DummyMemory(4096)).retrieve_os()

    # Test writing new `TEST` structure.
    test_struct = TEST_OFFSETS((1, 2), 2, two_offset=2)
    my_os.phys_write(0, TEST_OFFSETS, test_struct)

    # Test reading a nested structure with offsets.
    test_works = my_os.phys_read(0, NESTED_OFFSET_TEST).inner
    assert test_works.two_offset == 2


def test_string():
    proc = dummy.quick_process(4096, bytes([0x8]))
    proc_address = proc.info().address

    # Test writing a string using `bytes`
    proc.write(proc_address, c_char * 8, bytes("it works", "utf-8"))
    # Test reading a char null terminated string.
    test_works = proc.read_char_string(proc_address)
    assert test_works == "it works"


class TEST_SIZEOF(Structure):
    _fields_ = [("one", (c_uint32 * 2)), ("two", c_int64), ("ptr", POINTER32(POINT))]


def test_sizeof():
    assert sizeof(TEST_SIZEOF) == 0x14


def test_struct_array():
    proc = dummy.quick_process(4096, bytes([0x8]))
    proc_address = proc.info().address

    # Test writing an array of structures.
    proc.write(proc_address, POINT * 3, [POINT(1, 2), POINT(3, 4), POINT(5, 6)])
    # Test reading an array of structures.
    test_works = proc.read(proc_address, POINT * 3)
    assert test_works[0] == POINT(1, 2)
