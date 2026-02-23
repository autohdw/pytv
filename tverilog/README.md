# tverilog

`tverilog` is a Python API binding for PyTV. It converts `.pytv` templates through an internal native Rust bridge, then executes the generated Python to produce Verilog outputs.
Use `from tverilog import generate` in user code. The module `tverilog._core` is internal and may change without notice.

## Install (development)

1. Build and install native extension:

```bash
maturin develop -m tverilog/native/Cargo.toml
```

2. Install Python package:

```bash
pip install -e tverilog
```

## API

```python
from tverilog import generate

result = generate("examples/test.pytv")
print(result.output_file)
```
