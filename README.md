# memflow-py

[Python] support for [Memflow], a physical memory introspection framework written in [Rust].

## Installation

1. Install python package: `pip install memflow`
2. Install appropriate memflow components (see [memflowup]).

## Example

```py
from memflow import *
from ctypes import *

class COFFHeader(Structure):
    _fields_ = [
        ("_pad0x0", c_byte * 6),
        ("sections", c_short),
        ("timestamp", c_uint32),
    ]

    def __str__(self):
        return f"{self.sections}, {self.timestamp}"

inventory = Inventory()
os = inventory.create_os("native")
process = os.process_from_name("CalculatorApp.exe")
module = process.module_by_name("CalculatorApp.dll")
header = process.read(module.base + 0x40, COFFHeader)
print(header)
```

## Building from source

### Prerequisites

- Rust ([Compilation support](https://github.com/memflow/memflow#compilation-support))
- Python (3.7 and up)

### Advisory warning

It is advised to build in a [python virtual environment](https://docs.python.org/3/tutorial/venv.html), so that you do not accidentally break anything that depends on this package.

### Steps

1. Fetch repository: `git clone https://github.com/memflow/memflow-py`
2. Install maturin package: `pip install maturin`
3. Install dev packages: `pip install '.[dev]'`
4. Build wheels and install package: `maturin develop`
5. Repeat step 4 when you make changes to source.
6. Test your changes: `python -m pytest`

For more information on building please see [Maturin].

[Memflow]: https://github.com/memflow/memflow
[memflowup]: https://github.com/memflow/memflowup
[Rust]: https://rust-lang.org/
[Python]: https://python.org/
[Maturin]: https://github.com/PyO3/maturin
