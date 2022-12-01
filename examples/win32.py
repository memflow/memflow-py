from memflow import *
from ctypes import *


class COFFHeader(Structure):
    _fields_ = [
        ("_pad0x0", c_uint8 * 6),
        ("sections", c_uint16),
        ("timestamp", c_uint32),
    ]

    def __str__(self):
        return f"{self.sections}, {self.timestamp}"


inventory = Inventory()
os = inventory.os("native")
process = os.process_from_name("CalculatorApp.exe")
module = process.module_from_name("CalculatorApp.dll")
header = process.read(module.base + 0x40, COFFHeader)
print(header)
