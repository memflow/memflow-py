# memflow-py

[Python] support for [Memflow], a physical memory introspection framework written in [Rust].

## Installation

1. Install python package: `pip install memflow`
2. Install appropriate memflow components (see [memflowup]).

## Example

```py
from memflow import *

class COFFHeader(Structure):
    _fields_ = [
        ("_pad0x0", c_byte * 6),
        ("sections", c_short),
        ("timestamp", c_uint32),
    ]

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
- [python virtual environment](https://docs.python.org/3/tutorial/venv.html)

### Steps

1. Fetch repository: `git clone https://github.com/memflow/memflow-py`
2. Install maturin package: `pip install maturin`
3. Install dev packages: `pip install '.[dev]'`
4. Create virtualenv `virtualenv .`
5. Active virtualenv `source ./bin/activate`
6. Build wheels and install package: `maturin develop`
7. Repeat step 4 when you make changes to source.
7. Install pytest `pip install pytest`
8. Test your changes: `python -m pytest`

For more information on building please see [Maturin].

[Memflow]: https://github.com/memflow/memflow
[memflowup]: https://github.com/memflow/memflowup
[Rust]: https://rust-lang.org/
[Python]: https://python.org/
[Maturin]: https://github.com/PyO3/maturin
