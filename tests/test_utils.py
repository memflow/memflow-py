from memflow import *


class POINT(Structure):
    _fields_ = [("x", c_uint32), ("y", c_float)]


class TEST(Structure):
    _fields_ = [("one", (c_uint32 * 2)), ("two", c_int64), ("ptr", POINTER64(POINT))]


def test_sizeof():
    assert sizeof(TEST) == 0x18


def test_struct_repr():
    assert repr(POINT(1, 2)) == "POINT(x=1, y=2)"
    assert repr(TEST([1, 2], 3, 0x0)) == "TEST(one=[1, 2], two=3, ptr=0)"


def test_struct_str():
    assert str(POINT(1, 2)) == "x=1 y=2"
    assert str(TEST([1, 2], 3, 0x0)) == "one=[1, 2] two=3 ptr=0"
