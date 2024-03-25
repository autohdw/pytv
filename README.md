# PyTV
**Py**thon **T**emplated **V**erilog

[![Crates.io Version](https://img.shields.io/crates/v/pytv?style=for-the-badge)](https://crates.io/crates/pytv)
[![docs.rs](https://img.shields.io/docsrs/pytv?style=for-the-badge&label=docs.rs)](https://docs.rs/pytv)
[![GitHub](https://img.shields.io/github/license/autohdw/pytv?style=for-the-badge)](LICENSE)

## Package
The package `pytv` is available on [crates.io](https://crates.io/crates/pytv).
Documentation is available on [docs.rs](https://docs.rs/pytv).

To use the package, run
```sh
cargo add pytv
```

## Features
### Python Template
This is the basic feature of this package.

```pytv
//! a = 1 + 2;            #  Python inline
assign wire_`a` = wire_b; // Verilog with variable/expression substitute
/*!
b = a ** 2;               #  Python block
*/
```
The magic comment string can be configured (`!` as default).

### Instantiation
The crate feature `inst` is enabled by default.
YAML contents between `<INST>` and `</INST>` are used to provide instantiation information.

## Related Auto Generator Projects
- **FLAMES**: template-based C++ library for Vitis HLS
  [[website](https://flames.autohdw.com)]
  [[GitHub](https://github.com/autohdw/flames)]
  [[paper at IEEE](https://ieeexplore.ieee.org/document/10437992)]
  [[paper PDF](https://wqzhao.org/assets/zhao2024flexible.pdf)]
- **AHDW**: a DSL, the predecessor of this project
  [[paper at IEEE](https://ieeexplore.ieee.org/document/10396119)]
  [[paper PDF](https://wqzhao.org/assets/zhao2023automatic.pdf)]

## Author
[Teddy van Jerry](https://github.com/Teddy-van-Jerry) ([Wuqiong Zhao](https://wqzhao.org))
