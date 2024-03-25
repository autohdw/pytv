//! Python Templated Verilog
//! 
//! # Generation Process
//! ```txt
//! .pytv ----> .v.py --+-> .v
//!                     |
//!                     +-> .inst
//! ```
//! 
//! # Examples
//! ```txt
//! a = 1 + 2;                #  Python inline
//! assign wire_`a` = wire_b; // Verilog with variable/expression substitute
//! /*!
//! b = a ** 2;               #  Python block
//! */
//! ```
//! The magic comment string can be configured (`!` as default).
//! 
//! # Related Auto Generator Projects
//! - **FLAMES**: template-based C++ library for Vitis HLS
//!   [[website](https://flames.autohdw.com)]
//!   [[GitHub](https://github.com/autohdw/flames)]
//!   [[paper at IEEE](https://ieeexplore.ieee.org/document/10437992)]
//!   [[paper PDF](https://wqzhao.org/assets/zhao2024flexible.pdf)]
//! - **AHDW**: a DSL, the predecessor of this project
//!   [[paper at IEEE](https://ieeexplore.ieee.org/document/10396119)]
//!   [[paper PDF](https://wqzhao.org/assets/zhao2023automatic.pdf)]

mod config;
mod convert;

#[cfg(feature = "inst")]
mod inst;

pub use config::Config;
pub use config::FileOptions;
pub use convert::Convert;
