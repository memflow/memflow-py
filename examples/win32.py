from memflow import *
import logging


class COFFHeader(Structure):
    _fields_ = [
        ("_pad0x0", c_uint8 * 6),
        ("sections", c_uint16),
        ("timestamp", c_uint32),
    ]

    def __str__(self):
        return f"{self.sections}, {self.timestamp}"


# Setup logging
FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

inventory = Inventory()
os = inventory.create_os("native")
process = os.process_from_name("CalculatorApp.exe")
module = process.module_by_name("CalculatorApp.dll")
header = process.read(module.base + 0x40, COFFHeader)
print(header)
