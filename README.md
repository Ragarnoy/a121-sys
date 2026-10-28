# a121-sys

[![docs.rs](https://docs.rs/a121-sys/badge.svg)](https://docs.rs/a121-sys)
[![crates.io](https://img.shields.io/crates/v/a121-sys.svg)](https://crates.io/crates/a121-sys)
[![crates.io](https://img.shields.io/crates/d/a121-sys.svg)](https://crates.io/crates/a121-sys)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Current bindings version: 1.13.*


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

- The Acconeer A121 static libraries, from the same SDK version as the bindings.
- A C cross compiler for the target, which builds the logging bridge and provides the C library
  headers bindgen needs, and libclang for bindgen.

| Target | C compiler | Notes |
|---|---|---|
| `thumbv7em-none-eabihf` (Cortex-M4F) | `arm-none-eabi-gcc` | Ubuntu: `apt install gcc-arm-none-eabi libnewlib-arm-none-eabi` |
| `thumbv8m.main-none-eabihf` (Cortex-M33) | `arm-none-eabi-gcc` | |
| `riscv32imac-esp-espidf`, `riscv32imc-esp-espidf` | `riscv32-esp-elf-gcc` | Source the ESP-IDF `export.sh` |
| `riscv32imac-unknown-none-elf`, `riscv32imc-unknown-none-elf` | `riscv32-esp-elf-gcc` or `riscv32-unknown-elf-gcc` | Needs newlib headers |
| `xtensa-esp32s3-none-elf`, `-esp32s2-`, `-esp32-` | `xtensa-esp32s3-elf-gcc` etc. | Install with [espup](https://github.com/esp-rs/espup) and `source ~/export-esp.sh`, which also provides the libclang that knows Xtensa |

Set `CC_<target>` to use another compiler. The build reads the include paths from the compiler,
so no `BINDGEN_EXTRA_CLANG_ARGS` are needed.

## SDK location

Point `A121_RSS_LIB` (or `ACC_RSS_LIBS`) at the absolute path of the directory holding `libacconeer_a121.a` and the
detector libraries:

```bash
export A121_RSS_LIB=/path/to/a121-v1.13.0/lib
cargo build --target thumbv7em-none-eabihf
```

Without it, the build looks in `rss/lib`, `libs`, `staticlibs`, `../libs`, `/usr/local/lib/acconeer`
and `/usr/lib/acconeer`. The bindings are generated from the headers in this crate's `rss/include`;
set `A121_RSS_INCLUDE` to use another SDK's `include` directory, which must be from the same SDK
version.

With the `stub_library` feature, the crate builds and links against generated stubs instead, for
tests and CI without the Acconeer libraries.

## Supported Targets

Support depends on Acconeer providing static libraries for the target:

- `thumbv7em-none-eabihf`: Cortex-M4F, e.g. STM32L4
- `thumbv8m.main-none-eabihf`: Cortex-M33, e.g. STM32WBA65
- ESP RISC-V and Xtensa, with ESP-IDF or bare metal (esp-hal)

## Getting Started

To use `a121-sys` in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
a121-sys = "0.8"
```

Ensure all dependencies are correctly set up in your build environment.

For more details on utilizing `a121-sys`, refer to the [documentation](https://docs.rs/a121-sys).

## Development and Contribution

We welcome contributions to expand and improve `a121-sys`. Whether it's adding new features, enhancing existing ones, or improving the documentation, feel free to open issues and submit pull requests.

## License

Distributed under the MIT License. See [LICENSE](https://github.com/Ragarnoy/a121-rs/LICENSE) for more information.
