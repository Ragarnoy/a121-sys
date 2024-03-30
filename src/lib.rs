#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(dead_code)]

//! # a121-sys Crate Documentation
//!
//! Welcome to the `a121-sys` crate, the direct line to the Acconeer A121 radar sensor from your Rust applications.
//! This crate provides the raw FFI bindings needed to harness the power of the A121 sensor, enabling you to build advanced radar sensing capabilities into your projects.
//!
//! ## Quickstart
//!
//! Add `a121-sys` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! a121-sys = { version = "0.1", features = ["distance", "presence"] }
//! ```
//!
//! ## Features
//!
//! - **Distance Measurement**: Activate the `distance` feature to measure distances with high precision.
//! - **Presence Detection**: Use the `presence` feature to detect objects or people in specific areas.
//!
//! Enable these features as needed to keep your compile times short and your binary size small.
//!
//! ## Logging with the C Log Wrapper
//!
//! For those needing to integrate with the sensor's native logging, `a121-sys` offers a way to hook into the C library's logging mechanism. By defining a `rust_log` function in your Rust code, you can capture log messages from the sensor:
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


use core::concat;
use core::env;
use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

