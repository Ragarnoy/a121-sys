#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(dead_code)]

//! # a121-sys Crate Documentation
//!
//! Welcome to the `a121-sys` crate, providing raw FFI bindings for the Acconeer A121 radar sensor.
//! This crate offers low-level access to the A121 sensor's capabilities, designed for embedded systems
//! and `no_std` environments.
//!
//! ## Usage
//!
//! Add `a121-sys` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! # For real hardware
//! a121-sys = { version = "0.5", features = ["distance", "presence"] }
//!
//! # For testing/development with stub library
//! a121-sys = { version = "0.5", features = ["stub_library", "distance", "presence"] }
//! ```
//!
//! ## Features
//!
//! - **distance**: Enable distance measurement functionality
//! - **presence**: Enable presence detection functionality
//! - **stub_library**: Use stub implementations for testing/development without hardware
//! - **std**: Enable functionality requiring the standard library
//!
//! ## Library Requirements
//!
//! This crate requires the Acconeer A121 static libraries and the `arm-none-eabi-gcc` toolchain. You can either:
//!
//! 1. Use the real libraries:
//!    - Obtain the official Acconeer A121 SDK
//!    - Set `ACC_RSS_LIBS` environment variable to the library location
//!
//! 2. Use stub libraries (for testing/development):
//!    - Enable the `stub_library` feature
//!
//! ## Logging Integration
//!
//! The crate provides a way to integrate with the sensor's native logging through a C log wrapper.
//! Implement the `rust_log` function to capture log messages:
//!
//! ```no_run
//! use core::ffi::c_char;
//!
//! #[no_mangle]
//! pub extern "C" fn rust_log(level: u32, message: *const c_char) {
//!     // Your logging implementation here
//! }
//! ```
//!
//! ## Embedded Usage
//!
//! The crate is designed for embedded systems and supports:
//!
//! - `no_std` environments
//! - Cortex-M4 targets (specifically `thumbv7em-none-eabihf`, or risc)
//! - Both real hardware and stub implementations
//!
//! Example target configuration:
//!
//! ```toml
//! [build]
//! target = "thumbv7em-none-eabihf"
//!
//! [target.thumbv7em-none-eabihf]
//! rustflags = [
//!     "-C", "link-arg=-mcpu=cortex-m4",
//!     "-C", "link-arg=-mthumb",
//!     "-C", "link-arg=-mfloat-abi=hard",
//!     "-C", "link-arg=-mfpu=fpv4-sp-d16",
//! ]
//! ```
//!
//! ## Safety
//!
//! This crate provides raw FFI bindings which are inherently unsafe. Users should:
//!
//! - Take care when implementing the logging callback
//! - Ensure proper initialization of the sensor
//! - Follow proper memory management practices
//! - Consider using higher-level safe wrappers (like the `a121` crate)
//!
//! ## Development and Testing
//!
//! For development without hardware:
//!
//! 1. Enable the stub library feature:
//!    ```toml
//!    a121-sys = { version = "0.5", features = ["stub_library"] }
//!    ```
//!
//! 2. Install required toolchain:
//!    ```bash
//!    rustup target add thumbv7em-none-eabihf
//!    # Install ARM GCC toolchain for your platform
//!    ```
//!
//! ## Environment Variables
//!
//! - `ACC_RSS_LIBS`: Path to the Acconeer A121 libraries
//! - `CPATH`: Additional C header paths (usually set automatically)
//!
//! ## Version Compatibility
//!
//! - Current bindings version: 1.8.0
//! - Minimum Supported Rust Version (MSRV): 1.82.0
//!

use core::concat;
use core::env;
use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
