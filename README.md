# a121-sys

[![docs.rs](https://docs.rs/a121-sys/badge.svg)](https://docs.rs/a121-sys)
[![crates.io](https://img.shields.io/crates/v/a121-sys.svg)](https://crates.io/crates/a121-sys)
[![crates.io](https://img.shields.io/crates/d/a121-sys.svg)](https://crates.io/crates/a121-sys)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Current bindings version: 1.12.*


`a121-sys` is a Rust crate offering raw bindings for interfacing with the Acconeer A121 V-Band radar sensor. Designed for embedded systems, it provides low-level access to the sensor's functionalities, enabling the development of high-level abstractions in a `no_std` environment.

This crate is intended to serve as a foundation for accessing the detailed capabilities of the A121 radar sensor, including advanced configuration and sensor management.

## Features

The `a121-sys` crate exposes raw bindings necessary for implementing the following features:

- **Distance Measurement**: Direct access to configure the radar for precise distance measurements.
- **Presence Detection**: Direct access for detecting the presence of objects or individuals within a specific zone.

These features are made available through the crate's feature flags, allowing users to include only what they need for their application.

```toml
[features]
distance = []
presence = []
```

## Dependencies

- Acconeer A121 Static Library must be accessible in your build environment.
- `arm-none-eabi-gcc` is required for building the C wrapper for some examples.

```bash
# Ubuntu
sudo apt-get install gcc-arm-none-eabi
```

ESP targets require ESP toolchains, install via the VSCode extension from espressif.
Before you can build a121-sys, you need to set the environment variables by sourcing the applicable SDK, i.e.:
```
. ~/esp/v5.2.3/esp-idf/export.sh
```
Then make sure that the following gives you the correct sysroot path:
```
riscv32-esp-elf-gcc -print-sysroot
```

## Vendored SDK layout

You can point at an external Acconeer RSS package (headers + prebuilt `.a` files) instead of
copying libraries into this crate:

```bash
export A121_RSS_INCLUDE=/path/to/rss/include
export A121_RSS_LIB=/path/to/rss/lib
cargo build --target thumbv8m.main-none-eabihf
```

`ACC_RSS_LIBS` is also accepted (same as `A121_RSS_LIB`). The crate `rss/include/` tree is always
used for bindgen; only the **static libraries** need to live in `A121_RSS_LIB` or `rss/lib/`.

Typical layout:

```
rss/
  include/   # acc_*.h (shipped in this repo)
  lib/       # libacconeer_a121.a, optional detector libs (not always committed)
```

## Supported Targets

Support is dependent on the Acconeer A121 Static Library's availability:

- `thumbv7em-none-eabihf` — Cortex-M4F (e.g. STM32L4)
- `thumbv8m.main-none-eabihf` — Cortex-M33 (e.g. STM32WBA65)
- ESP RISC-V / Xtensa

## Getting Started

To use `a121-sys` in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
a121-sys = "0.2"
```

Ensure all dependencies are correctly set up in your build environment.

For more details on utilizing `a121-sys`, refer to the [documentation](https://docs.rs/a121-sys).

## Development and Contribution

We welcome contributions to expand and improve `a121-sys`. Whether it's adding new features, enhancing existing ones, or improving the documentation, feel free to open issues and submit pull requests.

## License

Distributed under the MIT License. See [LICENSE](https://github.com/Ragarnoy/a121-rs/LICENSE) for more information.
