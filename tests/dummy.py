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


my_os = dummy.os()
proc_info = my_os.process_info_list()[0]
proc = my_os.process_from_info(proc_info)

# Test writing new `TEST` structure.
test_struct = TEST((1, 2), 2, POINTER64(POINT)(proc_info.address + 0x7777))
proc.write(proc_info.address, TEST, test_struct)
proc.write(test_works.ptr.addr, POINT, POINT(55, 3.14))

# Test reading a structure.
test_works = proc.read(proc_info.address, TEST)
print(f"test does work: {test_works}")


# Test reading through a pointer.
point_works = proc.read_ptr(test_works.ptr)
print(f"point through a pointer: {point_works}")
