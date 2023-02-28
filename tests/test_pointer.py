from memflow import Pointer
import pytest


def test_init():
    Pointer(0x5)
    with pytest.raises(ValueError):
        Pointer(-0x5)


def test_ops():
    # Add offset
    assert Pointer(0x3) + 5 == Pointer(0x8)
    # Subtract offset
    assert Pointer(0x3) - 2 == Pointer(0x1)


def test_overflow():
    with pytest.raises(ValueError):
        Pointer(0x3) - 5
